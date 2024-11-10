use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerGrafana},
    static_analysis::{
        server_runtime::{
            AdminService, PgAccessKind, NomadJobKind, NomadJobStage, ProvisioningScriptTag,
            ServerRuntime, VaultSecretHandle, VaultSecretRequest, IntegrationTest, epl_architecture_to_nomad_architecture,
        },
        PlatformValidationError, L1Projections, networking::{region_monitoring_clusters, region_loki_clusters, admin_service_responds_test, first_region_server}, docker_images::image_handle_from_pin,
    },
};

pub fn deploy_grafana(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    // All dashboards are the same per region
    for region in db.region().rows_iter() {
        let mut dashboard_paths = Vec::new();
        for dashboard in db.grafana_dashboard().rows_iter() {
            let fname = db.grafana_dashboard().c_filename(dashboard);
            let contents = db.grafana_dashboard().c_contents(dashboard);
            let rp = runtime.add_provisioning_resource(
                region,
                "grafana-dashboard",
                fname.clone(),
                contents.clone(),
                false,
                Vec::new(),
            );
            dashboard_paths.push(rp.path().to_string());
        }

        runtime.add_provisioning_script(
            region,
            ProvisioningScriptTag::NonEssentialsProvision,
            "provision-grafana-dashboards.sh",
            generate_grafana_dashboard_provisioning_script(db, dashboard_paths.as_slice()),
        );
    }

    for graf in db.grafana().rows_iter() {
        let deployment_name = db.grafana().c_deployment_name(graf);
        let namespace = db.grafana().c_namespace(graf);
        let region = db.grafana().c_region(graf);
        let region_name = db.region().c_region_name(region);
        let workload_architecture = db.grafana().c_workload_architecture(graf);
        let docker_image_grafana_pin = db.grafana().c_docker_image_grafana(graf);
        let docker_image_promxy_pin = db.grafana().c_docker_image_promxy(graf);
        let db_depl = db.grafana().c_database(graf);
        let db_depl_r = db.pg_deployment_unmanaged_db().c_parent(db.grafana().c_database(graf));
        let db_region = db.pg_deployment().c_region(db_depl_r);
        if region != db_region {
            return Err(PlatformValidationError::GrafanaDatabaseIsOutsideSpecifiedRegion {
                grafana_deployment: deployment_name.clone(),
                grafana_region: db.region().c_region_name(region).clone(),
                pg_deployment: db.pg_deployment().c_deployment_name(db_depl_r).clone(),
                pg_deployment_region: db.region().c_region_name(db_region).clone(),
            });
        }
        let port = db.grafana().c_port(graf);
        let memory_mb = db.grafana().c_memory_mb(graf);
        let dba = runtime
            .fetch_pg_access(&PgAccessKind::Unmanaged(db_depl))
            .clone();
        let locked_port = runtime.lock_port_all_servers(port as u16, format!("Grafana {deployment_name}"))?;
        let memory = runtime.reserve_stateless_memory_mb(format!("Grafana {deployment_name}"), memory_mb);
        let promxy_memory = runtime.reserve_stateless_memory_mb(
            format!("Grafana promxy {deployment_name}"),
            db.grafana().c_promxy_memory_mb(graf),
        );
        let service_slug = format!("epl-grafana-{deployment_name}");
        let nomad_job_name = format!("grafana-{deployment_name}");
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &service_slug);
        runtime.expose_admin_service(
            db,
            AdminService {
                service_title: "Grafana".to_string(),
                service_kind: "grafana".to_string(),
                service_instance: deployment_name.clone(),
                service_internal_upstream: consul_service.clone(),
                service_internal_port: port.try_into().unwrap(),
                is_https: false,
            },
        )?;
        let mut secs = runtime.issue_vault_secret(region, &format!("grafana/{deployment_name}"));
        let admin_pwd = secs.request_secret(
            region,
            "admin_password",
            VaultSecretRequest::AlphanumericPassword42Symbols,
        );
        let postgres_password = secs.request_secret(
            region,
            "postgres_password",
            VaultSecretRequest::ExistingVaultSecret {
                handle: Box::new(dba.db_password.clone()),
                sprintf: None,
            },
        );
        let vs = secs.finalize();

        let monitoring_cluster = db.grafana().c_monitoring_cluster(graf);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::GrafanaMonitoringClusterDoesntExistInRegion {
            grafana_deployment: deployment_name.clone(),
            grafana_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = db.grafana().c_loki_cluster(graf);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::GrafanaLoggingClusterDoesntExistInRegion {
            grafana_deployment: deployment_name.clone(),
            grafana_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;


        let job = runtime.fetch_nomad_job(
            namespace,
            nomad_job_name.clone(),
            region,
            NomadJobKind::Stateless,
            NomadJobStage::SystemJob,
        );
        job.set_loki_cluster(loki_cluster);
        job.assign_vault_secrets(vs);
        let tg = job.fetch_task_group(format!("grafana"));
        tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
        let count = db.grafana().c_instance_count(graf) as usize;
        tg.set_count(count);
        tg.try_set_placement(
            db,
            region,
            db.grafana().c_placement(graf),
            &format!("grafana deployment named {deployment_name} in region {region_name}"),
            count,
            l1proj.label_database
        )?;
        tg.add_locked_port("svc", locked_port);
        tg.expose_port_as_tcp_service("svc", &consul_service);
        tg.collect_prometheus_metrics(&consul_service, monitoring_cluster, None);
        let main_config = generate_grafana_config(
            port,
            &admin_pwd,
            &dba.db_user,
            &dba.db_host,
            dba.db_master_port,
            &postgres_password,
            &dba.db_database,
        );
        let datasources_config = generate_datasources(db, db.grafana().c_promxy_port(graf));
        let docker_image_grafana = image_handle_from_pin(db, workload_architecture, docker_image_grafana_pin, "grafana")?;
        // let task = tg.fetch_task("grafana".to_string(), "grafana/grafana:9.5.2".to_string());
        let task = tg.fetch_task(
            format!("epl-grafana-service-{deployment_name}"),
            docker_image_grafana,
        );
        task.add_memory(memory);
        let cfg_abs_path = task.add_secure_config("grafana.ini".to_string(), main_config);
        let _ds_abs_path = task.add_secure_config(
            "provisioning/datasources/datasources.yml".to_string(),
            datasources_config,
        );
        task.set_env_variable("GF_PATHS_CONFIG", &cfg_abs_path);
        task.set_env_variable("GF_PATHS_PROVISIONING", "/secrets/provisioning");

        let docker_image_promxy = image_handle_from_pin(db, workload_architecture, docker_image_promxy_pin, "promxy")?;
        let task = tg.fetch_task(
            format!("epl-grafana-promxy-{deployment_name}"),
            docker_image_promxy,
        );

        let promxy_cfg = generate_promxy_config(db);
        let abs_cfg = task.add_secure_config("promxy-conf.yml".to_string(), promxy_cfg);
        task.set_arguments(vec![
            format!("--bind-addr=127.0.0.1:{}", db.grafana().c_promxy_port(graf)),
            format!("--config={abs_cfg}"),
        ]);
        task.add_memory(promxy_memory);

        grafana_tests(db, l1proj, runtime, graf);
    }

    Ok(())
}

fn grafana_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime, graf: TableRowPointerGrafana) {
    let graf_name = db.grafana().c_deployment_name(graf);
    let graf_snake = graf_name.to_case(convert_case::Case::Snake);
    let region = db.grafana().c_region(graf);
    let reg_srv = first_region_server(db, region);

    if let Some(reg_srv) = reg_srv {
        let srv_iface = l1proj.consul_network_iface.value(reg_srv);
        let srv_ip = db.network_interface().c_if_ip(*srv_iface);
        runtime.add_integration_test(
            format!("grafana_dns_record_expected_count_{graf_snake}"),
            IntegrationTest::DnsResolutionARecordCount {
                target_servers: vec![format!("{srv_ip}:53")],
                queries: vec![(format!("epl-grafana-{graf_name}.service.consul"), db.grafana().c_instance_count(graf) as usize)]
            },
        );
    }


    runtime.add_integration_test(
        format!("grafana_external_admin_panel_responds_{graf_snake}"),
        admin_service_responds_test(
            db,
            l1proj,
            format!("adm-grafana-{graf_name}"),
            "/",
            "<title>Grafana</title>"
        )
    );

    // TODO: extract dashboard uids from json?
    runtime.add_integration_test(
        format!("grafana_node_exporter_dashboard_loaded_{graf_snake}"),
        admin_service_responds_test(
            db,
            l1proj,
            format!("adm-grafana-{graf_name}"),
            "/api/dashboards/uid/rYdddlPWk",
            "\"title\":\"Node Exporter Full\""
        )
    );
}

