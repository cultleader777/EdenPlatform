use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use convert_case::Casing;

use crate::{
    codegen::{
        nixplan::{self, NixAllServerPlans, NomadHostVolume, ZfsDataset},
        secrets::{sec_files, SecretKind, SecretValue, SecretsStorage, tls_cert_expiration_days},
    },
    database::{TableRowPointerServer, TableRowPointerRegion, Database},
    static_analysis::{CheckedDB, L1Projections, server_runtime::ServerRuntime, networking::{first_region_server, prometheus_metric_exists_test, admin_service_responds_test}},
};

use super::utils::gen_reload_service_on_path_change;

pub(crate) fn provision_nomad(
    db: &CheckedDB,
    plans: &mut NixAllServerPlans,
    secrets: &mut SecretsStorage,
) {
    for server in db.db.server().rows_iter() {
        let dc = db.db.server().c_dc(server);
        let region = db.db.datacenter().c_region(dc);
        let plan = plans.fetch_plan(server);
        plan.add_nix_package("nomad");

        // register nomad volumes from db
        for v in db.db.server().c_children_server_volume(server) {
            let vname = db.db.server_volume().c_volume_name(*v);
            let mountpoint = db.db.server_volume().c_mountpoint(*v);
            let read_only = matches!(
                db.db
                    .server_volume_usage_contract()
                    .c_usage_contract(db.db.server_volume().c_intended_usage(*v))
                    .as_str(),
                "read_only"
            );
            plan.add_nomad_host_volume(vname.as_str(), mountpoint.as_str(), read_only);
        }

        let nomad_secrets = derive_nomad_secrets(db, secrets);
        let region_secrets = nomad_secrets.region_secrets.get(&region).unwrap();
        let config = generate_nomad_config(db, server, &nomad_secrets, plan.nomad_host_volumes());

        let sec_cfg = plan.add_secret_config(nixplan::root_secret_config(
            "nomad-config.hcl".to_string(),
            config,
        ));
        let cfg_path = sec_cfg.absolute_path();

        let is_server = db.db.server().c_is_nomad_master(server);

        plan.add_env_variable(
            "NOMAD_ADDR".to_string(),
            "https://nomad-servers.service.consul:4646".to_string(),
        );

        if db.db.server().c_is_consul_master(server) {
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-server-consul-acl-token.txt".to_string(),
                region_secrets.server_consul_acl_token.clone(),
            ));
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-client-consul-acl-token.txt".to_string(),
                region_secrets.client_consul_acl_token.clone(),
            ));
            plan.add_shell_package(
                "epl-nomad-consul-acl-bootstrap",
                consul_nomad_token_bootstrap_script(),
            );
        }

        plan.add_secret(nixplan::root_secret_key(
            "nomad-ca.crt".to_string(),
            nomad_secrets.tls_ca_certificate.clone(),
        ))
        .absolute_path();
        if is_server {
            plan.add_zfs_dataset("nomad".to_string(), ZfsDataset {
                compression_enabled: true,
                encryption_enabled: true,
                expose_to_containers: false,
                mountpoint: "/var/lib/nomad".to_string(),
                record_size: "4k".to_string(),
                zpool: "rpool".to_string(),
            });
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-server.crt".to_string(),
                region_secrets.server_tls_certificate.clone(),
            ));
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-server.key".to_string(),
                region_secrets.server_tls_key.clone(),
            ));
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-cli.crt".to_string(),
                nomad_secrets.cli_tls_certificate.clone(),
            ));
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-cli.key".to_string(),
                nomad_secrets.cli_tls_key.clone(),
            ));
        } else {
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-client.crt".to_string(),
                region_secrets.client_tls_certificate.clone(),
            ));
            let _ = plan.add_secret(nixplan::root_secret_key(
                "nomad-client.key".to_string(),
                region_secrets.client_tls_key.clone(),
            ));
        }

        plan.add_ca_cert_file(nomad_secrets.tls_ca_certificate.value().clone());

        plan.add_custom_nix_block(format!(
            r#"
    systemd.services.nomad = {{
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];
      path = [ pkgs.iproute2 ];

      serviceConfig = {{
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStartPre = [
            "+${{pkgs.coreutils}}/bin/mkdir -p /var/lib/nomad"
            "+${{pkgs.coreutils}}/bin/chmod 700 /var/lib/nomad"
        ];
        ExecStart = "${{pkgs.nomad}}/bin/nomad agent -config={cfg_path}";
        ExecReload = "/bin/kill -HUP $MAINPID";
        KillMode = "process";
        KillSignal = "SIGINT";
        LimitNOFILE = "infinity";
        LimitNPROC = "infinity";
        Restart = "always";
        RestartSec = "20";
        TasksMax = "infinity";
      }};

      enable = true;
    }};
"#
        ));

        let mut rld_cfg = String::new();
        gen_reload_service_on_path_change(
            &mut rld_cfg,
            cfg_path.as_str(),
            "nomad-restart",
            "nomad.service",
            true,
        );

        plan.add_custom_nix_block(rld_cfg);

        plan.add_shell_package(
            "epl-nomad-acl-bootstrap",
            r#"
            while ! curl -f -s https://nomad-servers.service.consul:4646 &>/dev/null
            do
                sleep 1
            done

            while [ "$( dig +short nomad-servers.service.consul | wc -l )" -lt 3 ]
            do
                sleep 1
            done

            while true
            do
              nomad acl bootstrap &> /run/secdir/nomad-bootstrap-output.txt.tmp
              if cat /run/secdir/nomad-bootstrap-output.txt.tmp | grep 'No cluster leader'
              then
                sleep 2
                continue
              fi

              if cat /run/secdir/nomad-bootstrap-output.txt.tmp | grep 'Secret ID'
              then
                mv -f /run/secdir/nomad-bootstrap-output.txt.tmp /run/secdir/nomad-bootstrap-output.txt
              fi

              break
            done
"#,
        );

        let nomad_prov = r#"

            if [ -z "$NOMAD_TOKEN" ]
            then
                echo Must set NOMAD_TOKEN for this script
                exit 7
            fi

            while ! curl -f -s https://nomad-servers.service.consul:4646 &>/dev/null
            do
                sleep 1
            done

            cat > /tmp/epl-nomad-anonymous-policy.hcl<<EOL
              namespace "*" {
                policy       = "read"
                capabilities = [
                  "list-jobs"
                ]
              }

              agent {
                policy = "read"
              }

              operator {
                policy = "read"
              }

              quota {
                policy = "read"
              }

              node {
                policy = "read"
              }

              host_volume "*" {
                policy = "read"
              }
            EOL

            nomad acl policy apply -description "Anonymous policy" anonymous /tmp/epl-nomad-anonymous-policy.hcl
