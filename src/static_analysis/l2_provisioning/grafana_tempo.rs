use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerTempoCluster},
    static_analysis::{
        server_runtime::{ServerRuntime, MinIOBucketPermission, NomadJobKind, NomadJobStage, epl_architecture_to_nomad_architecture, IntegrationTest},
        PlatformValidationError, L1Projections, networking::{region_monitoring_clusters, region_loki_clusters, first_region_server}, docker_images::image_handle_from_pin,
    },
};

pub fn deploy_tempo(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    // 1. deploy ScyllaDB for index
    for tempo_cluster in db.tempo_cluster().rows_iter() {
        let name = db.tempo_cluster().c_cluster_name(tempo_cluster);
        let namespace = db.tempo_cluster().c_namespace(tempo_cluster);
        let region = db.tempo_cluster().c_region(tempo_cluster);
        let region_name = db.region().c_region_name(region);
        let workload_architecture = db.tempo_cluster().c_workload_architecture(tempo_cluster);
        let docker_image_pin_tempo = db.tempo_cluster().c_docker_image(tempo_cluster);

        let http_port = db.tempo_cluster().c_http_port(tempo_cluster);
        let grpc_port = db.tempo_cluster().c_grpc_port(tempo_cluster);
        let p2p_port = db.tempo_cluster().c_p2p_port(tempo_cluster);
        let otlp_http_port = db.tempo_cluster().c_otlp_http_port(tempo_cluster);
        let otlp_grpc_port = db.tempo_cluster().c_otlp_grpc_port(tempo_cluster);
        let memory_mb = db.tempo_cluster().c_memory_mb(tempo_cluster);
        let trace_retention_days = db.tempo_cluster().c_trace_retention_days(tempo_cluster);
        let tempo_instances = db.tempo_cluster().c_tempo_instances(tempo_cluster) as usize;

        let minio_bucket = db.tempo_cluster().c_storage_bucket(tempo_cluster);
        let minio_bucket_name = db.minio_bucket().c_bucket_name(minio_bucket);
        let minio_cluster = db.minio_bucket().c_parent(minio_bucket);
        let minio_cluster_name = db
            .minio_cluster()
            .c_cluster_name(minio_cluster);
        let minio_port = db
            .minio_cluster()
            .c_lb_port(db.minio_bucket().c_parent(minio_bucket));
        let minio_region = db.minio_cluster().c_region(minio_cluster);

        if region != minio_region {
            return Err(PlatformValidationError::TempoClusterMinIOBucketIsOutsideSpecifiedRegion {
                tempo_cluster: name.clone(),
                tempo_cluster_region: db.region().c_region_name(region).clone(),
                minio_cluster: minio_cluster_name.clone(),
                minio_cluster_region: db.region().c_region_name(minio_region).clone(),
            });
        }

        let monitoring_cluster = db.tempo_cluster().c_monitoring_cluster(tempo_cluster);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::TempoMonitoringClusterDoesntExistInRegion {
            tempo_cluster: name.clone(),
            tempo_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;

        let output_loki_cluster = db.tempo_cluster().c_loki_cluster(tempo_cluster);
        let output_loki_cluster = l1proj.loki_clusters.pick(
            region, output_loki_cluster
        ).ok_or_else(|| PlatformValidationError::TempoLoggingClusterDoesntExistInRegion {
            tempo_cluster: name.clone(),
            tempo_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: output_loki_cluster.to_string(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        let tempo_service_slug = format!("epl-tempo-{name}");
        let nomad_job_name = format!("tempo-{name}");
        let consul_service_fqdn = format!("{tempo_service_slug}.service.consul");
        let tempo_consul_service =
            runtime.instantiate_and_seal_consul_service(region, &tempo_service_slug);
        let minio_user = format!("tempo_{name}").replace("-", "_");
        let tempo_memory = runtime.reserve_stateless_memory_mb(
            format!("EPL Tempo cluster {name}"),
            memory_mb,
        );

        let http_port = runtime.lock_port_all_servers(
            http_port as u16,
            format!("Tempo http port for cluster {name}"),
        )?;
        let grpc_port = runtime.lock_port_all_servers(
            grpc_port as u16,
            format!("Tempo grpc port for cluster {name}"),
        )?;
        let p2p_port = runtime.lock_port_all_servers(
            p2p_port as u16,
            format!("Tempo p2p port for cluster {name}"),
        )?;
        let otlp_http_port = runtime.lock_port_all_servers(
            otlp_http_port as u16,
            format!("Tempo otlp http port for cluster {name}"),
        )?;
        let otlp_grpc_port = runtime.lock_port_all_servers(
            otlp_grpc_port as u16,
            format!("Tempo otlp grpc port for cluster {name}"),
        )?;

        let mut tempo_secrets_builder = runtime.issue_vault_secret(region, &format!("tempo/{name}"));
        let minio_password = tempo_secrets_builder.fetch_minio_bucket_credentials(
            db,
            "minio_bucket_password",
            &minio_user,
            minio_bucket,
            MinIOBucketPermission::ReadWrite,
        );
        let fin_secrets = tempo_secrets_builder.finalize();

        let tempo_cfg = generate_tempo_config(
            name.as_str(),
            &consul_service_fqdn,
            &format!(
                "http://epl-mon-{}-victoriametrics.service.consul:{}",
                db.monitoring_cluster().c_cluster_name(monitoring_cluster),
                db.monitoring_cluster().c_victoriametrics_port(monitoring_cluster),
            ),
            minio_bucket_name.as_str(),
            &format!("epl-minio-{minio_cluster_name}.service.consul:{minio_port}"),
            &minio_user,
            &minio_password.template_expression(),
            http_port.value(),
            grpc_port.value(),
            p2p_port.value(),
            otlp_http_port.value(),
            otlp_grpc_port.value(),
            trace_retention_days
        );

        let tempo_job = runtime.fetch_nomad_job(
            namespace,
            nomad_job_name.clone(),
            region,
            NomadJobKind::Stateless,
            NomadJobStage::SystemJob,
        );

        if tempo_job.loki_cluster().is_none() {
            tempo_job.set_loki_cluster(output_loki_cluster);
        }
        tempo_job.assign_vault_secrets(fin_secrets);
        let tempo_tg = tempo_job.fetch_task_group(format!("tempo"));
        tempo_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
        tempo_tg.set_count(tempo_instances);
        tempo_tg.try_set_placement(
            db,
            region,
            db.tempo_cluster().c_placement(tempo_cluster),
            &format!("instances for tempo_cluster named {name} in region {region_name}"),
            tempo_instances,
            l1proj.label_database
        )?;
        tempo_tg.add_locked_port("http", http_port);
        tempo_tg.add_locked_port("grpc", grpc_port);
        tempo_tg.add_locked_port("peer", p2p_port);
        tempo_tg.add_locked_port("otlp_http", otlp_http_port);
        tempo_tg.add_locked_port("otlp_grpc", otlp_grpc_port);
        tempo_tg.expose_port_as_tcp_service("http", &tempo_consul_service);
        // healthcheck is not http because tempo first needs to register itself to consul
        // only then it can bootstrap itself and will reply ok in ready endpoint
        // tempo_tg.set_service_http_healthcheck(&tempo_consul_service, "/ready");
        tempo_tg.collect_prometheus_metrics(&tempo_consul_service, monitoring_cluster, None);
        let docker_image_tempo = image_handle_from_pin(
            db, workload_architecture,
            docker_image_pin_tempo, "grafana_tempo"
        )?;
        let task = tempo_tg.fetch_task(
            format!("tempo-{name}"),
            docker_image_tempo,
        );
        task.add_memory(tempo_memory);
        let cfg_path = task.add_secure_config("config.yml".to_string(), tempo_cfg.clone());
        task.set_arguments(vec![
            "-target=scalable-single-binary".to_string(),
            format!("-config.file={cfg_path}"),
        ]);

        tempo_tests(db, l1proj, runtime, tempo_cluster);
    }

    Ok(())
}

fn tempo_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime, cluster: TableRowPointerTempoCluster) {
    let region = db.tempo_cluster().c_region(cluster);
    let name = db.tempo_cluster().c_cluster_name(cluster);
    let cluster_snake = name.to_case(convert_case::Case::Snake);
    let query_port = db.tempo_cluster().c_http_port(cluster);
    let push_port = db.tempo_cluster().c_otlp_http_port(cluster);
    let srv = first_region_server(db, region);
    if let Some(srv) = srv {
        let dns_lan_iface = l1proj.consul_network_iface.value(srv);
        let dns_lan_ip = db.network_interface().c_if_ip(*dns_lan_iface);
        runtime.add_integration_test(
            format!("tempo_cluster_{cluster_snake}_storing_and_querying_traces_works"),
            IntegrationTest::TempoSpansWritable {
                dns_server: format!("{dns_lan_ip}:53"),
                service_name: format!("epl-tempo-{name}.service.consul"),
                push_port,
                query_port,
            }
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_tempo_config(
    cluster_name: &str,
    service_dns_name: &str,
    prometheus_url: &str,
    bucket_name: &str,
    bucket_endpoint: &str,
    s3_username: &str,
    s3_password: &str,
    http_port: u16,
    grpc_port: u16,
    p2p_port: u16,
    otlp_http_port: u16,
    otlp_grpc_port: u16,
    trace_retention_days: i64,
) -> String {
    let trace_retention_h = trace_retention_days * 24;
    format!(
        r#"
# for grafana
stream_over_http_enabled: true

server:
  http_listen_port: {http_port}
  http_listen_address: {{{{ env "meta.private_ip" }}}}
  grpc_listen_port: {grpc_port}
  grpc_listen_address: {{{{ env "meta.private_ip" }}}}

distributor:
  ring:
    instance_addr: {{{{ env "meta.private_ip" }}}}
  receivers:
    otlp:
      protocols:
        http:
          endpoint: {{{{ env "meta.private_ip" }}}}:{otlp_http_port}
        grpc:
          endpoint: {{{{ env "meta.private_ip" }}}}:{otlp_grpc_port}

ingester:
  max_block_duration: 5m

compactor:
  ring:
    instance_addr: {{{{ env "meta.private_ip" }}}}
  compaction:
    block_retention: {trace_retention_h}h

memberlist:
  node_name: {{{{ env "node.unique.name" }}}}
  abort_if_cluster_join_fails: false
  bind_port: {p2p_port}
  bind_addr:
  - {{{{ env "meta.private_ip" }}}}
  join_members:
  - {service_dns_name}:{p2p_port}

metrics_generator:
  ring:
    instance_addr: {{{{ env "meta.private_ip" }}}}
  registry:
    external_labels:
      source: tempo
      cluster: {cluster_name}
  storage:
    path: /tmp/tempo/generator/wal
    remote_write:
      - url: {prometheus_url}/api/v1/write
        send_exemplars: true

storage:
  trace:
    backend: s3                        # backend configuration to use
    wal:
      path: /tmp/tempo/wal             # where to store the the wal locally
    s3:
      bucket: {bucket_name}                    # how to store data in s3
      endpoint: {bucket_endpoint}
      access_key: {s3_username}
      secret_key: {s3_password}
      insecure: true

querier:
  frontend_worker:
    frontend_address: {service_dns_name}:{grpc_port}

overrides:
  defaults:
    metrics_generator:
      processors: ['service-graphs', 'span-metrics']
"#
    )
}
