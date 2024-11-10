use std::collections::{HashMap, BTreeMap};
use std::fmt::Write;

use convert_case::Casing;
use serde_json::Value;

use crate::codegen::nixplan::{write_nix_region_start, write_nix_region_end, ZfsDataset};
use crate::codegen::secrets::tls_cert_expiration_days;
use crate::database::TableRowPointerDatacenter;
use crate::{
    codegen::{
        nixplan::{self, custom_user_secret_key, NixAllServerPlans},
        secrets::{sec_files, SecretKind, SecretValue, SecretsStorage},
    },
    database::{TableRowPointerServer, TableRowPointerRegion, Database},
    static_analysis::{CheckedDB, L1Projections, server_runtime::ServerRuntime, networking::{first_region_server, admin_service_responds_test}},
};

use super::routing::inter_dc_routes;
use super::utils::{gen_reload_service_on_path_change, pad_string};

pub(crate) fn provision_consul(
    db: &CheckedDB,
    plans: &mut NixAllServerPlans,
    secrets: &mut SecretsStorage,
) {
    let secrets = derive_consul_secrets(db, secrets);

    for server in db.db.server().rows_iter() {
        let plan = plans.fetch_plan(server);
        let region = db.db.datacenter().c_region(db.db.server().c_dc(server));
        let region_secrets = secrets.region_secrets.get(&region).unwrap();

        let ca_cert_key = "consul-tls-ca-cert.pem";
        let abs_server_ca_path = plan
            .add_secret(custom_user_secret_key(
                "consul".to_string(),
                ca_cert_key.to_string(),
                secrets.tls_ca_certificate.clone(),
            ))
            .absolute_path();

        let is_server = db.db.server().c_is_consul_master(server);

        // for registering services on every node
        plan.add_secret(nixplan::root_secret_key(
            "consul-agent-token.txt".to_string(),
            region_secrets.consul_agent_acl_token.clone(),
        ));

        let certs_path = if is_server {
            let server_secrets = secrets.server_keys.get(&server).unwrap();

            let server_tls_cert_key = "consul-tls-server-cert.pem";
            let server_tls_pkey_key = "consul-tls-server-pkey.pem";
            let abs_server_cert_path = plan
                .add_secret(custom_user_secret_key(
                    "consul".to_string(),
                    server_tls_cert_key.to_string(),
                    server_secrets.tls_certificate.clone(),
                ))
                .absolute_path();
            let abs_server_pkey_path = plan
                .add_secret(custom_user_secret_key(
                    "consul".to_string(),
                    server_tls_pkey_key.to_string(),
                    server_secrets.tls_private_key.clone(),
                ))
                .absolute_path();

            let certs_path = ConsulServerCertsPath {
                abs_server_cert_path,
                abs_server_pkey_path,
            };

            // for creating these tokens from any consul server
            plan.add_secret(nixplan::root_secret_key(
                "consul-management-token.txt".to_string(),
                region_secrets.consul_initial_management_acl_token.clone(),
            ));
            plan.add_secret(nixplan::root_secret_key(
                "consul-default-token.txt".to_string(),
                // after consul 1.18 split off dns token from default
                secrets.consul_dns_token.clone(),
            ));
            plan.add_secret(nixplan::root_secret_key(
                "consul-fast-l1-token.txt".to_string(),
                region_secrets.consul_fast_l1_acl_token.clone(),
            ));
            //plan.add_secret(nixplan::root_secret_key(
            //    "consul-dns-token.txt".to_string(),
            //    secrets.consul_dns_token.clone(),
            //));

            for dc in db.db.region().c_referrers_datacenter__region(region) {
                let dc_name = db.db.datacenter().c_dc_name(*dc);
                let token = region_secrets.consul_vrrp_acl_tokens.get(dc).unwrap();
                plan.add_secret(nixplan::root_secret_key(
                    format!("consul-vrrp-token-{dc_name}.txt"),
                    token.clone(),
                ));
            }

            Some(certs_path)
        } else {
            None
        };

        let config = serde_json::to_string_pretty(&generate_consul_config(
            db,
            server,
            &secrets,
            certs_path.as_ref(),
            &abs_server_ca_path,
        ))
        .unwrap();

        let abs_config_path = plan
            .add_secret_config(nixplan::custom_user_secret_config(
                "consul".to_string(),
                "consul-config.json".to_string(),
                config,
            ))
            .absolute_path();

        plan.add_shell_package("epl-wait-for-consul", consul_wait_script());

        plan.add_custom_nix_block(format!(
            r#"
    services.consul = {{
      enable = true;
      webUi = true;
      forceAddrFamily = "ipv4";
      extraConfigFiles = [
        "{abs_config_path}"
      ];
    }};
    users.users.consul.extraGroups = ["keys"];
"#
        ));

        if is_server {
            plan.add_zfs_dataset("consul".to_string(), ZfsDataset {
                compression_enabled: true,
                encryption_enabled: true,
                expose_to_containers: false,
                mountpoint: "/var/lib/consul".to_string(),
                record_size: "4k".to_string(),
                zpool: "rpool".to_string(),
            });
        }

        let mut restart_block = String::new();
        gen_reload_service_on_path_change(
            &mut restart_block,
            &abs_config_path,
            "consul-restart",
            "consul.service",
            true,
        );

        plan.add_custom_nix_block(restart_block);

        plan.add_shell_package(
            "epl-consul-bootstrap",
            consul_token_policies_bootstrap_script(),
        );
        plan.add_shell_package(
            "epl-consul-vrrp-acl",
            &consul_vrrp_token_policy_bootstrap_script(db, region),
        );
    }

    // 1. add consul certs
    // 2. add consul config
}