"#;

        plan.add_shell_package("epl-nomad-acl-policies", nomad_prov);

        plan.add_shell_package("epl-nomad-vault-policies", r#"
            if [ -z "$VAULT_TOKEN" ]
            then
                echo Must set VAULT_TOKEN for this script
                exit 7
            fi

            while ! curl -f -s https://vault.service.consul:8200 &>/dev/null
            do
                sleep 1
            done

            cat > /tmp/epl-nomad-vault-token-policy.json<<EOL
            {
                "disallowed_policies": "nomad-server",
                "token_explicit_max_ttl": 0,
                "name": "nomad-cluster",
                "orphan": true,
                "token_period": 259200,
                "renewable": true
            }
            EOL

            cat > /tmp/epl-nomad-vault-policy.hcl<<EOL
            # Allow creating tokens under "nomad-cluster" token role. The token role name
            # should be updated if "nomad-cluster" is not used.
            path "auth/token/create/nomad-cluster" {
                capabilities = ["update"]
            }

            # Allow looking up "nomad-cluster" token role. The token role name should be
            # updated if "nomad-cluster" is not used.
            path "auth/token/roles/nomad-cluster" {
                capabilities = ["read"]
            }

            # Allow looking up the token passed to Nomad to validate # the token has the
            # proper capabilities. This is provided by the "default" policy.
            path "auth/token/lookup-self" {
                capabilities = ["read"]
            }

            # Allow looking up incoming tokens to validate they have permissions to access
            # the tokens they are requesting. This is only required if
            # allow_unauthenticated is set to false.
            path "auth/token/lookup" {
                capabilities = ["update"]
            }

            # Allow revoking tokens that should no longer exist. This allows revoking
            # tokens for dead tasks.
            path "auth/token/revoke-accessor" {
                capabilities = ["update"]
            }

            # Allow checking the capabilities of our own token. This is used to validate the
            # token upon startup.
            path "sys/capabilities-self" {
                capabilities = ["update"]
            }

            # Allow our own token to be renewed.
            path "auth/token/renew-self" {
                capabilities = ["update"]
            }
            EOL

            vault policy write nomad-server /tmp/epl-nomad-vault-policy.hcl
            vault write /auth/token/roles/nomad-cluster @/tmp/epl-nomad-vault-token-policy.json

            ORIGINAL_TOKEN=$VAULT_TOKEN
            export VAULT_TOKEN=$1
            if ! vault token lookup
            then
                # token invalid, needs to be created
                export VAULT_TOKEN=$ORIGINAL_TOKEN
                NEW_TOKEN=$( vault token create -policy nomad-server -period 72h -orphan | grep 'hvs.' | sed -E 's/^.* hvs/hvs/' )
                echo "NOMAD_VAULT_TOKEN $NEW_TOKEN"
            fi
"#);
    }
}

