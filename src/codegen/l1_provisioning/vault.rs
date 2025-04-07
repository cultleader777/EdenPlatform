use std::fmt::Write;
use std::collections::HashMap;

use convert_case::Casing;

use crate::{
    codegen::{
        nixplan::{self, NixAllServerPlans, ZfsDataset},
        secrets::{sec_files, SecretFile, SecretKind, SecretValue, SecretsStorage, tls_cert_expiration_days},
    },
    static_analysis::{CheckedDB, L1Projections, server_runtime::ServerRuntime, networking::{first_region_server, prometheus_metric_exists_test, admin_service_responds_test}}, database::{TableRowPointerRegion, Database, TableRowPointerServer},
};

pub(crate) fn provision_vault(
    db: &CheckedDB,
    plans: &mut NixAllServerPlans,
    secrets: &mut SecretsStorage,
) {
    let vault_ca_secrets = derive_vault_ca_secrets(secrets);
    for region in db.db.region().rows_iter() {
        let region_name = db.db.region().c_region_name(region);
        let region_vault_servers = db.sync_res.network.vault_masters.get(&region).unwrap();

        for dc in db.db.region().c_referrers_datacenter__region(region) {
            for server in db.db.datacenter().c_referrers_server__dc(*dc) {
                let tls_ca_file = {
                    let plan = plans.fetch_plan(*server);
                    plan.add_env_variable(
                        "VAULT_ADDR".to_string(),
                        "https://vault.service.consul:8200".to_string(),
                    );
                    plan.add_ca_cert_file(vault_ca_secrets.ca_certificate.value().clone());

                    plan.add_secret(nixplan::all_users_readable_key(
                        "vault-ca.crt".to_string(),
                        vault_ca_secrets.ca_certificate.clone(),
                    ))
                        .absolute_path()
                };

                let consul_acl_token = secrets.fetch_secret(
                    format!("vault_region_{region_name}_consul_acl_token"),
                    SecretKind::Guid,
                );

                let is_consul_master = db.db.server().c_is_consul_master(*server);
                let is_vault_instance = db.db.server().c_is_vault_instance(*server);

                if is_consul_master {
                    let plan = plans.fetch_plan(*server);

                    let _ = plan.add_secret(nixplan::root_secret_key(
                        "vault-service-consul-acl-token.txt".to_string(),
                        consul_acl_token.clone(),
                    ));
                    plan.add_shell_package(
                        "epl-vault-consul-acl-bootstrap",
                        vault_consul_token_bootstrap_script(),
                    );
                }

                if is_vault_instance {
                    let hostname = db.db.server().c_hostname(*server);
                    let fqdn = db.projections.server_fqdns.value(*server);
                    let vault_iface = db.projections.consul_network_iface.value(*server);
                    let vault_service_ip = db.db.network_interface().c_if_ip(*vault_iface);

                    let ca_sec_files = vault_ca_sec_files();
                    let mut output_sec_files = Vec::with_capacity(2);
                    let vault_tls_pkey_key = format!("vault_region_{region_name}_tls_instance_{hostname}_private_key");
                    let vault_tls_cert_key = format!("vault_region_{region_name}_tls_instance_{hostname}_certificate");

                    if let Some(curr_cert) = secrets.get_secret(&vault_tls_cert_key) {
                        if tls_cert_expiration_days(&curr_cert) < 14 {
                            println!("Renewing TLS cert for key {vault_tls_cert_key}");
                            secrets.delete_secret(&vault_tls_pkey_key);
                            secrets.delete_secret(&vault_tls_cert_key);
                        }
                    }

                    output_sec_files.push(SecretFile {
                        kind: SecretKind::TlsPrivateKey,
                        key: vault_tls_pkey_key,
                        file_name: format!("{fqdn}-key.pem"),
                    });
                    output_sec_files.push(SecretFile {
                        kind: SecretKind::TlsCertificate,
                        key: vault_tls_cert_key,
                        file_name: format!("{fqdn}.pem"),
                    });

                    let mut server_tls_vec = secrets.multi_secret_derive(
                        &[
                            (
                                "server-cfssl.json",
                                r#"
                        {
                            "signing": {
                                "default": {
                                    "expiry": "8760h",
                                    "usages": ["signing", "key encipherment", "server auth", "client auth"]
                                }
                            }
                        }
                        "#,
                            )
                        ],
                        ca_sec_files,
                        output_sec_files,
                        format!(r#"
                    echo {{}} | \
                    cfssl gencert -ca=vault-ca.pem -ca-key=vault-ca-key.pem -config=server-cfssl.json \
                        -hostname="{fqdn},vault.service.consul,*.vault.service.consul,vault.service.{region_name}.consul,*.vault.service.{region_name}.consul,localhost,127.0.0.1" - | cfssljson -bare {fqdn}
                "#).as_str(),
                    );

                    let instance_tls_cert = server_tls_vec.pop().unwrap();
                    let instance_tls_key = server_tls_vec.pop().unwrap();

                    let plan = plans.fetch_plan(*server);
                    plan.add_zfs_dataset("vault".to_string(), ZfsDataset {
                        compression_enabled: true,
                        encryption_enabled: true,
                        expose_to_containers: false,
                        mountpoint: "/var/lib/vault".to_string(),
                        record_size: "4k".to_string(),
                        zpool: "rpool".to_string(),
                    });

                    let tls_instance_cert = plan
                        .add_secret(nixplan::custom_user_secret_key(
                            "vault".to_string(),
                            "vault-instance.crt".to_string(),
                            instance_tls_cert,
                        ))
                        .absolute_path();
                    let tls_instance_key = plan
                        .add_secret(nixplan::custom_user_secret_key(
                            "vault".to_string(),
                            "vault-instance.key".to_string(),
                            instance_tls_key,
                        ))
                        .absolute_path();

                    let full_vault_cfg = generate_vault_config(
                        db,
                        *server,
                        region_vault_servers.as_slice(),
                        consul_acl_token.value().as_str(),
                        tls_ca_file.as_str(),
                        tls_instance_key.as_str(),
                        tls_instance_cert.as_str(),
                        vault_service_ip,
                        fqdn,
                        &region_name,
                    );

                    let sec_cfg = nixplan::custom_user_secret_config(
                        "vault".to_string(),
                        "vault_config.hcl".to_string(),
                        full_vault_cfg,
                    );
                    let cfg = plan.add_secret_config(sec_cfg);

                    let cfg_path = cfg.absolute_path();

                    plan.add_custom_nix_block(format!(
                        r#"
    users.users.vault = {{
        isSystemUser = true;
        description = "Vault service";
        extraGroups = ["keys"];
        group = "vault";
    }};
    users.groups.vault = {{}};

    systemd.services.vault = {{
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];

      serviceConfig = {{
        User = "vault";
        Group = "vault";
        Type = "simple";
        ExecStartPre = [
            "+${{pkgs.coreutils}}/bin/mkdir -p /var/lib/vault"
            "+${{pkgs.coreutils}}/bin/chown vault:vault /var/lib/vault"
            "+${{pkgs.coreutils}}/bin/chmod 700 /var/lib/vault"
        ];
        ExecStart = "${{pkgs.vault-bin}}/bin/vault server -config={cfg_path} -log-level=info";
        Restart = "always";
        RestartSec = "20";
        TasksMax = "infinity";
      }};

      enable = true;
    }};
"#
                    ));

                    plan.add_shell_package(
                        "epl-vault-operator-init",
                        r#"
export VAULT_ADDR=$1
# pass initial text as argument or init vault

${pkgs.vault-bin}/bin/vault operator init &> /run/secdir/vault-init-output.txt.tmp

if grep -e 'Initial Root Token:' /run/secdir/vault-init-output.txt.tmp &> /dev/null
then
    mv /run/secdir/vault-init-output.txt.tmp /run/secdir/vault-init-output.txt

    echo Unsealing initial vault after 3 seconds...
    sleep 3

    for UK in $(seq 1 3)
    do
        UNSEAL_KEY=$( cat /run/secdir/vault-init-output.txt | grep "Unseal Key $UK:" | sed -E 's/^.*: //' )
        ${pkgs.vault-bin}/bin/vault operator unseal $UNSEAL_KEY
    done

    export VAULT_TOKEN=$( cat /run/secdir/vault-init-output.txt | grep "Initial Root Token:" | sed -E 's/^.*: //' )
    ${pkgs.vault-bin}/bin/vault secrets enable -path=epl kv-v2 || true
fi
"#,
                    );

                    plan.add_shell_package(
                        "epl-vault-operator-unseal",
                        &format!(r#"
export VAULT_ADDR=$1
# pass initial text as argument or init vault
VAULT_KEY_BASE64=$2

export VAULT_INIT_RES=$( echo "$VAULT_KEY_BASE64" | base64 -d )

if [ -z "$VAULT_INIT_RES" ]
then
    echo Failed to initialize vault/key is not set
    exit 7
fi

# make sure vault is responsive before we try to unseal
while ! curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"initialized":'
do
  sleep 3
done

# make sure vault is initialized
while ! curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"initialized":true'
do
  sleep 3
done

# in case we need to restart due to raft logs
if sudo journalctl -u vault.service --since "$(systemctl show vault.service -p ExecMainStartTimestamp | cut -d= -f2)" | grep 'no TLS config found' &>/dev/null
then
  echo "Restarting vault and waiting 10 seconds"
  sudo systemctl restart vault.service
  sleep 10
fi

if curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"sealed":true'
then
  for UK in $(seq 1 3)
  do
    UNSEAL_KEY=$( echo "$VAULT_INIT_RES" | grep "Unseal Key $UK:" | sed -E 's/^.*: //' )
    ${{pkgs.vault-bin}}/bin/vault operator unseal $UNSEAL_KEY
  done
fi

export VAULT_TOKEN=$( echo "$VAULT_INIT_RES" | grep "Initial Root Token:" | sed -E 's/^.*: //' )
${{pkgs.vault-bin}}/bin/vault secrets enable -path=epl kv-v2 || true
"#),
                    );
                }
            }
        }
    }
}

pub fn vault_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut vault_master_ips: HashMap<TableRowPointerRegion, (String, Vec<String>)> = HashMap::new();
    // test main consul services for region
    for r in db.region().rows_iter() {
        let reg_name = db.region().c_region_name(r);
        let reg_name_snake = reg_name.to_case(convert_case::Case::Snake);
        let mut master_ips_for_region: Vec<String> = Vec::new();
        let fr_server = first_region_server(db, r);
        if let Some(fr_server) = fr_server {
            let fr_net_iface = l1proj.consul_network_iface.value(fr_server);
            let fr_ip = format!("{}:53", db.network_interface().c_if_ip(*fr_net_iface));
            for dc in db.region().c_referrers_datacenter__region(r) {
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    if db.server().c_is_vault_instance(*server) {
                        let iface = l1proj.consul_network_iface.value(*server);
                        let ip = db.network_interface().c_if_ip(*iface);
                        master_ips_for_region.push(ip.clone());
                    }
                }
            }

            let expected_actives = 1;
            if master_ips_for_region.is_empty() {
                // our job is not to validate but add tests if everything okay
                continue;
            }

            let expected_standbys = master_ips_for_region.len() - expected_actives;

            runtime.add_integration_test(
                format!("vault_servers_available_region_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                    target_servers: vec![fr_ip.clone()],
                    queries: vec![
                        ("vault.service.consul".to_string(), master_ips_for_region.clone())
                    ]
                },
            );

            runtime.add_integration_test(
                format!("vault_active_server_available_region_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionARecordCount {
                    target_servers: vec![fr_ip.clone()],
                    queries: vec![
                        ("active.vault.service.consul".to_string(), expected_actives)
                    ]
                },
            );

            runtime.add_integration_test(
                format!("vault_standby_server_available_region_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionARecordCount {
                    target_servers: vec![fr_ip.clone()],
                    queries: vec![
                        ("standby.vault.service.consul".to_string(), expected_standbys)
                    ]
                },
            );

            runtime.add_integration_test(
                format!("vault_ui_responds_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::HttpGetRespondsString {
                    hostname: None,
                    expected_string: "<title>Vault</title>".to_string(),
                    http_server_port: 8200,
                    is_https: true,
                    path: "/ui/".to_string(),
                    server_ips: master_ips_for_region.clone(),
                    use_admin_panel_credentials: Some(crate::static_analysis::server_runtime::IntegrationTestCredentials::AdminPanel),
                },
            );


            runtime.add_integration_test(
                format!("vault_ui_external_responds_{reg_name_snake}"),
                admin_service_responds_test(
                    db,
                    l1proj,
                    format!("adm-vault-{reg_name}"),
                    "/ui/",
                    "<title>Vault</title>"
                )
            );

            if let Some(reg_mon_cluster) = l1proj.monitoring_clusters.region_default(r) {
                runtime.add_integration_test(
                    format!("vault_region_{reg_name_snake}_prometheus_metrics_gathered"),
                    prometheus_metric_exists_test(db, l1proj, reg_mon_cluster, "vault_barrier_list_count")
                );
            }

            assert!(vault_master_ips.insert(r, (fr_ip.clone(), master_ips_for_region)).is_none());
        }
    }

    // test cross region consul services
    for source_region in db.region().rows_iter() {
        let src_reg_name = db.region().c_region_name(source_region);
        let src_reg_name_snake = src_reg_name.to_case(convert_case::Case::Snake);
        if let Some((src_first, _)) = vault_master_ips.get(&source_region) {
            for target_region in db.region().rows_iter() {
                let tg_reg_name = db.region().c_region_name(target_region);
                let tg_reg_name_snake = tg_reg_name.to_case(convert_case::Case::Snake);
                let (_, tg_ips) = vault_master_ips.get(&target_region).unwrap();

                runtime.add_integration_test(
                    format!("vault_servers_available_from_region_{src_reg_name_snake}_to_region_{tg_reg_name_snake}"),
                    crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                        target_servers: vec![src_first.clone()],
                        queries: vec![
                            (format!("vault.service.{tg_reg_name}.consul"), tg_ips.clone())
                        ]
                    },
                );
            }
        }
    }
}