fn consul_wait_script() -> &'static str {
    r#"
while ! ${pkgs.consul}/bin/consul members
do
  sleep 5
done
"#
}

pub fn consul_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut consul_master_ips: HashMap<TableRowPointerRegion, (String, Vec<String>)> = HashMap::new();
    // test main consul services for region
    for r in db.region().rows_iter() {
        let reg_name = db.region().c_region_name(r);
        let reg_name_snake = reg_name.to_case(convert_case::Case::Snake);
        let mut ips_for_region: Vec<String> = Vec::new();
        let fr_server = first_region_server(db, r);
        if let Some(fr_server) = fr_server {
            let fr_net_iface = l1proj.consul_network_iface.value(fr_server);
            let fr_ip = format!("{}:53", db.network_interface().c_if_ip(*fr_net_iface));
            for dc in db.region().c_referrers_datacenter__region(r) {
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    if db.server().c_is_consul_master(*server) {
                        let iface = l1proj.consul_network_iface.value(*server);
                        let ip = db.network_interface().c_if_ip(*iface);
                        ips_for_region.push(ip.clone());
                    }
                }
            }

            runtime.add_integration_test(
                format!("consul_servers_available_region_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                    target_servers: vec![fr_ip.clone()],
                    queries: vec![
                        ("consul.service.consul".to_string(), ips_for_region.clone())
                    ]
                },
            );

            runtime.add_integration_test(
                format!("consul_ui_responds_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::HttpGetRespondsString {
                    hostname: None,
                    expected_string: "<title>Consul by HashiCorp</title>".to_string(),
                    http_server_port: 8501,
                    is_https: true,
                    path: "/ui/".to_string(),
                    server_ips: ips_for_region.clone(),
                    use_admin_panel_credentials: Some(crate::static_analysis::server_runtime::IntegrationTestCredentials::AdminPanel),
                },
            );

            runtime.add_integration_test(
                format!("consul_ui_external_responds_{reg_name_snake}"),
                admin_service_responds_test(
                    db,
                    l1proj,
                    format!("adm-consul-{reg_name}"),
                    "/ui/",
                    "<title>Consul by HashiCorp</title>"
                )
            );

            assert!(consul_master_ips.insert(r, (fr_ip.clone(), ips_for_region)).is_none());
        }
    }

    // test cross region consul services
    for source_region in db.region().rows_iter() {
        let src_reg_name = db.region().c_region_name(source_region);
        let src_reg_name_snake = src_reg_name.to_case(convert_case::Case::Snake);
        if let Some((src_first, _)) = consul_master_ips.get(&source_region) {
            for target_region in db.region().rows_iter() {
                let tg_reg_name = db.region().c_region_name(target_region);
                let tg_reg_name_snake = tg_reg_name.to_case(convert_case::Case::Snake);
                let (_, tg_ips) = consul_master_ips.get(&target_region).unwrap();

                runtime.add_integration_test(
                    format!("consul_servers_available_from_region_{src_reg_name_snake}_to_region_{tg_reg_name_snake}"),
                    crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                        target_servers: vec![src_first.clone()],
                        queries: vec![
                            (format!("consul.service.{tg_reg_name}.consul"), tg_ips.clone())
                        ]
                    },
                );
            }
        }
    }
}

