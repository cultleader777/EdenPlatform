use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerLokiCluster},
    static_analysis::{
        server_runtime::{
            CustomConsulPolicyBuilder, MinIOBucketPermission, NomadJobKind, NomadJobStage,
            ServerRuntime, VaultSecretHandle, VaultSecretRequest, IntegrationTest, epl_architecture_to_nomad_architecture,
        },
        PlatformValidationError, L1Projections, networking::{region_monitoring_clusters, region_loki_clusters, prometheus_metric_exists_test, first_region_server}, docker_images::image_handle_from_pin,
    },
};

pub fn deploy_loki(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    for loki_cluster in db.loki_cluster().rows_iter() {
        let name = db.loki_cluster().c_cluster_name(loki_cluster);
        let namespace = db.loki_cluster().c_namespace(loki_cluster);
        let region = db.loki_cluster().c_region(loki_cluster);
        let region_name = db.region().c_region_name(region);
        let workload_architecture = db.loki_cluster().c_workload_architecture(loki_cluster);
        let docker_image_pin_loki = db.loki_cluster().c_docker_image_loki(loki_cluster);
        let retention_hours = 24 * db.loki_cluster().c_retention_period_days(loki_cluster);
        let loki_writer_http_port = db.loki_cluster().c_loki_writer_http_port(loki_cluster);
        let loki_writer_grpc_port = db.loki_cluster().c_loki_writer_grpc_port(loki_cluster);
        let loki_reader_http_port = db.loki_cluster().c_loki_reader_http_port(loki_cluster);
        let loki_reader_grpc_port = db.loki_cluster().c_loki_reader_grpc_port(loki_cluster);
        let loki_backend_http_port = db.loki_cluster().c_loki_backend_http_port(loki_cluster);
        let loki_backend_grpc_port = db.loki_cluster().c_loki_backend_grpc_port(loki_cluster);
        let reader_count = db.loki_cluster().c_loki_readers(loki_cluster) as usize;
        let writer_count = db.loki_cluster().c_loki_writers(loki_cluster) as usize;
        let minio_bucket = db.loki_cluster().c_storage_bucket(loki_cluster);
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
            return Err(PlatformValidationError::LokiClusterMinIOBucketIsOutsideSpecifiedRegion {
                loki_cluster: name.clone(),
                loki_cluster_region: db.region().c_region_name(region).clone(),
                minio_cluster: minio_cluster_name.clone(),
                minio_cluster_region: db.region().c_region_name(minio_region).clone(),
            });
        }

        let monitoring_cluster = db.loki_cluster().c_monitoring_cluster(loki_cluster);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::LokiMonitoringClusterDoesntExistInRegion {
            loki_cluster: name.clone(),
            loki_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let output_loki_cluster = db.loki_cluster().c_loki_cluster(loki_cluster);
        let output_loki_cluster = l1proj.loki_clusters.pick(
            region, output_loki_cluster
        ).ok_or_else(|| PlatformValidationError::LokiLoggingClusterDoesntExistInRegion {
            loki_cluster: name.clone(),
            loki_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: output_loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        let loki_service_slug = format!("epl-loki-{name}-loki");
        let nomad_job_name = format!("loki-{name}");
        let loki_writer_service_slug = format!("{loki_service_slug}-writer");
        let loki_reader_service_slug = format!("{loki_service_slug}-reader");
        let loki_backend_service_slug = format!("{loki_service_slug}-backend");
        let consul_kv_prefix = format!("nomad-loki/{loki_service_slug}");
        let minio_user = format!("loki_{}", name.replace("-", "_"));
        let loki_writer_consul_service =
            runtime.instantiate_and_seal_consul_service(region, &loki_writer_service_slug);
        let loki_reader_consul_service =
            runtime.instantiate_and_seal_consul_service(region, &loki_reader_service_slug);
        let loki_backend_consul_service =
            runtime.instantiate_and_seal_consul_service(region, &loki_backend_service_slug);
        let loki_writer_memory = runtime.reserve_stateless_memory_mb(
            format!("EPL Loki cluster {name} writer"),
            db.loki_cluster().c_loki_writer_memory_mb(loki_cluster),
        );
        let loki_reader_memory = runtime.reserve_stateless_memory_mb(
            format!("EPL Loki cluster {name} reader"),
            db.loki_cluster().c_loki_reader_memory_mb(loki_cluster),
        );
        let loki_backend_memory = runtime.reserve_stateless_memory_mb(
            format!("EPL Loki cluster {name} backend"),
            db.loki_cluster().c_loki_backend_memory_mb(loki_cluster),
        );
        let res_writer_http_port = runtime.lock_port_all_servers(
            loki_writer_http_port as u16,
            format!("Loki writer http for cluster {name}"),
        )?;
        let res_writer_grpc_port = runtime.lock_port_all_servers(
            loki_writer_grpc_port as u16,
            format!("Loki writer grpc for cluster {name}"),
        )?;
        let res_reader_http_port = runtime.lock_port_all_servers(
            loki_reader_http_port as u16,
            format!("Loki reader http for cluster {name}"),
        )?;
        let res_reader_grpc_port = runtime.lock_port_all_servers(
            loki_reader_grpc_port as u16,
            format!("Loki reader grpc for cluster {name}"),
        )?;
        let res_backend_http_port = runtime.lock_port_all_servers(
            loki_backend_http_port as u16,
            format!("Loki backend http for cluster {name}"),
        )?;
        let res_backend_grpc_port = runtime.lock_port_all_servers(
            loki_backend_grpc_port as u16,
            format!("Loki backend grpc for cluster {name}"),
        )?;

        let mut loki_secrets_builder = runtime.issue_vault_secret(region, &format!("loki/{name}"));
        let mut consul_policy_builder = CustomConsulPolicyBuilder::new(loki_service_slug.clone());
        consul_policy_builder.add_kw_read_write(&consul_kv_prefix);
        let consul_policy = consul_policy_builder.build();
        let consul_token = loki_secrets_builder.request_secret(
            region,
            "consul_token",
            VaultSecretRequest::ConsulTokenWithPolicy {
                policy: consul_policy,
            },
        );
        let minio_password = loki_secrets_builder.fetch_minio_bucket_credentials(
            db,
            "minio_bucket_password",
            &minio_user,
            minio_bucket,
            MinIOBucketPermission::ReadWrite,
        );
        let fin_secrets = loki_secrets_builder.finalize();

        let loki_cfg = generate_loki_config(
            &consul_token,
            &consul_kv_prefix,
            retention_hours,
            minio_bucket_name,
            &format!("epl-minio-{minio_cluster_name}.service.consul:{minio_port}"),
            &minio_user,
            &minio_password,
        );

        let loki_job = runtime.fetch_nomad_job(
            namespace,
            nomad_job_name.clone(),
            region,
            NomadJobKind::Stateless,
            NomadJobStage::SystemJob,
        );
        if loki_job.loki_cluster().is_none() {
            loki_job.set_loki_cluster(output_loki_cluster);
        }
        loki_job.assign_vault_secrets(fin_secrets);
        let writer_tg = loki_job.fetch_task_group(format!("writer"));
        writer_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
        writer_tg.set_count(writer_count);
        writer_tg.try_set_placement(
            db,
            region,
            db.loki_cluster().c_writer_placement(loki_cluster),
            &format!("writers for loki_cluster named {name} in region {region_name}"),
            writer_count,
            l1proj.label_database
        )?;
        writer_tg.add_locked_port("http", res_writer_http_port);
        writer_tg.add_locked_port("grpc", res_writer_grpc_port);
        writer_tg.expose_port_as_tcp_service("http", &loki_writer_consul_service);
        writer_tg.set_service_http_healthcheck(&loki_writer_consul_service, "/ready");
        writer_tg.collect_prometheus_metrics(&loki_writer_consul_service, monitoring_cluster, None);
        let docker_image_loki = image_handle_from_pin(db, workload_architecture, docker_image_pin_loki, "grafana_loki")?;
        let task = writer_tg.fetch_task(
            format!("loki-writer-{name}"),
            docker_image_loki.clone(),
        );
        task.add_memory(loki_writer_memory);
        let cfg_path = task.add_secure_config("config.yml".to_string(), loki_cfg.clone());
        task.set_arguments(vec![
            format!("-config.file={cfg_path}"),
            "-target=write".to_string(),
            format!("-server.http-listen-port={loki_writer_http_port}"),
            "-server.http-listen-address=${meta.private_ip}".to_string(),
            format!("-server.grpc-listen-port={loki_writer_grpc_port}"),
            "-server.grpc-listen-address=${meta.private_ip}".to_string(),
            "-ingester.wal-enabled=false".to_string(),
            format!("-ring.prefix={consul_kv_prefix}/"),
            "-legacy-read-mode=false".to_string(),
            format!(
                "-common.compactor-grpc-address={}:{}",
                loki_backend_consul_service.service_fqdn(),
                loki_backend_grpc_port,
            ),
            "-memberlist.advertise-addr=${meta.private_ip}".to_string(),
        ]);

        let reader_tg = loki_job.fetch_task_group(format!("reader"));
        reader_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
        reader_tg.set_count(reader_count);
        reader_tg.try_set_placement(
            db,
            region,
            db.loki_cluster().c_reader_placement(loki_cluster),
            &format!("readers for loki_cluster named {name} in region {region_name}"),
            reader_count,
            l1proj.label_database
        )?;
        reader_tg.add_locked_port("http", res_reader_http_port);
        reader_tg.add_locked_port("grpc", res_reader_grpc_port);
        reader_tg.expose_port_as_tcp_service("http", &loki_reader_consul_service);
        reader_tg.set_service_http_healthcheck(&loki_reader_consul_service, "/ready");
        reader_tg.collect_prometheus_metrics(&loki_reader_consul_service, monitoring_cluster, None);
        let task = reader_tg.fetch_task(
            format!("loki-reader-{name}"),
            docker_image_loki.clone(),
        );
        task.add_memory(loki_reader_memory);
        let cfg_path = task.add_secure_config("config.yml".to_string(), loki_cfg.clone());
        task.set_arguments(vec![
            format!("-config.file={cfg_path}"),
            "-target=read".to_string(),
            format!("-server.http-listen-port={loki_reader_http_port}"),
            "-server.http-listen-address=${meta.private_ip}".to_string(),
            format!("-server.grpc-listen-port={loki_reader_grpc_port}"),
            "-server.grpc-listen-address=${meta.private_ip}".to_string(),
            format!("-ring.prefix={consul_kv_prefix}/"),
            "-legacy-read-mode=false".to_string(),
            format!(
                "-common.compactor-grpc-address={}:{}",
                loki_backend_consul_service.service_fqdn(),
                loki_backend_grpc_port,
            ),
            "-memberlist.advertise-addr=${meta.private_ip}".to_string(),
        ]);

        let backend_tg = loki_job.fetch_task_group(format!("backend"));
        backend_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
        backend_tg.try_set_placement(
            db,
            region,
            db.loki_cluster().c_backend_placement(loki_cluster),
            &format!("backend for loki_cluster named {name} in region {region_name}"),
            reader_count,
            l1proj.label_database
        )?;
        backend_tg.add_locked_port("http", res_backend_http_port);
        backend_tg.add_locked_port("grpc", res_backend_grpc_port);
        backend_tg.expose_port_as_tcp_service("http", &loki_backend_consul_service);
        backend_tg.set_service_http_healthcheck(&loki_backend_consul_service, "/ready");
        backend_tg.collect_prometheus_metrics(&loki_backend_consul_service, monitoring_cluster, None);
        let task = backend_tg.fetch_task(
            format!("loki-backend-{name}"),
            docker_image_loki,
        );
        task.add_memory(loki_backend_memory);
        let cfg_path = task.add_secure_config("config.yml".to_string(), loki_cfg.clone());
        task.set_arguments(vec![
            format!("-config.file={cfg_path}"),
            "-target=backend".to_string(),
            format!("-server.http-listen-port={loki_backend_http_port}"),
            "-server.http-listen-address=${meta.private_ip}".to_string(),
            format!("-server.grpc-listen-port={loki_backend_grpc_port}"),
            "-server.grpc-listen-address=${meta.private_ip}".to_string(),
            "-legacy-read-mode=false".to_string(),
            "-memberlist.advertise-addr=${meta.private_ip}".to_string(),
        ]);

        loki_tests(db, l1proj, runtime, loki_cluster);
    }

    Ok(())
}


fn loki_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime, cluster: TableRowPointerLokiCluster) {
    let cluster_name = db.loki_cluster().c_cluster_name(cluster);
    let cluster_snake = cluster_name.to_case(convert_case::Case::Snake);
    let region = db.loki_cluster().c_region(cluster);
    let mon = db.loki_cluster().c_monitoring_cluster(cluster);
    let reg_srv = first_region_server(db, region);
    if reg_srv.is_none() {
        return;
    }
    let reg_srv = reg_srv.unwrap();

    let iface = l1proj.consul_network_iface.value(reg_srv);
    let ip = db.network_interface().c_if_ip(*iface);

    runtime.add_integration_test(
        format!("loki_cluster_{cluster_snake}_writer_dns_exist"),
        IntegrationTest::DnsResolutionARecordCount {
            target_servers: vec![format!("{}:53", ip)],
            queries: vec![
                (format!("epl-loki-{cluster_name}-loki-writer.service.consul"), db.loki_cluster().c_loki_writers(cluster) as usize)
            ]
        },
    );

    runtime.add_integration_test(
        format!("loki_cluster_{cluster_snake}_reader_dns_exist"),
        IntegrationTest::DnsResolutionARecordCount {
            target_servers: vec![format!("{}:53", ip)],
            queries: vec![
                (format!("epl-loki-{cluster_name}-loki-reader.service.consul"), db.loki_cluster().c_loki_readers(cluster) as usize)
            ]
        },
    );

    runtime.add_integration_test(
        format!("loki_cluster_{cluster_snake}_read_write_test"),
        IntegrationTest::LokiWriterReaderTest {
            dns_server: format!("{}:53", ip),
            reader_dns_name: format!("epl-loki-{cluster_name}-loki-reader.service.consul"),
            writer_dns_name: format!("epl-loki-{cluster_name}-loki-writer.service.consul"),
            reader_port: db.loki_cluster().c_loki_reader_http_port(cluster),
            writer_port: db.loki_cluster().c_loki_writer_http_port(cluster),
        },
    );

    if Some(cluster) == l1proj.loki_clusters.region_default(region) {
        runtime.add_integration_test(
            format!("loki_cluster_{cluster_snake}_has_journald_stream"),
            IntegrationTest::LokiStreamExists {
                dns_server: format!("{}:53", ip),
                reader_dns_name: format!("epl-loki-{cluster_name}-loki-reader.service.consul"),
                reader_port: db.loki_cluster().c_loki_reader_http_port(cluster),
                stream_identifiers: vec![
                    ("source_type".to_string(), "journald".to_string()),
                ],
            }
        );

        runtime.add_integration_test(
            format!("loki_cluster_{cluster_snake}_has_l1_provisioning_stream"),
            IntegrationTest::LokiStreamExists {
                dns_server: format!("{}:53", ip),
                reader_dns_name: format!("epl-loki-{cluster_name}-loki-reader.service.consul"),
                reader_port: db.loki_cluster().c_loki_reader_http_port(cluster),
                stream_identifiers: vec![
                    ("source_type".to_string(), "l1_provisioning".to_string()),
                ],
            }
        );

        runtime.add_integration_test(
            format!("loki_cluster_{cluster_snake}_has_l2_provisioning_stream"),
            IntegrationTest::LokiStreamExists {
                dns_server: format!("{}:53", ip),
                reader_dns_name: format!("epl-loki-{cluster_name}-loki-reader.service.consul"),
                reader_port: db.loki_cluster().c_loki_reader_http_port(cluster),
                stream_identifiers: vec![
                    ("source_type".to_string(), "l2_provisioning".to_string()),
                ],
            }
        );
    }


    if let Some(mon_c) = l1proj.monitoring_clusters.pick(region, &mon) {
        runtime.add_integration_test(
            format!("loki_cluster_{cluster_snake}_writer_prometheus_metrics_exist"),
            prometheus_metric_exists_test(
                db,
                l1proj,
                mon_c,
                &format!("loki_distributor_ingester_appends_total{{job='epl-loki-{cluster_name}-loki-writer'}}")
            )
        );

        runtime.add_integration_test(
            format!("loki_cluster_{cluster_snake}_reader_prometheus_metrics_exist"),
            prometheus_metric_exists_test(
                db,
                l1proj,
                mon_c,
                &format!("loki_querier_tail_active{{job='epl-loki-{cluster_name}-loki-reader'}}")
            )
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_loki_config(
    consul_acl_token: &VaultSecretHandle,
    kv_prefix: &str,
    retention_hours: i64,
    minio_bucket: &str,
    minio_url: &str,
    minio_user: &str,
    minio_password: &VaultSecretHandle,
) -> String {
    let consul_acl_token = consul_acl_token.template_expression();
    let minio_password = minio_password.template_expression();
    format!(
        r#"
# TODO: enable auth maybe?
auth_enabled: false

limits_config:
    retention_period: {retention_hours}h

common:
  replication_factor: 1
  ring:
    instance_addr: {{{{ env "meta.private_ip" }}}}
    kvstore:
      store: consul
      prefix: {kv_prefix}/
      consul:
        host: 127.0.0.1:8500
        acl_token: {consul_acl_token}

ingester:
  # https://github.com/grafana/loki/issues/8615
  autoforget_unhealthy: true

schema_config:
  configs:
  - from: 2020-05-15
    store: tsdb
    object_store: s3
    schema: v13
    index:
      prefix: index_
      period: 24h

compactor:
  working_directory: /alloc/tmp/compactor
  compaction_interval: 5m

storage_config:
  tsdb_shipper:
    active_index_directory: /alloc/tmp/index
    cache_location: /alloc/tmp/index_cache
  aws:
    s3: s3://{minio_user}:{minio_password}@{minio_url}/{minio_bucket}
    s3forcepathstyle: true
"#
    )
}
