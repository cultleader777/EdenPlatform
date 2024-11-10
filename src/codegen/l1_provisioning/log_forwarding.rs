use super::utils::gen_reload_service_on_path_change;
use crate::{
    codegen::nixplan::{custom_user_secret_config, root_secret_config, NixAllServerPlans},
    static_analysis::CheckedDB, database::{TableRowPointerRegion, TableRowPointerLokiCluster},
};

pub(crate) fn provision_log_forwarding(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    for server in db.db.server().rows_iter() {
        let dc = db.db.server().c_dc(server);
        let region = db.db.datacenter().c_region(dc);
        let monitoring_cluster = db.projections.monitoring_clusters.region_default(region);
        let loki_cluster_ptr = db.projections.loki_clusters.region_default(region).unwrap();

        let plan = plans.fetch_plan(server);
        plan.add_nix_package("vector");
        let vector_iface = db.projections.consul_network_iface.value(server);
        let log_forwarding_ip = db.db.network_interface().c_if_ip(*vector_iface);
        let port = 9281;
        let fqdn = db.projections.server_fqdns.value(server);

        if let Some(monitoring_cluster) = &monitoring_cluster {
            let monitoring_cluster = db.db.monitoring_cluster().c_cluster_name(*monitoring_cluster);
            let sec_conf = plan.add_secret_config(root_secret_config(
            "epl-vector-service.hcl".to_string(),
            format!(
                r#"
service {{
  name = "epl-vector"
  id   = "epl-vector"
  port = {port}
  tags = ["epl-mon-{monitoring_cluster}"]

  tagged_addresses = {{
    lan = {{
      address = "{log_forwarding_ip}"
      port    = {port}
    }}
  }}

  meta = {{
    metrics_path = "/metrics"
  }}

  checks = [
    {{
        id       = "home"
        tcp      = "{log_forwarding_ip}:{port}"
        interval = "15s"
    }},
  ]
}}
"#
                ),
            ));

            let abs_service_path = sec_conf.absolute_path();
            plan.add_post_second_round_secrets_shell_hook(format!(
                r#"
# wait for consul to be up and running if restarted for up to 10 seconds
for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

export CONSUL_HTTP_TOKEN=$( cat /run/keys/consul-agent-token.txt )
for I in $(seq 1 5); do
  consul services register {abs_service_path} && break || true
  # try a few times if consul is down
  sleep 1
done
"#));
        }

        let sec_toml = plan.add_secret_config(custom_user_secret_config(
            "vector".to_string(),
            "vector.toml".to_string(),
            vector_config(
                db,
                region,
                fqdn,
                log_forwarding_ip,
                port,
                loki_cluster_ptr,
            ),
        ));

        let sec_toml_path = sec_toml.absolute_path();

        let mut rld_cfg = String::new();
        gen_reload_service_on_path_change(
            &mut rld_cfg,
            sec_toml_path.as_str(),
            "vector-restart",
            "vector.service",
            true,
        );

        plan.add_custom_nix_block(rld_cfg);
        plan.add_post_second_round_secrets_shell_hook(r#"
# create for vector
mkdir --mode 700 -p /var/lib/vector
chown vector:vector /var/lib/vector
"#.to_string());

        plan.add_custom_nix_block(format!(
            r#"
    users.users.vector = {{
        isSystemUser = true;
        description = "Vector service";
        extraGroups = ["keys" "systemd-journal" "docker" "epl-prov" ];
        group = "vector";
    }};
    users.groups.vector = {{}};

    systemd.services.vector = {{
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {{
        User = "vector";
        Group = "vector";
        Type = "simple";
        ExecStartPre = "${{pkgs.vector}}/bin/vector validate --config-toml={sec_toml_path}";
        ExecStart = "${{pkgs.vector}}/bin/vector --threads=4 --config-toml={sec_toml_path}";
        Restart = "always";
        RestartSec = "10";
      }};

      enable = true;
    }};
"#
        ));
    }
}

fn vector_config(
    db: &CheckedDB,
    region: TableRowPointerRegion,
    node_name: &str,
    node_ip: &str,
    prometheus_port: i64,
    default_loki_cluster: TableRowPointerLokiCluster,
) -> String {
    use std::fmt::Write;

    let default_loki_cluster_name = db.db.loki_cluster().c_cluster_name(default_loki_cluster);
    let default_loki_port = db.db.loki_cluster().c_loki_writer_http_port(default_loki_cluster);
    let mut res = String::new();
    write!(&mut res, r#"
# ----------------------------------
# prometheus metrics
# ----------------------------------
[sources.internal_metrics]
type = "internal_metrics"
scrape_interval_secs = 2

[sinks.prometheus_exporter_sink]
inputs = ["internal_metrics"]
type = "prometheus_exporter"
address = "{node_ip}:{prometheus_port}"
"#).unwrap();

    write!(
        &mut res,
        r#"
# ---------------------------------------------------------
# journald source
# ---------------------------------------------------------
[sources.journald]
type = "journald"
current_boot_only = true
exclude_units = [
  "dbus.service",
  "init.scope",
  "systemd-journald.service",
  "systemd-udevd.service",
]

# ----------------------------------
# docker source
# ----------------------------------
[sources.docker]
type = "docker_logs"

# ----------------------------------
# l1 provisioning sources
# ----------------------------------
[sources.l1_provisioning_logs]
type = "file"
include = [ "/var/log/epl-l1-prov/*.log" ]
read_from = "beginning"
remove_after_secs = 86400

[transforms.l1_provisioning_logs_extra]
type = "remap"
inputs = ["l1_provisioning_logs"]
source = """
segments = split!(.file, "/")
fname = split!(get!(segments, [-1]), ".")
.filename = get!(segments, [-1])
.provisioning_id = get!(fname, [-2])
"""

# ----------------------------------
# l2 provisioning sources
# ----------------------------------
[sources.l2_provisioning_logs]
type = "file"
include = [ "/var/log/epl-l2-prov/*/*.log" ]
read_from = "beginning"

[transforms.l2_provisioning_logs_extra]
type = "remap"
inputs = ["l2_provisioning_logs"]
source = """
segments = split!(.file, "/")
.filename = get!(segments, [-1])
.provisioning_id = get!(segments, [-2])
"""

# ----------------------------------
# loki journald sink
# ----------------------------------
[sinks.loki_journald]
type = "loki"
inputs = [ "journald" ]
endpoint = "http://epl-loki-{default_loki_cluster_name}-loki-writer.service.consul:{default_loki_port}"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_journald.buffer]
type = "disk"
max_size = 268435488
when_full = "block"

[sinks.loki_journald.labels]
source_type = "journald"
host = "{node_name}"
systemd_unit = "{{{{ _SYSTEMD_UNIT }}}}"

# ----------------------------------
# loki l1 provisioning sink
# ----------------------------------
[sinks.loki_l1_provisioning]
type = "loki"
inputs = [ "l1_provisioning_logs_extra" ]
endpoint = "http://epl-loki-{default_loki_cluster_name}-loki-writer.service.consul:{default_loki_port}"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l1_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l1_provisioning.labels]
source_type = "l1_provisioning"
host = "{node_name}"
file = "{{{{ filename }}}}"
provisioning_id = "{{{{ provisioning_id }}}}"

# ----------------------------------
# loki l2 provisioning sink
# ----------------------------------
[sinks.loki_l2_provisioning]
type = "loki"
inputs = [ "l2_provisioning_logs_extra" ]
endpoint = "http://epl-loki-{default_loki_cluster_name}-loki-writer.service.consul:{default_loki_port}"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l2_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l2_provisioning.labels]
source_type = "l2_provisioning"
host = "{node_name}"
file = "{{{{ filename }}}}"
provisioning_id = "{{{{ provisioning_id }}}}"
"#).unwrap();

    // generate routes
      write!(
                &mut res,
                r#"
# ----------------------------------
# loki nomad docker sink for {default_loki_cluster_name}
# ----------------------------------
[transforms.loki_nomad_docker_router]
type = "route"
inputs = [ "docker" ]
[transforms.loki_nomad_docker_router.route]
"#
        ).unwrap();

    for loki_cluster in db.db.region().c_referrers_loki_cluster__region(region) {
        let loki_cluster = db.db.loki_cluster().c_cluster_name(*loki_cluster);
        write!(&mut res, "{loki_cluster} = '.label.epl_loki_cluster == \"{loki_cluster}\"'\n").unwrap();
    }
    res += "\n";

    for this_loki_cluster in db.db.region().c_referrers_loki_cluster__region(region) {
        let loki_cluster = db.db.loki_cluster().c_cluster_name(*this_loki_cluster);
        let loki_port = db.db.loki_cluster().c_loki_writer_http_port(*this_loki_cluster);
        let mut inputs = vec![format!("\"loki_nomad_docker_router.{loki_cluster}\"")];
        if *this_loki_cluster == default_loki_cluster {
            inputs.push("\"loki_nomad_docker_router._unmatched\"".to_string());
        }
        let inputs = inputs.join(", ");
        write!(
                &mut res,
                r#"
# ----------------------------------
# loki nomad docker sink for {loki_cluster}
# ----------------------------------
[sinks.loki_nomad_docker_{loki_cluster}]
type = "loki"
inputs = [ {inputs} ]
endpoint = "http://epl-loki-{loki_cluster}-loki-writer.service.consul:{loki_port}"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_nomad_docker_{loki_cluster}.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_nomad_docker_{loki_cluster}.labels]
source_type = "nomad_docker"
host = "{node_name}"
namespace = "{{{{ label.\"com.hashicorp.nomad.namespace\" }}}}"
job_name = "{{{{ label.\"com.hashicorp.nomad.job_name\" }}}}"
task_group_name = "{{{{ label.\"com.hashicorp.nomad.task_group_name\" }}}}"
task_name = "{{{{ label.\"com.hashicorp.nomad.task_name\" }}}}"
alloc_id = "{{{{ label.\"com.hashicorp.nomad.alloc_id\" }}}}"
image = "{{{{ image }}}}"
"#
        ).unwrap();
    }

    res
}