struct ConsulServerCertsPath {
    abs_server_cert_path: String,
    abs_server_pkey_path: String,
}

fn generate_consul_config(
    db: &CheckedDB,
    server: TableRowPointerServer,
    secrets: &ConsulSecrets,
    certs_path: Option<&ConsulServerCertsPath>,
    abs_server_ca_path: &String,
) -> serde_json::Value {
    let dc = db.db.server().c_dc(server);
    let region = db.db.datacenter().c_region(dc);
    let region_secrets = secrets.region_secrets.get(&region).unwrap();
    let region_name = db.db.region().c_region_name(region);
    let is_server = db.db.server().c_is_consul_master(server);
    let server_consul_ip = db
        .db
        .network_interface()
        .c_if_ip(*db.projections.consul_network_iface.value(server));

    use serde_json::*;

    let mut retry_join: Vec<Value> = Vec::new();
    let consul_masters = db.sync_res.network.consul_masters.get(&region).unwrap();
    for s in consul_masters {
        assert!(db.db.server().c_is_consul_master(*s));
        let s_consul_ip = db
            .db
            .network_interface()
            .c_if_ip(*db.projections.consul_network_iface.value(*s));
        retry_join.push(Value::String(s_consul_ip.clone()));
    }
    let retry_join = Value::Array(retry_join);
    let mut retry_join_wan: Vec<Value> = Vec::new();
    for other_region in db.db.region().rows_iter() {
        if region != other_region {
            let other_consul_masters = db.sync_res.network.consul_masters.get(&other_region).unwrap();
            for s in other_consul_masters {
                let s_consul_ip = db
                    .db
                    .network_interface()
                    .c_if_ip(*db.projections.consul_network_iface.value(*s));
                retry_join_wan.push(Value::String(s_consul_ip.clone()));
            }
        }
    }
    let retry_join_wan = Value::Array(retry_join_wan);

    let mut all_args = json!({
        // patroni started failing in prod without this
        // couldn't deregister old service
        "limits": {
            "rpc_max_conns_per_client": 1000
        },
        "addresses": {
            "dns": "127.0.0.1",
            "grpc": "127.0.0.1",
            "http": "127.0.0.1",
            "https": server_consul_ip,
        },
        "advertise_addr": server_consul_ip,
        "advertise_addr_wan": server_consul_ip,
        "bind_addr": server_consul_ip,
        "client_addr": "127.0.0.1",
        "data_dir": "/var/lib/consul",
        // this is not a mistake, we use region
        // name as datacenter name because
        // because consul couldn't lookup dns names
        // like postgres.service.consul if they're in the
        // same region but different datacenter
        "datacenter": region_name,
        "disable_update_check": false,
        "domain": "consul",
        "enable_local_script_checks": false,
        "enable_script_checks": false,
        "encrypt": secrets.consul_encrypt_token.value(),
        "encrypt_verify_incoming": true,
        "encrypt_verify_outgoing": true,
        "log_level": "INFO",
        "log_rotate_bytes": 0u16,
        "log_rotate_duration": "24h",
        "log_rotate_max_files": 0u16,
        "node_name": db.db.server().c_hostname(server),
        "performance": {
          "leave_drain_time": "5s",
          "raft_multiplier": 1u16,
          "rpc_hold_timeout": "7s",
        },
        "ports": {
          "dns": 8600u16,
          "grpc": -1i16,
          "http": 8500u16,
          "https": 8501u16,
          "serf_lan": 8301u16,
          "serf_wan": 8302u16,
          "server": 8300u16,
        },
        "raft_protocol": 3u16,
        "retry_interval": "30s",
        "retry_join": retry_join,
        "retry_max": 0u16,
        "server": is_server,
        "translate_wan_addrs": false,
        "ui_config": {
            "enabled": true,
        },
    });

    if is_server {
        let quorum = db.sync_res.network.consul_masters.get(&region).unwrap();
        let paths = certs_path.unwrap();
        let server_args = json!({
            "connect": {
                "enabled": true,
            },
            "bootstrap": false,
            "bootstrap_expect": quorum.len(),
            "auto_encrypt": {
                "allow_tls": true,
            },
            "retry_join_wan": retry_join_wan,
            "tls": {
                "defaults": {
                    "cert_file": paths.abs_server_cert_path,
                    "key_file": paths.abs_server_pkey_path,
                    "ca_file": abs_server_ca_path,
                    "verify_incoming": false,
                    "verify_outgoing": true,
                    "tls_min_version": "TLSv1_2",
                },
                "internal_rpc": {
                    "verify_incoming": false,
                    "verify_server_hostname": true,
                },
                "https": {
                    "verify_incoming": false,
                },
            },
            "acl": {
                "default_policy": "deny",
                "enable_token_persistence": true,
                "enabled": true,
                "tokens": {
                    "agent": region_secrets.consul_agent_acl_token.value(),
                    // TODO: in later consul versions split default token dns to
                    // separate dns oken
                    "default": secrets.consul_dns_token.value(),
                    // "dns": secrets.consul_dns_token.value(),
                    "initial_management": region_secrets.consul_initial_management_acl_token.value(),
                },
            },
        });

        merge_json(&mut all_args, &server_args);
    } else {
        let client_args = json!({
            "auto_encrypt": {
                "tls": true,
            },
            "tls": {
                "defaults": {
                    "ca_file": abs_server_ca_path,
                    "verify_incoming": false,
                    "verify_outgoing": true,
                    "tls_min_version": "TLSv1_2",
                },
                "internal_rpc": {
                    "verify_incoming": false,
                    "verify_server_hostname": true,
                },
                "https": {
                    "verify_incoming": false,
                },
            },
            "acl": {
                "default_policy": "deny",
                "enable_token_persistence": true,
                "enabled": true,
                "tokens": {
                    "agent": region_secrets.consul_agent_acl_token.value(),
                    "default": secrets.consul_dns_token.value(),
                },
            },
        });

        merge_json(&mut all_args, &client_args);
    }

    all_args
}

fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                assert!(a.insert(k.clone(), v.clone()).is_none());
            }
        }
        _ => {
            panic!("Oy vey...")
        }
    }
}

struct ConsulServerSecret {
    tls_private_key: SecretValue,
    tls_certificate: SecretValue,
}

pub struct ConsulRegionSecrets {
    consul_initial_management_acl_token: SecretValue,
    consul_agent_acl_token: SecretValue,
    consul_fast_l1_acl_token: SecretValue,
    pub consul_vrrp_acl_tokens: BTreeMap<TableRowPointerDatacenter, SecretValue>,
}

pub struct ConsulSecrets {
    pub region_secrets: BTreeMap<TableRowPointerRegion, ConsulRegionSecrets>,
    consul_encrypt_token: SecretValue,
    consul_dns_token: SecretValue,
    tls_ca_certificate: SecretValue,
    server_keys: HashMap<TableRowPointerServer, ConsulServerSecret>,
}

pub fn derive_consul_secrets(db: &CheckedDB, secrets: &mut SecretsStorage) -> ConsulSecrets {
    let consul_encrypt_token = secrets.fetch_secret(
        "global_consul_encrypt_token".to_string(),
        SecretKind::ConsulEncryptionKey,
    );
    // TODO: in newer consul version there is separate
    // consul dns slot for token
    let consul_dns_token = secrets.fetch_secret(
        "global_consul_dns_token".to_string(),
        SecretKind::Guid,
    );

    let consul_ca_sec_files = sec_files(&[
        (
            SecretKind::TlsPrivateKey,
            "global_consul_tls_ca_private_key",
            "consul-agent-ca-key.pem",
        ),
        (
            SecretKind::TlsCertificate,
            "global_consul_tls_ca_certificate",
            "consul-agent-ca.pem",
        ),
    ]);
    let mut consul_ca_secrets = secrets.multi_secret_derive(
        &[],
        sec_files(&[]),
        consul_ca_sec_files.clone(),
        r#"
            consul tls ca create -days=6205
        "#,
    );

    let tls_ca_certificate = consul_ca_secrets.pop().unwrap();

    let mut server_keys = HashMap::new();
    let mut region_secrets = BTreeMap::new();

    for region in db.db.region().rows_iter() {
        let mut has_servers = false;
        let region_name = db.db.region().c_region_name(region);
        for dc in db.db.region().c_referrers_datacenter__region(region) {
            for server in db.db.datacenter().c_referrers_server__dc(*dc) {
                has_servers = true;
                if db.db.server().c_is_consul_master(*server) {
                    let hostname = db.db.server().c_hostname(*server);
                    let tls_pkey_key = format!("consul_region_{region_name}_server_{hostname}_consul_tls_private_key");
                    let tls_cert_key = format!("consul_region_{region_name}_server_{hostname}_consul_tls_certificate");

                    if let Some(curr_cert) = secrets.get_secret(&tls_cert_key) {
                        if tls_cert_expiration_days(&curr_cert) < 14 {
                            println!("Renewing TLS cert for key {tls_cert_key}");
                            secrets.delete_secret(&tls_pkey_key);
                            secrets.delete_secret(&tls_cert_key);
                        }
                    }

                    let mut keys = secrets.multi_secret_derive(
                        &[],
                        consul_ca_sec_files.clone(),
                        sec_files(&[
                            (
                                SecretKind::TlsPrivateKey,
                                &tls_pkey_key,
                                &format!("{region_name}-server-consul-0-key.pem"),
                            ),
                            (
                                SecretKind::TlsCertificate,
                                &tls_cert_key,
                                &format!("{region_name}-server-consul-0.pem"),
                            ),
                        ]),
                        &format!(
                            r#"
                                consul tls cert create -server -dc {region_name} -node {hostname}
                            "#
                        ),
                    );

                    let tls_certificate = keys.pop().unwrap();
                    let tls_private_key = keys.pop().unwrap();

                    let r = server_keys.insert(
                        *server,
                        ConsulServerSecret {
                            tls_private_key,
                            tls_certificate,
                        },
                    );
                    assert!(r.is_none());
                }
            }
        }

        if has_servers {
            let consul_agent_acl_token = secrets.fetch_secret(
                format!("consul_region_{region_name}_acl_agent_token"),
                SecretKind::Guid,
            );
            let consul_fast_l1_acl_token = secrets.fetch_secret(
                format!("consul_region_{region_name}_acl_fast_l1_token"),
                SecretKind::Guid,
            );
            let consul_initial_management_acl_token = secrets.fetch_secret(
                format!("consul_region_{region_name}_acl_management_token"),
                SecretKind::Guid,
            );

            let mut consul_vrrp_acl_tokens = BTreeMap::new();

            for dc in db.db.region().c_referrers_datacenter__region(region) {
                let dc_name = db.db.datacenter().c_dc_name(*dc);
                assert!(consul_vrrp_acl_tokens.insert(
                    *dc,
                    secrets.fetch_secret(
                        format!("consul_region_{region_name}_dc_{dc_name}_acl_vrrp_token"),
                        SecretKind::Guid
                    )
                ).is_none());
            }

            assert!(region_secrets.insert(region, ConsulRegionSecrets {
                consul_initial_management_acl_token,
                consul_agent_acl_token,
                consul_fast_l1_acl_token,
                consul_vrrp_acl_tokens,
            }).is_none());
        }
    }

    ConsulSecrets {
        consul_encrypt_token,
        consul_dns_token,
        tls_ca_certificate,
        server_keys,
        region_secrets,
    }
}

