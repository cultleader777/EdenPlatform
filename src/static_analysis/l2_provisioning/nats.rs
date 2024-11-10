use std::fmt::Write;
use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerRegion, TableRowPointerNatsCluster, TableRowPointerMonitoringCluster},
    static_analysis::{
        server_runtime::{NomadJobKind, NomadJobStage, ProvisioningScriptTag, ServerRuntime, IntegrationTest, epl_architecture_to_nomad_architecture},
        PlatformValidationError, L1Projections, networking::{region_loki_clusters, region_monitoring_clusters, server_region, consul_services_exists_integration_test, prometheus_metric_exists_test, check_servers_regional_distribution}, docker_images::image_handle_from_pin,
    },
};

pub fn deploy_nats_instances(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    for nats_cluster in db.nats_cluster().rows_iter() {
        let cluster_name = db.nats_cluster().c_cluster_name(nats_cluster);
        let namespace = db.nats_cluster().c_namespace(nats_cluster);
        let region = db.nats_cluster().c_region(nats_cluster);
        let workload_architecture = db.nats_cluster().c_workload_architecture(nats_cluster);
        let docker_image_pin_nats = db.nats_cluster().c_docker_image_nats(nats_cluster);
        let docker_image_pin_nats_exporter = db.nats_cluster().c_docker_image_nats_exporter(nats_cluster);
        let service_slug = format!("epl-nats-{cluster_name}");
        let nomad_job_name = format!("nats-{cluster_name}");
        let prom_service_slug = format!("epl-nats-{cluster_name}-prom");
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &service_slug);
        let prom_consul_service = runtime.instantiate_and_seal_consul_service(region, &prom_service_slug);
        let job_kind = NomadJobKind::BoundStateful;
        let monitoring_cluster = db.nats_cluster().c_monitoring_cluster(nats_cluster);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::NatsMonitoringClusterDoesntExistInRegion {
            nats_cluster: cluster_name.clone(),
            nats_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = db.nats_cluster().c_loki_cluster(nats_cluster);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::NatsLoggingClusterDoesntExistInRegion {
            nats_cluster: cluster_name.clone(),
            nats_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        let clients_port = db.nats_cluster().c_nats_clients_port(nats_cluster);
        let cluster_port = db.nats_cluster().c_nats_cluster_port(nats_cluster);
        let http_port = db.nats_cluster().c_nats_http_mon_port(nats_cluster);
        let prom_port = db.nats_cluster().c_nats_prometheus_port(nats_cluster);

        let instance_count = db
            .nats_cluster()
            .c_children_nats_deployment_instance(nats_cluster)
            .len();

        if instance_count != 3 && instance_count != 5 {
            return Err(PlatformValidationError::NatsClusterInstancesCountMustBeThreeOrFive {
                nats_cluster: cluster_name.clone(),
                nats_region: db.region().c_region_name(region).clone(),
                nats_instance_count: instance_count,
            })
        }

        assert!(instance_count > 0, "No NATS instances deployed for cluster {cluster_name}, TODO: move this into platform validator error");
        assert!(instance_count == 3 || instance_count == 5, "Only 3 or 5 NATS instances can be deployed for cluster {cluster_name}, TODO: move this into platform validation error");

        let mut nats_route_ips = vec![];

        for deployment in db
            .nats_cluster()
            .c_children_nats_deployment_instance(nats_cluster)
        {
            let server_volume = db.nats_deployment_instance().c_nats_server(*deployment);
            let server = db.server_volume().c_parent(server_volume);
            let instance_region = server_region(db, server);
            if region != instance_region {
                return Err(PlatformValidationError::NatsClusterInstanceIsOutsideSpecifiedRegion {
                    nats_cluster: cluster_name.clone(),
                    nats_cluster_region: db.region().c_region_name(region).clone(),
                    server: db.server().c_hostname(server).clone(),
                    server_region: db.region().c_region_name(instance_region).clone(),
                })
            }
            let hostname = db.server().c_hostname(server);
            let mut nats_ip = None;
            for ni in db.server().c_children_network_interface(server) {
                if db
                    .network()
                    .c_network_name(db.network_interface().c_if_network(*ni))
                    == "lan"
                {
                    assert!(nats_ip.is_none());
                    nats_ip = Some(db.network_interface().c_if_ip(*ni));
                }
            }

            assert!(
                nats_ip.is_some(),
                "Cannot find LAN network interface to deploy NATS node for server {hostname}"
            );
            nats_route_ips.push(nats_ip.unwrap());
        }

        if db.nats_cluster().c_distribute_over_dcs(nats_cluster) {
            check_servers_regional_distribution(
                db,
                region,
                db.nats_cluster().c_children_nats_deployment_instance(nats_cluster).iter().map(|i| {
                    let srv_volume = db.nats_deployment_instance().c_nats_server(*i);
                    db.server_volume().c_parent(srv_volume)
                }),
                format!("nats_cluster=>{cluster_name}")
            )?;
        }

        nats_route_ips.sort();

        let nats_routes_joined = nats_route_ips
            .iter()
            .map(|i| format!("nats://{i}:{cluster_port}"))
            .collect::<Vec<_>>()
            .join(",");

        let rolling_stagger = 60;
        let mut update_delay = 0;
        for deployment in db
            .nats_cluster()
            .c_children_nats_deployment_instance(nats_cluster)
        {
            let target_mount_path = "/data";

            let server_volume = db.nats_deployment_instance().c_nats_server(*deployment);
            let server = db.server_volume().c_parent(server_volume);
            let hostname = db.server().c_hostname(server);

            let server_data = runtime.fetch_server_data(db, server);
            let volume_lock = server_data.server_volume_write_lock(
                db,
                server_volume,
                format!("Exclusive epl-nats-{cluster_name} volume lock"),
            )?;

            let mut port_locks = Vec::new();
            {
                port_locks.push((
                    "nats_client",
                    server_data.lock_port(
                        db,
                        clients_port.try_into().unwrap(),
                        format!("NATS client port for {cluster_name}"),
                    )?,
                ));
                port_locks.push((
                    "nats_cluster",
                    server_data.lock_port(
                        db,
                        cluster_port.try_into().unwrap(),
                        format!("NATS cluster port for {cluster_name}"),
                    )?,
                ));
                port_locks.push((
                    "nats_http_mon",
                    server_data.lock_port(
                        db,
                        http_port.try_into().unwrap(),
                        format!("NATS http monitoring port for {cluster_name}"),
                    )?,
                ));
                port_locks.push((
                    "nats_prom_port",
                    server_data.lock_port(
                        db,
                        prom_port.try_into().unwrap(),
                        format!("NATS prometheus exporter port for {cluster_name}"),
                    )?,
                ));
            }

            let locked_mem = server_data.reserve_memory_mb(
                format!("nats memory {cluster_name}"),
                db.nats_cluster().c_instance_memory_mb(nats_cluster),
            )?;
            let prom_mem =
                server_data.reserve_memory_mb("nats prometheus exporter memory".to_string(), 32)?;

            let server_lock = runtime.lock_server_with_label(
                db,
                format!("epl-nats-{hostname}-{cluster_name}"),
                server,
            )?;

            let instance_id = db
                .nats_deployment_instance()
                .c_instance_id(*deployment);
            let tg_name = format!("nats-{instance_id}");

            let nomad_job =
                runtime.fetch_nomad_job(
                    namespace, nomad_job_name.clone(), region, job_kind, NomadJobStage::SystemJob
                );
            if nomad_job.loki_cluster().is_none() {
                nomad_job.set_loki_cluster(loki_cluster);
            }
            let tg = nomad_job.fetch_task_group(tg_name.clone());
            // don't restart all at once
            tg.set_shutdown_delay_seconds(update_delay);
            update_delay += rolling_stagger;
            tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
            tg.assign_server_lock(server_lock);
            for (mnemonic, lp) in port_locks {
                tg.add_locked_port(mnemonic, lp);
            }

            tg.expose_port_as_tcp_service("nats_cluster", &consul_service);
            tg.expose_port_as_tcp_service("nats_prom_port", &prom_consul_service);
            tg.collect_prometheus_metrics(&prom_consul_service, monitoring_cluster, None);

            let docker_image_nats = image_handle_from_pin(db, workload_architecture, docker_image_pin_nats, "nats")?;
            let nats_task = tg.fetch_task(
                format!("nats-{cluster_name}-daemon"),
                docker_image_nats,
            );
            nats_task.add_memory(locked_mem);
            nats_task.bind_volume(volume_lock, target_mount_path.to_string());

            let arguments = vec![
                format!("--name={cluster_name}-{hostname}"),
                "--jetstream".to_string(),
                format!("--store_dir={target_mount_path}/nats"),
                format!("--port={clients_port}"),
                format!("--http_port={http_port}"),
                "--addr=${meta.private_ip}".to_string(),
                format!("--cluster_name={cluster_name}"),
                format!("--cluster=nats://${{meta.private_ip}}:{cluster_port}"),
                format!("--cluster_advertise=${{meta.private_ip}}:{cluster_port}"),
                format!("--routes={nats_routes_joined}"),
            ];

            nats_task.set_arguments(arguments);

            let docker_image_nats_exporter = image_handle_from_pin(db, workload_architecture, docker_image_pin_nats_exporter, "nats_exporter")?;
            let nats_prom_task =
                tg.fetch_task(
                    format!("nats-{cluster_name}-prom-exp"),
                    docker_image_nats_exporter,
                );
            nats_prom_task.add_memory(prom_mem);
            let arguments = vec![
                "-addr=${meta.private_ip}".to_string(),
                "-channelz".to_string(),
                "-connz_detailed".to_string(),
                "-healthz".to_string(),
                "-gatewayz".to_string(),
                "-leafz".to_string(),
                "-routez".to_string(),
                "-serverz".to_string(),
                "-subz".to_string(),
                "-varz".to_string(),
                "-use_internal_server_id".to_string(),
                "-use_internal_server_name".to_string(),
                format!("-p={prom_port}"),
                format!("http://${{meta.private_ip}}:{http_port}"),
            ];
            nats_prom_task.set_arguments(arguments);
        }

        nats_cluster_tests(db, l1proj, nats_cluster, monitoring_cluster, runtime);
    }

    for region in db.region().rows_iter() {
        provision_nats_resources(db, runtime, region);
    }

    Ok(())
}

fn nats_cluster_tests(
    db: &Database,
    l1proj: &L1Projections,
    cluster: TableRowPointerNatsCluster,
    mon_cluster: TableRowPointerMonitoringCluster,
    runtime: &mut ServerRuntime
) {
    let cluster_name = db.nats_cluster().c_cluster_name(cluster);
    let job_name = format!("epl-nats-{cluster_name}-prom");
    let name = cluster_name.to_case(convert_case::Case::Snake);
    let region = db.nats_cluster().c_region(cluster);
    let init_port = db.nats_cluster().c_nats_clients_port(cluster);
    let servers =
        db.nats_cluster()
            .c_children_nats_deployment_instance(cluster)
            .iter()
            .map(|i| {
                let disk = db.nats_deployment_instance().c_nats_server(*i);
                db.server_volume().c_parent(disk)
            })
            .collect::<Vec<_>>();
    runtime.add_integration_test(
        format!("nats_cluster_{name}_instances_available_in_dns"),
        consul_services_exists_integration_test(db, l1proj, region, format!("epl-nats-{cluster_name}.service.consul"), &servers)
    );

    let mut sockets: Vec<String> = Vec::new();
    for port in init_port..=db.nats_cluster().c_nats_http_mon_port(cluster) {
        let this_sockets = servers.iter().map(|i| {
            let ip = db.network_interface().c_if_ip(*l1proj.consul_network_iface.value(*i));
            format!("{ip}:{port}")
        }).collect::<Vec<_>>();
        sockets.extend(this_sockets);
    }

    runtime.add_integration_test(
        format!("nats_cluster_{name}_tcp_sockets_open"),
        IntegrationTest::TcpSocketsOpen { target_sockets: sockets },
    );

    runtime.add_integration_test(
        format!("nats_cluster_{name}_prometheus_metrics_gathered"),
        prometheus_metric_exists_test(
            db, l1proj, mon_cluster,
            &format!("gnatsd_varz_jetstream_meta_cluster_size{{job=\\\"{job_name}\\\"}}")
        )
    );
}

fn provision_nats_resources(db: &Database, runtime: &mut ServerRuntime, region: TableRowPointerRegion) {
    let mut script = String::new();

    script += "#!/bin/sh\n";
    script += "\n";

    for nats_cluster in db.nats_cluster().rows_iter() {
        let this_region = db.nats_cluster().c_region(nats_cluster);
        if region != this_region {
            continue;
        }
        script += "export NATS_URL=nats://epl-nats-";
        script += db.nats_cluster().c_cluster_name(nats_cluster);
        script += &format!(
            ".service.consul:{}\n",
            db.nats_cluster().c_nats_clients_port(nats_cluster)
        );
        script += "\n";
        script += "while ! nats account info\n";
        script += "do\n";
        script += "  echo Waiting for the NATS cluster to be up at $NATS_URL\n";
        script += "  sleep 2\n";
        script += "done\n";
        script += "while ! nats stream ls\n";
        script += "do\n";
        script += "  echo Waiting for the NATS cluster list streams $NATS_URL\n";
        script += "  sleep 2\n";
        script += "done\n";

        for nats_stream in db
            .nats_cluster()
            .c_children_nats_jetstream_stream(nats_cluster)
        {
            let stream_name = db.nats_jetstream_stream().c_stream_name(*nats_stream);
            let max_msg_size = db.nats_jetstream_stream().c_max_msg_size(*nats_stream);
            let enable_subjects = db.nats_jetstream_stream().c_enable_subjects(*nats_stream);
            let subjects =
                if enable_subjects {
                    format!("{stream_name}.*")
                } else {
                    stream_name.clone()
                };
            let config_block = format!(
                r#"{{
  "config": {{
    "subjects": ["{subjects}"],
    "retention": "limits",
    "max_consumers": -1,
    "max_msgs_per_subject": -1,
    "max_msgs": -1,
    "max_bytes": -1,
    "max_age": 604800000000000,
    "max_msg_size": {max_msg_size},
    "storage": "file",
    "discard": "old",
    "num_replicas": 3,
    "duplicate_window": 120000000000,
    "sealed": false,
    "deny_delete": false,
    "deny_purge": false,
    "allow_rollup_hdrs": true,
    "allow_direct": true,
    "mirror_direct": false
  }}
}}
"#
            );

            writeln!(&mut script, "STREAM_CFG=$( cat << EOF | base64 -w 0").unwrap();
            script += &config_block;
            script += "EOF\n";
            script += ")\n";

            let grep_repeatable_errors = "tee /dev/stderr | grep 'no suitable peers for placement'";

            writeln!(&mut script, "while echo $STREAM_CFG | base64 -d | nats stream add '{stream_name}' --subjects '{subjects}' --config=/dev/stdin 2>&1 | {grep_repeatable_errors}").unwrap();
            writeln!(&mut script, "do").unwrap();
            writeln!(&mut script, "  echo retriable stream adding error, retrying in 3 seconds").unwrap();
            writeln!(&mut script, "  sleep 3").unwrap();
            writeln!(&mut script, "done").unwrap();

            script += "\n";

            script += "ADD_SUCCEEDED=\"${PIPESTATUS[2]}\"\n";
            script += "if [ \"$ADD_SUCCEEDED\" -eq 0 ];\n";
            script += "then\n";
            writeln!(&mut script, "  while echo $STREAM_CFG | base64 -d | nats stream edit {stream_name} --config=/dev/stdin --force 2>&1 | {grep_repeatable_errors}").unwrap();
            writeln!(&mut script, "  do").unwrap();
            writeln!(&mut script, "    echo retriable stream editing error, retrying in 3 seconds").unwrap();
            writeln!(&mut script, "    sleep 3").unwrap();
            writeln!(&mut script, "  done").unwrap();
            script += "fi\n";

            for importer in db.nats_jetstream_stream().c_referrers_ch_nats_stream_import__stream(*nats_stream) {
                let importer_name = db.ch_nats_stream_import().c_consumer_name(*importer);
                let importer_db = db.ch_nats_stream_import().c_parent(*importer);
                let importer_db_name = db.ch_deployment_schemas().c_db_name(importer_db);
                let ch_deployment = db.ch_deployment_schemas().c_parent(importer_db);
                let ch_deployment_name = db.ch_deployment().c_deployment_name(ch_deployment);
                writeln!(&mut script, "nats consumer add {stream_name} ch_imp_{importer_name} --filter '{stream_name}' --target ch_imp.{ch_deployment_name}.{importer_db_name}.{importer_name} --deliver-group ch_imp_{importer_name} --ack none --deliver all --replay instant --defaults || true").unwrap();
            }
        }
    }

    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::SystemResourceProvision,
        "provision-nats-resources.sh",
        script,
    );
}
