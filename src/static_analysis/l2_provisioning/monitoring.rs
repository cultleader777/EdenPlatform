use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerMonitoringCluster},
    static_analysis::{
        alerts,
        server_runtime::{AdminService, NomadJobKind, NomadJobStage, ServerRuntime, IntegrationTest, epl_architecture_to_nomad_architecture},
        PlatformValidationError, L1Projections, networking::{server_region, admin_service_responds_test, prometheus_metric_exists_test, prometheus_metric_doesnt_exist_test, check_servers_regional_distribution}, docker_images::image_handle_from_pin, get_global_settings,
    },
};
use std::fmt::Write;

pub fn deploy_monitoring_instances(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    let job_kind = NomadJobKind::BoundStateful;
    let gs = get_global_settings(db);

    for mon_c in db.monitoring_cluster().rows_iter() {
        let mon_instances = db.monitoring_cluster().c_children_monitoring_instance(mon_c).len();
        if mon_instances < 2 || mon_instances > 3 {
            return Err(PlatformValidationError::MonitoringInstancesMustAtLeastTwoToThree {
                cluster_name: db.monitoring_cluster().c_cluster_name(mon_c).clone(),
                minimum_count: 2,
                maximum_count: 3,
                current_count: mon_instances,
            });
        }

        let am_instances = db.monitoring_cluster().c_children_alertmanager_instance(mon_c).len();
        if am_instances != 3 && am_instances != 5 {
            return Err(PlatformValidationError::AlertmanagerInstancesMustBeThreeOrFive {
                cluster_name: db.monitoring_cluster().c_cluster_name(mon_c).clone(),
                valid_counts: vec![3, 5],
                current_count: am_instances,
            });
        }

        let cluster_name = db.monitoring_cluster().c_cluster_name(mon_c);
        let namespace = db.monitoring_cluster().c_namespace(mon_c);
        let region = db.monitoring_cluster().c_region(mon_c);
        let workload_architecture = db.monitoring_cluster().c_workload_architecture(mon_c);
        let docker_image_prometheus_pin = db.monitoring_cluster().c_docker_image_prometheus(mon_c);
        let docker_image_alertmanager_pin = db.monitoring_cluster().c_docker_image_alertmanager(mon_c);
        let docker_image_victoriametrics_pin = db.monitoring_cluster().c_docker_image_victoriametrics(mon_c);
        let mon_region_default = l1proj.monitoring_clusters.region_default(region);
        let is_this_region_default = Some(mon_c) == mon_region_default;
        let pm_port = db.monitoring_cluster().c_prometheus_port(mon_c);
        let vm_port = db.monitoring_cluster().c_victoriametrics_port(mon_c);
        let am_port = db.monitoring_cluster().c_alertmanager_port(mon_c);
        let vm_retention_months = db.monitoring_cluster().c_victoriametrics_retention_months(mon_c);

        let mut am_ips = Vec::with_capacity(5);
        for inst in db.monitoring_cluster().c_children_alertmanager_instance(mon_c) {
            let sv = db.alertmanager_instance().c_alertmanager_server(*inst);
            let server = db.server_volume().c_parent(sv);
            let server_region = server_region(db, server);
            if region != server_region {
                return Err(PlatformValidationError::MonitoringInstanceIsOutsideSpecifiedRegion {
                    monitoring_cluster: db.monitoring_cluster().c_cluster_name(mon_c).clone(),
                    monitoring_cluster_region: db.region().c_region_name(region).clone(),
                    server: db.server().c_hostname(server).clone(),
                    server_region: db.region().c_region_name(server_region).clone(),
                });
            }
            let this_if = l1proj.consul_network_iface.value(server);
            let this_ip = db.network_interface().c_if_ip(*this_if);
            am_ips.push(this_ip.clone());
        }

        check_servers_regional_distribution(
            db,
            region,
            db.monitoring_cluster().c_children_monitoring_instance(mon_c).iter().map(|i| {
                let srv_volume = db.monitoring_instance().c_monitoring_server(*i);
                db.server_volume().c_parent(srv_volume)
            }),
            format!("mon_cluster=>{cluster_name}")
        )?;

        check_servers_regional_distribution(
            db,
            region,
            db.monitoring_cluster().c_children_alertmanager_instance(mon_c).iter().map(|i| {
                let srv_volume = db.alertmanager_instance().c_alertmanager_server(*i);
                db.server_volume().c_parent(srv_volume)
            }),
            format!("mon_cluster alertmanager=>{cluster_name}")
        )?;

        let service_slug = format!("epl-mon-{}", db.monitoring_cluster().c_cluster_name(mon_c));
        let nomad_job_name = format!("mon-{}", db.monitoring_cluster().c_cluster_name(mon_c));

        let prom_service_slug = format!("{service_slug}-prometheus");
        let vm_service_slug = format!("{service_slug}-victoriametrics");
        let am_service_slug = format!("{service_slug}-alertmanager");
        let prom_consul_service = runtime.instantiate_and_seal_consul_service(region, &prom_service_slug);
        let vm_consul_service = runtime.instantiate_and_seal_consul_service(region, &vm_service_slug);
        let am_consul_service = runtime.instantiate_and_seal_consul_service(region, &am_service_slug);

        let adm_service_name = format!("adm-prometheus-{cluster_name}");
        runtime.expose_admin_service(
            db,
            AdminService {
                service_title: "Prometheus clusters".to_string(),
                service_kind: "prometheus".to_string(),
                service_instance: cluster_name.clone(),
                service_internal_upstream: prom_consul_service.clone(),
                service_internal_port: db
                    .monitoring_cluster()
                    .c_prometheus_port(mon_c)
                    .try_into()
                    .unwrap(),
                is_https: false,
            },
        )?;
        runtime.expose_admin_service(
            db,
            AdminService {
                service_title: "Alertmanager".to_string(),
                service_kind: "alertmanager".to_string(),
                service_instance: cluster_name.clone(),
                service_internal_upstream: am_consul_service.clone(),
                service_internal_port: db
                    .monitoring_cluster()
                    .c_alertmanager_port(mon_c)
                    .try_into()
                    .unwrap(),
                is_https: false,
            },
        )?;

        let rolling_stagger = 60;
        let mut update_delay = 0;
        for inst in db.monitoring_cluster().c_children_monitoring_instance(mon_c) {
            let instance_id = db.monitoring_instance().c_instance_id(*inst);
            let sv = db.monitoring_instance().c_monitoring_server(*inst);
            let server = db.server_volume().c_parent(sv);
            let hostname = db.server().c_hostname(server);
            let server_data = runtime.fetch_server_data(db, server);
            let volume_lock = server_data.server_volume_write_lock(
                db,
                sv,
                format!("Exclusive epl-mon-{cluster_name} volume lock"),
            )?;

            let mut this_ip = None;
            for ni in db.server().c_children_network_interface(server) {
                if db
                    .network()
                    .c_network_name(db.network_interface().c_if_network(*ni))
                    == "lan"
                {
                    assert!(this_ip.is_none());
                    this_ip = Some(db.network_interface().c_if_ip(*ni));
                }
            }
            let this_ip = this_ip.unwrap();

            let prometheus_port = server_data.lock_port(
                db,
                db.monitoring_cluster()
                    .c_prometheus_port(mon_c)
                    .try_into()
                    .unwrap(),
                format!("Prometheus port for {cluster_name}"),
            )?;
            let victoriametrics_port = server_data.lock_port(
                db,
                db.monitoring_cluster()
                    .c_victoriametrics_port(mon_c)
                    .try_into()
                    .unwrap(),
                format!("VictoriaMetrics port for {cluster_name}"),
            )?;

            let locked_prometheus_mem = server_data.reserve_memory_mb(
                format!("Prometheus memory {cluster_name}"),
                db.monitoring_cluster().c_prometheus_memory_mb(mon_c),
            )?;
            let locked_victoriametrics_mem = server_data.reserve_memory_mb(
                format!("VictoriaMetrics memory {cluster_name}"),
                db.monitoring_cluster().c_victoriametrics_memory_mb(mon_c),
            )?;

            let server_lock = runtime.lock_server_with_label(
                db,
                format!("epl-mon-{hostname}-{cluster_name}"),
                server,
            )?;

            let nomad_job =
                runtime.fetch_nomad_job(namespace, nomad_job_name.clone(), region, job_kind, NomadJobStage::SystemJob);
            // prometheus + victoriametrics
            let tg_name = format!("monitoring-{instance_id}");
            let tg = nomad_job.fetch_task_group(tg_name.clone());
            tg.set_shutdown_delay_seconds(update_delay);
            update_delay += rolling_stagger;
            tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
            tg.assign_server_lock(server_lock);
            tg.add_locked_port("prom", prometheus_port);
            tg.add_locked_port("vm", victoriametrics_port);
            tg.expose_port_as_tcp_service("prom", &prom_consul_service);
            tg.set_service_http_healthcheck(&prom_consul_service, "/-/healthy");
            tg.expose_port_as_tcp_service("vm", &vm_consul_service);
            tg.set_service_http_healthcheck(&vm_consul_service, "/health");

            let docker_image_prometheus =
                image_handle_from_pin(db, workload_architecture, docker_image_prometheus_pin, "prometheus")?;
            let prometheus_task =
                tg.fetch_task(
                    format!("mon-{cluster_name}-prometheus"),
                    docker_image_prometheus,
                );
            let alert_rules_yml_path =
                if !db
                    .monitoring_cluster()
                    .c_children_monitoring_cluster_alert_group(mon_c)
                    .is_empty()
                {
                    Some(prometheus_task.add_secure_config(
                        "alert_rules.yml".to_string(),
                        alert_rules_yml(db, mon_c),
                    ))
                } else {
                    None
                };
            let prometheus_yml_path = prometheus_task.add_secure_config(
                "prometheus.yml".to_string(),
                prometheus_yml(
                    &alert_rules_yml_path,
                    this_ip.as_str(),
                    pm_port,
                    vm_port,
                    am_port,
                    &am_ips,
                    cluster_name,
                    is_this_region_default,
                ),
            );
            let admin_domain = db.tld().c_domain(gs.admin_tld);
            prometheus_task.add_memory(locked_prometheus_mem);
            prometheus_task.bind_volume(volume_lock.clone(), "/volume".to_string());
            prometheus_task.set_arguments(vec![
                format!("--web.listen-address={this_ip}:{pm_port}"),
                format!("--config.file={prometheus_yml_path}"),
                "--storage.tsdb.path=/volume/prometheus-data".to_string(),
                "--storage.tsdb.retention.time=15d".to_string(),
                "--web.console.libraries=/opt/bitnami/prometheus/conf/console_libraries"
                    .to_string(),
                "--web.console.templates=/opt/bitnami/prometheus/conf/consoles".to_string(),
                format!("--web.external-url=https://{adm_service_name}.{admin_domain}"),
            ]);

            let docker_image_victoriametrics =
                image_handle_from_pin(db, workload_architecture, docker_image_victoriametrics_pin, "victoria_metrics")?;
            let vm_task =
                tg.fetch_task(
                    format!("mon-{cluster_name}-vm"),
                    docker_image_victoriametrics,
                );

            // task multibind volume? oy vey!
            vm_task.add_memory(locked_victoriametrics_mem);
            vm_task.bind_volume(volume_lock.clone(), "/volume".to_string());
            vm_task.set_arguments(vec![
                "-storageDataPath=/volume/victoriametrics-data".to_string(),
                format!("-retentionPeriod={vm_retention_months}"),
                format!("-httpListenAddr={this_ip}:{vm_port}"),
            ]);
        }

        let rolling_stagger = 60;
        let mut update_delay = 0;
        for inst in db.monitoring_cluster().c_children_alertmanager_instance(mon_c) {
            let instance_id = db.alertmanager_instance().c_instance_id(*inst);
            let sv = db.alertmanager_instance().c_alertmanager_server(*inst);
            let server = db.server_volume().c_parent(sv);
            let hostname = db.server().c_hostname(server);
            let server_data = runtime.fetch_server_data(db, server);
            let volume_lock = server_data.server_volume_write_lock(
                db,
                sv,
                format!("Exclusive epl-mon-{cluster_name}-am volume lock"),
            )?;
            let locked_alertmanager_mem = server_data.reserve_memory_mb(
                format!("Alertmanager memory {cluster_name}"),
                db.monitoring_cluster().c_alertmanager_memory_mb(mon_c),
            )?;

            let this_if = l1proj.consul_network_iface.value(server);
            let this_ip = db.network_interface().c_if_ip(*this_if);

            let alertmanager_port = server_data.lock_port(
                db,
                db.monitoring_cluster()
                    .c_alertmanager_port(mon_c)
                    .try_into()
                    .unwrap(),
                format!("Alertmanager port for {cluster_name}"),
            )?;
            let _alertmanager_p2p_port = server_data.lock_port(
                db,
                db.monitoring_cluster()
                    .c_alertmanager_p2p_port(mon_c)
                    .try_into()
                    .unwrap(),
                format!("Alertmanager p2p port for {cluster_name}"),
            )?;

            let server_lock = runtime.lock_server_with_label(
                db,
                format!("epl-mon-{hostname}-am-{cluster_name}"),
                server,
            )?;

            let nomad_job =
                runtime.fetch_nomad_job(namespace, nomad_job_name.clone(), region, job_kind, NomadJobStage::SystemJob);

            // alertmanager
            let am_tg_name = format!("alertmanager-{instance_id}");
            let am_tg = nomad_job.fetch_task_group(am_tg_name.clone());
            am_tg.set_shutdown_delay_seconds(update_delay);
            update_delay += rolling_stagger;
            am_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
            am_tg.assign_server_lock(server_lock);
            am_tg.add_locked_port("am", alertmanager_port);
            am_tg.expose_port_as_tcp_service("am", &am_consul_service);
            am_tg.set_service_http_healthcheck(&am_consul_service, "/-/healthy");
            let am_p2p_port = db.monitoring_cluster().c_alertmanager_p2p_port(mon_c);
            let docker_image_alertmanager =
                image_handle_from_pin(db, workload_architecture, docker_image_alertmanager_pin, "alertmanager")?;
            let am_task =
                am_tg.fetch_task(
                    format!("mon-{cluster_name}-am"),
                    docker_image_alertmanager,
                );

            let am_cfg_path =
                am_task.add_secure_config("alertmanager.yml".to_string(), alertmanager_yml(db, mon_c));

            am_task.add_memory(locked_alertmanager_mem);
            am_task.bind_volume(volume_lock.clone(), "/volume".to_string());
            let mut am_args = vec![
                format!("--web.listen-address={this_ip}:{am_port}"),
                format!("--config.file={am_cfg_path}"),
                format!("--storage.path=/volume/alertmanager"),
                format!("--cluster.listen-address={this_ip}:{am_p2p_port}"),
                format!("--cluster.advertise-address={this_ip}:{am_p2p_port}"),
            ];

            for ip in &am_ips {
                if ip != this_ip {
                    am_args.push(format!("--cluster.peer={ip}:{am_p2p_port}"));
                }
            }

            am_task.set_arguments(am_args);
        }

        monitoring_tests(db, l1proj, runtime, mon_c);
    }

    Ok(())
}