fn consul_token_policies_bootstrap_script() -> &'static str {
    r#"
            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^agent-policy:$' &>/dev/null && exit 0

            cat > /tmp/epl-consul-agent-policy.hcl<<EOL
            node_prefix "" {
                policy = "write"
            }
            service_prefix "" {
                policy = "write"
            }
            EOL

            cat > /tmp/epl-consul-default-policy.hcl<<EOL
            # allow prometheus target scrapes
            agent_prefix "" {
                policy = "read"
            }
            node_prefix "" {
                policy = "read"
            }
            service_prefix "" {
                policy = "read"
            }
            # For DNS policiy, remove in the future when
            # we separate default token from DNS token
            query_prefix "" {
                policy = "read"
            }

            # inter DC routing, allow every node to access routes
            key_prefix "epl-interdc-routes/" {
                policy = "list"
            }

            # all l1 provisioning plans are sodium encrypted doesnt matter
            # if anyone reads, only intended node can decrypt
            key_prefix "epl-l1-plans/" {
                policy = "list"
            }
            EOL

            cat > /tmp/epl-consul-fast-l1-admin-policy.hcl<<EOL
            # allow plans upload for every server
            key_prefix "epl-l1-plans/" {
                policy = "write"
            }
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "agent-policy" \
                -description "Agent Token Policy" \
                -rules @/tmp/epl-consul-agent-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Agent Token" \
                -policy-name "agent-policy" \
                -secret=$( sudo cat /run/keys/consul-agent-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "default-token" \
                -description "Default Token Policy" \
                -rules @/tmp/epl-consul-default-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Default Token" \
                -policy-name "default-token" \
                -secret=$( sudo cat /run/keys/consul-default-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "fast-l1-token" \
                -description "Fast L1 Admin Policy" \
                -rules @/tmp/epl-consul-fast-l1-admin-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Fast L1 Admin" \
                -policy-name "fast-l1-token" \
                -secret=$( sudo cat /run/keys/consul-fast-l1-token.txt )
"#
}

