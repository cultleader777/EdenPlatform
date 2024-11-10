use crate::{
    codegen::nixplan::{root_secret_config, NixAllServerPlans},
    static_analysis::{CheckedDB, networking::server_region},
};

pub(crate) fn provision_node_exporter(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    for server in db.db.server().rows_iter() {
        let monitoring_cluster = db.projections.monitoring_clusters.region_default(server_region(&db.db, server));

        let plan = plans.fetch_plan(server);
        plan.add_nix_package("prometheus-node-exporter");
        let node_exp_iface = db.projections.consul_network_iface.value(server);
        let node_exp_service_ip = db.db.network_interface().c_if_ip(*node_exp_iface);
        let port = 9100;

        if let Some(monitoring_cluster) = &monitoring_cluster {
            let monitoring_cluster = db.db.monitoring_cluster().c_cluster_name(*monitoring_cluster);
            let sec_conf = plan.add_secret_config(root_secret_config(
                "epl-node-exporter-service.hcl".to_string(),
                format!(
                r#"
service {{
  name = "epl-node-exporter"
  id   = "epl-node-exporter"
  port = {port}
  tags = ["epl-mon-{monitoring_cluster}"]

  meta = {{
    metrics_path = "/metrics"
  }}

  tagged_addresses = {{
    lan = {{
      address = "{node_exp_service_ip}"
      port    = {port}
    }}
  }}

  checks = [
    {{
        id       = "home"
        name     = "/"
        http     = "http://{node_exp_service_ip}:{port}/"
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

export CONSUL_HTTP_TOKEN=$( cat /run/keys/consul-agent-token.txt )
for I in $(seq 1 5); do
  consul services register {abs_service_path} && break || true
  # try a few times if consul is down
  sleep 1
done
"#));
        }

        // textfile collector dir
        plan.add_post_second_round_secrets_shell_hook(r#"
mkdir -p /var/lib/node_exporter
chown -R node-exp:node-exp /var/lib/node_exporter
chmod 700 /var/lib/node_exporter
"#.to_string());

        plan.add_custom_nix_block(format!(
            r#"
    users.users.node-exp = {{
        isSystemUser = true;
        description = "Vault service";
        extraGroups = ["keys"];
        group = "node-exp";
    }};
    users.groups.node-exp = {{}};

    systemd.services.node_exporter = {{
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" ];

      serviceConfig = {{
        User = "node-exp";
        Group = "node-exp";
        Type = "simple";
        ExecStart = "${{pkgs.prometheus-node-exporter}}/bin/node_exporter" +
          " --collector.systemd" +
          " --collector.textfile" +
          " --collector.textfile.directory=/var/lib/node_exporter" +
          " --web.listen-address={node_exp_service_ip}:{port}" +
          " --web.telemetry-path=/metrics";
        Restart = "always";
        RestartSec = "1";
        SyslogIdentifier = "node_exporter";
        ProtectHome = "yes";
        NoNewPrivileges = "yes";
        ProtectSystem = "strict";
        ProtectControlGroups = "true";
        ProtectKernelModules = "true";
        ProtectKernelTunables = "yes";
      }};

      enable = true;
    }};
"#
        ));
    }
}
