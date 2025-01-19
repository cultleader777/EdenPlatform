use std::{fmt::Write, collections::HashMap};

use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerChKeeperDeployment, TableRowPointerMonitoringCluster, TableRowPointerChDeployment, TableRowPointerChSchema, TableRowPointerRegion},
    static_analysis::{
        server_runtime::{
            ServerRuntime, NomadJobStage, NomadJobKind, epl_architecture_to_nomad_architecture, IntegrationTest, VaultSecretRequest, VaultSecretHandle, ProvisioningResourcePath, ProvisioningScriptTag, ClickhouseDbCredentials, ChAccessKind,
        },
        PlatformValidationError, L1Projections, networking::{region_monitoring_clusters, region_loki_clusters, server_region, prometheus_metric_exists_test, check_servers_regional_distribution}, docker_images::image_handle_from_pin, databases::clickhouse::{parse_clickhouse_ddl_segments, SUPPORTED_TABLE_ENGINES}, bw_compat_types::parser::ValidVersionedStructType,
    },
};

pub fn deploy_clickhouse_instances(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    deploy_clickhouse_keepers(db, runtime, l1proj)?;
    deploy_clickhouse_dbs(db, runtime, l1proj)?;
    Ok(())
}

fn deploy_clickhouse_keepers(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    let job_kind = NomadJobKind::BoundStateful;

    for keeper_depl in db.ch_keeper_deployment().rows_iter() {
        let depl_name = db.ch_keeper_deployment().c_deployment_name(keeper_depl);
        let namespace = db.ch_keeper_deployment().c_namespace(keeper_depl);
        let region = db.ch_keeper_deployment().c_region(keeper_depl);
        let memory_mb = db.ch_keeper_deployment().c_memory_mb(keeper_depl);
        let workload_architecture = db.ch_keeper_deployment().c_workload_architecture(keeper_depl);
        let docker_image = db.ch_keeper_deployment().c_docker_image(keeper_depl);

        let monitoring_cluster = db.ch_keeper_deployment().c_monitoring_cluster(keeper_depl);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::ChKeeperDeploymentMonitoringClusterDoesntExistInRegion {
            ch_keeper_deployment: depl_name.clone(),
            ch_keeper_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = db.ch_keeper_deployment().c_loki_cluster(keeper_depl);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::ChKeeperDeploymentLoggingClusterDoesntExistInRegion {
            ch_keeper_deployment: depl_name.clone(),
            ch_keeper_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        let service_slug = format!("epl-ch-keeper-{depl_name}");
        let nomad_job_name = format!("ch-keeper-{depl_name}");
        let keeper_tcp_port = db.ch_keeper_deployment().c_keeper_port(keeper_depl);
        let keeper_raft_port = db.ch_keeper_deployment().c_raft_port(keeper_depl);
        let prometheus_port = db.ch_keeper_deployment().c_prometheus_port(keeper_depl);
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &service_slug);

        let mut update_delay = 0;
        let replica_rolling_update_delay_seconds = 15;
        let instances = db.ch_keeper_deployment().c_children_ch_keeper_deployment_instance(keeper_depl);
        let instance_count = instances.len();

        if instance_count != 3 && instance_count != 5 {
            return Err(PlatformValidationError::ChKeeperClusterInstancesCountMustBeThreeOrFive {
                ch_keeper_deployment: depl_name.clone(),
                ch_keeper_region: db.region().c_region_name(region).clone(),
                ch_keeper_instance_count: instance_count,
            })
        }

        for depl_child in instances {
            let server_volume = db.ch_keeper_deployment_instance().c_keeper_server(*depl_child);
            let server = db.server_volume().c_parent(server_volume);
            let server_region = server_region(db, server);
            let hostname = db.server().c_hostname(server);
            if region != server_region {
                return Err(PlatformValidationError::ChKeeperDeploymentInstanceIsOutsideSpecifiedRegion {
                    ch_keeper_deployment: depl_name.clone(),
                    ch_keeper_region: db.region().c_region_name(region).clone(),
                    server: hostname.clone(),
                    server_region: db.region().c_region_name(server_region).clone(),
                });
            }

            let instance_id = db.ch_keeper_deployment_instance().c_instance_id(*depl_child);

            let lan_iface = l1proj.consul_network_iface.value(server);
            let lan_ip = db.network_interface().c_if_ip(*lan_iface);
            let keeper_conf = generate_keeper_config(
                keeper_tcp_port, keeper_raft_port, prometheus_port, lan_ip.as_str(),
                instance_id, db, keeper_depl, l1proj,
            );

            let server_lock = runtime.lock_server_with_label(
                db,
                format!("epl-ch-keeper-{hostname}-{depl_name}"),
                server,
            )?;
            let server_data = runtime.fetch_server_data(db, server);
            let volume_lock = server_data.server_volume_write_lock(
                db,
                server_volume,
                format!("Exclusive epl-ch-keeper-{depl_name} database volume lock"),
            )?;

            let mut mem_blocks = Vec::new();
            {
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("Clickhouse Keeper {depl_name}-{instance_id}"),
                    memory_mb,
                )?);
            }

            let mut port_locks = Vec::new();
            {
                port_locks.push((
                    "tcp_port",
                    server_data.lock_port(
                        db,
                        keeper_tcp_port as u16,
                        format!("Clickhouse keeper TCP port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "raft_port",
                    server_data.lock_port(
                        db,
                        keeper_raft_port as u16,
                        format!("Clickhouse keeper RAFT port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "prom_port",
                    server_data.lock_port(
                        db,
                        prometheus_port as u16,
                        format!("Clickhouse keeper Prometheus port for {depl_name}"),
                    )?,
                ));
            }


            let nomad_job =
                runtime.fetch_nomad_job(
                    namespace,
                    nomad_job_name.clone(),
                    region,
                    job_kind,
                    NomadJobStage::SystemJob,
                );

            if nomad_job.loki_cluster().is_none() {
                nomad_job.set_loki_cluster(loki_cluster);
            }

            let depl_tg = nomad_job.fetch_task_group(format!("chk-{instance_id}"));
            depl_tg.set_shutdown_delay_seconds(update_delay);
            update_delay += replica_rolling_update_delay_seconds;
            depl_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
            depl_tg.assign_server_lock(server_lock);
            for (pm, lp) in port_locks {
                depl_tg.add_locked_port(pm, lp);
            }
            depl_tg.expose_port_as_tcp_service("prom_port", &consul_service);
            depl_tg.collect_prometheus_metrics(&consul_service, monitoring_cluster, None);
            let docker_image_clickhouse = image_handle_from_pin(
                db, &workload_architecture, docker_image, "clickhouse"
            )?;

            let task =
                    depl_tg.fetch_task(
                        format!("chk-{depl_name}-{instance_id}"),
                        docker_image_clickhouse,
                    );

            for mb in mem_blocks {
                task.add_memory(mb);
            }

            let cfg_file = task.add_local_file("keeper_config.xml".to_string(), keeper_conf);
            task.bind_volume(volume_lock, "/var/lib/clickhouse".to_string());
            task.set_entrypoint(vec!["/usr/bin/clickhouse-keeper".to_string()]);
            task.set_arguments(vec![
                format!("--config-file={cfg_file}"),
            ]);
        }

        if db.ch_keeper_deployment().c_distribute_over_dcs(keeper_depl) {
            check_servers_regional_distribution(
                db,
                region,
                db.ch_keeper_deployment().c_children_ch_keeper_deployment_instance(keeper_depl).iter().map(|i| {
                    let srv_volume = db.ch_keeper_deployment_instance().c_keeper_server(*i);
                    db.server_volume().c_parent(srv_volume)
                }),
                format!("ch_keeper_deployment=>{depl_name}")
            )?;
        }

        keeper_deployment_tests(db, l1proj, keeper_depl, monitoring_cluster, runtime);
    }

    Ok(())
}

fn deploy_clickhouse_dbs(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    let job_kind = NomadJobKind::BoundStateful;

    // schema migrations
    for region in db.region().rows_iter() {
        let mut schema_prov_scripts: HashMap<TableRowPointerChSchema, ProvisioningResourcePath> =
            HashMap::new();
        for schema in db.ch_schema().rows_iter() {
            if l1proj.ch_schemas_in_region.value(region).contains(&schema) {
                let db_name = db.ch_schema().c_schema_name(schema);
                let fname = format!("up_{db_name}.sh");
                let migration = generate_ch_migration_script(db, schema);
                schema_prov_scripts.insert(
                    schema,
                    runtime.add_provisioning_resource(region, "ch-migrations", fname, migration, true, Vec::default()),
                );
            }
        }
        runtime.add_provisioning_script(
            region,
            ProvisioningScriptTag::SystemResourceProvision,
            "provision-ch-instances.sh",
            generate_ch_init_script(db, region, &schema_prov_scripts, l1proj),
        );
    }

    for ch_depl in db.ch_deployment().rows_iter() {
        let depl_name = db.ch_deployment().c_deployment_name(ch_depl);
        let namespace = db.ch_deployment().c_namespace(ch_depl);
        let region = db.ch_deployment().c_region(ch_depl);
        let workload_architecture = db.ch_deployment().c_workload_architecture(ch_depl);
        let docker_image = db.ch_deployment().c_docker_image(ch_depl);
        let extra_memory_mb = db.ch_deployment().c_extra_memory_mb(ch_depl);
        let tuning = ChTuningArgs {
            mark_cache_size_mb: db.ch_deployment().c_mark_cache_size_mb(ch_depl),
            index_mark_cache_size_mb: db.ch_deployment().c_index_mark_cache_size_mb(ch_depl),
            uncompressed_cache_size_mb: db.ch_deployment().c_uncompressed_cache_size_mb(ch_depl),
            compiled_expression_cache_size_mb: db.ch_deployment().c_compiled_expression_cache_size_mb(ch_depl),
            query_cache_size_mb: db.ch_deployment().c_query_cache_size_mb(ch_depl),
            max_thread_pool_size: db.ch_deployment().c_max_thread_pool_size(ch_depl),
            max_bytes_to_merge_at_max_space_in_pool_mb: db.ch_deployment().c_max_bytes_to_merge_at_max_space_in_pool_mb(ch_depl),
            max_query_execution_time_seconds: db.ch_deployment().c_max_query_execution_time_seconds(ch_depl),
            queue_max_wait_ms: db.ch_deployment().c_queue_max_wait_ms(ch_depl),
            max_concurrent_queries: db.ch_deployment().c_max_concurrent_queries(ch_depl),
            merge_max_block_size: db.ch_deployment().c_merge_max_block_size(ch_depl)
        };

        let monitoring_cluster = db.ch_deployment().c_monitoring_cluster(ch_depl);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::ChDeploymentMonitoringClusterDoesntExistInRegion {
            ch_deployment: depl_name.clone(),
            ch_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = db.ch_deployment().c_loki_cluster(ch_depl);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::ChDeploymentLoggingClusterDoesntExistInRegion {
            ch_deployment: depl_name.clone(),
            ch_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        let service_slug = format!("epl-clickhouse-{depl_name}");
        let nomad_job_name = format!("clickhouse-{depl_name}");
        let native_port = db.ch_deployment().c_native_port(ch_depl);
        let http_port = db.ch_deployment().c_http_port(ch_depl);
        let replication_port = db.ch_deployment().c_replication_port(ch_depl);
        let prometheus_port = db.ch_deployment().c_prometheus_port(ch_depl);
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &service_slug);
        let mut component_secrets_builder = runtime.issue_vault_secret(region, &format!("clickhouse/{depl_name}"));

        let interserver_pass = component_secrets_builder.request_secret(
            region, "interserver_password", VaultSecretRequest::AlphanumericPassword42Symbols
        );
        let admin_pass = component_secrets_builder.request_secret(
            region, "admin_password", VaultSecretRequest::AlphanumericPassword42Symbols
        );

        let mut to_add_ch_access: Vec<(ChAccessKind, ClickhouseDbCredentials)> = Vec::new();
        for schema in db.ch_deployment().c_children_ch_deployment_schemas(ch_depl) {
            let db_name = db.ch_deployment_schemas().c_db_name(*schema);
            let _db_admin_pass = component_secrets_builder.request_secret(
                region, &format!("db_{db_name}_admin"), VaultSecretRequest::AlphanumericPassword42Symbols
            );
            let db_rw_user = format!("db_{db_name}_rw");
            let db_rw_pass = component_secrets_builder.request_secret(
                region, &db_rw_user, VaultSecretRequest::AlphanumericPassword42Symbols
            );
            let db_ro_user = format!("db_{db_name}_ro");
            let db_ro_pass = component_secrets_builder.request_secret(
                region, &db_ro_user, VaultSecretRequest::AlphanumericPassword42Symbols
            );

            to_add_ch_access.push((
                ChAccessKind::ManagedReadWrite(*schema),
                ClickhouseDbCredentials {
                    db_host: consul_service.service_fqdn(),
                    db_http_port: http_port as u16,
                    db_database: db_name.clone(),
                    db_user: db_rw_user,
                    db_password: db_rw_pass.clone(),
                },
            ));

            to_add_ch_access.push((
                ChAccessKind::ManagedReadOnly(*schema),
                ClickhouseDbCredentials {
                    db_host: consul_service.service_fqdn(),
                    db_http_port: http_port as u16,
                    db_database: db_name.clone(),
                    db_user: db_ro_user,
                    db_password: db_ro_pass.clone(),
                },
            ));
        }

        let fin_secrets = component_secrets_builder.finalize();

        let mut update_delay = 0;
        let replica_rolling_update_delay_seconds = 30;

        let nomad_job =
            runtime.fetch_nomad_job(
                namespace,
                nomad_job_name.clone(),
                region,
                job_kind,
                NomadJobStage::SystemJob,
            );
        nomad_job.assign_vault_secrets(fin_secrets);

        for (k, v) in to_add_ch_access {
            runtime.add_ch_access(k, v);
        }

        let instances = db.ch_deployment().c_children_ch_deployment_instance(ch_depl);
        let instance_count = instances.len();
        if instance_count < 2 || instance_count > 10 {
            return Err(PlatformValidationError::ChDeploymentInstancesCountMustBeAtLeastTwoButNoMoreThan10 {
                ch_deployment: depl_name.clone(),
                ch_region: db.region().c_region_name(region).clone(),
                ch_instance_count: instance_count,
                min_instance_count: 2,
                max_instance_count: 10,
            })
        }

        for depl_child in instances {
            let server_volume = db.ch_deployment_instance().c_ch_server(*depl_child);
            let server = db.server_volume().c_parent(server_volume);
            let server_region = server_region(db, server);
            let hostname = db.server().c_hostname(server);
            if region != server_region {
                return Err(PlatformValidationError::ChDeploymentInstanceIsOutsideSpecifiedRegion {
                    ch_deployment: depl_name.clone(),
                    ch_region: db.region().c_region_name(region).clone(),
                    server: hostname.clone(),
                    server_region: db.region().c_region_name(server_region).clone(),
                });
            }

            let instance_id = db.ch_deployment_instance().c_instance_id(*depl_child);

            let lan_iface = l1proj.consul_network_iface.value(server);
            let lan_ip = db.network_interface().c_if_ip(*lan_iface);
            let users_xml_file_contents = default_clickhouse_profile(
                tuning.queue_max_wait_ms,
                tuning.max_query_execution_time_seconds,
                &admin_pass
            );

            let server_lock = runtime.lock_server_with_label(
                db,
                format!("epl-clickhouse-{hostname}-{depl_name}"),
                server,
            )?;
            let server_data = runtime.fetch_server_data(db, server);
            let volume_lock = server_data.server_volume_write_lock(
                db,
                server_volume,
                format!("Exclusive epl-clickhouse-{depl_name} database volume lock"),
            )?;

            let mut mem_blocks = Vec::new();
            {
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("Clickhouse mark_cache_size {depl_name}-{instance_id}"),
                    tuning.mark_cache_size_mb,
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("Clickhouse index_mark_cache_size {depl_name}-{instance_id}"),
                    tuning.index_mark_cache_size_mb,
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("Clickhouse uncompressed_cache_size {depl_name}-{instance_id}"),
                    tuning.uncompressed_cache_size_mb,
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("Clickhouse compiled_expression_cache_size {depl_name}-{instance_id}"),
                    tuning.compiled_expression_cache_size_mb,
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("Clickhouse query_cache_size {depl_name}-{instance_id}"),
                    tuning.query_cache_size_mb,
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("Clickhouse extra memory {depl_name}-{instance_id}"),
                    extra_memory_mb,
                )?);
            }

            let mut total_memory_bytes: i64 = 0;
            for mb in &mem_blocks {
                total_memory_bytes += mb.bytes();
            }

            let ch_conf = generate_clickhouse_config(
                native_port, http_port, replication_port, prometheus_port,
                lan_ip.as_str(), instance_id,
                &interserver_pass,
                db, ch_depl, l1proj, &tuning, "/secrets/users_config.xml",
                total_memory_bytes - 32 * 1024 * 1024,
            );

            let mut port_locks = Vec::new();
            {
                port_locks.push((
                    "native_port",
                    server_data.lock_port(
                        db,
                        native_port as u16,
                        format!("Clickhouse native port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "http_port",
                    server_data.lock_port(
                        db,
                        http_port as u16,
                        format!("Clickhouse HTTP port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "prom_port",
                    server_data.lock_port(
                        db,
                        prometheus_port as u16,
                        format!("Clickhouse Prometheus port for {depl_name}"),
                    )?,
                ));
            }


            let nomad_job =
                runtime.fetch_nomad_job(
                    namespace,
                    nomad_job_name.clone(),
                    region,
                    job_kind,
                    NomadJobStage::SystemJob,
                );

            if nomad_job.loki_cluster().is_none() {
                nomad_job.set_loki_cluster(loki_cluster);
            }

            let depl_tg = nomad_job.fetch_task_group(format!("ch-{instance_id}"));
            depl_tg.set_shutdown_delay_seconds(update_delay);
            update_delay += replica_rolling_update_delay_seconds;
            depl_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
            depl_tg.assign_server_lock(server_lock);
            for (pm, lp) in port_locks {
                depl_tg.add_locked_port(pm, lp);
            }
            depl_tg.expose_port_as_tcp_service("prom_port", &consul_service);
            depl_tg.collect_prometheus_metrics(&consul_service, monitoring_cluster, None);
            let docker_image_clickhouse = image_handle_from_pin(
                db, &workload_architecture, docker_image, "clickhouse"
            )?;

            let task =
                    depl_tg.fetch_task(
                        format!("ch-{depl_name}-{instance_id}"),
                        docker_image_clickhouse,
                    );

            for mb in mem_blocks {
                task.add_memory(mb);
            }

            let _ = task.add_secure_config("users_config.xml".to_string(), users_xml_file_contents);
            let cfg_file = task.add_secure_config("clickhouse_config.xml".to_string(), ch_conf);
            task.add_secure_env_variables(
                "env_vars".to_string(), &[
                    ("CH_ADMIN_PASSWORD", &admin_pass)
                ]
            );
            let entrypoint = task.add_executable_local_file(
                "init".to_string(),
                format!(r#"#!/bin/sh
# helper executable
echo '#!/bin/sh

 clickhouse-client -h {lan_ip} --port {native_port} --password $CH_ADMIN_PASSWORD
' > /usr/local/bin/connect
chmod +x /usr/local/bin/connect

exec /usr/bin/clickhouse-server --config-file={cfg_file}
"#)
            );
            task.bind_volume(volume_lock, "/var/lib/clickhouse".to_string());
            task.set_entrypoint(vec![entrypoint]);
        }

        if db.ch_deployment().c_distribute_over_dcs(ch_depl) {
            check_servers_regional_distribution(
                db,
                region,
                db.ch_deployment().c_children_ch_deployment_instance(ch_depl).iter().map(|i| {
                    let srv_volume = db.ch_deployment_instance().c_ch_server(*i);
                    db.server_volume().c_parent(srv_volume)
                }),
                format!("ch_keeper_deployment=>{depl_name}")
            )?;
        }

        server_deployment_tests(db, l1proj, ch_depl, monitoring_cluster, runtime);
    }

    Ok(())
}

fn generate_keeper_config(
    keeper_tcp_port: i64,
    keeper_raft_port: i64,
    prometheus_port: i64,
    lan_ip: &str,
    server_id: i64,
    db: &Database,
    depl: TableRowPointerChKeeperDeployment,
    l1proj: &L1Projections,
) -> String {
    let mut res = String::new();

    write!(&mut res, r#"
<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>{lan_ip}</listen_host>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>{prometheus_port}</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <keeper_server>
        <tcp_port>{keeper_tcp_port}</tcp_port>
        <server_id>{server_id}</server_id>
        <log_storage_path>/var/lib/clickhouse/coordination/log</log_storage_path>
        <snapshot_storage_path>/var/lib/clickhouse/coordination/snapshots</snapshot_storage_path>
        <enable_reconfiguration>true</enable_reconfiguration>

        <coordination_settings>
            <operation_timeout_ms>10000</operation_timeout_ms>
            <session_timeout_ms>30000</session_timeout_ms>
            <raft_logs_level>information</raft_logs_level>
        </coordination_settings>

        <raft_configuration>
"#).unwrap();

    for instance in db.ch_keeper_deployment().c_children_ch_keeper_deployment_instance(depl) {
        let server_id = db.ch_keeper_deployment_instance().c_instance_id(*instance);
        let srv_disk = db.ch_keeper_deployment_instance().c_keeper_server(*instance);
        let srv = db.server_volume().c_parent(srv_disk);
        let lan_iface = l1proj.consul_network_iface.value(srv);
        let lan_ip = db.network_interface().c_if_ip(*lan_iface);
        write!(&mut res, r#"
            <server>
                <id>{server_id}</id>
                <hostname>{lan_ip}</hostname>
                <port>{keeper_raft_port}</port>
            </server>
"#).unwrap();

    }

    write!(&mut res, r#"
        </raft_configuration>
    </keeper_server>
</clickhouse>
"#).unwrap();

    res
}

fn keeper_deployment_tests(
    db: &Database,
    l1proj: &L1Projections,
    cluster: TableRowPointerChKeeperDeployment,
    mon_cluster: TableRowPointerMonitoringCluster,
    runtime: &mut ServerRuntime
) {
    let depl_name = db.ch_keeper_deployment().c_deployment_name(cluster);
    let job_name = format!("epl-ch-keeper-{depl_name}");
    let name = depl_name.to_case(convert_case::Case::Snake);
    let keeper_port = db.ch_keeper_deployment().c_keeper_port(cluster);
    let servers =
        db.ch_keeper_deployment()
            .c_children_ch_keeper_deployment_instance(cluster)
            .iter()
            .map(|i| {
                let disk = db.ch_keeper_deployment_instance().c_keeper_server(*i);
                db.server_volume().c_parent(disk)
            })
            .collect::<Vec<_>>();

    let sockets = servers.iter().map(|i| {
        let ip = db.network_interface().c_if_ip(*l1proj.consul_network_iface.value(*i));
        format!("{ip}:{keeper_port}")
    }).collect::<Vec<_>>();

    runtime.add_integration_test(
        format!("clickhouse_keeper_{name}_tcp_socket_open"),
        IntegrationTest::TcpSocketsOpen { target_sockets: sockets },
    );

    runtime.add_integration_test(
        format!("clickhouse_keeper_{name}_prometheus_metrics_gathered"),
        prometheus_metric_exists_test(
            db, l1proj, mon_cluster,
            &format!("ClickHouseAsyncMetrics_KeeperZnodeCount{{job=\\\"{job_name}\\\"}}")
        )
    );
}

fn server_deployment_tests(
    db: &Database,
    l1proj: &L1Projections,
    cluster: TableRowPointerChDeployment,
    mon_cluster: TableRowPointerMonitoringCluster,
    runtime: &mut ServerRuntime
) {
    let depl_name = db.ch_deployment().c_deployment_name(cluster);
    let job_name = format!("epl-clickhouse-{depl_name}");

    let name = depl_name.to_case(convert_case::Case::Snake);
    let native_port = db.ch_deployment().c_native_port(cluster);
    let servers =
        db.ch_deployment()
            .c_children_ch_deployment_instance(cluster)
            .iter()
            .map(|i| {
                let disk = db.ch_deployment_instance().c_ch_server(*i);
                db.server_volume().c_parent(disk)
            })
            .collect::<Vec<_>>();

    let sockets = servers.iter().map(|i| {
        let ip = db.network_interface().c_if_ip(*l1proj.consul_network_iface.value(*i));
        format!("{ip}:{native_port}")
    }).collect::<Vec<_>>();

    runtime.add_integration_test(
        format!("clickhouse_server_{name}_native_port_open"),
        IntegrationTest::TcpSocketsOpen { target_sockets: sockets },
    );

    runtime.add_integration_test(
        format!("clickhouse_server_{name}_prometheus_metrics_gathered"),
        prometheus_metric_exists_test(
            db, l1proj, mon_cluster,
            &format!("ClickHouseProfileEvents_FileOpen{{job=\\\"{job_name}\\\"}}")
        )
    );
}

pub fn generate_ch_migration_script(db: &Database, schema: TableRowPointerChSchema) -> String {
    let mut res = String::new();

    res += "#!/bin/sh\n";
    res += "set -e\n";
    res += "\n";

    res += "[ -n \"$CH_DB_URL\" ] || { echo CH_DB_URL environment variable is required; exit 7; }\n";
    res += "\n";

    writeln!(&mut res, "echo \"{}\" | \\", epl_migrations_table()).unwrap();
    writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_DB_URL").unwrap();
    res += "\n";

    for migration in db.ch_schema().c_children_ch_migration(schema) {
        let upg_sql = db.ch_migration().c_upgrade(*migration);
        let logical_time = db.ch_migration().c_time(*migration);
        let segments = parse_clickhouse_ddl_segments(&upg_sql);

        assert!(segments.len() < 1000, "Really?");

        const MIGRATIONS_DELIM: &'static str = "WatUpWitItVanillaFace";

        let mut seg_idx = 0;
        for seg in segments {
            let seg = preprocess_segment(seg);
            assert!(!seg.contains(MIGRATIONS_DELIM), "Really?");
            let logical_time = logical_time * 1000 + seg_idx as i64;
            writeln!(&mut res, "if ! echo \"SELECT logical_time,'already_executed' FROM epl_schema_migrations WHERE logical_time = {logical_time}\" | curl --data-binary @- -s --fail-with-body $CH_DB_URL | grep already_executed").unwrap();
            writeln!(&mut res, "then").unwrap();
            writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'{MIGRATIONS_DELIM}'").unwrap();
            res += &seg;
            writeln!(&mut res, "\n{MIGRATIONS_DELIM}").unwrap();
            writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'{MIGRATIONS_DELIM}'").unwrap();
            writeln!(&mut res, "  INSERT INTO epl_schema_migrations(logical_time) VALUES({logical_time})").unwrap();
            writeln!(&mut res, "\n{MIGRATIONS_DELIM}").unwrap();
            writeln!(&mut res, "fi").unwrap();
            res += "\n";
            seg_idx += 1;
        }
        //let segments = cli
    }

    res
}

pub fn generate_ch_init_script(
    db: &Database,
    region: TableRowPointerRegion,
    schema_map: &HashMap<TableRowPointerChSchema, ProvisioningResourcePath>,
    l1proj: &L1Projections,
) -> String {
    let mut res = String::new();

    res += r#"
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
"#;

    for ch_deployment in db.ch_deployment().rows_iter() {
        let depl_region = db.ch_deployment().c_region(ch_deployment);
        if region != depl_region {
            continue;
        }

        // tests fail otherwise
        if db.ch_deployment().c_children_ch_deployment_instance(ch_deployment).is_empty() {
            continue;
        }

        let first_replica = db.ch_deployment().c_children_ch_deployment_instance(ch_deployment)[0];
        let ch_volume = db.ch_deployment_instance().c_ch_server(first_replica);
        let ch_server = db.server_volume().c_parent(ch_volume);
        let ch_ip = l1proj.consul_network_iface.value(ch_server);
        let ch_ip = db.network_interface().c_if_ip(*ch_ip);
        let deployment_name = db.ch_deployment().c_deployment_name(ch_deployment);
        let http_port = db.ch_deployment().c_http_port(ch_deployment);
        res += "export CHUSER=default\n";
        writeln!(&mut res, "export ADMIN_PASSWORD=$( vault kv get -field=admin_password epl/clickhouse/{deployment_name} )").unwrap();
        writeln!(&mut res, "export CH_URL=\"http://$CHUSER:$ADMIN_PASSWORD@{ch_ip}:{http_port}\"").unwrap();
        writeln!(&mut res, "while ! curl -s $CH_URL/?query=SELECT+1277712 | grep 1277712").unwrap();
        res += "do\n";
        writeln!(&mut res, "    echo Waiting for clickhouse deployment {deployment_name} to be up...").unwrap();
        res += "    sleep 5\n";
        res += "done\n";

        res += "echo Provisioning users and databases...\n";
        for schema in db.ch_deployment().c_children_ch_deployment_schemas(ch_deployment) {
            let db_name = db.ch_deployment_schemas().c_db_name(*schema);
            let admin_user = format!("db_{db_name}_admin");
            let rw_user = format!("db_{db_name}_rw");
            let ro_user = format!("db_{db_name}_ro");

            // create the db
            writeln!(&mut res, "echo \"CREATE DATABASE IF NOT EXISTS {db_name} ON CLUSTER default\" | \\").unwrap();
            writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_URL").unwrap();

            let mut define_user = |user_name: &str| {
                writeln!(&mut res, "export DB_USER_PASSWORD_HASH=$( vault kv get -field={user_name} epl/clickhouse/{deployment_name} | sha256sum | awk '{{print $1}}' )").unwrap();
                writeln!(&mut res, "export DB_USER_NAME='{user_name}'").unwrap();
                writeln!(&mut res, "echo \"CREATE USER IF NOT EXISTS $DB_USER_NAME IDENTIFIED WITH sha256_hash BY '$DB_USER_PASSWORD_HASH'\" | \\").unwrap();
                writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_URL").unwrap();
            };

            define_user(&admin_user);
            define_user(&rw_user);
            define_user(&ro_user);

            let mut db_grant = |user_name: &str, rights: &str| {
                writeln!(&mut res, "echo \"GRANT {rights} ON {db_name}.* TO {user_name}\" | \\").unwrap();
                writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_URL").unwrap();
            };

            for user in &[
                &admin_user,
                &ro_user,
                &rw_user,
            ] {
                db_grant(user, "SELECT, SHOW");
            }

            for user in &[
                &admin_user,
                &rw_user,
            ] {
                db_grant(user, "INSERT, OPTIMIZE");
            }

            for user in &[
                &admin_user,
            ] {
                db_grant(user, "ALTER TABLE, ALTER VIEW, CREATE TABLE, CREATE VIEW, DROP TABLE, DROP VIEW, TRUNCATE");
            }

            writeln!(&mut res, "echo \"GRANT SOURCES, CLUSTER ON *.* TO {admin_user}\" | \\").unwrap();
            writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_URL").unwrap();
            writeln!(&mut res, "echo \"GRANT TABLE ENGINE ON * TO {admin_user}\" | \\").unwrap();
            writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_URL").unwrap();
        }

        writeln!(&mut res, "echo Performing table migrations...").unwrap();
        for schema in db.ch_deployment().c_children_ch_deployment_schemas(ch_deployment) {
            let db_name = db.ch_deployment_schemas().c_db_name(*schema);
            let db_schema = db.ch_deployment_schemas().c_ch_schema(*schema);
            let admin_user = format!("db_{db_name}_admin");
            let upg_script = schema_map.get(&db_schema).unwrap();
            writeln!(&mut res, "TARGET_DB_PASSWORD=$( vault kv get -field=db_{db_name}_admin epl/clickhouse/{deployment_name} )").unwrap();
            writeln!(&mut res, "CH_DB_URL=\"http://{admin_user}:$TARGET_DB_PASSWORD@{ch_ip}:{http_port}/?database={db_name}\" {} &", upg_script.path()).unwrap();
        }

        res += "echo Migrations scheduled, waiting for finish...\n";
        res += "wait\n";

        res += "echo Provisioning NATS consumers...\n";
        for schema in db.ch_deployment().c_children_ch_deployment_schemas(ch_deployment) {
            let db_name = db.ch_deployment_schemas().c_db_name(*schema);
            for import in db.ch_deployment_schemas().c_children_ch_nats_stream_import(*schema) {
                let consumer_name = db.ch_nats_stream_import().c_consumer_name(*import);
                let target_table = db.ch_nats_stream_import().c_into_table(*import);
                let nats_stream = db.ch_nats_stream_import().c_stream(*import);
                let stream_type = db.nats_jetstream_stream().c_stream_type(nats_stream);
                let vtype = l1proj.versioned_types.value(stream_type);
                let last_type = vtype.last().unwrap();
                let types = last_type.the_type().fields.iter().filter_map(|(fname, ftype)| {
                    let ch_type = match &ftype.field_type {
                        ValidVersionedStructType::Bool => "Bool",
                        ValidVersionedStructType::F64 => "Float64",
                        ValidVersionedStructType::I64 => "Int64",
                        ValidVersionedStructType::String => "String",
                        _other => {
                            // unsupported type, this will be checked later after async checks
                            return None;
                        }
                    };
                    return Some(format!("{fname} {ch_type}"));
                }).collect::<Vec<_>>().join(", ");
                let nats_cluster = db.nats_jetstream_stream().c_parent(nats_stream);
                let cluster_name = db.nats_cluster().c_cluster_name(nats_cluster);
                let nats_port = db.nats_cluster().c_nats_clients_port(nats_cluster);
                let consul_fqdn = format!("epl-nats-{cluster_name}.service.consul:{nats_port}");
                let nats_subject = format!("ch_imp.{deployment_name}.{db_name}.{consumer_name}");
                // nats table
                writeln!(&mut res, "echo \"CREATE TABLE IF NOT EXISTS nats_ch_imp_queue_{consumer_name} ( {types} ) ENGINE = NATS settings nats_url = '{consul_fqdn}', nats_queue_group = 'ch_imp_{consumer_name}', nats_subjects = '{nats_subject}', nats_format = 'JSONEachRow' \" | \\").unwrap();
                writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_URL/?database={db_name}").unwrap();
                // mat view
                writeln!(&mut res, "echo \"CREATE MATERIALIZED VIEW IF NOT EXISTS nats_consumer_{consumer_name} TO {target_table} AS SELECT * FROM nats_ch_imp_queue_{consumer_name} \" | \\").unwrap();
                writeln!(&mut res, "  curl --data-binary @- -s --fail-with-body $CH_URL/?database={db_name}").unwrap();
            }
        }
        res += "wait\n";

        res += "echo All migrations ran successfully\n";
    }

    res
}

fn preprocess_segment(mut seg: String) -> String {
    for eng in SUPPORTED_TABLE_ENGINES {
        let wspace = format!(" {eng}");
        assert!(!eng.starts_with(" "));
        if seg.contains(&wspace) {
            seg = seg.replace(&wspace, &format!(" Replicated{eng}"));
        }
    }

    perform_on_cluster_replacements(seg)
}

fn perform_on_cluster_replacements(mut input: String) -> String {
    // only CREATE,DROP,ALTER,RENAME statements need to be considered
    lazy_static! {
        static ref CREATE_TABLE_IF_NOT_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+([a-zA-Z0-9_]+)\s*\(").unwrap();
        static ref CREATE_VIEW_IF_NOT_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*CREATE\s+(OR\s+REPLACE\s+)?VIEW\s+IF\s+NOT\s+EXISTS\s+([a-zA-Z0-9_]+)\s+AS\s+").unwrap();
        static ref CREATE_MAT_VIEW_IF_NOT_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*CREATE\s+MATERIALIZED\s+VIEW\s+IF\s+NOT\s+EXISTS\s+([a-zA-Z0-9_]+)\s+").unwrap();
        static ref DROP_TABLE_IF_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*DROP\s+TABLE\s+IF\s+EXISTS\s+([a-zA-Z0-9_]+)").unwrap();
        static ref DROP_VIEW_IF_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*DROP\s+VIEW\s+IF\s+EXISTS\s+([a-zA-Z0-9_]+)").unwrap();
        static ref ALTER_TABLE_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*ALTER\s+(TEMPORARY\s+)?TABLE\s+([a-zA-Z0-9_]+)").unwrap();
        static ref RENAME_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*RENAME\s+TABLE\s+").unwrap();

        static ref IS_CREATE_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*CREATE\s+(TABLE|VIEW|MATERIALIZED\s+VIEW)\s+").unwrap();
        static ref IS_ALTER_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*ALTER\s+(TABLE|VIEW)\s+").unwrap();
        static ref IS_DROP_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*DROP\s+(TABLE|VIEW)\s+").unwrap();
        static ref IS_RENAME_REGEX: regex::Regex = regex::Regex::new(r"(?im)^\s*RENAME\s+TABLE\s+").unwrap();
    }

    assert!(!input.contains(";"));

    if CREATE_TABLE_IF_NOT_EXISTS_REGEX.is_match(&input) {
        input = CREATE_TABLE_IF_NOT_EXISTS_REGEX.replace_all(&input, "\nCREATE TABLE IF NOT EXISTS $1 ON CLUSTER default (").to_string();
    }

    if CREATE_VIEW_IF_NOT_EXISTS_REGEX.is_match(&input) {
        input = CREATE_VIEW_IF_NOT_EXISTS_REGEX.replace_all(&input, "\nCREATE $1 VIEW IF NOT EXISTS $2 ON CLUSTER default AS ").to_string();
    }

    if CREATE_MAT_VIEW_IF_NOT_EXISTS_REGEX.is_match(&input) {
        input = CREATE_MAT_VIEW_IF_NOT_EXISTS_REGEX.replace_all(&input, "\nCREATE MATERIALIZED VIEW IF NOT EXISTS $1 ON CLUSTER default ").to_string();
    }

    if DROP_TABLE_IF_EXISTS_REGEX.is_match(&input) {
        input = DROP_TABLE_IF_EXISTS_REGEX.replace_all(&input, "\nDROP TABLE IF EXISTS $1 ON CLUSTER default ").to_string();
    }

    if DROP_VIEW_IF_EXISTS_REGEX.is_match(&input) {
        input = DROP_VIEW_IF_EXISTS_REGEX.replace_all(&input, "\nDROP VIEW IF EXISTS $1 ON CLUSTER default ").to_string();
    }

    if ALTER_TABLE_REGEX.is_match(&input) {
        input = ALTER_TABLE_REGEX.replace_all(&input, "\nALTER $1 TABLE $2 ON CLUSTER default ").to_string();
    }

    if RENAME_REGEX.is_match(&input) {
        input += " ON CLUSTER default ";
    }

    // double check with more lax regexes if we didn't screw up
    if IS_CREATE_REGEX.is_match(&input) ||
        IS_ALTER_REGEX.is_match(&input) ||
        IS_DROP_REGEX.is_match(&input) ||
        IS_RENAME_REGEX.is_match(&input)
    {
        assert!(input.contains(" ON CLUSTER default"), "Fail at regex on cluster default replaces... Query unprocessed [{input}]");
    }

    input
}

#[test]
fn test_sql_preprocess_replacements_create_table() {
    pretty_assertions::assert_eq!(
        preprocess_segment(r#"
CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id
"#.to_string()), r#"
CREATE TABLE IF NOT EXISTS foo ON CLUSTER default (
            id Int32,
            a String
          ) ENGINE = ReplicatedMergeTree() ORDER BY id
"#
    );
}

#[test]
fn test_sql_preprocess_replacements_replacing_create_table() {
    pretty_assertions::assert_eq!(
        preprocess_segment(r#"
CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String
          ) ENGINE = ReplacingMergeTree() ORDER BY id
"#.to_string()), r#"
CREATE TABLE IF NOT EXISTS foo ON CLUSTER default (
            id Int32,
            a String
          ) ENGINE = ReplicatedReplacingMergeTree() ORDER BY id
"#
    );
}

#[test]
fn test_sql_preprocess_replacements_create_view() {
    pretty_assertions::assert_eq!(
        preprocess_segment(r#"
CREATE OR REPLACE VIEW IF NOT EXISTS some_view AS
SELECT 1
"#.to_string()), r#"
CREATE OR REPLACE  VIEW IF NOT EXISTS some_view ON CLUSTER default AS SELECT 1
"#
    );
}

#[test]
fn test_sql_preprocess_replacements_create_materialized_view() {
    pretty_assertions::assert_eq!(
        preprocess_segment(r#"
CREATE MATERIALIZED VIEW IF NOT EXISTS some_view TO whatever
SELECT 1
"#.to_string()), r#"
CREATE MATERIALIZED VIEW IF NOT EXISTS some_view ON CLUSTER default TO whatever
SELECT 1
"#
    );
}

#[test]
fn test_sql_preprocess_replacements_drop_table() {
    pretty_assertions::assert_eq!(
        preprocess_segment(r#"
DROP TABLE IF EXISTS some_table"#.to_string()),
        r#"
DROP TABLE IF EXISTS some_table ON CLUSTER default "#
    );
}

#[test]
fn test_sql_preprocess_replacements_alter_table() {
    pretty_assertions::assert_eq!(
        preprocess_segment(r#"
ALTER  TEMPORARY  TABLE some_table DROP COLUMN abc
"#.to_string()), r#"
ALTER TEMPORARY   TABLE some_table ON CLUSTER default  DROP COLUMN abc
"#
    );
}

#[test]
fn test_sql_preprocess_replacements_rename_table() {
    pretty_assertions::assert_eq!(
        preprocess_segment(r#"
RENAME TABLE some_table TO some_table_b, some_table_d TO some_table_e
"#.to_string()), r#"
RENAME TABLE some_table TO some_table_b, some_table_d TO some_table_e
 ON CLUSTER default "#
    );
}

fn epl_migrations_table() -> &'static str {
    "CREATE TABLE IF NOT EXISTS epl_schema_migrations ON CLUSTER default (\
       logical_time Int64, \
       time_started DateTime DEFAULT now() \
     ) ENGINE = ReplicatedReplacingMergeTree \
     ORDER BY logical_time"
}

struct ChTuningArgs {
    mark_cache_size_mb: i64,
    index_mark_cache_size_mb: i64,
    uncompressed_cache_size_mb: i64,
    compiled_expression_cache_size_mb: i64,
    max_thread_pool_size: i64,
    query_cache_size_mb: i64,
    max_bytes_to_merge_at_max_space_in_pool_mb: i64,
    max_concurrent_queries: i64,
    merge_max_block_size: i64,
    queue_max_wait_ms: i64,
    max_query_execution_time_seconds: i64,
}

fn generate_clickhouse_config(
    native_port: i64,
    http_port: i64,
    replication_port: i64,
    prometheus_port: i64,
    lan_ip: &str,
    server_id: i64,
    interserver_pass: &VaultSecretHandle,
    db: &Database,
    depl: TableRowPointerChDeployment,
    l1proj: &L1Projections,
    tuning: &ChTuningArgs,
    users_xml_file: &str,
    total_memory_bytes: i64,
) -> String {
    let mut res = String::new();
    let depl_name = db.ch_deployment().c_deployment_name(depl);
    let keeper_depl = db.ch_deployment().c_keeper(depl);
    let keeper_port = db.ch_keeper_deployment().c_keeper_port(keeper_depl);
    // let's start with only one shard in the beginning
    let shard_name = "01";

    let interserver_pass = interserver_pass.template_expression();
    let mark_cache_size = tuning.mark_cache_size_mb * 1024 * 1024;
    let index_mark_cache_size = tuning.index_mark_cache_size_mb * 1024 * 1024;
    let uncompressed_cache_size = tuning.uncompressed_cache_size_mb * 1024 * 1024;
    let compiled_expression_cache_size = tuning.compiled_expression_cache_size_mb * 1024 * 1024;
    let query_cache_size = tuning.query_cache_size_mb * 1024 * 1024;
    let max_thread_pool_size = tuning.max_thread_pool_size;
    let max_concurrent_queries = tuning.max_concurrent_queries;
    let merge_max_block_size = tuning.merge_max_block_size;
    let max_bytes_to_merge_at_max_space_in_pool = tuning.max_bytes_to_merge_at_max_space_in_pool_mb * 1024 * 1024;
    let zk_path_prefix = format!("/ch-{depl_name}");

    write!(&mut res, r#"
<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>{lan_ip}</listen_host>
    <http_port>{http_port}</http_port>
    <tcp_port>{native_port}</tcp_port>
    <interserver_http_port>{replication_port}</interserver_http_port>
    <interserver_http_host>{lan_ip}</interserver_http_host>
    <!-- decrease idle CPU usage https://github.com/ClickHouse/ClickHouse/issues/60016 -->
    <asynchronous_metrics_update_period_s>60</asynchronous_metrics_update_period_s>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>{prometheus_port}</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <user_directories>
        <users_xml>
            <path>{users_xml_file}</path>
        </users_xml>
        <replicated>
            <zookeeper_path>{zk_path_prefix}/access/</zookeeper_path>
        </replicated>
    </user_directories>

    <zookeeper>
        <sessions_path>{zk_path_prefix}/sessions</sessions_path>
"#).unwrap();

    for keeper_instance in db.ch_keeper_deployment().c_children_ch_keeper_deployment_instance(keeper_depl) {
        let srv_disk = db.ch_keeper_deployment_instance().c_keeper_server(*keeper_instance);
        let srv = db.server_volume().c_parent(srv_disk);
        let lan_iface = l1proj.consul_network_iface.value(srv);
        let lan_ip = db.network_interface().c_if_ip(*lan_iface);
        write!(&mut res, r#"
            <node>
                <host>{lan_ip}</host>
                <port>{keeper_port}</port>
            </node>
"#).unwrap();

    }

    write!(&mut res, r#"
    </zookeeper>

    <macros>
        <shard>{shard_name}</shard>
        <replica>{depl_name}-{shard_name}-{server_id}</replica>
    </macros>

    <default_replica_path>{zk_path_prefix}/tables/{{database}}/{{table}}</default_replica_path>
    <default_replica_name>{{replica}}</default_replica_name>

    <remote_servers>
        <default>
            <shard>
                <!-- Optional. Whether to write data to just one of the replicas. Default: false (write data to all replicas). -->
                <internal_replication>true</internal_replication>
"#).unwrap();

    for ch_instance in db.ch_deployment().c_children_ch_deployment_instance(depl) {
        let srv_disk = db.ch_deployment_instance().c_ch_server(*ch_instance);
        let srv = db.server_volume().c_parent(srv_disk);
        let lan_iface = l1proj.consul_network_iface.value(srv);
        let lan_ip = db.network_interface().c_if_ip(*lan_iface);
        write!(&mut res, r#"
                <replica>
                    <host>{lan_ip}</host>
                    <port>{native_port}</port>
                </replica>
"#).unwrap();
    }

    write!(&mut res, r#"
            </shard>
        </default>
    </remote_servers>

    <!-- You can specify credentials for authenthication between replicas.
         This is required when interserver_https_port is accessible from untrusted networks,
         and also recommended to avoid SSRF attacks from possibly compromised services in your network.
      -->
    <interserver_http_credentials>
        <user>interserver</user>
        <password>{interserver_pass}</password>
    </interserver_http_credentials>

    <max_connections>4096</max_connections>

    <!-- For 'Connection: keep-alive' in HTTP 1.1 -->
    <keep_alive_timeout>10</keep_alive_timeout>

    <!-- The maximum number of query processing threads, excluding threads for retrieving data from remote servers, allowed to run all queries.
         This is not a hard limit. In case if the limit is reached the query will still get at least one thread to run.
         Query can upscale to desired number of threads during execution if more threads become available.
    -->
    <concurrent_threads_soft_limit_num>0</concurrent_threads_soft_limit_num>
    <concurrent_threads_soft_limit_ratio_to_cores>2</concurrent_threads_soft_limit_ratio_to_cores>

    <!-- Maximum number of concurrent queries. -->
    <max_concurrent_queries>{max_concurrent_queries}</max_concurrent_queries>

    <!-- Maximum memory usage (resident set size) for server process.
         Zero value or unset means default. Default is "max_server_memory_usage_to_ram_ratio" of available physical RAM.
         If the value is larger than "max_server_memory_usage_to_ram_ratio" of available physical RAM, it will be cut down.

         The constraint is checked on query execution time.
         If a query tries to allocate memory and the current memory usage plus allocation is greater
          than specified threshold, exception will be thrown.

         It is not practical to set this constraint to small values like just a few gigabytes,
          because memory allocator will keep this amount of memory in caches and the server will deny service of queries.
      -->
    <max_server_memory_usage>{total_memory_bytes}</max_server_memory_usage>

    <!-- Maximum number of threads in the Global thread pool.
    This will default to a maximum of 10000 threads if not specified.
    This setting will be useful in scenarios where there are a large number
    of distributed queries that are running concurrently but are idling most
    of the time, in which case a higher number of threads might be required.
    -->

    <max_thread_pool_size>{max_thread_pool_size}</max_thread_pool_size>

    <!-- Configure other thread pools: -->

    <!-- Enables asynchronous loading of databases and tables to speedup server startup.
         Queries to not yet loaded entity will be blocked until load is finished.
      -->
    <!-- <async_load_databases>true</async_load_databases> -->

    <!-- On memory constrained environments you may have to set this to value larger than 1.
      -->
    <max_server_memory_usage_to_ram_ratio>0.9</max_server_memory_usage_to_ram_ratio>

    <!-- Simple server-wide memory profiler. Collect a stack trace at every peak allocation step (in bytes).
         Data will be stored in system.trace_log table with query_id = empty string.
         Zero means disabled.
      -->
    <total_memory_profiler_step>0</total_memory_profiler_step>

    <!-- Collect random allocations and deallocations and write them into system.trace_log with 'MemorySample' trace_type.
         The probability is for every alloc/free regardless to the size of the allocation.
         Note that sampling happens only when the amount of untracked memory exceeds the untracked memory limit,
          which is 4 MiB by default but can be lowered if 'total_memory_profiler_step' is lowered.
         You may want to set 'total_memory_profiler_step' to 1 for extra fine grained sampling.
      -->
    <total_memory_tracker_sample_probability>0</total_memory_tracker_sample_probability>

    <!-- Set limit on number of open files (default: maximum). This setting makes sense on Mac OS X because getrlimit() fails to retrieve
         correct maximum value. -->
    <!-- <max_open_files>262144</max_open_files> -->

    <!-- Size of cache of uncompressed blocks of data, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         Cache is used when 'use_uncompressed_cache' user setting turned on (off by default).
         Uncompressed cache is advantageous only for very short queries and in rare cases.

         Note: uncompressed cache can be pointless for lz4, because memory bandwidth
         is slower than multi-core decompression on some server configurations.
         Enabling it can sometimes paradoxically make queries slower.
      -->
    <uncompressed_cache_size>{uncompressed_cache_size}</uncompressed_cache_size>

    <!-- Approximate size of mark cache, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         You should not lower this value.
      -->
    <mark_cache_size>{mark_cache_size}</mark_cache_size>

    <!-- For marks of secondary indices.
      -->
    <index_mark_cache_size>{index_mark_cache_size}</index_mark_cache_size>

    <!-- If you enable the `min_bytes_to_use_mmap_io` setting,
         the data in MergeTree tables can be read with mmap to avoid copying from kernel to userspace.
         It makes sense only for large files and helps only if data reside in page cache.
         To avoid frequent open/mmap/munmap/close calls (which are very expensive due to consequent page faults)
         and to reuse mappings from several threads and queries,
         the cache of mapped files is maintained. Its size is the number of mapped regions (usually equal to the number of mapped files).
         The amount of data in mapped files can be monitored
         in system.metrics, system.metric_log by the MMappedFiles, MMappedFileBytes metrics
         and in system.asynchronous_metrics, system.asynchronous_metrics_log by the MMapCacheCells metric,
         and also in system.events, system.processes, system.query_log, system.query_thread_log, system.query_views_log by the
         CreatedReadBufferMMap, CreatedReadBufferMMapFailed, MMappedFileCacheHits, MMappedFileCacheMisses events.
         Note that the amount of data in mapped files does not consume memory directly and is not accounted
         in query or server memory usage - because this memory can be discarded similar to OS page cache.
         The cache is dropped (the files are closed) automatically on removal of old parts in MergeTree,
         also it can be dropped manually by the SYSTEM DROP MMAP CACHE query.
      -->
    <mmap_cache_size>1000</mmap_cache_size>

    <!-- Cache size in bytes for compiled expressions.-->
    <compiled_expression_cache_size>{compiled_expression_cache_size}</compiled_expression_cache_size>

    <!-- Cache size in elements for compiled expressions.-->
    <compiled_expression_cache_elements_size>10000</compiled_expression_cache_elements_size>

    <!-- Cache path for custom (created from SQL) cached disks -->
    <custom_cached_disks_base_directory>/var/lib/clickhouse/caches/</custom_cached_disks_base_directory>

    <validate_tcp_client_information>false</validate_tcp_client_information>

    <!-- Path to data directory, with trailing slash. -->
    <path>/var/lib/clickhouse/</path>

    <!-- Path to temporary data for processing hard queries. -->
    <tmp_path>/var/lib/clickhouse/tmp/</tmp_path>

    <!-- Disable AuthType plaintext_password and no_password for ACL. -->
    <allow_plaintext_password>1</allow_plaintext_password>
    <allow_no_password>1</allow_no_password>
    <allow_implicit_no_password>1</allow_implicit_no_password>

    <!-- When a user does not specify a password type in the CREATE USER query, the default password type is used.
         Accepted values are: 'plaintext_password', 'sha256_password', 'double_sha1_password', 'bcrypt_password'.
      -->
    <default_password_type>sha256_password</default_password_type>

    <!-- Work factor for bcrypt_password authentication type-->
    <bcrypt_workfactor>12</bcrypt_workfactor>

    <!-- Directory with user provided files that are accessible by 'file' table function. -->
    <user_files_path>/var/lib/clickhouse/user_files/</user_files_path>

    <!-- Default profile of settings. -->
    <default_profile>default</default_profile>

    <!-- Comma-separated list of prefixes for user-defined settings.
         The server will allow to set these settings, and retrieve them with the getSetting function.
         They are also logged in the query_log, similarly to other settings, but have no special effect.
         The "SQL_" prefix is introduced for compatibility with MySQL - these settings are being set by Tableau.
    -->
    <custom_settings_prefixes>SQL_</custom_settings_prefixes>
    <default_database>default</default_database>

    <timezone>UTC</timezone>

    <!-- You can specify umask here (see "man umask"). Server will apply it on startup.
         Number is always parsed as octal. Default umask is 027 (other users cannot read logs, data files, etc; group can only read).
    -->
    <!-- <umask>022</umask> -->

    <!-- Perform mlockall after startup to lower first queries latency
          and to prevent clickhouse executable from being paged out under high IO load.
         Enabling this option is recommended but will lead to increased startup time for up to a few seconds.
    -->
    <mlock_executable>true</mlock_executable>

    <!-- Reallocate memory for machine code ("text") using huge pages. Highly experimental. -->
    <remap_executable>false</remap_executable>

    <!-- Substitutions for parameters of replicated tables.
          Optional. If you don't use replicated tables, you could omit that.

         See https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication/#creating-replicated-tables
      -->
    <!-- Replica group name for database Replicated.
          The cluster created by Replicated database will consist of replicas in the same group.
          DDL queries will only wail for the replicas in the same group.
          Empty by default.
    -->
    <!--
    <replica_group_name><replica_group_name>
    -->


    <!-- Reloading interval for embedded dictionaries, in seconds. Default: 3600. -->
    <builtin_dictionaries_reload_interval>3600</builtin_dictionaries_reload_interval>


    <!-- Maximum session timeout, in seconds. Default: 3600. -->
    <max_session_timeout>3600</max_session_timeout>

    <!-- Default session timeout, in seconds. Default: 60. -->
    <default_session_timeout>60</default_session_timeout>

    <!-- Sending data to Graphite for monitoring. Several sections can be defined. -->
    <!--
        interval - send every X second
        root_path - prefix for keys
        hostname_in_path - append hostname to root_path (default = true)
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Serve endpoint for Prometheus monitoring. -->
    <!--
        endpoint - mertics path (relative to root, statring with "/")
        port - port to setup server. If not defined or 0 than http_port used
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Query log. Used only for queries with setting log_queries = 1. -->
    <query_log>
        <!-- What table to insert data. If table is not exist, it will be created.
             When query log structure is changed after system update,
              then old table will be renamed and new table will be created automatically.
        -->
        <database>system</database>
        <table>query_log</table>
        <!--
            PARTITION BY expr: https://clickhouse.com/docs/en/table_engines/mergetree-family/custom_partitioning_key/
            Example:
                event_date
                toMonday(event_date)
                toYYYYMM(event_date)
                toStartOfHour(event_time)
        -->
        <partition_by>toYYYYMM(event_date)</partition_by>
        <!--
            Table TTL specification: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree/#mergetree-table-ttl
            Example:
                event_date + INTERVAL 1 WEEK
                event_date + INTERVAL 7 DAY DELETE
                event_date + INTERVAL 2 WEEK TO DISK 'bbb'

        <ttl>event_date + INTERVAL 30 DAY DELETE</ttl>
        -->

        <!--
            ORDER BY expr: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree#order_by
            Example:
                event_date, event_time
                event_date, type, query_id
                event_date, event_time, initial_query_id

        <order_by>event_date, event_time, initial_query_id</order_by>
        -->

        <!-- Instead of partition_by, you can provide full engine expression (starting with ENGINE = ) with parameters,
             Example: <engine>ENGINE = MergeTree PARTITION BY toYYYYMM(event_date) ORDER BY (event_date, event_time) SETTINGS index_granularity = 1024</engine>
          -->

        <!-- Interval of flushing data. -->
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <!-- Maximal size in lines for the logs. When non-flushed logs amount reaches max_size, logs dumped to the disk. -->
        <max_size_rows>1048576</max_size_rows>
        <!-- Pre-allocated size in lines for the logs. -->
        <reserved_size_rows>8192</reserved_size_rows>
        <!-- Lines amount threshold, reaching it launches flushing logs to the disk in background. -->
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>

        <!-- example of using a different storage policy for a system table -->
        <!-- storage_policy>local_ssd</storage_policy -->
    </query_log>

    <!-- Trace log. Stores stack traces collected by query profilers.
         See query_profiler_real_time_period_ns and query_profiler_cpu_time_period_ns settings. -->
    <trace_log>
        <database>system</database>
        <table>trace_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>
    </trace_log>

    <!-- Query thread log. Has information about all threads participated in query execution.
         Used only for queries with setting log_query_threads = 1. -->
    <query_thread_log>
        <database>system</database>
        <table>query_thread_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </query_thread_log>

    <!-- Query views log. Has information about all dependent views associated with a query.
         Used only for queries with setting log_query_views = 1. -->
    <query_views_log>
        <database>system</database>
        <table>query_views_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </query_views_log>

    <!-- Uncomment if use part log.
         Part log contains information about all actions with parts in MergeTree tables (creation, deletion, merges, downloads).-->
    <part_log>
        <database>system</database>
        <table>part_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </part_log>

    <!-- Uncomment to write text log into table.
         Text log contains all information from usual server log but stores it in structured and efficient way.
         The level of the messages that goes to the table can be limited (<level>), if not specified all messages will go to the table.
    <text_log>
        <database>system</database>
        <table>text_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <level></level>
    </text_log>
    -->

    <!-- Metric log contains rows with current values of ProfileEvents, CurrentMetrics collected with "collect_interval_milliseconds" interval. -->
    <metric_log>
        <database>system</database>
        <table>metric_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <collect_interval_milliseconds>1000</collect_interval_milliseconds>
        <flush_on_crash>false</flush_on_crash>
    </metric_log>

    <!--
        Asynchronous metric log contains values of metrics from
        system.asynchronous_metrics.
    -->
    <asynchronous_metric_log>
        <database>system</database>
        <table>asynchronous_metric_log</table>
        <flush_interval_milliseconds>7000</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </asynchronous_metric_log>

    <!--
        OpenTelemetry log contains OpenTelemetry trace spans.
    -->
    <opentelemetry_span_log>
        <!--
            The default table creation code is insufficient, this <engine> spec
            is a workaround. There is no 'event_time' for this log, but two times,
            start and finish. It is sorted by finish time, to avoid inserting
            data too far away in the past (probably we can sometimes insert a span
            that is seconds earlier than the last span in the table, due to a race
            between several spans inserted in parallel). This gives the spans a
            global order that we can use to e.g. retry insertion into some external
            system.
        -->
        <engine>
            engine MergeTree
            partition by toYYYYMM(finish_date)
            order by (finish_date, finish_time_us, trace_id)
        </engine>
        <database>system</database>
        <table>opentelemetry_span_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </opentelemetry_span_log>


    <!-- Crash log. Stores stack traces for fatal errors.
         This table is normally empty. -->
    <crash_log>
        <database>system</database>
        <table>crash_log</table>

        <partition_by />
        <flush_interval_milliseconds>1000</flush_interval_milliseconds>
        <max_size_rows>1024</max_size_rows>
        <reserved_size_rows>1024</reserved_size_rows>
        <buffer_size_rows_flush_threshold>512</buffer_size_rows_flush_threshold>
        <flush_on_crash>true</flush_on_crash>
    </crash_log>

    <!-- Session log. Stores user log in (successful or not) and log out events.

        Note: session log has known security issues and should not be used in production.
    -->
    <!-- <session_log>
        <database>system</database>
        <table>session_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </session_log> -->

    <!-- Profiling on Processors level. -->
    <processors_profile_log>
        <database>system</database>
        <table>processors_profile_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </processors_profile_log>

    <!-- Log of asynchronous inserts. It allows to check status
         of insert query in fire-and-forget mode.
    -->
    <asynchronous_insert_log>
        <database>system</database>
        <table>asynchronous_insert_log</table>

        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <partition_by>event_date</partition_by>
        <ttl>event_date + INTERVAL 3 DAY</ttl>
    </asynchronous_insert_log>

    <!-- Backup/restore log.
    -->
    <backup_log>
        <database>system</database>
        <table>backup_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </backup_log>

    <!-- Storage S3Queue log.
    -->
    <s3queue_log>
        <database>system</database>
        <table>s3queue_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </s3queue_log>

    <!-- Blob storage object operations log.
    -->
    <blob_storage_log>
        <database>system</database>
        <table>blob_storage_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <ttl>event_date + INTERVAL 30 DAY</ttl>
    </blob_storage_log>

    <!-- <top_level_domains_path>/var/lib/clickhouse/top_level_domains/</top_level_domains_path> -->
    <!-- Custom TLD lists.
         Format: <name>/path/to/file</name>

         Changes will not be applied w/o server restart.
         Path to the list is under top_level_domains_path (see above).
    -->
    <top_level_domains_lists>
        <!--
        <public_suffix_list>/path/to/public_suffix_list.dat</public_suffix_list>
        -->
    </top_level_domains_lists>

    <!-- Configuration of external dictionaries. See:
         https://clickhouse.com/docs/en/sql-reference/dictionaries/external-dictionaries/external-dicts
    -->
    <dictionaries_config>*_dictionary.*ml</dictionaries_config>

    <!-- Load dictionaries lazily, i.e. a dictionary will be loaded when it's used for the first time.
         "false" means ClickHouse will start loading dictionaries immediately at startup.
    -->
    <dictionaries_lazy_load>true</dictionaries_lazy_load>

    <!-- Wait at startup until all the dictionaries finish their loading (successfully or not)
         before receiving any connections. Affects dictionaries only if "dictionaries_lazy_load" is false.
         Setting this to false can make ClickHouse start faster, however some queries can be executed slower.
    -->
    <wait_dictionaries_load_at_startup>true</wait_dictionaries_load_at_startup>

    <!-- Configuration of user defined executable functions -->
    <user_defined_executable_functions_config>*_function.*ml</user_defined_executable_functions_config>

    <!-- Path in ZooKeeper to store user-defined SQL functions created by the command CREATE FUNCTION.
     If not specified they will be stored locally. -->
    <user_defined_zookeeper_path>{zk_path_prefix}/user_defined</user_defined_zookeeper_path>

    <!-- Uncomment if you want data to be compressed 30-100% better.
         Don't do that if you just started using ClickHouse.
      -->
    <!--
    <compression>
        <!- - Set of variants. Checked in order. Last matching case wins. If nothing matches, lz4 will be used. - ->
        <case>

            <!- - Conditions. All must be satisfied. Some conditions may be omitted. - ->
            <min_part_size>10000000000</min_part_size>        <!- - Min part size in bytes. - ->
            <min_part_size_ratio>0.01</min_part_size_ratio>   <!- - Min size of part relative to whole table size. - ->

            <!- - What compression method to use. - ->
            <method>zstd</method>
        </case>
    </compression>
    -->

    <!-- Allow to execute distributed DDL queries (CREATE, DROP, ALTER, RENAME) on cluster.
         Works only if ZooKeeper is enabled. Comment it if such functionality isn't required. -->
    <distributed_ddl>
        <!-- Path in ZooKeeper to queue with DDL queries -->
        <path>{zk_path_prefix}/task_queue/ddl</path>

        <!-- Settings from this profile will be used to execute DDL queries -->
        <!-- <profile>default</profile> -->

        <!-- Controls how much ON CLUSTER queries can be run simultaneously. -->
        <!-- <pool_size>1</pool_size> -->

        <!--
             Cleanup settings (active tasks will not be removed)
        -->

        <!-- Controls task TTL (default 1 week) -->
        <!-- <task_max_lifetime>604800</task_max_lifetime> -->

        <!-- Controls how often cleanup should be performed (in seconds) -->
        <!-- <cleanup_delay_period>60</cleanup_delay_period> -->

        <!-- Controls how many tasks could be in the queue -->
        <!-- <max_tasks_in_queue>1000</max_tasks_in_queue> -->

        <!-- Host name of the current node. If specified, will only compare and not resolve hostnames inside the DDL tasks -->
        <host_name>{lan_ip}</host_name>
    </distributed_ddl>

    <!-- Settings to fine-tune MergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <merge_tree>
        <number_of_free_entries_in_pool_to_lower_max_size_of_merge>0</number_of_free_entries_in_pool_to_lower_max_size_of_merge>
        <!-- <max_suspicious_broken_parts>5</max_suspicious_broken_parts> -->
        <!--
        Choose a value between 1024 and 4096.
        The default is 8192.
        -->
        <merge_max_block_size>{merge_max_block_size}</merge_max_block_size>
        <max_bytes_to_merge_at_max_space_in_pool>{max_bytes_to_merge_at_max_space_in_pool}</max_bytes_to_merge_at_max_space_in_pool>
    </merge_tree>

    <!-- Settings to fine-tune ReplicatedMergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <!--
    <replicated_merge_tree>
        <max_replicated_fetches_network_bandwidth>1000000000</max_replicated_fetches_network_bandwidth>
    </replicated_merge_tree>
    -->

    <!-- Settings to fine-tune Distributed tables. See documentation in source code, in DistributedSettings.h -->
    <!--
    <distributed>
        <flush_on_detach>false</flush_on_detach>
    </distributed>
    -->

    <!-- Protection from accidental DROP.
         If size of a MergeTree table is greater than max_table_size_to_drop (in bytes) than table could not be dropped with any DROP query.
         If you want do delete one table and don't want to change clickhouse-server config, you could create special file <clickhouse-path>/flags/force_drop_table and make DROP once.
         By default max_table_size_to_drop is 50GB; max_table_size_to_drop=0 allows to DROP any tables.
         The same for max_partition_size_to_drop.
         Uncomment to disable protection.
    -->
    <!-- <max_table_size_to_drop>0</max_table_size_to_drop> -->
    <!-- <max_partition_size_to_drop>0</max_partition_size_to_drop> -->

    <!-- Example of parameters for GraphiteMergeTree table engine -->

    <!-- Directory in <clickhouse-path> containing schema files for various input formats.
         The directory will be created if it doesn't exist.
      -->
    <format_schema_path>/var/lib/clickhouse/format_schemas/</format_schema_path>

    <!-- Directory containing the proto files for the well-known Protobuf types.
      -->
    <google_protos_path>/usr/share/clickhouse/protos/</google_protos_path>

    <!-- Configuration for the query cache -->
    <query_cache>
        <max_size_in_bytes>{query_cache_size}</max_size_in_bytes>
        <max_entries>1024</max_entries>
        <max_entry_size_in_bytes>1048576</max_entry_size_in_bytes>
        <max_entry_size_in_rows>30000000</max_entry_size_in_rows>
    </query_cache>

    <backups>
        <allowed_path>backups</allowed_path>

        <!-- If the BACKUP command fails and this setting is true then the files
             copied before the failure will be removed automatically.
        -->
        <remove_backup_files_after_failure>true</remove_backup_files_after_failure>
    </backups>

    <!-- This allows to disable exposing addresses in stack traces for security reasons.
         Please be aware that it does not improve security much, but makes debugging much harder.
         The addresses that are small offsets from zero will be displayed nevertheless to show nullptr dereferences.
         Regardless of this configuration, the addresses are visible in the system.stack_trace and system.trace_log tables
         if the user has access to these tables.
         I don't recommend to change this setting.
    <show_addresses_in_stack_traces>false</show_addresses_in_stack_traces>
    -->

</clickhouse>
"#).unwrap();

    res
}

fn default_clickhouse_profile(
    queue_max_wait_ms: i64,
    max_execution_time_seconds: i64,
    admin_password: &VaultSecretHandle,
) -> String {
    // this is reasonably safe because this entire xml is stored in /secrets only in memory
    // we can add extra admin passwords where only users know sha256sum and not even EPL knows the password
    let admin_password = admin_password.template_expression();
    format!(r#"
<clickhouse>
    <!-- Profiles of settings. -->
    <profiles>
        <!-- Default settings. -->
        <default>
        </default>

        <!-- Profile that allows only read queries. -->
        <readonly>
            <readonly>1</readonly>
        </readonly>
    </profiles>

    <!-- Users and ACL. -->
    <users>
        <!-- If user name was not specified, 'default' user is used. -->
        <default>
            <!-- See also the files in users.d directory where the password can be overridden.

                 Password could be specified in plaintext or in SHA256 (in hex format).

                 If you want to specify password in plaintext (not recommended), place it in 'password' element.
                 Example: <password>qwerty</password>.
                 Password could be empty.

                 If you want to specify SHA256, place it in 'password_sha256_hex' element.
                 Example: <password_sha256_hex>65e84be33532fb784c48129675f9eff3a682b27168c0ea744b2cf58ee02337c5</password_sha256_hex>
                 Restrictions of SHA256: impossibility to connect to ClickHouse using MySQL JS client (as of July 2019).

                 If you want to specify double SHA1, place it in 'password_double_sha1_hex' element.
                 Example: <password_double_sha1_hex>e395796d6546b1b65db9d665cd43f0e858dd4303</password_double_sha1_hex>

                 If you want to specify a previously defined LDAP server (see 'ldap_servers' in the main config) for authentication,
                  place its name in 'server' element inside 'ldap' element.
                 Example: <ldap><server>my_ldap_server</server></ldap>

                 If you want to authenticate the user via Kerberos (assuming Kerberos is enabled, see 'kerberos' in the main config),
                  place 'kerberos' element instead of 'password' (and similar) elements.
                 The name part of the canonical principal name of the initiator must match the user name for authentication to succeed.
                 You can also place 'realm' element inside 'kerberos' element to further restrict authentication to only those requests
                  whose initiator's realm matches it.
                 Example: <kerberos />
                 Example: <kerberos><realm>EXAMPLE.COM</realm></kerberos>

                 How to generate decent password:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha256sum | tr -d '-'
                 In first line will be password and in second - corresponding SHA256.

                 How to generate double SHA1:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha1sum | tr -d '-' | xxd -r -p | sha1sum | tr -d '-'
                 In first line will be password and in second - corresponding double SHA1.
            -->
            <password>{admin_password}</password>

            <!-- List of networks with open access.

                 To open access from everywhere, specify:
                    <ip>::/0</ip>

                 To open access only from localhost, specify:
                    <ip>::1</ip>
                    <ip>127.0.0.1</ip>

                 Each element of list has one of the following forms:
                 <ip> IP-address or network mask. Examples: 213.180.204.3 or 10.0.0.1/8 or 10.0.0.1/255.255.255.0
                     2a02:6b8::3 or 2a02:6b8::3/64 or 2a02:6b8::3/ffff:ffff:ffff:ffff::.
                 <host> Hostname. Example: server01.clickhouse.com.
                     To check access, DNS query is performed, and all received addresses compared to peer address.
                 <host_regexp> Regular expression for host names. Example, ^server\d\d-\d\d-\d\.clickhouse\.com$
                     To check access, DNS PTR query is performed for peer address and then regexp is applied.
                     Then, for result of PTR query, another DNS query is performed and all received addresses compared to peer address.
                     Strongly recommended that regexp is ends with $
                 All results of DNS requests are cached till server restart.
            -->
            <networks>
                <!-- eden platform subnet -->
                <ip>10.0.0.0/8</ip>
            </networks>

            <!-- Settings profile for user. -->
            <profile>default</profile>

            <!-- Quota for user. -->
            <quota>default</quota>

            <!-- User can create other users and grant rights to them. -->
            <access_management>1</access_management>

            <!-- User can manipulate named collections. -->
            <named_collection_control>1</named_collection_control>

            <!-- User permissions can be granted here -->
            <!--
            <grants>
                <query>GRANT ALL ON *.*</query>
            </grants>
            -->
        </default>
    </users>

    <!-- Quotas. -->
    <quotas>
        <!-- Name of quota. -->
        <default>
            <!-- Limits for time interval. You could specify many intervals with different limits. -->
            <interval>
                <!-- Length of interval. -->
                <duration>3600</duration>

                <!-- No limits. Just calculate resource usage for time interval. -->
                <queries>0</queries>
                <errors>0</errors>
                <result_rows>0</result_rows>
                <read_rows>0</read_rows>
                <execution_time>0</execution_time>
                <queue_max_wait_ms>{queue_max_wait_ms}</queue_max_wait_ms>
                <max_execution_time>{max_execution_time_seconds}</max_execution_time>
            </interval>
        </default>
    </quotas>
</clickhouse>
"#)
}