struct VaultCaSecrets {
    ca_certificate: SecretValue,
}

fn vault_ca_sec_files() -> Vec<SecretFile> {
    sec_files(&[
        (
            SecretKind::TlsPrivateKey,
            "global_vault_tls_ca_private_key",
            "vault-ca-key.pem",
        ),
        (
            SecretKind::TlsCertificate,
            "global_vault_tls_ca_certificate",
            "vault-ca.pem",
        ),
    ])
}

fn derive_vault_ca_secrets(secrets: &mut SecretsStorage) -> VaultCaSecrets {
    let ca_sec_files = vault_ca_sec_files();

    let mut ca_vec = secrets.multi_secret_derive(
        &[(
            "ca-conf.json",
            r#"
                {
                    "CN": "vault",
                    "CA": {
                        "expiry": "148920h",
                        "pathlen": 0
                    },
                    "hosts": [
                        "vault"
                    ],
                    "key": {
                        "algo": "ecdsa",
                        "size": 256
                    },
                    "names": []
                }
                "#,
        )],
        Vec::new(),
        ca_sec_files,
        r#"
            cfssl gencert -initca ca-conf.json > ca-keys.json
            cat ca-keys.json | cfssljson -bare vault-ca
        "#,
    );

    let ca_certificate = ca_vec.pop().unwrap();
    let _ca_private_key = ca_vec.pop().unwrap();

    VaultCaSecrets {
        ca_certificate,
    }
}