fn consul_vrrp_token_policy_bootstrap_script(db: &CheckedDB, region: TableRowPointerRegion) -> String {
    let mut res = String::new();
    let nix_region = "consul_vrrp_bootstrap_script";
    write_nix_region_start(nix_region, &mut res);
    write!(&mut res, r#"
export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

while :
do
    consul members | grep alive &>/dev/null && break
    sleep 1
done
"#).unwrap();

    for dc in db.db.region().c_referrers_datacenter__region(region) {
        let dc_name = db.db.datacenter().c_dc_name(*dc);
        let dc_net = db.sync_res.network.networking_answers.dcs.get(&dc).unwrap();
        if dc_net.is_consul_vrrp {
            let inter_dc_routes = inter_dc_routes(db, *dc);
            let dc_route_key = format!("epl-interdc-routes/{dc_name}");

            write!(&mut res, r#"
if ! ${{pkgs.consul}}/bin/consul acl policy list | grep '^vrrp-policy-{dc_name}:$'
then
    cat > /tmp/epl-consul-vrrp-{dc_name}-policy.hcl<<EOL
    key_prefix "{dc_route_key}" {{
        policy = "write"
    }}
EOL

    ${{pkgs.consul}}/bin/consul acl policy create \
        -name "vrrp-policy-{dc_name}" \
        -description "VRRP policy for datacenter {dc_name}" \
        -rules @/tmp/epl-consul-vrrp-{dc_name}-policy.hcl

    ${{pkgs.consul}}/bin/consul acl token create \
        -description "VRRP Token for datacenter {dc_name}" \
        -policy-name "vrrp-policy-{dc_name}" \
        -secret=$( sudo cat /run/keys/consul-vrrp-token-{dc_name}.txt )
fi

"#).unwrap();

            if dc_net.params.use_l3_hop_for_vpn_gateways {
                let route_key = format!("{dc_route_key}/l3_vpn_hop");
                assert_eq!(inter_dc_routes.len(), 1);
                for routes in inter_dc_routes.values() {
                    if !routes.is_empty() {
                        let first_route = routes[0].route_script();
                        let script = pad_string(&first_route, "            ");

                        write!(&mut res, r#"
${{pkgs.consul}}/bin/consul kv get {route_key} || echo '{script}
' | ${{pkgs.consul}}/bin/consul kv put {route_key} -
"#).unwrap();
                    };
                }
            } else {
                for (subnet, routes) in inter_dc_routes {
                    let subnet = subnet.to_string().replace("/", "p");
                    let route_key = format!("{dc_route_key}/{subnet}");
                    if !routes.is_empty() {
                        let first_route = routes[0].route_script();
                        let script = pad_string(&first_route, "            ");

                        write!(&mut res, r#"
${{pkgs.consul}}/bin/consul kv get {route_key} || echo '{script}
' | ${{pkgs.consul}}/bin/consul kv put {route_key} -
"#).unwrap();
                    };
                }
            }

            write!(&mut res, r#"

# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-{dc_name}.txt

"#).unwrap();
        }
    }

    write_nix_region_end(nix_region, &mut res);

    res
}