pub fn provisioning_nomad_namespaces_script(db: &Database) -> String {
    let mut res = r#"#!/bin/sh
while ! nomad status &> /dev/null
do
  echo Nomad not ready yet, sleeping...
  sleep 3
done
"#.to_string();
    for ns in db.nomad_namespace().rows_iter() {
        let ns_name = db.nomad_namespace().c_namespace(ns);
        let ns_desc = db.nomad_namespace().c_description(ns);
        writeln!(&mut res, r#"nomad namespace apply -description "{ns_desc}" {ns_name}"#).unwrap();
    }
    res
}

pub fn nomad_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut nomad_master_ips: HashMap<TableRowPointerRegion, (String, Vec<String>)> = HashMap::new();
    // test main consul services for region
    for r in db.region().rows_iter() {
        let reg_name = db.region().c_region_name(r);
        let reg_name_snake = reg_name.to_case(convert_case::Case::Snake);
        let mut master_ips_for_region: Vec<String> = Vec::new();
        let mut client_ips_for_region: Vec<String> = Vec::new();
        let fr_server = first_region_server(db, r);
        if let Some(fr_server) = fr_server {
            let fr_net_iface = l1proj.consul_network_iface.value(fr_server);
            let fr_ip = format!("{}:53", db.network_interface().c_if_ip(*fr_net_iface));
            for dc in db.region().c_referrers_datacenter__region(r) {
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    let iface = l1proj.consul_network_iface.value(*server);
                    let ip = db.network_interface().c_if_ip(*iface);
                    client_ips_for_region.push(ip.clone());
                    if db.server().c_is_nomad_master(*server) {
                        master_ips_for_region.push(ip.clone());
                    }
                }
            }

            runtime.add_integration_test(
                format!("nomad_servers_available_region_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                    target_servers: vec![fr_ip.clone()],
                    queries: vec![
                        ("nomad-servers.service.consul".to_string(), master_ips_for_region.clone())
                    ]
                },
            );

            // if few enough clients all should appear in consul dns
            if client_ips_for_region.len() <= 7 {
                runtime.add_integration_test(
                    format!("nomad_clients_available_region_{reg_name_snake}"),
                    crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                        target_servers: vec![fr_ip.clone()],
                        queries: vec![
                            ("nomad-clients.service.consul".to_string(), client_ips_for_region.clone())
                        ]
                    },
                );
            }

            runtime.add_integration_test(
                format!("nomad_ui_responds_{reg_name_snake}"),
                crate::static_analysis::server_runtime::IntegrationTest::HttpGetRespondsString {
                    hostname: None,
                    expected_string: "<title>Nomad</title>".to_string(),
                    http_server_port: 4646,
                    is_https: true,
                    path: "/ui/".to_string(),
                    server_ips: master_ips_for_region.clone(),
                    use_admin_panel_credentials: Some(crate::static_analysis::server_runtime::IntegrationTestCredentials::AdminPanel),
                },
            );

            runtime.add_integration_test(
                format!("nomad_ui_external_responds_{reg_name_snake}"),
                admin_service_responds_test(
                    db,
                    l1proj,
                    format!("adm-nomad-{reg_name}"),
                    "/",
                    "<title>Nomad</title>"
                )
            );

            if let Some(reg_mon_cluster) = l1proj.monitoring_clusters.region_default(r) {
                runtime.add_integration_test(
                    format!("nomad_region_{reg_name_snake}_prometheus_metrics_gathered"),
                    prometheus_metric_exists_test(db, l1proj, reg_mon_cluster, "nomad_client_allocs_cpu_allocated")
                );
            }

            assert!(nomad_master_ips.insert(r, (fr_ip.clone(), master_ips_for_region)).is_none());
        }
    }

    // test cross region consul services
    for source_region in db.region().rows_iter() {
        let src_reg_name = db.region().c_region_name(source_region);
        let src_reg_name_snake = src_reg_name.to_case(convert_case::Case::Snake);
        if let Some((src_first, _)) = nomad_master_ips.get(&source_region) {
            for target_region in db.region().rows_iter() {
                let tg_reg_name = db.region().c_region_name(target_region);
                let tg_reg_name_snake = tg_reg_name.to_case(convert_case::Case::Snake);
                let (_, tg_ips) = nomad_master_ips.get(&target_region).unwrap();

                runtime.add_integration_test(
                    format!("nomad_servers_available_from_region_{src_reg_name_snake}_to_region_{tg_reg_name_snake}"),
                    crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                        target_servers: vec![src_first.clone()],
                        queries: vec![
                            (format!("nomad-servers.service.{tg_reg_name}.consul"), tg_ips.clone())
                        ]
                    },
                );
            }
        }
    }
}