fn generate_grafana_dashboard_provisioning_script(db: &Database, dashboards: &[String]) -> String {
    let mut res = String::new();

    res += r#"
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
"#;

    for graf in db.grafana().rows_iter() {
        let deployment_name = db.grafana().c_deployment_name(graf);
        let port = db.grafana().c_port(graf);
        let grafana_url = format!("http://epl-grafana-{deployment_name}.service.consul:{port}");
        res += "# wait for instance to be healthy\n";
        res += &format!("while ! curl -f {grafana_url}/api/health\n");
        res += "do\n";
        res += "  echo Waiting for grafana healthcheck to be up...\n";
        res += "  sleep 5\n";
        res += "done\n";

        res += &format!("ADMIN_PASSWORD=$( vault kv get -field=admin_password epl/grafana/{deployment_name} )\n");
        for dashboard in dashboards {
            // Ensure dashboard is overwritten
            res += "\n";
            res += &format!(
                "jq \".dashboard.id = null | .overwrite = true\" {dashboard} > {dashboard}.fixed\n"
            );
            res += "while ! curl -f -u admin:$ADMIN_PASSWORD -XPOST -H 'Content-Type: application/json' --data @";
            res += dashboard;
            res += &format!(".fixed {grafana_url}/api/dashboards/db");
            res += "\n";
            res += "do\n";
            res += &format!("  echo Can\\'t upload grafana dashboard {dashboard} from first time, trying again after second...\n");
            res += "  sleep 1\n";
            res += "done\n";
        }
    }

    res
}