fn monitoring_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime, mon: TableRowPointerMonitoringCluster) {
    let cluster_name = db.monitoring_cluster().c_cluster_name(mon);
    let cluster_snake = cluster_name.to_case(convert_case::Case::Snake);
    let mut inst_ips = Vec::new();
    let mut am_tcp_p2p_sockets = Vec::new();
    let am_p2p_port = db.monitoring_cluster().c_alertmanager_p2p_port(mon);
    let region = db.monitoring_cluster().c_region(mon);

    runtime.add_integration_test(
        format!("monitoring_cluster_external_admin_panel_responds_{cluster_snake}"),
        admin_service_responds_test(
            db,
            l1proj,
            format!("adm-prometheus-{cluster_name}"),
            "/graph",
            "<title>Prometheus Time Series Collection and Processing Server</title>"
        )
    );

    for mon in db.monitoring_cluster().c_children_monitoring_instance(mon) {
        let vol = db.monitoring_instance().c_monitoring_server(*mon);
        let srv = db.server_volume().c_parent(vol);
        let iface = l1proj.consul_network_iface.value(srv);
        let ip = db.network_interface().c_if_ip(*iface);
        inst_ips.push(ip.clone());
        am_tcp_p2p_sockets.push(format!("{ip}:{am_p2p_port}"));
    }

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_dns_exists"),
        IntegrationTest::DnsResolutionWorksARecords {
            target_servers: vec![format!("{}:53", inst_ips[0].clone())],
            queries: vec![
                (format!("epl-mon-{cluster_name}-prometheus.service.consul"), inst_ips.clone())
            ]
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_prometheus_healthcheck_ok"),
        IntegrationTest::HttpGetRespondsOk {
            server_ips: inst_ips.clone(),
            http_server_port: db.monitoring_cluster().c_prometheus_port(mon),
            path: "/-/healthy".to_string(),
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_prometheus_ui_works"),
        IntegrationTest::HttpGetRespondsString {
            hostname: None,
            server_ips: inst_ips.clone(),
            http_server_port: db.monitoring_cluster().c_prometheus_port(mon),
            path: "/graph".to_string(),
            is_https: false,
            expected_string: "<title>Prometheus Time Series Collection and Processing Server</title>".to_string(),
            use_admin_panel_credentials: Some(crate::static_analysis::server_runtime::IntegrationTestCredentials::AdminPanel),
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_prometheus_exposes_metrics"),
        IntegrationTest::HttpGetRespondsString {
            hostname: None,
            server_ips: inst_ips.clone(),
            http_server_port: db.monitoring_cluster().c_prometheus_port(mon),
            path: "/metrics".to_string(),
            is_https: false,
            expected_string: "prometheus_notifications_queue_capacity 10000".to_string(),
            use_admin_panel_credentials: Some(crate::static_analysis::server_runtime::IntegrationTestCredentials::AdminPanel),
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_victoriametrics_healthcheck_ok"),
        IntegrationTest::HttpGetRespondsOk {
            server_ips: inst_ips.clone(),
            http_server_port: db.monitoring_cluster().c_victoriametrics_port(mon),
            path: "/health".to_string(),
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_alertmanager_healthcheck_ok"),
        IntegrationTest::HttpGetRespondsOk {
            server_ips: inst_ips.clone(),
            http_server_port: db.monitoring_cluster().c_alertmanager_port(mon),
            path: "/-/healthy".to_string(),
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_alertmanager_ui"),
        IntegrationTest::HttpGetRespondsString {
            hostname: None,
            server_ips: inst_ips.clone(),
            http_server_port: db.monitoring_cluster().c_alertmanager_port(mon),
            path: "/#/alerts".to_string(),
            is_https: false,
            expected_string: "<title>Alertmanager</title>".to_string(),
            use_admin_panel_credentials: Some(crate::static_analysis::server_runtime::IntegrationTestCredentials::AdminPanel),
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_alertmanager_peer_ports"),
        IntegrationTest::TcpSocketsOpen {
            target_sockets: am_tcp_p2p_sockets,
        },
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_prometheus_metrics_exist"),
        prometheus_metric_exists_test(db, l1proj, mon, "prometheus_target_scrape_pool_targets")
    );

    runtime.add_integration_test(
        format!("monitoring_cluster_{cluster_snake}_no_scrape_targets_are_down"),
        prometheus_metric_doesnt_exist_test(db, l1proj, mon, "up == 0")
    );

    // check if at least first alert from alert group exists
    for ag in db.monitoring_cluster().c_children_monitoring_cluster_alert_group(mon) {
        let ag = db.monitoring_cluster_alert_group().c_alert_group_name(*ag);
        let ag_name = db.alert_group().c_alert_group_name(ag);
        let ag_snake = ag_name.to_case(convert_case::Case::Snake);
        let children = db.alert_group().c_children_alert(ag);
        if !children.is_empty() {
            let first_alert = children[0];
            let alert_name = db.alert().c_alert_name(first_alert);
            runtime.add_integration_test(
                format!("monitoring_cluster_{cluster_snake}_{ag_snake}_prometheus_alert_exists"),
                IntegrationTest::HttpGetRespondsString {
                    hostname: None,
                    server_ips: inst_ips.clone(),
                    http_server_port: db.monitoring_cluster().c_prometheus_port(mon),
                    path: "/api/v1/rules".to_string(),
                    is_https: false,
                    expected_string: format!("\"name\":\"{alert_name}\""),
                    use_admin_panel_credentials: Some(crate::static_analysis::server_runtime::IntegrationTestCredentials::AdminPanel),
                },
            );
        }
    }

    // if region default
    if Some(mon) == l1proj.monitoring_clusters.region_default(region) {
        runtime.add_integration_test(
            format!("monitoring_cluster_{cluster_snake}_node_exporter_metrics_exist"),
            prometheus_metric_exists_test(db, l1proj, mon, "node_cpu_seconds_total")
        );

        runtime.add_integration_test(
            format!("monitoring_cluster_{cluster_snake}_cadvisor_metrics_exist"),
            prometheus_metric_exists_test(db, l1proj, mon, "container_cpu_user_seconds_total")
        );

        runtime.add_integration_test(
            format!("monitoring_cluster_{cluster_snake}_has_vector_metrics"),
            prometheus_metric_exists_test(db, l1proj, mon, "vector_buffer_events")
        );

        runtime.add_integration_test(
            format!("monitoring_cluster_{cluster_snake}_has_epl_l1_provisioning_id_metrics"),
            prometheus_metric_exists_test(db, l1proj, mon, "epl_l1_provisioning_id")
        );
    }
}

fn prometheus_yml(
    alert_rules_yml_path: &Option<String>,
    instance_internal_ip: &str,
    prometheus_port: i64,
    vm_port: i64,
    alertmanager_port: i64,
    alertmanager_ips: &[String],
    cluster_name: &str,
    scrape_system_services: bool,
) -> String {
    let mut result = String::new();
    let maybe_alert_rules = if let Some(ar) = alert_rules_yml_path {
        format!(
            r#"
rule_files:
  - {ar}
"#
        )
    } else {
        "".to_string()
    };
    let maybe_system_services = if scrape_system_services {
        r#"
  - job_name: 'consul-nomad'
    scheme: https
    tls_config:
      insecure_skip_verify: true
    metrics_path: /v1/metrics
    params:
      format: ['prometheus']
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        services:
          - nomad-clients
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance

  - job_name: 'consul-vault'
    scheme: https
    tls_config:
      insecure_skip_verify: true
    metrics_path: /v1/sys/metrics
    params:
      format: ['prometheus']
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        services:
          - vault
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance

"#
    } else { "" };
    let _ = write!(
        &mut result,
        r#"
global:
  scrape_interval: 15s
  evaluation_interval: 15s

remote_write:
  - url: http://{instance_internal_ip}:{vm_port}/api/v1/write

{maybe_alert_rules}

scrape_configs:
  - job_name: "prometheus"
    static_configs:
      - targets: ["{instance_internal_ip}:{prometheus_port}"]

  - job_name: 'consul'
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        tags:
          - epl-mon-{cluster_name}
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance
      - source_labels: [__meta_consul_service_metadata_metrics_path]
        regex: ^(/.+)$
        action: replace
        target_label: __metrics_path__

{maybe_system_services}

alerting:
  alertmanagers:
    - static_configs:
      - targets:
"#
    );

    for ip in alertmanager_ips {
        let _ = writeln!(&mut result, "        - {ip}:{alertmanager_port}");
    }

    result
}

fn alert_rules_yml(db: &Database, mon_c: TableRowPointerMonitoringCluster) -> String {
    let mut res = String::new();

    res += "groups:\n";

    for ag in db.monitoring_cluster().c_children_monitoring_cluster_alert_group(mon_c) {
        let ag = db.monitoring_cluster_alert_group().c_alert_group_name(*ag);
        res += "- name: ";
        res += db.alert_group().c_alert_group_name(ag);
        res += "\n";
        res += "  rules:\n";

        for alert in db.alert_group().c_children_alert(ag) {
            res += &alerts::generate_alert_rule(db, *alert);
        }
    }

    // escape for nomad
    res.replace(" {{ ", " {{\"{{\"}} ")
        .replace(" }} ", " {{\"}}\"}} ")
}

fn alertmanager_yml(db: &Database, mon_c: TableRowPointerMonitoringCluster) -> String {
    let mut res = String::new();

    let _ = write!(
        &mut res,
        r#"
# Inhibition rules allow to mute a set of alerts given that another alert is
# firing.
# We use this to mute any warning-level notifications if the same alert is
# already critical.
inhibit_rules:
  - source_matchers: [severity="critical"]
    target_matchers: [severity="warning"]
    equal: [alertname, cluster, service]

# The root route on which each incoming alert enters.
route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 1h
  # this receiver will never be used as we explicitly list all alert groups
  # to which channel they belong
  receiver: unused
"#
    );

    if db.monitoring_cluster().c_children_monitoring_cluster_alert_group(mon_c).len() > 0 {
        write!(
            &mut res,
            r#"
  routes:"#
        ).unwrap();
        for ag in db.monitoring_cluster().c_children_monitoring_cluster_alert_group(mon_c) {
            let ag_orig = db.monitoring_cluster_alert_group().c_alert_group_name(*ag);
            let mut alert_vec = db.alert_group().c_children_alert(ag_orig)
                .iter()
                .map(|alert| {
                    db.alert().c_alert_name(*alert).as_str()
                })
                .collect::<Vec<_>>();
            alert_vec.sort();
            let joined = alert_vec.join("|");
            let group_name = db.alert_group().c_alert_group_name(ag_orig);
            write!(
                &mut res,
                r#"
  - receiver: {group_name}
    matchers:
    - alertname=~"^({joined})$""#
            ).unwrap();
        }
    }

    write!(
        &mut res,
        r#"
receivers:"#
    ).unwrap();

    for ag in db.monitoring_cluster().c_children_monitoring_cluster_alert_group(mon_c) {
        let ag_orig = db.monitoring_cluster_alert_group().c_alert_group_name(*ag);
        let tg_bot = db.monitoring_cluster_alert_group().c_telegram_bot(*ag);
        let tg_channel = db.monitoring_cluster_alert_group().c_telegram_channel(*ag);
        let tg_channel_id = db.telegram_channel().c_channel_id(tg_channel);
        let tg_bot_token = db.telegram_bot().c_bot_token(tg_bot);
        let group_name = db.alert_group().c_alert_group_name(ag_orig);

        write!(
            &mut res,
            r#"
  - name: '{group_name}'
    telegram_configs:
      - bot_token: {tg_bot_token}
        chat_id: {tg_channel_id}
        api_url: https://api.telegram.org
        parse_mode: ''
"#
        ).unwrap();
    }

    write!(
        &mut res,
        r#"
  - name: 'unused'
    telegram_configs:
      - bot_token: bad_bot_token
        chat_id: -123456789
        api_url: https://api.telegram.org
        parse_mode: ''
"#
    ).unwrap();

    res
}
