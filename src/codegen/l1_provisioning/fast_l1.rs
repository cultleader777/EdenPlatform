#[cfg(not(test))]
use std::fmt::Write;
use std::collections::BTreeMap;
use crate::{
    codegen::{
        Directory, L1ProvOutputs, CodegenSecrets, secrets::{SecretKind, SecretValue, SecretsStorage}, FastProvSecrets, nixplan::{NixAllServerPlans, root_secret_key}
    },
    static_analysis::CheckedDB, database::TableRowPointerServer
};


// mock for testing, we don't need those
#[cfg(test)]
pub fn generate_fast_l1_provisioning_part(
    _prov_dir: &mut Directory, _checked: &CheckedDB,
    _l1_outputs: &L1ProvOutputs, _secrets: &CodegenSecrets,
)
{
}

#[cfg(not(test))]
pub fn generate_fast_l1_provisioning_part(
    prov_dir: &mut Directory, checked: &CheckedDB,
    l1_outputs: &L1ProvOutputs, secrets: &CodegenSecrets,
)
{
    use std::io::Write;

    let admin_key_bytes = base64::decode(
        secrets.fast_prov_secrets.admin_provisioning_encryption_key.value()
    ).unwrap();
    let admin_secret_key = sodiumoxide::crypto::box_::SecretKey::from_slice(&admin_key_bytes).unwrap();
    let l1_prov_id = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    prov_dir.create_file("epl-prov-id", l1_prov_id.clone());
    for (region, servers) in &l1_outputs.regions {
        let region_name = checked.db.region().c_region_name(*region);
        let region_dir = prov_dir.create_directory(&region_name);

        let mut publish_script = "#!/bin/sh

function upload_plan_with_retry() {
  KV_PATH=$1
  KV_FILE=$2
  RAW_FILE=$( echo $KV_FILE | tr -d '@' )

  # if server not bootstrapped yet file will be skipped
  if [ ! -f \"$RAW_FILE\" ]
  then
    echo \"File $RAW_FILE doesn't exist, skipping\"
    return 0
  fi

  for I in $(seq 1 3)
  do
    if consul kv put $KV_PATH $KV_FILE
    then
      break
    fi
    echo Failed consul plan upload, trying again in 3 seconds...
    sleep 3
  done
}

UPLOAD_START=$( date +%s%N )
".to_string();

        for (srv, values) in servers {
            let hostname = checked.db.server().c_hostname(*srv);
            let srv_private_key = derive_server_fast_l1_provisioning_private_key_b64(
                checked, &secrets.fast_prov_secrets, *srv
            );
            let srv_public_key = srv_private_key.public_key();
            let filename = format!("plan_{hostname}.bin");

            writeln!(&mut publish_script, r#"upload_plan_with_retry epl-l1-plans/{hostname} @{filename}"#).unwrap();

            let mut buffer = String::with_capacity(values.preconditions.len() + values.provisioning.len());

            buffer += &values.preconditions;
            // TODO: optimize these replaces for big plans
            // maybe make default value for these values instead of replacing them
            buffer += &values.provisioning
                             .replace("L1_EPL_PROVISIONING_ID", &l1_prov_id)
                             .replace("L1_PROVISIONING_TOLERATE_REBUILD_FAIL", "false")
                             .replace("L1_RESTART_CONSUL_POST_SECRETS", "false");

            let mut e = zstd::Encoder::new(Vec::new(), 7).expect("Can't create zstd encoder");
            let _ = e.write_all(buffer.as_bytes()).unwrap();
            let finished = e.finish().expect("Can't finish encoding");
            //println!("srv {} privkey {}", hostname, base64::encode(srv_private_key));
            //println!("{:.2} ratio {} -> {}",
            //         finished.len() as f64 / buffer.len() as f64, buffer.len(), finished.len());

            let nonce = sodiumoxide::crypto::box_::gen_nonce();
            let enc = sodiumoxide::crypto::box_::seal(
                &finished, &nonce, &srv_public_key, &admin_secret_key
            );

            //println!("Nonce {:?} payload len {} checksum {}", hex::encode(nonce.0), enc.len(), hex::encode(hmac_sha256::Hash::hash(&enc)));

            let mut final_buf = Vec::with_capacity(nonce.0.len() + enc.len());
            final_buf.extend_from_slice(&nonce.0);
            final_buf.extend_from_slice(&enc);

            assert_eq!(final_buf.len(), nonce.0.len() + enc.len());

            region_dir.create_file_binary_always_overwrite(filename.as_str(), final_buf);
        }

        writeln!(&mut publish_script, r#"UPLOAD_END=$( date +%s%N )"#).unwrap();
        writeln!(&mut publish_script, r#"UPLOAD_MS=$(( ( UPLOAD_END - UPLOAD_START ) / 1000000 ))"#).unwrap();
        writeln!(&mut publish_script, r#"echo Upload took ${{UPLOAD_MS}}ms"#).unwrap();

        region_dir.create_executable_file("consul-push.sh", publish_script);
    }
}

#[cfg(test)]
pub fn generate_fast_prov_secrets(checked: &CheckedDB, secrets: &mut SecretsStorage) -> FastProvSecrets {
    let mut regions = BTreeMap::new();
    for region in checked.db.region().rows_iter() {
        let region_name = checked.db.region().c_region_name(region);
        let secret = secrets.fetch_secret(
            format!("fast_l1_region_{region_name}_decryption_seed"),
            SecretKind::StrongPassword42Symbols
        );
        assert!(regions.insert(region, secret).is_none());
    }
    FastProvSecrets {
        admin_provisioning_encryption_key: SecretValue::from_string("admin_provisioning_encryption_key".to_string()),
        admin_provisioning_public_key: "admin_provisioning_public_key".to_string(),
        region_provisioning_decryption_seeds: regions,
    }
}

#[cfg(not(test))]
pub fn generate_fast_prov_secrets(checked: &CheckedDB, secrets: &mut SecretsStorage) -> FastProvSecrets {
    let admin_provisioning_encryption_key =
        secrets.fetch_secret("admin_fast_l1_secret_key".to_string(), SecretKind::LibsodiumBoxPrivateKey);
    let sodium_key_bytes = base64::decode(admin_provisioning_encryption_key.value()).unwrap();
    let sec_admin_key = sodiumoxide::crypto::box_::SecretKey::from_slice(&sodium_key_bytes).unwrap();
    let pub_admin_key = sec_admin_key.public_key();
    let admin_provisioning_public_key = base64::encode(pub_admin_key.0);
    let mut regions = BTreeMap::new();
    for region in checked.db.region().rows_iter() {
        let region_name = checked.db.region().c_region_name(region);
        let secret = secrets.fetch_secret(
            format!("fast_l1_region_{region_name}_decryption_seed"),
            SecretKind::StrongPassword42Symbols
        );
        assert!(regions.insert(region, secret).is_none());
    }
    FastProvSecrets {
        admin_provisioning_encryption_key,
        admin_provisioning_public_key,
        region_provisioning_decryption_seeds: regions,
    }
}

pub fn derive_server_fast_l1_provisioning_private_key_b64(
    checked: &CheckedDB, fps: &FastProvSecrets, server: TableRowPointerServer,
) -> sodiumoxide::crypto::box_::SecretKey {
    let hostname = checked.db.server().c_hostname(server);
    let dc = checked.db.server().c_dc(server);
    let region = checked.db.datacenter().c_region(dc);
    let region_name = checked.db.region().c_region_name(region);
    let region_seed = fps.region_provisioning_decryption_seeds.get(&region).unwrap();
    let region_sec = region_seed.value();

    let seed_str = format!("{region_sec}.{region_name}.{hostname}");
    let hash = hmac_sha256::Hash::hash(seed_str.as_bytes());
    let seed = sodiumoxide::crypto::box_::Seed(hash);
    let (_pk, sk) = sodiumoxide::crypto::box_::keypair_from_seed(&seed);
    sk
}

const SIG_CHECKER_BYTES_B64: &str = include_str!("sigchecker.b64");

pub fn provision_fast_l1_provisioning(
    checked: &CheckedDB,
    plans: &mut NixAllServerPlans,
    server: TableRowPointerServer,
    cgen_secrets: &CodegenSecrets,
) {
    let hostname = checked.db.server().c_hostname(server);
    let plan = plans.fetch_plan(server);
    let derived = derive_server_fast_l1_provisioning_private_key_b64(
        checked, &cgen_secrets.fast_prov_secrets, server
    );
    plan.add_secret(root_secret_key(
        "l1-fast-prov-decryption-key".to_string(),
        SecretValue::from_string(base64::encode(derived.0))
    ));
    plan.add_secret(root_secret_key(
        "l1-fast-prov-admin-pub-key".to_string(),
        SecretValue::from_string(cgen_secrets.fast_prov_secrets.admin_provisioning_public_key.clone()),
    ));
    plan.add_pre_l1_provisioning_shell_hook(format!(r#"
mkdir -p /etc/nixos/l1-checker
pushd /etc/nixos/l1-checker
echo {SIG_CHECKER_BYTES_B64} | base64 -d | gunzip | tar x -C .
popd
"#));

    plan.add_custom_nix_block(format!(r#"
     # l1 agent
     systemd.services.l1-fast-agent = {{
       wantedBy = [ "multi-user.target" ];
       requires = [ "network-online.target" ];
       after = [ "network-online.target" "consul.service" ];
       script =
       let
         l1Checker = import ./l1-checker/default.nix {{ pkgs = pkgs; }};
       in
       ''
         export PATH=/run/current-system/sw/bin:$PATH
         # wait for consul to become available
         while ! ${{pkgs.consul}}/bin/consul kv get epl-l1-plans/{hostname}
         do
           sleep 7
         done

         ${{pkgs.consul}}/bin/consul watch \
           -type=key -key=epl-l1-plans/{hostname} \
           ${{l1Checker}}/checker \
             /run/keys/l1-fast-prov-decryption-key \
             /run/keys/l1-fast-prov-admin-pub-key \
             /run/secdir/l1-fast-plan.zst
       '';

       serviceConfig = {{
         User = "root";
         Group = "root";
         Type = "simple";
         Restart = "always";
         RestartSec = "20";
       }};

       enable = true;
     }};
"#));
}
