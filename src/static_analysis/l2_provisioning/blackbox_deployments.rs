use std::collections::{BTreeSet, BTreeMap};

use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerServer, TableRowPointerBlackboxDeploymentGroup, TableRowPointerBlackboxDeploymentPort, TableRowPointerBlackboxDeploymentServiceRegistration, TableRowPointerBlackboxDeploymentTask, TableRowPointerBlackboxDeploymentTaskMount, TableRowPointerPgDeploymentUnmanagedDb, TableRowPointerMinioBucket, TableRowPointerNatsCluster, TableRowPointerRegion, TableRowPointerBlackboxDeployment, TableRowPointerBlackboxDeploymentEnvVariable},
    static_analysis::{
        server_runtime::{ServerRuntime, NomadJobKind, NomadJobStage, LockedServerLabel, LockedPort, ConsulServiceHandle, ReservedMemory, epl_architecture_to_nomad_architecture, SuccessfulVolumeLock, IntegrationTest, FinalizedVaultSecrets, PgAccessKind, VaultSecretRequest, VaultSecretHandle, MinIOBucketPermission},
        L1Projections, PlatformValidationError, networking::{region_monitoring_clusters, region_loki_clusters, first_region_server}, docker_images::image_handle_from_pin, projections::Projection
    },
};

type BlackboxCliArguments = Vec<String>;