fn vault_consul_token_bootstrap_script() -> &'static str {
    r#"
            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^vault-service:$' &>/dev/null && exit 0

            cat > /tmp/epl-consul-vault-service-policy.hcl<<EOL
            service "vault" {
              policy = "write"
            }

            agent_prefix "" {
              policy = "read"
            }

            session_prefix "" {
              policy = "write"
            }
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "vault-service" \
                -description "Vault Service Policy" \
                -rules @/tmp/epl-consul-vault-service-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Vault Service Token" \
                -policy-name "vault-service" \
                -secret=$( sudo cat /run/keys/vault-service-consul-acl-token.txt )
"#
}

fn generate_vault_config(
    db: &CheckedDB,
    this_srv: TableRowPointerServer,
    region_vault_servers: &[TableRowPointerServer],
    vault_consul_acl_token: &str,
    tls_ca_cert_path: &str,
    tls_key_path: &str,
    tls_cert_path: &str,
    vault_service_ip: &str,
    fqdn: &str,
    region: &str,
) -> String {
    let mut this_srv_raft_peers = String::new();

    for other_srv in region_vault_servers {
        let other_fqdn = db.projections.server_fqdns.value(*other_srv);
        if *other_srv != this_srv {
            write!(&mut this_srv_raft_peers, r#"
  retry_join {{
    leader_api_addr = "https://{other_fqdn}:8200"
    leader_ca_cert_file = "{tls_ca_cert_path}"
    leader_client_cert_file = "{tls_cert_path}"
    leader_client_key_file = "{tls_key_path}"
  }}
"#).unwrap();
        }
    }

    format!(
        r#"
# Eden platform machines assume we never use swap
disable_mlock = true

cluster_name = "{region}"
max_lease_ttl = "768h"
default_lease_ttl = "768h"

disable_clustering = "False"
cluster_addr = "https://{fqdn}:8201"
api_addr = "https://{fqdn}:8200"

plugin_directory = "/usr/local/lib/vault/plugins"

listener "tcp" {{
  address = "{vault_service_ip}:8200"
  cluster_address = "{vault_service_ip}:8201"
  tls_cert_file = "{tls_cert_path}"
  tls_key_file = "{tls_key_path}"
  tls_client_ca_file = "{tls_ca_cert_path}"
  tls_min_version  = "tls12"
  tls_disable = "false"
  telemetry {{
    unauthenticated_metrics_access = "true"
  }}
}}

storage "raft" {{
  path = "/var/lib/vault"
  node_id = "{fqdn}"
{this_srv_raft_peers}
}}

service_registration "consul" {{
  address = "127.0.0.1:8500"
  check_timeout = "5s"
  disable_registration = "False"
  scheme = "http"
  service = "vault"
  service_tags = ""
  service_address = "{vault_service_ip}"
  token = "{vault_consul_acl_token}"
}}

ui = true

telemetry {{
  prometheus_retention_time = "3h"
}}
"#
    )
}