fn generate_nomad_config(
    db: &CheckedDB,
    server: TableRowPointerServer,
    secrets: &NomadSecrets,
    host_volumes: &BTreeMap<String, NomadHostVolume>,
) -> String {
    let mut res = String::new();
    let dc = db.db.server().c_dc(server);
    let dc_name = db.db.datacenter().c_dc_name(dc);
    let region = db.db.datacenter().c_region(dc);
    let disable_log_collection = db.db.region().c_nomad_disable_log_collection(region);
    let region_name = db.db.region().c_region_name(region);
    let region_secrets = secrets.region_secrets.get(&region).unwrap();

    let nomad_ip = db
        .db
        .network_interface()
        .c_if_ip(*db.projections.consul_network_iface.value(server));
    let is_server = db.db.server().c_is_nomad_master(server);

    let raft_protocol_version = "3";

    let hostname = db.db.server().c_hostname(server);
    let nomad_consul_acl_token = if is_server {
        region_secrets.server_consul_acl_token.value()
    } else {
        region_secrets.client_consul_acl_token.value()
    };
    let encryption_key = secrets.nomad_encryption_key.value();
    let nomad_masters = db.sync_res.network.nomad_masters.get(&region).unwrap();
    let quorum_count = nomad_masters.len();

    res += &format!(
        r#"

name = "{hostname}"
region = "{region_name}"
datacenter = "{dc_name}"

enable_debug = false
disable_update_check = false


bind_addr = "{nomad_ip}"
advertise {{
    http = "{nomad_ip}:4646"
    rpc = "{nomad_ip}:4647"
    serf = "{nomad_ip}:4648"
}}

ports {{
    http = 4646
    rpc = 4647
    serf = 4648
}}

consul {{
    # The address to the Consul agent.
    address = "127.0.0.1:8500"
    ssl = false
    ca_file = ""
    cert_file = ""
    key_file = ""
    token = "{nomad_consul_acl_token}"
    # The service name to register the server and client with Consul.
    server_service_name = "nomad-servers"
    client_service_name = "nomad-clients"
    tags = {{
    }}

    # Enables automatically registering the services.
    auto_advertise = true

    # Enabling the server and client to bootstrap using Consul.
    server_auto_join = true
    client_auto_join = true
}}

data_dir = "/var/lib/nomad"

log_level = "INFO"
enable_syslog = true

leave_on_terminate = true
leave_on_interrupt = false

"#
    );

    if is_server {
        res += r#"
tls {
    http = true
    rpc = true
    ca_file = "/run/keys/nomad-ca.crt"
    cert_file = "/run/keys/nomad-server.crt"
    key_file = "/run/keys/nomad-server.key"
    rpc_upgrade_mode = false
    verify_server_hostname = "true"
    verify_https_client = "false"
}
"#;
    } else {
        res += r#"
tls {
    http = true
    rpc = true
    ca_file = "/run/keys/nomad-ca.crt"
    cert_file = "/run/keys/nomad-client.crt"
    key_file = "/run/keys/nomad-client.key"
    rpc_upgrade_mode = false
    verify_server_hostname = "true"
    verify_https_client = "false"
}
"#;
    }

    for global_nomad_vault_token in &region_secrets.nomad_vault_token {
        let maybe_token = if is_server {
            format!(
                "    token = \"{global_nomad_vault_token}\"
"
            )
        } else {
            "".to_string()
        };
        res += &format!(
            r#"

vault {{
    enabled = true
    address = "https://vault.service.consul:8200"
    allow_unauthenticated = false
    create_from_role = "nomad-cluster"
    task_token_ttl = ""
    ca_file = "/run/keys/vault-ca.crt"
    ca_path = ""
    cert_file = ""
    key_file = ""
    tls_server_name = ""
    tls_skip_verify = false
    namespace = ""
{maybe_token}
}}
"#
        );
    }

    res += r#"
client {
  enabled = true

  node_class = ""
  no_host_uuid = false

  max_kill_timeout = "3600s"

  network_speed = 0
  cpu_total_compute = 0

  gc_interval = "1m"
  gc_disk_usage_threshold = 80
  gc_inode_usage_threshold = 70
  gc_parallel_destroys = 2

  reserved {
    cpu = 0
    memory = 0
    disk = 0
  }

  network_interface = ""#;

    let consul_iface = db.projections.consul_network_iface.value(server);
    let private_ip = db.db.network_interface().c_if_ip(*consul_iface);

    if db.db.network_interface().c_if_vlan(*consul_iface) < 0 {
        res += db.db.network_interface().c_if_name(*consul_iface);
    } else {
        write!(&mut res, "vlan{}", db.db.network_interface().c_if_vlan(*consul_iface)).unwrap();
    }

    res += r#""

  meta = {
"#;

    write!(&mut res, "    \"private_ip\" = \"{private_ip}\"\n").unwrap();

    if db.db.server().c_run_unassigned_workloads(server) {
        res += "    \"run_unassigned_workloads\" = \"1\"\n";
    }

    for (lbl, srv) in db.projections.server_runtime.server_label_locks() {
        if srv.contains(&server) {
            res += "    \"lock_";
            res += lbl.label_name();
            res += "\" = \"1\"\n";
        }
    }

    for label in db.db.server().c_children_server_label(server) {
        let label_key = db.db.server_label().c_label_name(*label);
        let label_name = db.db.valid_server_labels().c_label_name(label_key);
        let value = db.db.server_label().c_label_value(*label);
        res += "    \"label_";
        res += label_name;
        res += "\" = \"";
        res += value;
        res += "\"\n";
    }

    res += r#"  }
"#;

    for vol in host_volumes.values() {
        let mountpoint = &vol.mountpoint;
        let name = &vol.name;
        let read_only = &vol.read_only;
        res += &format!(
            r#"
  host_volume "{name}" {{
    path = "{mountpoint}"
    read_only = {read_only}
  }}
"#
        );
    }

    for net in db.db.network().rows_iter() {
        if db.db.network().c_network_name(net) == "lan" {
            let cidr = db.db.network().c_cidr(net);
            res += &format!(
                r#"
  host_network "lan" {{
    cidr = "{cidr}"
  }}
"#
            );
        }
    }

    res += r#"
}
"#;

    res += r#"
acl {
    enabled = true
    token_ttl = "30s"
    policy_ttl = "30s"
    replication_token = ""
}

telemetry {
    disable_hostname = "false"
    collection_interval = "1s"
    use_node_name = "false"
    publish_allocation_metrics = "true"
    publish_node_metrics = "true"
    filter_default = "true"
    prefix_filter = []
    disable_dispatched_job_summary_metrics = "false"
    statsite_address = ""
    statsd_address = ""
    datadog_address = ""
    datadog_tags = []
    prometheus_metrics = "true"
    circonus_api_token = ""
    circonus_api_app = "nomad"
    circonus_api_url = "https://api.circonus.com/v2"
    circonus_submission_interval = "10s"
    circonus_submission_url = ""
    circonus_check_id = ""
    circonus_check_force_metric_activation = "false"
    circonus_check_instance_id = ""
    circonus_check_search_tag = ""
    circonus_check_display_name = ""
    circonus_check_tags = ""
    circonus_broker_id = ""
    circonus_broker_select_tag = ""
}

plugin "docker" {
    config {
"#;

    write!(&mut res, r#"
        # this might use a lot of memory if enabled, we have grafana loki
        # and this can be disabled for memory savings, but if grafana loki doesn't
        # work this is nice for debugging
        disable_log_collection = {disable_log_collection}
"#).unwrap();

    res += r#"
        extra_labels = ["*"]
        logging {
            type = "json-file"
            config {
                max-file = 3
                max-size = "30m"
            }
        }
    }
}
"#;

    if is_server {
        res += &format!(
            r#"
server {{
    enabled = true

    bootstrap_expect = {quorum_count}

    rejoin_after_leave = false

    enabled_schedulers = ["service", "batch", "system"]
    num_schedulers = 128

    node_gc_threshold = "24h"
    eval_gc_threshold = "1h"
    job_gc_threshold = "4h"
    deployment_gc_threshold = "1h"

    encrypt = "{encryption_key}"

    raft_protocol = {raft_protocol_version}

    default_scheduler_config {{
        scheduler_algorithm             = "binpack"
        memory_oversubscription_enabled = true
        reject_job_registration         = false
        pause_eval_broker               = false

        preemption_config {{
            batch_scheduler_enabled    = true
            system_scheduler_enabled   = true
            service_scheduler_enabled  = true
            sysbatch_scheduler_enabled = true
        }}
    }}
}}
"#
        );
    }

    res
}

struct NomadRegionSecrets {
    server_tls_key: SecretValue,
    server_tls_certificate: SecretValue,
    client_tls_key: SecretValue,
    client_tls_certificate: SecretValue,
    nomad_vault_token: Option<SecretValue>,
    #[allow(dead_code)]
    nomad_acl_bootstrap_token: SecretValue,
    server_consul_acl_token: SecretValue,
    client_consul_acl_token: SecretValue,
}

struct NomadSecrets {
    tls_ca_certificate: SecretValue,
    region_secrets: BTreeMap<TableRowPointerRegion, NomadRegionSecrets>,
    nomad_encryption_key: SecretValue,
    cli_tls_key: SecretValue,
    cli_tls_certificate: SecretValue,
}

fn derive_nomad_secrets(db: &CheckedDB, secrets: &mut SecretsStorage) -> NomadSecrets {
    let mut region_secrets = BTreeMap::new();

    let ca_sec_files = sec_files(&[
        (
            SecretKind::TlsPrivateKey,
            "global_nomad_tls_ca_private_key",
            "nomad-ca-key.pem",
        ),
        (
            SecretKind::TlsCertificate,
            "global_nomad_tls_ca_certificate",
            "nomad-ca.pem",
        ),
    ]);

    let mut ca_vec = secrets.multi_secret_derive(
        &[(
            "ca-conf.json",
            r#"
                {
                    "CN": "nomad",
                    "CA": {
                        "expiry": "148920h",
                        "pathlen": 0
                    },
                    "hosts": [
                        "nomad"
                    ],
                    "key": {
                        "algo": "ecdsa",
                        "size": 256
                    },
                    "names": []
                }
                "#,
        )],
        vec![],
        ca_sec_files.clone(),
        r#"
            cfssl gencert -initca ca-conf.json > ca-keys.json
            cat ca-keys.json | cfssljson -bare nomad-ca
        "#,
    );

    let tls_ca_certificate = ca_vec.pop().unwrap();

    let cli_tls_pkey_key = "global_nomad_tls_cli_private_key".to_string();
    let cli_tls_cert_key = "global_nomad_tls_cli_certificate".to_string();

    if let Some(curr_cert) = secrets.get_secret(&cli_tls_cert_key) {
        if tls_cert_expiration_days(&curr_cert) < 14 {
            println!("Renewing TLS cert for key {cli_tls_cert_key}");

            secrets.delete_secret(&cli_tls_pkey_key);
            secrets.delete_secret(&cli_tls_cert_key);
        }
    }

    let mut cli_vec = secrets.multi_secret_derive(
        &[
            (
                "cli-cfssl.json",
                r#"
                {
                    "signing": {
                        "default": {
                            "expiry": "8760h"
                        },
                        "profiles": {
                            "client": {
                                "expiry": "8760h",
                                "usages": ["signing", "key encipherment", "server auth", "client auth"]
                            }
                        }
                    }
                }
                "#
            )
        ],
        ca_sec_files.clone(),
        sec_files(&[
            (SecretKind::TlsPrivateKey, &cli_tls_pkey_key, "cli-key.pem"),
            (SecretKind::TlsCertificate, &cli_tls_cert_key, "cli.pem"),
        ]),
        r#"
          echo '{}' | \
            cfssl gencert -ca=nomad-ca.pem -ca-key=nomad-ca-key.pem -config=cli-cfssl.json -profile=client - | \
            cfssljson -bare cli
        "#,
    );

    let cli_tls_certificate = cli_vec.pop().unwrap();
    let cli_tls_key = cli_vec.pop().unwrap();

    let nomad_encryption_key = secrets.fetch_secret(
        format!("nomad_global_gossip_encryption_key"),
        SecretKind::NomadEncryptionKey,
    );

    for region in db.db.region().rows_iter() {
        if db.db.region().c_referrers_datacenter__region(region).is_empty() {
            // empty region
            continue;
        } else {
            let mut servers = 0;
            for dc in db.db.region().c_referrers_datacenter__region(region) {
                servers += db.db.datacenter().c_referrers_server__dc(*dc).len();
            }
            if servers == 0 {
                // region datacenters have no servers
                continue;
            }
        }

        let region_name = db.db.region().c_region_name(region);
        let server_consul_acl_token = secrets.fetch_secret(
            format!("nomad_region_{region_name}_consul_server_acl_token"),
            SecretKind::Guid,
        );
        let client_consul_acl_token = secrets.fetch_secret(
            format!("nomad_region_{region_name}_consul_client_acl_token"),
            SecretKind::Guid,
        );
        // just generate, doesn't end up in sources
        let nomad_acl_bootstrap_token = secrets.fetch_secret(
            format!("nomad_region_{region_name}_bootstrap_acl_token"),
            SecretKind::Guid,
        );
        let nomad_vault_token = secrets.get_secret(&format!("nomad_region_{region_name}_vault_token"));

        let server_tls_pkey_key = format!("nomad_region_{region_name}_tls_server_private_key");
        let server_tls_cert_key = format!("nomad_region_{region_name}_tls_server_certificate");

        if let Some(curr_cert) = secrets.get_secret(&server_tls_cert_key) {
            if tls_cert_expiration_days(&curr_cert) < 14 {
                println!("Renewing TLS cert for key {server_tls_cert_key}");

                secrets.delete_secret(&server_tls_pkey_key);
                secrets.delete_secret(&server_tls_cert_key);
            }
        }

        let mut server_vec = secrets.multi_secret_derive(
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
                    "#
                )
            ],
            ca_sec_files.clone(),
            sec_files(&[
                (SecretKind::TlsPrivateKey, &server_tls_pkey_key, "server-key.pem"),
                (SecretKind::TlsCertificate, &server_tls_cert_key, "server.pem"),
            ]),
            &format!(r#"
                echo '{{}}' | \
                    cfssl gencert -ca=nomad-ca.pem -ca-key=nomad-ca-key.pem -config=server-cfssl.json \
                        -hostname="server.{region_name}.nomad,nomad-servers.service.consul,localhost,127.0.0.1" - | \
                    cfssljson -bare server
            "#),
        );

        let server_tls_certificate = server_vec.pop().unwrap();
        let server_tls_key = server_vec.pop().unwrap();

        let client_tls_pkey_key = format!("nomad_region_{region_name}_tls_client_private_key");
        let client_tls_cert_key = format!("nomad_region_{region_name}_tls_client_certificate");

        if let Some(curr_cert) = secrets.get_secret(&client_tls_cert_key) {
            if tls_cert_expiration_days(&curr_cert) < 14 {
                println!("Renewing TLS cert for key {client_tls_cert_key}");

                secrets.delete_secret(&client_tls_pkey_key);
                secrets.delete_secret(&client_tls_cert_key);
            }
        }

        let mut client_vec = secrets.multi_secret_derive(
            &[(
                "client-cfssl.json",
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
            )],
            ca_sec_files.clone(),
            sec_files(&[
                (
                    SecretKind::TlsPrivateKey,
                    &client_tls_pkey_key,
                    "client-key.pem",
                ),
                (
                    SecretKind::TlsCertificate,
                    &client_tls_cert_key,
                    "client.pem",
                ),
            ]),
            &format!(r#"
                echo '{{}}' | \
                    cfssl gencert -ca=nomad-ca.pem -ca-key=nomad-ca-key.pem -config=client-cfssl.json \
                        -hostname="client.{region_name}.nomad,localhost,127.0.0.1" - | \
                    cfssljson -bare client
            "#),
        );

        let client_tls_certificate = client_vec.pop().unwrap();
        let client_tls_key = client_vec.pop().unwrap();

        assert!(region_secrets.insert(region, NomadRegionSecrets {
            server_tls_key,
            server_tls_certificate,
            client_tls_key,
            client_tls_certificate,
            nomad_acl_bootstrap_token,
            nomad_vault_token,
            server_consul_acl_token,
            client_consul_acl_token,
        }).is_none());
    }

    NomadSecrets {
        region_secrets,
        tls_ca_certificate,
        nomad_encryption_key,
        cli_tls_key,
        cli_tls_certificate,
    }
}

fn consul_nomad_token_bootstrap_script() -> &'static str {
    r#"
            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^nomad-server:$' &>/dev/null && exit 0

            cat > /tmp/epl-nomad-server-consul-acl-policy.hcl<<EOL
            agent_prefix "" {
                policy = "read"
            }

            node_prefix "" {
                policy = "read"
            }

            service_prefix "" {
                policy = "write"
            }

            # TODO: remove after nomad 1.9 and use consul identities instead
            key_prefix "epl-kv/" {
                policy = "read"
            }

            acl = "write"
            EOL

            cat > /tmp/epl-nomad-client-consul-acl-policy.hcl<<EOL
            agent_prefix "" {
                policy = "read"
            }

            node_prefix "" {
                policy = "read"
            }

            service_prefix "" {
                policy = "write"
            }

            acl = "write"
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "nomad-server" \
                -description "Nomad Server Policy" \
                -rules @/tmp/epl-nomad-server-consul-acl-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Nomad Server Token" \
                -policy-name "nomad-server" \
                -secret=$( sudo cat /run/keys/nomad-server-consul-acl-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "nomad-client" \
                -description "Nomad Client Policy" \
                -rules @/tmp/epl-nomad-client-consul-acl-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Nomad Client Token" \
                -policy-name "nomad-client" \
                -secret=$( sudo cat /run/keys/nomad-client-consul-acl-token.txt )
"#
}