pub fn deploy_blackbox_deployments(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {

    for bb_depl in db.blackbox_deployment().rows_iter() {
        let depl_name = db.blackbox_deployment().c_deployment_name(bb_depl);
        let ns = db.blackbox_deployment().c_namespace(bb_depl);
        let region = db.blackbox_deployment().c_region(bb_depl);
        let region_name = db.region().c_region_name(region);
        let monitoring_cluster = db.blackbox_deployment().c_monitoring_cluster(bb_depl);
        let loki_cluster = db.blackbox_deployment().c_loki_cluster(bb_depl);
        let nomad_job_name = format!("bb-{depl_name}");

        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::BlackboxDeploymentMonitoringClusterDoesntExistInRegion {
            bb_deployment: depl_name.clone(),
            bb_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::BlackboxDeploymentLoggingClusterDoesntExistInRegion {
            bb_deployment: depl_name.clone(),
            bb_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        if db.blackbox_deployment().c_children_blackbox_deployment_group(bb_depl).is_empty() {
            return Err(PlatformValidationError::BlackboxDeploymentHasNoGroups {
                bb_deployment: depl_name.clone(),
                bb_region: db.region().c_region_name(region).clone(),
                groups_count: db.blackbox_deployment().c_children_blackbox_deployment_group(bb_depl).len(),
                groups_minimum: 1,
            });
        }

        let (env_variable_wiring, needs_secrets) = check_blackbox_deployment_wiring(db, bb_depl, l1proj)?;

        let mut finalized_vault_secrets: Option<FinalizedVaultSecrets> = None;
        let mut env_var_handles: BTreeMap<TableRowPointerBlackboxDeploymentEnvVariable, VaultSecretHandle> = BTreeMap::new();
        if needs_secrets {
            // get vault secret here
            let secret_path = format!("bb-depl/{}", db.blackbox_deployment().c_deployment_name(bb_depl));
            let mut builder = runtime.issue_vault_secret(region, &secret_path);
            for (env_var, the_source) in &env_variable_wiring {
                let var_key = format!("env_var_{}", db.blackbox_deployment_env_variable().c_var_name(*env_var).to_lowercase());
                match the_source {
                    BlackBoxDeploymentResource::Pg { pg_db } => {
                        let access = builder.fetch_pg_access(&PgAccessKind::UnmanagedRw(*pg_db));
                        let left = format!("postgresql://{}:", access.db_user);
                        let right = format!("@master.{}:{}/{}", access.db_host, access.db_master_port, access.db_database);
                        let postgres_password = builder.request_secret(
                            region,
                            &var_key,
                            VaultSecretRequest::ExistingVaultSecret {
                                handle: Box::new(access.db_password.clone()),
                                sprintf: None,
                            },
                        );
                        let postgres_url = postgres_password.surround_expression(left, right);
                        assert!(env_var_handles.insert(*env_var, postgres_url).is_none());
                    }
                    BlackBoxDeploymentResource::MinIOBucket { bucket } => {
                        let user = format!("bb-depl-{}", db.blackbox_deployment().c_deployment_name(bb_depl));
                        let permission = MinIOBucketPermission::ReadWrite;
                        let minio_password = builder.fetch_minio_bucket_credentials(
                            db, &var_key, &user, *bucket, permission
                        );

                        let cluster = db.minio_bucket().c_parent(*bucket);
                        let service_slug = db.minio_cluster().c_consul_service_name(cluster);
                        let port = db.minio_cluster().c_lb_port(cluster);
                        let minio_url = minio_password.surround_expression(
                            format!("s3://{}:", user),
                            // password vault expression in the middle
                            format!("@{}.service.consul:{}/{}", service_slug, port, db.minio_bucket().c_bucket_name(*bucket)),
                        );

                        assert!(env_var_handles.insert(*env_var, minio_url).is_none());
                    }
                    BlackBoxDeploymentResource::NatsCluster { .. } => {
                        // no passwords from vault, just set env variable
                    }
                }
            }

            finalized_vault_secrets = Some(builder.finalize());
        }

        let mut group_server_locks: BTreeMap<TableRowPointerBlackboxDeploymentGroup, Vec<LockedServerLabel>> = BTreeMap::new();
        let mut port_locks: BTreeMap<TableRowPointerBlackboxDeploymentGroup, Vec<(String, LockedPort)>> = BTreeMap::new();
        let mut volume_locks: BTreeMap<TableRowPointerBlackboxDeploymentGroup, BTreeMap<TableRowPointerBlackboxDeploymentTaskMount, SuccessfulVolumeLock>> = BTreeMap::new();
        let mut nomad_port_names: BTreeMap<TableRowPointerBlackboxDeploymentGroup, BTreeMap<TableRowPointerBlackboxDeploymentPort, String>> = BTreeMap::new();
        let mut tasks_memory: BTreeMap<TableRowPointerBlackboxDeploymentTask, ReservedMemory> = BTreeMap::new();

        let mut is_stateless = true;
        for grp in db.blackbox_deployment().c_children_blackbox_deployment_group(bb_depl) {
            let mut port_set: BTreeSet<i64> = BTreeSet::new();
            for port in db.blackbox_deployment_group().c_children_blackbox_deployment_port(*grp) {
                assert!(port_set.insert(db.blackbox_deployment_port().c_port(*port)));
            }

            let port_set = port_set.iter().collect::<Vec<_>>();

            for wnd in port_set.windows(2) {
                if wnd[1] - wnd[0] != 1 {
                    return Err(PlatformValidationError::BlackboxDeploymentPortsAreNotSequential {
                        bb_deployment: depl_name.clone(),
                        bb_region: db.region().c_region_name(region).clone(),
                        port_a: *wnd[0],
                        non_sequential_port_b: *wnd[1],
                        deployment_ports: port_set.iter().map(|i| **i).collect::<Vec<_>>(),
                    });
                }
            }

            let mut server_set: BTreeSet<TableRowPointerServer> = BTreeSet::new();
            let group_count = db.blackbox_deployment_group().c_count(*grp);
            let group_name = db.blackbox_deployment_group().c_group_name(*grp);

            if db.blackbox_deployment_group().c_children_blackbox_deployment_task(*grp).is_empty() {
                return Err(PlatformValidationError::BlackboxDeploymentGroupHasNoTasks {
                    bb_deployment: depl_name.clone(),
                    bb_region: db.region().c_region_name(region).clone(),
                    group_name: group_name.clone(),
                    tasks_count: db.blackbox_deployment_group().c_children_blackbox_deployment_task(*grp).len(),
                    tasks_minimum: 1,
                });
            }

            let mut group_server: Option<TableRowPointerServer> = None;
            for task in db.blackbox_deployment_group().c_children_blackbox_deployment_task(*grp) {
                for mnt in db.blackbox_deployment_task().c_children_blackbox_deployment_task_mount(*task) {
                    is_stateless = false;

                    if group_count != 1 {
                        return Err(PlatformValidationError::BlackboxDeploymentStatefulGroupMustHaveCountOfOne {
                            bb_deployment: depl_name.clone(),
                            bb_region: db.region().c_region_name(region).clone(),
                            group_name: group_name.clone(),
                            current_count: group_count,
                            expected_count: 1,
                        });
                    }

                    let server_vol = db.blackbox_deployment_task_mount().c_server_volume(*mnt);
                    let server = db.server_volume().c_parent(server_vol);
                    let server_dc = db.server().c_dc(server);
                    let server_region = db.datacenter().c_region(server_dc);
                    if server_region != region {
                        return Err(PlatformValidationError::BlackboxDeploymentGroupBoundToOtherRegion {
                            bb_deployment: depl_name.clone(),
                            bb_region: db.region().c_region_name(region).clone(),
                            group_name: group_name.clone(),
                            group_bound_server: db.server().c_hostname(server).clone(),
                            group_bound_server_region: db.region().c_region_name(server_region).clone(),
                        });
                    }
                    let _ = server_set.insert(server);
                    if server_set.len() > 1 {
                        return Err(PlatformValidationError::BlackboxDeploymentGroupBoundToMultipleServers {
                            bb_deployment: depl_name.clone(),
                            bb_region: db.region().c_region_name(region).clone(),
                            group_name: group_name.clone(),
                            group_bound_server_a: db.server().c_hostname(*server_set.iter().next().unwrap()).clone(),
                            group_bound_server_b: db.server().c_hostname(*server_set.iter().skip(1).next().unwrap()).clone(),
                            maximum_servers: 1,
                        });
                    }

                    group_server = Some(server);
                }
            }

            if let Some(group_server) = group_server {
                if !db.blackbox_deployment_group().c_placement(*grp).is_empty() {
                    return Err(PlatformValidationError::BlackboxDeploymentPlacementsAreValidOnlyForStatelessWorkloads {
                        bb_deployment: depl_name.clone(),
                        bb_region: db.region().c_region_name(region).clone(),
                        group_name: group_name.clone(),
                        group_placement: db.blackbox_deployment_group().c_placement(*grp).clone(),
                        only_valid_placement: "".to_string(),
                        already_bound_server: db.server().c_hostname(group_server).clone(),
                    });
                }

                let hostname = db.server().c_hostname(group_server);
                // server lock
                let server_lock = runtime.lock_server_with_label(db, format!("epl-bb-{depl_name}-{hostname}-{group_name}"), group_server)?;
                group_server_locks.entry(*grp).or_default().push(server_lock);

                let server_data = runtime.fetch_server_data(db, group_server);

                let port_locks = port_locks.entry(*grp).or_default();
                let mut port_idx = 0;
                let this_grp_port_names = nomad_port_names.entry(*grp).or_default();
                for depl_port in db.blackbox_deployment_group().c_children_blackbox_deployment_port(*grp) {
                    port_idx += 1;
                    let port_value = db.blackbox_deployment_port().c_port(*depl_port);
                    let port_description = db.blackbox_deployment_port().c_port_description(*depl_port);
                    let nomad_port_name = format!("port_{port_idx}");
                    this_grp_port_names.insert(*depl_port, nomad_port_name.clone());
                    port_locks.push((
                        nomad_port_name,
                        server_data.lock_port(
                            db,
                            port_value as u16,
                            format!("Blackbox deployment {depl_name}>{port_description}"),
                        )?,
                    ))
                }

                for task in db.blackbox_deployment_group().c_children_blackbox_deployment_task(*grp) {
                    let memory_mb = db.blackbox_deployment_task().c_memory_mb(*task);
                    let task_name = db.blackbox_deployment_task().c_task_name(*task);
                    assert!(
                        tasks_memory.insert(
                            *task,
                            server_data.reserve_memory_mb(
                                format!("Blackbox deployment {depl_name}>{group_name}>{task_name}"), memory_mb
                            )?
                        ).is_none()
                    );

                    for mnt in db.blackbox_deployment_task().c_children_blackbox_deployment_task_mount(*task) {
                        let server_vol = db.blackbox_deployment_task_mount().c_server_volume(*mnt);
                        let vol_name = db.server_volume().c_volume_name(server_vol);
                        let server_host = db.server().c_hostname(db.server_volume().c_parent(server_vol));
                        let mountpoint = db.blackbox_deployment_task_mount().c_target_path(*mnt);
                        let vol_locks_grp = volume_locks.entry(*grp).or_default();
                        let vol_lock = server_data.server_volume_write_lock(
                            db, server_vol, format!("Blackbox deployment {depl_name}>{group_name}>{task_name} at server {server_host} volume {vol_name} with mountpoint {mountpoint}")
                        )?;
                        assert!(vol_locks_grp.insert(*mnt, vol_lock).is_none());
                    }
                }
            } else {
                let port_locks = port_locks.entry(*grp).or_default();
                let mut port_idx = 0;
                let this_grp_port_names = nomad_port_names.entry(*grp).or_default();
                for depl_port in db.blackbox_deployment_group().c_children_blackbox_deployment_port(*grp) {
                    port_idx += 1;
                    let port_value = db.blackbox_deployment_port().c_port(*depl_port);
                    let port_description = db.blackbox_deployment_port().c_port_description(*depl_port);
                    let nomad_port_name = format!("port_{port_idx}");
                    this_grp_port_names.insert(*depl_port, nomad_port_name.clone());
                    port_locks.push((
                        nomad_port_name,
                        runtime.lock_port_all_servers(
                            port_value as u16,
                            format!("Blackbox deployment {depl_name}>{port_description}"),
                        )?,
                    ))
                }

                for task in db.blackbox_deployment_group().c_children_blackbox_deployment_task(*grp) {
                    let memory_mb = db.blackbox_deployment_task().c_memory_mb(*task);
                    let task_name = db.blackbox_deployment_task().c_task_name(*task);
                    assert!(
                        tasks_memory.insert(
                            *task,
                            runtime.reserve_stateless_memory_mb(
                                format!("Blackbox deployment {depl_name}>{group_name}>{task_name}"),
                                memory_mb
                            )
                        ).is_none()
                    );
                }
            }
        }

        let mut consul_service_registrations: BTreeMap<TableRowPointerBlackboxDeploymentServiceRegistration, ConsulServiceHandle> = BTreeMap::new();

        for svc in db.blackbox_deployment().c_children_blackbox_deployment_service_registration(bb_depl) {
            let min_instances = db.blackbox_deployment_service_registration().c_min_instances(*svc);
            let svc_instances = db.blackbox_deployment_service_registration().c_referrers_blackbox_deployment_service_instance__service_registration(*svc);
            if (svc_instances.len() as i64) < min_instances {
                return Err(PlatformValidationError::BlackboxDeploymentServiceRegistrationHasNotEnoughInstances {
                    bb_deployment: depl_name.clone(),
                    bb_region: db.region().c_region_name(region).clone(),
                    service_name: db.blackbox_deployment_service_registration().c_service_name(*svc).clone(),
                    current_service_instances: svc_instances.len(),
                    min_service_instances: min_instances,
                });
            }

            for svc_instance in svc_instances {
                let the_port = db.blackbox_deployment_service_instance().c_port(*svc_instance);
                let protocol = db.blackbox_deployment_port().c_protocol(the_port);
                if db.blackbox_deployment_service_registration().c_scrape_prometheus_metrics(*svc) {
                    if protocol != "http" {
                        return Err(PlatformValidationError::BlackboxDeploymentPrometheusMetricsCanBeScrapedOnlyFromHttpPorts {
                            bb_deployment: depl_name.clone(),
                            bb_region: db.region().c_region_name(region).clone(),
                            group_name: db.blackbox_deployment_group().c_group_name(db.blackbox_deployment_service_instance().c_parent(*svc_instance)).clone(),
                            expected_protocol: "http".to_string(),
                            port_protocol: protocol.clone(),
                            service_name: db.blackbox_deployment_service_registration().c_service_name(*svc).clone(),
                            group_port: db.blackbox_deployment_port().c_port(the_port),
                        });
                    }
                }
            }
            let svc_name = db.blackbox_deployment_service_registration().c_service_name(*svc);
            consul_service_registrations.insert(
                *svc,
                runtime.instantiate_and_seal_consul_service(
                    region, svc_name.as_str()
                )
            );
        }

        // verification done, forming should should succeed
        let job_kind = if is_stateless {
            NomadJobKind::Stateless
        } else { NomadJobKind::BoundStateful };

        let job = runtime.fetch_nomad_job(ns, nomad_job_name, region, job_kind, NomadJobStage::Application);
        if let Some(secrets) = finalized_vault_secrets {
            job.assign_vault_secrets(secrets);
        }

        job.set_loki_cluster(loki_cluster);

        for grp in db.blackbox_deployment().c_children_blackbox_deployment_group(bb_depl) {
            let group_name = db.blackbox_deployment_group().c_group_name(*grp);
            let nomad_grp = job.fetch_task_group(group_name.clone());
            let workload_architecture = db.blackbox_deployment_group().c_workload_architecture(*grp);

            if let Some(locks) = group_server_locks.remove(grp) {
                for lock in locks {
                    nomad_grp.assign_server_lock(lock);
                }
            }

            if let Some(port_locks) = port_locks.remove(grp) {
                for (pm, lp) in port_locks {
                    nomad_grp.add_locked_port(&pm, lp);
                }
            }

            let count = db.blackbox_deployment_group().c_count(*grp) as usize;
            nomad_grp.constrain_architecture(
                epl_architecture_to_nomad_architecture(
                    &db.blackbox_deployment_group().c_workload_architecture(*grp)
                )
            );
            nomad_grp.set_count(count);
            nomad_grp.try_set_placement(
                db,
                region,
                db.blackbox_deployment_group().c_placement(*grp),
                &format!("Blackbox deployment {depl_name}>{group_name} in region {region_name}"),
                count,
                l1proj.label_database
            )?;

            let mut volume_locks = volume_locks.remove(grp);

            for task in db.blackbox_deployment_group().c_children_blackbox_deployment_task(*grp) {
                let task_name = db.blackbox_deployment_task().c_task_name(*task);
                let docker_image_set = db.blackbox_deployment_task().c_docker_image_set(*task);
                let docker_image_set_name = db.docker_image_set().c_set_name(docker_image_set);
                let docker_image = db.blackbox_deployment_task().c_docker_image(*task);

                let docker_image_handle = image_handle_from_pin(
                    db, &workload_architecture, docker_image, docker_image_set_name.as_str(),
                )?;

                let nomad_task = nomad_grp.fetch_task(format!("bb-{task_name}"), docker_image_handle);
                if let Some(mem) = tasks_memory.remove(task) {
                    nomad_task.add_memory(mem);
                } else {
                    panic!("Blackbox task {task_name} has no memory")
                }

                for mount in db.blackbox_deployment_task().c_children_blackbox_deployment_task_mount(*task) {
                    if let Some(volume_locks) = &mut volume_locks {
                        let the_lock = volume_locks.remove(mount).unwrap();
                        nomad_task.bind_volume(the_lock, db.blackbox_deployment_task_mount().c_target_path(*mount).clone());
                    }
                }

                let args = db.blackbox_deployment_task().c_args(*task);
                if !args.is_empty() {
                    let parsed_args = serde_yaml::from_str::<BlackboxCliArguments>(args).map_err(|e| {
                        PlatformValidationError::BlackboxDeploymentCantParseTaskArguments {
                            bb_deployment: depl_name.clone(),
                            bb_region: db.region().c_region_name(region).clone(),
                            group_name: db.blackbox_deployment_group().c_group_name(*grp).clone(),
                            task_arguments: args.clone(),
                            task_name: task_name.clone(),
                            example_arguments_yaml: "
- /bin/sleep
- '123'
".to_string(),
                            parsing_error: e.to_string(),
                        }
                    })?;
                    nomad_task.set_arguments(parsed_args);
                }
                let entrypoint = db.blackbox_deployment_task().c_entrypoint(*task);
                if !entrypoint.is_empty() {
                    let parsed_entrypoint = serde_yaml::from_str::<BlackboxCliArguments>(&entrypoint).map_err(|e| {
                        PlatformValidationError::BlackboxDeploymentCantParseTaskEntrypoint {
                            bb_deployment: depl_name.clone(),
                            bb_region: db.region().c_region_name(region).clone(),
                            group_name: db.blackbox_deployment_group().c_group_name(*grp).clone(),
                            task_entrypoint: entrypoint.clone(),
                            task_name: task_name.clone(),
                            example_entrypoint_yaml: "
- /bin/sleep
- '123'
".to_string(),
                            parsing_error: e.to_string(),
                        }
                    })?;
                    nomad_task.set_entrypoint(parsed_entrypoint);
                }

                let memory_oversubscription_mb = db.blackbox_deployment_task().c_memory_oversubscription_mb(*task);
                if memory_oversubscription_mb != 128 {
                    nomad_task.set_memory_oversubscription_mb(memory_oversubscription_mb as u32);
                }

                for local_file in db.blackbox_deployment_task().c_children_blackbox_deployment_local_file(*task) {
                    let fname = db.blackbox_deployment_local_file().c_local_file_name(*local_file);
                    let mode = db.blackbox_deployment_local_file().c_mode(*local_file);
                    let contents = db.blackbox_deployment_local_file().c_local_file_contents(*local_file);

                    match mode.as_str() {
                        "644" => {
                            nomad_task.add_local_file(fname.clone(), contents.clone());
                        }
                        "755" => {
                            nomad_task.add_executable_local_file(fname.clone(), contents.clone());
                        }
                        other => {
                            panic!("Unknown file mode: {other}")
                        }
                    }
                }

                let mut sec_env_vars: Vec<(&str, &VaultSecretHandle)> = Vec::new();
                for env_var in db.blackbox_deployment_task().c_children_blackbox_deployment_env_variable(*task) {
                    if !db.blackbox_deployment_env_variable().c_value_source(*env_var).is_empty() {
                        let the_resource = env_variable_wiring.get(env_var).expect("We should have info about all wired variables here");
                        match *the_resource {
                            BlackBoxDeploymentResource::MinIOBucket { .. } => {
                                let sec_handle = env_var_handles.get(env_var).expect("This should have been set before");
                                sec_env_vars.push((db.blackbox_deployment_env_variable().c_var_name(*env_var), sec_handle));
                            }
                            BlackBoxDeploymentResource::Pg { .. } => {
                                let sec_handle = env_var_handles.get(env_var).expect("This should have been set before");
                                sec_env_vars.push((db.blackbox_deployment_env_variable().c_var_name(*env_var), sec_handle));
                            }
                            BlackBoxDeploymentResource::NatsCluster { cluster } => {
                                nomad_task.set_env_variable(
                                    db.blackbox_deployment_env_variable().c_var_name(*env_var),
                                    &format!("nats://epl-nats-{}.service.consul:{}",
                                            db.nats_cluster().c_cluster_name(*cluster),
                                            db.nats_cluster().c_nats_clients_port(*cluster))
                                );
                            }
                        }
                    } else {
                        assert!(!db.blackbox_deployment_env_variable().c_raw_value(*env_var).is_empty(),
                                "We assume raw value here");
                        nomad_task.set_env_variable(
                            db.blackbox_deployment_env_variable().c_var_name(*env_var),
                            db.blackbox_deployment_env_variable().c_raw_value(*env_var),
                        );
                    }
                }
                if !sec_env_vars.is_empty() {
                    nomad_task.add_secure_env_variables("epl-env-secrets.env".to_string(), &sec_env_vars);
                }
            }


            for svc in db.blackbox_deployment_group().c_children_blackbox_deployment_service_instance(*grp) {
                let svc_reg = db.blackbox_deployment_service_instance().c_service_registration(*svc);
                let the_port = db.blackbox_deployment_service_instance().c_port(*svc);
                let port_map = nomad_port_names.get(grp).unwrap();
                let nomad_port_name = port_map.get(&the_port).unwrap();
                let consul_handle = consul_service_registrations.get(&svc_reg).unwrap();
                nomad_grp.expose_port_as_tcp_service(&nomad_port_name, consul_handle);
                if db.blackbox_deployment_service_registration().c_scrape_prometheus_metrics(svc_reg) {
                    let prom_path = db.blackbox_deployment_service_registration().c_prometheus_metrics_path(svc_reg);
                    let prom_path =
                        if prom_path == "/metrics" {
                            None
                        } else { Some(prom_path.as_str()) };
                    nomad_grp.collect_prometheus_metrics(consul_handle, monitoring_cluster, prom_path);
                }
            }
        }
    }

    blackbox_deployments_integration_tests(db, runtime, l1proj);

    Ok(())
}

lazy_static! {
    pub static ref VALID_ENV_NAME_REGEX: regex::Regex = regex::Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
}

// bool returns if secrets are needed
fn check_blackbox_deployment_wiring<'a>(
    db: &Database,
    bb_depl: TableRowPointerBlackboxDeployment,
    l1proj: &'a L1Projections,
) -> Result<(BTreeMap<TableRowPointerBlackboxDeploymentEnvVariable, &'a BlackBoxDeploymentResource>, bool), PlatformValidationError> {
    let mut res = BTreeMap::new();

    let mut needs_secrets = false;
    for tg in db.blackbox_deployment().c_children_blackbox_deployment_group(bb_depl) {
        for task in db.blackbox_deployment_group().c_children_blackbox_deployment_task(*tg) {
            for env_var in db.blackbox_deployment_task().c_children_blackbox_deployment_env_variable(*task) {
                let var_name = db.blackbox_deployment_env_variable().c_var_name(*env_var);
                if !VALID_ENV_NAME_REGEX.is_match(&var_name) {
                    return Err(PlatformValidationError::BlackboxDeploymentEnvironmentVariableNameIsInvalid {
                        bb_deployment: db.blackbox_deployment().c_deployment_name(bb_depl).clone(),
                        bb_region: db.region().c_region_name(db.blackbox_deployment().c_region(bb_depl)).clone(),
                        group_name: db.blackbox_deployment_group().c_group_name(*tg).clone(),
                        task_name: db.blackbox_deployment_task().c_task_name(*task).clone(),
                        env_variable_name: var_name.clone(),
                        must_match_regex: VALID_ENV_NAME_REGEX.to_string(),
                    });
                }

                if db.blackbox_deployment_env_variable().c_raw_value(*env_var).is_empty()
                    && db.blackbox_deployment_env_variable().c_value_source(*env_var).is_empty()
                {
                    return Err(PlatformValidationError::BlackboxDeploymentRawValueOrValueSourceMustBeNotEmpty {
                        bb_deployment: db.blackbox_deployment().c_deployment_name(bb_depl).clone(),
                        bb_region: db.region().c_region_name(db.blackbox_deployment().c_region(bb_depl)).clone(),
                        group_name: db.blackbox_deployment_group().c_group_name(*tg).clone(),
                        task_name: db.blackbox_deployment_task().c_task_name(*task).clone(),
                        env_variable_name: var_name.clone(),
                        raw_value: db.blackbox_deployment_env_variable().c_raw_value(*env_var).clone(),
                        value_source: db.blackbox_deployment_env_variable().c_value_source(*env_var).clone(),
                    });
                }

                if !db.blackbox_deployment_env_variable().c_raw_value(*env_var).is_empty()
                    && !db.blackbox_deployment_env_variable().c_value_source(*env_var).is_empty()
                {
                    return Err(PlatformValidationError::BlackboxDeploymentRawValueAndValueSourceAreMutuallyExclusive {
                        bb_deployment: db.blackbox_deployment().c_deployment_name(bb_depl).clone(),
                        bb_region: db.region().c_region_name(db.blackbox_deployment().c_region(bb_depl)).clone(),
                        group_name: db.blackbox_deployment_group().c_group_name(*tg).clone(),
                        task_name: db.blackbox_deployment_task().c_task_name(*task).clone(),
                        env_variable_name: var_name.clone(),
                        raw_value: db.blackbox_deployment_env_variable().c_raw_value(*env_var).clone(),
                        value_source: db.blackbox_deployment_env_variable().c_value_source(*env_var).clone(),
                    });
                }

                // here we just check that if value_source is set that it exists
                let val_source = db.blackbox_deployment_env_variable().c_value_source(*env_var);
                if !val_source.is_empty() {
                    let available_resources =
                        l1proj.bb_depl_resources_per_region.value(db.blackbox_deployment().c_region(bb_depl));

                    let Some(resource) = available_resources.get(val_source) else {
                        return Err(PlatformValidationError::BlackboxDeploymentValueSourceNotFoundInRegion {
                            bb_deployment: db.blackbox_deployment().c_deployment_name(bb_depl).clone(),
                            bb_region: db.region().c_region_name(db.blackbox_deployment().c_region(bb_depl)).clone(),
                            group_name: db.blackbox_deployment_group().c_group_name(*tg).clone(),
                            task_name: db.blackbox_deployment_task().c_task_name(*task).clone(),
                            env_variable_name: var_name.clone(),
                            value_source: val_source.clone(),
                        });
                    };

                    match resource {
                        BlackBoxDeploymentResource::MinIOBucket { .. } | BlackBoxDeploymentResource::Pg { .. } => {
                            needs_secrets |= true;
                        }
                        BlackBoxDeploymentResource::NatsCluster { .. } => {}
                    }

                    assert!(
                        res.insert(*env_var, resource).is_none(),
                        "must be unique"
                    );
                }
            }
        }
    }

    Ok((res, needs_secrets))
}

fn blackbox_deployments_integration_tests(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) {
    for bb_depl in db.blackbox_deployment().rows_iter() {
        let bb_depl_name = db.blackbox_deployment().c_deployment_name(bb_depl).to_case(convert_case::Case::Snake);
        let region = db.blackbox_deployment().c_region(bb_depl);
        if let Some(first_region_server) = first_region_server(db, region) {
            let consul_iface = l1proj.consul_network_iface.value(first_region_server);
            let dns_ip = db.network_interface().c_if_ip(*consul_iface);
            for svc_reg in db.blackbox_deployment().c_children_blackbox_deployment_service_registration(bb_depl) {
                let svc_name = db.blackbox_deployment_service_registration().c_service_name(*svc_reg);
                let svc_name_snake = svc_name.to_case(convert_case::Case::Snake);
                let count = db.blackbox_deployment_service_registration().c_referrers_blackbox_deployment_service_instance__service_registration(*svc_reg).len();
                runtime.add_integration_test(
                    format!("bb_depl_{bb_depl_name}_service_{svc_name_snake}_instances_exist"),
                    IntegrationTest::DnsResolutionARecordCount {
                        target_servers: vec![format!("{}:53", dns_ip.clone())],
                        queries: vec![
                            (format!("{svc_name}.service.consul"), count)
                        ]
                    },
                )
            }
        }
    }
}

pub enum BlackBoxDeploymentResource {
    Pg {
        pg_db: TableRowPointerPgDeploymentUnmanagedDb,
        // TODO: add read only user access once such case is needed
        // write_allowed: bool,
    },
    MinIOBucket {
        bucket: TableRowPointerMinioBucket,
        // TODO: add read only permissions to minio
        // write_allowed: bool,
    },
    // NATS unauthenticated for now, rw by default
    NatsCluster {
        cluster: TableRowPointerNatsCluster,
    },
    // TODO: if we need implement clickhouse unmanaged database
    // ClickhouseCluster {
    //     ...
    // },
}

pub fn compute_available_resources_projection(db: &Database) -> Projection<TableRowPointerRegion, BTreeMap<String, BlackBoxDeploymentResource>> {
    Projection::create(db.region().rows_iter(), |region| {
        let mut res: BTreeMap<String, BlackBoxDeploymentResource> =
            BTreeMap::new();

        for minio_cluster in db.region().c_referrers_minio_cluster__region(region) {
            for bucket in db.minio_cluster().c_children_minio_bucket(*minio_cluster) {
                let key_rw = format!(
                    "s3:rw:{}:{}",
                    db.minio_cluster().c_cluster_name(*minio_cluster),
                    db.minio_bucket().c_bucket_name(*bucket),
                );
                //let key_ro = format!(
                //    "s3:ro:{}:{}",
                //    db.minio_cluster().c_cluster_name(*minio_cluster),
                //    db.minio_bucket().c_bucket_name(*bucket),
                //);

                assert!(
                    res.insert(key_rw, BlackBoxDeploymentResource::MinIOBucket {
                        bucket: *bucket,
                        // write_allowed: true,
                    }).is_none(),
                    "must be unique"
                );
                //assert!(
                //    res.insert(key_ro, BlackBoxDeploymentResource::MinIOBucket {
                //        bucket: *bucket,
                //        write_allowed: false,
                //    }).is_none(),
                //    "must be unique"
                //);
            }
        }

        for pg_depl in db.region().c_referrers_pg_deployment__region(region) {
            // only unmanaged databases are candidates
            for unm_db in db.pg_deployment().c_children_pg_deployment_unmanaged_db(*pg_depl) {
                let key_rw = format!(
                    "pg:rw:{}:{}",
                    db.pg_deployment().c_deployment_name(*pg_depl),
                    db.pg_deployment_unmanaged_db().c_db_name(*unm_db),
                );

                assert!(
                    res.insert(key_rw, BlackBoxDeploymentResource::Pg {
                        pg_db: *unm_db,
                    }).is_none(),
                    "must be unique"
                );
            }
        }

        for nats_cluster in db.region().c_referrers_nats_cluster__region(region) {
            let key_rw = format!(
                "nats:rw:{}",
                db.nats_cluster().c_cluster_name(*nats_cluster),
            );
            assert!(
                res.insert(key_rw, BlackBoxDeploymentResource::NatsCluster {
                    cluster: *nats_cluster,
                }).is_none(),
                "must be unique"
            );
        }

        res
    })
}
