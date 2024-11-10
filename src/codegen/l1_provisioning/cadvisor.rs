use crate::{
    codegen::nixplan::{root_secret_config, NixAllServerPlans},
    static_analysis::{CheckedDB, networking::server_region},
};

pub(crate) fn provision_cadvisor(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    for server in db.db.server().rows_iter() {
        let monitoring_cluster = db.projections.monitoring_clusters.region_default(server_region(&db.db, server));

        let plan = plans.fetch_plan(server);
        plan.add_nix_package("cadvisor");
        let cadvisor_iface = db.projections.consul_network_iface.value(server);
        let cadvisor_service_ip = db.db.network_interface().c_if_ip(*cadvisor_iface);
        let port = 9280;

        if let Some(monitoring_cluster) = &monitoring_cluster {
            let monitoring_cluster = db.db.monitoring_cluster().c_cluster_name(*monitoring_cluster);
            let sec_conf = plan.add_secret_config(root_secret_config(
                "epl-cadvisor-service.hcl".to_string(),
                format!(
                r#"
service {{
  name = "epl-cadvisor"
  id   = "epl-cadvisor"
  port = {port}
  tags = ["epl-mon-{monitoring_cluster}"]

  tagged_addresses = {{
    lan = {{
      address = "{cadvisor_service_ip}"
      port    = {port}
    }}
  }}

  meta = {{
    metrics_path = "/metrics"
  }}

  checks = [
    {{
        id       = "healthcheck"
        name     = "/healthz"
        http     = "http://{cadvisor_service_ip}:9280/"
        interval = "15s"
    }},
  ]
}}
"#
                ),
            ));
            let abs_service_path = sec_conf.absolute_path();

            plan.add_post_second_round_secrets_shell_hook(format!(r#"
# wait for consul to be up and running if restarted for up to 10 seconds
for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

export CONSUL_HTTP_TOKEN=$(cat /run/keys/consul-agent-token.txt)
for I in $(seq 1 5); do
  consul services register {abs_service_path} && break || true
  # try a few times if consul is down
  sleep 1
done
"#));
        }

        plan.add_custom_nix_block(format!(r#"
    systemd.services.cadvisor = {{
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {{
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStart = "${{pkgs.cadvisor}}/bin/cadvisor" +
          " --listen_ip={cadvisor_service_ip}" +
          " --port={port}" +
          " --prometheus_endpoint=/metrics" +
          " --docker_only" +
          " --store_container_labels=false" +
          " --whitelisted_container_labels=com.hashicorp.nomad.job.name,com.hashicorp.nomad.node_name,com.hashicorp.nomad.namespace";
        Restart = "always";
        RestartSec = "1";
      }};

      enable = true;
    }};
"#));
    }
}
