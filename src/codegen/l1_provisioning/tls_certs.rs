use std::collections::HashMap;

use crate::{
    codegen::{
        nixplan::{root_secret_key, NixAllServerPlans},
        secrets::{sec_files, SecretKind, SecretValue, SecretsStorage},
    },
    static_analysis::CheckedDB,
};

struct PublicCert {
    private_key: SecretValue,
    certificate: SecretValue,
}

pub(crate) fn provision_tls_certificates(
    db: &CheckedDB,
    plans: &mut NixAllServerPlans,
    secrets: &mut SecretsStorage,
) {
    assert_eq!(
        db.db.tld().len(),
        1,
        "Now only one TLD is assumed, should expand in the future"
    );

    let ca_key = "public_tls_self_signed_ca_certificate";
    let pkey_key = "public_tls_self_signed_ca_private_key";

    let ca_sec_files = sec_files(&[
        (SecretKind::TlsCertificate, ca_key, "ca.pem"),
        (SecretKind::TlsPrivateKey, pkey_key, "ca-key.pem"),
    ]);

    let ca_conf = r#"{
            "CN": "CA Key",
            "CA": {
                "expiry": "148920h",
                "pathlen": 0
            },
            "hosts": [],
            "key": {
                "algo": "ecdsa",
                "size": 256
            },
            "names": []
        }"#;

    let _ = secrets.multi_secret_derive(
        &[("ca-conf.json", ca_conf)],
        vec![],
        ca_sec_files.clone(),
        r#"
        cfssl gencert -initca ca-conf.json > ca-keys.json
        cat ca-keys.json | cfssljson -bare ca
    "#,
    );

    let mut res = HashMap::new();
    for tld in db.db.tld().rows_iter() {
        let full_tld_name = db.db.tld().c_domain(tld);
        let certificates_request = format!("{},*.{}", full_tld_name, full_tld_name);

        let cert_key = format!("public_tls_self_signed_certificate_{full_tld_name}");
        let pkey_key = format!("public_tls_self_signed_private_key_{full_tld_name}");

        let mut server_vec = secrets.multi_secret_derive(
            &[
                (
                    "https-cfssl.json",
                    r#"
                    {
                        "signing": {
                            "default": {
                                "expiry": "148920h",
                                "usages": ["signing", "key encipherment", "server auth", "client auth"]
                            }
                        }
                    }
                    "#
                )
            ],
            ca_sec_files.clone(),
            sec_files(&[
                (SecretKind::TlsPrivateKey, &pkey_key, "https-key.pem"),
                (SecretKind::TlsCertificate, &cert_key, "https.pem"),
            ]),
            &format!(r#"
                echo '{{}}' | \
                    cfssl gencert -ca=ca.pem -ca-key=ca-key.pem -config=https-cfssl.json \
                        -hostname="{certificates_request},localhost,127.0.0.1" - | \
                    cfssljson -bare https
            "#),
        );

        let cert = server_vec.pop().unwrap();
        let pkey = server_vec.pop().unwrap();
        assert!(server_vec.is_empty());

        res.insert(
            tld,
            PublicCert {
                private_key: pkey,
                certificate: cert,
            },
        );
    }

    // Put certificates for every server
    for server in db.db.server().rows_iter() {
        let dc = db.db.server().c_dc(server);
        let region = db.db.datacenter().c_region(dc);
        let tld = db.db.region().c_tld(region);
        let certs = res
            .get(&tld)
            .expect("We assume all TLD public certs are generated");
        let plan = plans.fetch_plan(server);
        plan.add_ca_cert_file(certs.certificate.to_string());
        // add as config inside mkdir?
        let mut sec_vol_vec = Vec::new();
        let tls_key = plan.add_secret(root_secret_key(
            "public_tls_key.pem".to_string(),
            certs.private_key.clone(),
        ));
        sec_vol_vec.push(tls_key.absolute_path());
        let tls_conf = plan.add_secret(root_secret_key(
            "public_tls_cert.pem".to_string(),
            certs.certificate.clone(),
        ));
        sec_vol_vec.push(tls_conf.absolute_path());
        plan.create_secrets_nomad_volume("ssl_certs", &sec_vol_vec);
    }
}