fn generate_grafana_config(
    port: i64,
    admin_pwd: &VaultSecretHandle,
    postgres_user: &str,
    postgres_host: &str,
    postgres_port: u16,
    postgres_password: &VaultSecretHandle,
    postgres_database: &str,
) -> String {
    let admin_pwd = admin_pwd.template_expression();
    let postgres_password = postgres_password.template_expression();
    format!(
        r#"
[paths]
data = /var/lib/grafana
plugins = /var/lib/grafana/plugins

[server]
protocol = http
http_port = {port}

[database]
type = postgres
host = {postgres_host}:{postgres_port}
user = {postgres_user}
password = {postgres_password}
name = {postgres_database}

[datasources]

[users]
allow_sign_up = false

[security]
admin_user = admin
admin_password = {admin_pwd}

[auth]

[auth.anonymous]
enabled = false

[auth.basic]
enabled = true

[log]
mode = console

[metrics]
enabled = true
"#
    )
}

fn generate_datasources(db: &Database, promxy_port: i64) -> String {
    use std::fmt::Write;

    let mut res = "apiVersion: 1\n".to_string();
    res += "datasources:\n";

    let _ = write!(
        &mut res,
        r#"
  - name: promxy all victoria metrics
    type: prometheus
    access: proxy
    url: http://127.0.0.1:{promxy_port}
    isDefault: true
"#
    );

    for mon in db.monitoring_cluster().rows_iter() {
        // TODO: add local promxy for viewing all clusters
        let mc_name = db.monitoring_cluster().c_cluster_name(mon);
        let mc_vm_port = db.monitoring_cluster().c_victoriametrics_port(mon);
        let mc_prom_port = db.monitoring_cluster().c_prometheus_port(mon);
        let region = db.monitoring_cluster().c_region(mon);
        let region_name = db.region().c_region_name(region);
        let _ = write!(
            &mut res,
            r#"
  - name: {mc_name} victoria metrics
    type: prometheus
    access: proxy
    url: http://epl-mon-{mc_name}-victoriametrics.service.{region_name}.consul:{mc_vm_port}
"#
        );

        let _ = write!(
            &mut res,
            r#"
  - name: {mc_name} prometheus
    type: prometheus
    access: proxy
    url: http://epl-mon-{mc_name}-victoriametrics.service.{region_name}.consul:{mc_prom_port}
"#
        );
    }

    for loki in db.loki_cluster().rows_iter() {
        let name = db.loki_cluster().c_cluster_name(loki);
        let region = db.loki_cluster().c_region(loki);
        let region_name = db.region().c_region_name(region);
        let reader_port = db.loki_cluster().c_loki_reader_http_port(loki);
        let _ = write!(
            &mut res,
            r#"
  - name: {name} loki cluster
    type: loki
    access: proxy
    url: http://epl-loki-{name}-loki-reader.service.{region_name}.consul:{reader_port}
    jsonData:
      maxLines: 1000
"#
        );
    }

    for tempo in db.tempo_cluster().rows_iter() {
        let name = db.tempo_cluster().c_cluster_name(tempo);
        let region = db.tempo_cluster().c_region(tempo);
        let region_name = db.region().c_region_name(region);
        let tempo_port = db.tempo_cluster().c_http_port(tempo);
        let _ = write!(
            &mut res,
            r#"
  - name: {name} tempo cluster
    type: tempo
    url: http://epl-tempo-{name}.service.{region_name}.consul:{tempo_port}
    access: proxy
    basicAuth: false

"#
        );
    }

    res
}

fn generate_promxy_config(db: &Database) -> String {
    let mut res = String::new();

    res += r#"
global:
  evaluation_interval: 5s
  external_labels:
    source: promxy

promxy:
  server_groups:
"#;

    for mc in db.monitoring_cluster().rows_iter() {
        let cname = db.monitoring_cluster().c_cluster_name(mc);
        let vm_port = db.monitoring_cluster().c_victoriametrics_port(mc);
        res += &format!(
            r#"
    - static_configs:
        - targets:
          - epl-mon-{cname}-victoriametrics.service.consul:{vm_port}
      # labels to be added to metrics retrieved from this server_group
      labels:
        epl_mc: {cname}
      anti_affinity: 10s
      timeout: 5s
      query_params:
        nocache: 1
"#
        );
    }

    res
}
