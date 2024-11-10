use std::collections::HashMap;

use crate::{
    database::{
        Database,
        TableRowPointerBackendApplicationDeploymentIngress,
        TableRowPointerFrontendApplicationDeployment,
        TableRowPointerFrontendApplicationDeploymentIngress,
        TableRowPointerFrontendApplicationExternalLink,
        TableRowPointerFrontendApplicationExternalPage,
        TableRowPointerFrontendApplicationUsedEndpoint,
    },
    static_analysis::{
        server_runtime::{
            PgAccessKind, ChAccessKind, NomadJobKind, NomadJobStage, ServerRuntime, VaultSecretHandle,
            VaultSecretRequest, epl_architecture_to_nomad_architecture, MinIOBucketPermission,
        },
        L1Projections, PlatformValidationError, networking::{region_monitoring_clusters, region_loki_clusters, region_tempo_clusters}, docker_images::{image_handle_from_backend_app_deployment, image_handle_from_frontend_app_deployment},
    },
};

pub fn deploy_epl_applications(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    deploy_backend_applications(
        db,
        runtime,
        l1proj,
    )?;
    deploy_frontend_applications(
        db,
        runtime,
        l1proj,
    )?;

    Ok(())
}

fn deploy_backend_applications(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    for depl in db.backend_application_deployment().rows_iter() {
        let depl_name = db.backend_application_deployment().c_deployment_name(depl);
        let ns = db.backend_application_deployment().c_namespace(depl);
        let app_ptr = db.backend_application_deployment().c_application_name(depl);
        let region = db.backend_application_deployment().c_region(depl);
        let region_name = db.region().c_region_name(region);
        let workload_architecture = db.backend_application_deployment().c_workload_architecture(depl);
        let app_name = db.backend_application().c_application_name(app_ptr);
        let pg_wiring = l1proj.application_deployment_pg_wirings
            .get(&depl)
            .expect("PG wiring must have been computed at this point");
        let ch_wiring = l1proj.application_deployment_ch_wirings
            .get(&depl)
            .expect("CH wiring must have been computed at this point");
        let bucket_wiring = l1proj.application_deployment_bucket_wirings
            .get(&depl)
            .expect("Bucket wiring must have been computed at this point");
        let queues_wiring = l1proj.application_deployment_stream_wirings
            .get(&depl)
            .expect("queues wiring must have been computed at this point");
        let app_configs = l1proj.application_deployment_configs
            .get(&depl)
            .expect("application configs must have been computed at this point");
        let consul_service_slug = format!("epl-app-{depl_name}");
        let nomad_job_name = format!("app-{depl_name}");
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &consul_service_slug);
        let monitoring_cluster = db.backend_application_deployment().c_monitoring_cluster(depl);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::ApplicationMonitoringClusterDoesntExistInRegion {
            application_deployment: depl_name.clone(),
            application_name: app_name.clone(),
            application_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = db.backend_application_deployment().c_loki_cluster(depl);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::ApplicationLoggingClusterDoesntExistInRegion {
            application_deployment: depl_name.clone(),
            application_name: app_name.clone(),
            application_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;
        let tracing_cluster = db.backend_application_deployment().c_tracing_cluster(depl);
        let tracing_cluster = l1proj.tempo_clusters.pick(
            region, tracing_cluster
        ).ok_or_else(|| PlatformValidationError::ApplicationTracingClusterDoesntExistInRegion {
            application_deployment: depl_name.clone(),
            application_name: app_name.clone(),
            application_region: db.region().c_region_name(region).clone(),
            not_found_tempo_cluster: tracing_cluster.to_string(),
            available_tempo_clusters: region_tempo_clusters(db, region),
        })?;

        let tempo_cluster_name = db.tempo_cluster().c_cluster_name(tracing_cluster);
        let tempo_grpc_push_port = db.tempo_cluster().c_otlp_grpc_port(tracing_cluster);
        let otlp_url = format!("http://epl-tempo-{tempo_cluster_name}.service.consul:{tempo_grpc_push_port}");

        let mut env_variables: Vec<(String, String)> = Vec::new();
        let mut secret_requests: Vec<(String, String, VaultSecretRequest)> = Vec::new();
        for pg_shard in db
            .backend_application()
            .c_children_backend_application_pg_shard(app_ptr)
        {
            // task.add_secure_env_variables(fname, secrets)
            // postgresql://{db_name}:{db_password}@epl-pg-{db_depl_name}:{master_port}/{db_name}

            let shard_name = db.backend_application_pg_shard().c_shard_name(*pg_shard);
            let w = pg_wiring.get(pg_shard).unwrap();
            let db_depl = db.pg_deployment_schemas().c_parent(*w);
            let db_region = db.pg_deployment().c_region(db_depl);
            if region != db_region {
                return Err(PlatformValidationError::ApplicationPgWiringDatabaseIsInDifferentRegion {
                    application_deployment: depl_name.clone(),
                    application_name: app_name.clone(),
                    application_region: db.region().c_region_name(region).clone(),
                    application_db_name: shard_name.clone(),
                    application_db_wired_deployment: db.pg_deployment().c_deployment_name(db_depl).clone(),
                    application_db_wired_region: db.region().c_region_name(db_region).clone(),
                });
            }
            let creds = runtime.fetch_pg_access(&PgAccessKind::Managed(*w));
            secret_requests.push((
                format!("EPL_PG_CONN_{}", shard_name.to_uppercase()),
                format!("pg_shard_{shard_name}"),
                VaultSecretRequest::ExistingVaultSecret {
                    handle: Box::new(creds.db_password.clone()),
                    sprintf: Some(format!(
                        "postgresql://{}:%s@{}:{}/{}",
                        creds.db_user, creds.db_host, creds.db_master_port, creds.db_database
                    )),
                },
            ));
        }

        for ch_shard in db
            .backend_application()
            .c_children_backend_application_ch_shard(app_ptr)
        {
            let shard_name = db.backend_application_ch_shard().c_shard_name(*ch_shard);
            let w = ch_wiring.get(ch_shard).unwrap();
            let db_depl = db.ch_deployment_schemas().c_parent(*w);
            let db_region = db.ch_deployment().c_region(db_depl);
            let queries = l1proj.application_ch_shard_queries.value(*ch_shard);
            if region != db_region {
                return Err(PlatformValidationError::ApplicationChWiringDatabaseIsInDifferentRegion {
                    application_deployment: depl_name.clone(),
                    application_name: app_name.clone(),
                    application_region: db.region().c_region_name(region).clone(),
                    application_db_name: shard_name.clone(),
                    application_db_wired_deployment: db.ch_deployment().c_deployment_name(db_depl).clone(),
                    application_db_wired_region: db.region().c_region_name(db_region).clone(),
                });
            }

            let creds =
                if !queries.inserters.is_empty() || !queries.mutators.is_empty() {
                    runtime.fetch_ch_access(&ChAccessKind::ManagedReadWrite(*w))
                } else {
                    runtime.fetch_ch_access(&ChAccessKind::ManagedReadOnly(*w))
                };

            env_variables.push((
                format!("EPL_CH_{}_URL", shard_name.to_uppercase()),
                format!("http://{}:{}", creds.db_host, creds.db_http_port),
            ));
            env_variables.push((
                format!("EPL_CH_{}_USER", shard_name.to_uppercase()),
                creds.db_user.clone(),
            ));
            env_variables.push((
                format!("EPL_CH_{}_DATABASE", shard_name.to_uppercase()),
                creds.db_database.clone(),
            ));
            secret_requests.push((
                format!("EPL_CH_{}_PASSWORD", shard_name.to_uppercase()),
                format!("ch_shard_{shard_name}_password"),
                VaultSecretRequest::ExistingVaultSecret {
                    handle: Box::new(creds.db_password.clone()),
                    sprintf: None,
                },
            ));
        }

        let mut builder = runtime.issue_vault_secret(region, &format!("app/{depl_name}"));
        let mut secret_handles: Vec<(String, VaultSecretHandle)> = Vec::new();
        for (env_var, srk, req) in secret_requests {
            let handle = builder.request_secret(region, &srk, req);
            secret_handles.push((env_var, handle));
        }

        struct MinIOBucketData {
            bucket: String,
            url: String,
            user: String,
            password: VaultSecretHandle,
            pw_env_name: String,
            app_bucket: String,
        }

        let mut minio_creds: Vec<MinIOBucketData> = Vec::new();
        for s3_bucket in db
            .backend_application()
            .c_children_backend_application_s3_bucket(app_ptr)
        {
            let s3_bucket_name = db.backend_application_s3_bucket().c_bucket_name(*s3_bucket);
            let s3_bucket_name_uppercase = s3_bucket_name.to_uppercase();
            let final_bucket = bucket_wiring.get(s3_bucket).unwrap();
            let final_bucket_name = db.minio_bucket().c_bucket_name(*final_bucket);
            let minio_cluster = db.minio_bucket().c_parent(*final_bucket);
            let minio_cluster_name = db.minio_cluster().c_cluster_name(minio_cluster);
            let minio_region = db.minio_cluster().c_region(minio_cluster);
            if minio_region != region {
                return Err(PlatformValidationError::ApplicationS3BucketWiringIsInDifferentRegion {
                    application_deployment: depl_name.clone(),
                    application_name: app_name.clone(),
                    application_region: db.region().c_region_name(region).clone(),
                    application_bucket_name: s3_bucket_name.clone(),
                    application_bucket_wired_minio_region: db.region().c_region_name(minio_region).clone(),
                    application_bucket_wired_minio_cluster: minio_cluster_name.clone(),
                    application_bucket_wired_minio_bucket: final_bucket_name.clone(),
                });
            }
            let minio_port = db.minio_cluster().c_lb_port(minio_cluster);
            let minio_service = db.minio_cluster().c_consul_service_name(minio_cluster);
            let minio_user = format!("epl_app_{}", depl_name.replace("-", "_"));
            let minio_password = builder.fetch_minio_bucket_credentials(
                db,
                &format!("minio_bucket_{}_password", s3_bucket_name),
                &minio_user,
                *final_bucket,
                MinIOBucketPermission::ReadWrite,
            );

            minio_creds.push(MinIOBucketData {
                url: format!("http://{minio_service}.service.consul:{minio_port}"),
                bucket: final_bucket_name.clone(),
                password: minio_password,
                user: minio_user,
                pw_env_name: format!("EPL_S3_{s3_bucket_name_uppercase}_PASSWORD"),
                app_bucket: s3_bucket_name.clone(),
            })
        }

        let fin_secrets = builder.finalize();

        let db_creds = secret_handles
            .iter()
            .map(|(a, b)| (a.as_str(), b))
            .collect::<Vec<_>>();

        let minio_secrets = minio_creds
            .iter()
            .map(|i| {
                (i.pw_env_name.as_str(), &i.password)
            })
            .collect::<Vec<_>>();

        let memory = runtime.reserve_stateless_memory_mb(
            format!("EPL application deployment {depl_name}"),
            db.backend_application_deployment().c_memory_mb(depl),
        );
        let port = db.backend_application_deployment().c_http_port(depl);
        let locked = runtime.lock_port_all_servers(
            port.try_into().unwrap(),
            format!("EPL application deployment {depl_name}"),
        )?;
        let job = runtime.fetch_nomad_job(
            ns,
            nomad_job_name.clone(),
            region,
            NomadJobKind::Stateless,
            NomadJobStage::Application,
        );
        job.set_loki_cluster(loki_cluster);
        job.assign_vault_secrets(fin_secrets);
        let tg = job.fetch_task_group(format!("app"));
        tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
        tg.add_locked_port("app", locked);
        tg.expose_port_as_tcp_service("app", &consul_service);
        tg.collect_prometheus_metrics(&consul_service, monitoring_cluster, None);
        let count =
            db.backend_application_deployment()
                .c_count(depl)
                .try_into()
                .unwrap();
        tg.set_count(count);
        tg.try_set_placement(
            db,
            region,
            db.backend_application_deployment().c_placement(depl),
            &format!("backend application deployment named {depl_name} in region {region_name}"),
            count,
            l1proj.label_database
        )?;
        let docker_image = image_handle_from_backend_app_deployment(db, depl)?;
        let task = tg.fetch_task(format!("app-{depl_name}"), docker_image);
        task.add_memory(memory);
        if db_creds.len() > 0 {
            task.add_secure_env_variables("dbcreds".to_string(), &db_creds);
        }
        if minio_secrets.len() > 0 {
            task.add_secure_env_variables("minio".to_string(), &minio_secrets);
        }
        for minio_cred in &minio_creds {
            let bucket_uppercase = minio_cred.app_bucket.to_uppercase();
            task.set_env_variable(&format!("EPL_S3_{bucket_uppercase}_BUCKET"), &minio_cred.bucket);
            task.set_env_variable(&format!("EPL_S3_{bucket_uppercase}_URI"), &minio_cred.url);
            task.set_env_variable(&format!("EPL_S3_{bucket_uppercase}_USER"), &minio_cred.user);
        }
        // ugly, TODO: unify env variable configs in the future
        for (env_var_k, env_var_v) in env_variables {
            task.set_env_variable(&env_var_k, &env_var_v);
        }
        for (cfg_key, cfg_val) in app_configs {
            let cfg_name = db.backend_application_config().c_config_name(*cfg_key);
            let replaced = cfg_val.replace("\"", "\\\"");
            task.set_env_variable(
                &format!("EPL_CFG_{}", cfg_name.to_uppercase()),
                &replaced,
            );
        }
        task.set_env_variable("EPL_HTTP_SOCKET", &format!("${{meta.private_ip}}:{}", port));
        task.set_env_variable("EPL_DEPLOYMENT_NAME", depl_name);
        task.set_env_variable("OTEL_EXPORTER_OTLP_ENDPOINT", &otlp_url);
        task.set_env_variable("RUST_LOG", "info");

        for nats_stream in db
            .backend_application()
            .c_children_backend_application_nats_stream(app_ptr)
        {
            let stream_name = db
                .backend_application_nats_stream()
                .c_stream_name(*nats_stream);
            let w = queues_wiring.get(nats_stream).unwrap();
            let nats_cluster = db.nats_jetstream_stream().c_parent(*w);
            let nats_region = db.nats_cluster().c_region(nats_cluster);
            if region != nats_region {
                return Err(PlatformValidationError::ApplicationStreamWiringNatsClusterIsInDifferentRegion {
                    application_deployment: depl_name.clone(),
                    application_name: app_name.clone(),
                    application_region: db.region().c_region_name(region).clone(),
                    application_nats_stream_name: stream_name.clone(),
                    application_nats_wired_cluster: db.nats_cluster().c_cluster_name(nats_cluster).clone(),
                    application_nats_wired_region: db.region().c_region_name(nats_region).clone(),
                });
            }
            let cluster_stream_name = db.nats_jetstream_stream().c_stream_name(*w);
            let nats_cluster = db.nats_jetstream_stream().c_parent(*w);
            let nats_conn = format!(
                "nats://epl-nats-{}.service.consul:{}",
                db.nats_cluster().c_cluster_name(nats_cluster),
                db.nats_cluster().c_nats_clients_port(nats_cluster)
            );
            task.set_env_variable(
                &format!("EPL_NATS_CONN_{}", stream_name.to_uppercase()),
                &nats_conn,
            );
            task.set_env_variable(
                &format!("EPL_NATS_STREAM_{}", stream_name.to_uppercase()),
                cluster_stream_name.as_str(),
            );
        }
    }

    Ok(())
}

fn deploy_frontend_applications(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    for depl in db.frontend_application_deployment().rows_iter() {
        let depl_name = db.frontend_application_deployment().c_deployment_name(depl);
        let ns = db.frontend_application_deployment().c_namespace(depl);
        let workload_backend_architecture = db.frontend_application_deployment().c_workload_backend_architecture(depl);
        let app_ptr = db
            .frontend_application_deployment()
            .c_application_name(depl);
        let region = db.frontend_application_deployment().c_region(depl);
        let region_name = db.region().c_region_name(region);
        let app_name = db.frontend_application().c_application_name(app_ptr);
        let consul_service_slug = format!("epl-app-{depl_name}");
        let nomad_job_name = format!("app-{depl_name}");
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &consul_service_slug);
        let loki_cluster = db.frontend_application_deployment().c_loki_cluster(depl);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::FrontendApplicationLoggingClusterDoesntExistInRegion {
            application_deployment: depl_name.clone(),
            application_name: app_name.clone(),
            application_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        let memory = runtime.reserve_stateless_memory_mb(
            format!("EPL frontend application deployment {depl_name}"),
            db.frontend_application_deployment().c_memory_mb(depl),
        );
        let port = db.frontend_application_deployment().c_http_port(depl);
        let locked = runtime.lock_port_all_servers(
            port.try_into().unwrap(),
            format!("EPL frontend application deployment {depl_name}"),
        )?;
        let job = runtime.fetch_nomad_job(
            ns,
            nomad_job_name.clone(),
            region,
            NomadJobKind::Stateless,
            NomadJobStage::Application,
        );
        job.set_loki_cluster(loki_cluster);
        let tg = job.fetch_task_group(format!("app"));
        tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_backend_architecture));

        tg.add_locked_port("app", locked);
        tg.expose_port_as_tcp_service("app", &consul_service);
        let count =
            db.frontend_application_deployment()
                .c_count(depl)
                .try_into()
                .unwrap();
        tg.set_count(count);
        tg.try_set_placement(
            db,
            region,
            db.frontend_application_deployment().c_placement(depl),
            &format!("frontend application deployment named {depl_name} in region {region_name}"),
            count,
            l1proj.label_database
        )?;

        let docker_image = image_handle_from_frontend_app_deployment(db, depl)?;
        let task = tg.fetch_task(format!("app-{depl_name}"), docker_image);
        task.add_memory(memory);
        task.set_env_variable("EPL_HTTP_SOCKET", &format!("${{meta.private_ip}}:{}", port));
        task.set_env_variable("EPL_DEPLOYMENT_NAME", depl_name);
        task.set_env_variable(
            "EPL_ENDPOINT_MAPPING",
            &generate_endpoint_mappings(db, depl, l1proj.frontend_deployment_endpoint_wirings),
        );
        task.set_env_variable(
            "EPL_EXTPAGES_MAPPING",
            &generate_page_mappings(db, depl, l1proj.frontend_deployment_page_wirings),
        );
        task.set_env_variable(
            "EPL_EXTLINKS_MAPPING",
            &generate_link_mappings(db, depl, l1proj.frontend_deployment_link_wirings),
        );
        for ing in db
            .frontend_application_deployment()
            .c_referrers_frontend_application_deployment_ingress__deployment(depl)
        {
            let mountpoint = db
                .frontend_application_deployment_ingress()
                .c_mountpoint(*ing);
            if mountpoint != "/" {
                assert!(mountpoint.ends_with('/'));
                let root_path = mountpoint.strip_suffix('/').unwrap();
                // we assume only one ingress max
                task.set_env_variable("OVERRIDE_ROOT_PATH", root_path);
            }
        }
    }

    Ok(())
}

fn generate_endpoint_mappings(
    db: &Database,
    deployment: TableRowPointerFrontendApplicationDeployment,
    endpoints: &HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationUsedEndpoint,
            TableRowPointerBackendApplicationDeploymentIngress,
        >,
    >,
) -> String {
    let mut res = "{".to_string();

    let app_ptr = db
        .frontend_application_deployment()
        .c_application_name(deployment);

    let endpoints = endpoints.get(&deployment).unwrap();
    for (idx, ue) in db
        .frontend_application()
        .c_children_frontend_application_used_endpoint(app_ptr)
        .iter()
        .enumerate()
    {
        let is_last = idx
            == db
                .frontend_application()
                .c_children_frontend_application_used_endpoint(app_ptr)
                .len()
                - 1;
        let endpoint_name = db.frontend_application_used_endpoint().c_endpoint_name(*ue);
        let v = endpoints.get(ue).unwrap();
        let mountpoint = db.backend_application_deployment_ingress().c_mountpoint(*v);

        res += endpoint_name;
        res += ":\'";
        res += mountpoint;
        res += "'";

        if !is_last {
            res += ",";
        }
    }

    res += "}";
    res
}

fn generate_page_mappings(
    db: &Database,
    deployment: TableRowPointerFrontendApplicationDeployment,
    endpoints: &HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationExternalPage,
            TableRowPointerFrontendApplicationDeploymentIngress,
        >,
    >,
) -> String {
    let mut res = "{".to_string();

    let app_ptr = db
        .frontend_application_deployment()
        .c_application_name(deployment);

    let endpoints = endpoints.get(&deployment).unwrap();
    for (idx, ue) in db
        .frontend_application()
        .c_children_frontend_application_external_page(app_ptr)
        .iter()
        .enumerate()
    {
        let is_last = idx
            == db
                .frontend_application()
                .c_children_frontend_application_external_page(app_ptr)
                .len()
                - 1;
        let endpoint_name = db.frontend_application_external_page().c_link_name(*ue);
        let v = endpoints.get(ue).unwrap();
        let tld = db.frontend_application_deployment_ingress().c_tld(*v);
        let subdomain = db.frontend_application_deployment_ingress().c_subdomain(*v);
        let mountpoint = db
            .frontend_application_deployment_ingress()
            .c_mountpoint(*v);

        res += endpoint_name;
        res += ":\'https://";
        if !subdomain.is_empty() {
            res += subdomain;
            res += ".";
        }
        res += db.tld().c_domain(tld);
        res += mountpoint;
        res += "'";

        if !is_last {
            res += ",";
        }
    }

    res += "}";
    res
}

fn generate_link_mappings(
    db: &Database,
    deployment: TableRowPointerFrontendApplicationDeployment,
    endpoints: &HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationExternalLink,
            TableRowPointerBackendApplicationDeploymentIngress,
        >,
    >,
) -> String {
    let mut res = "{".to_string();

    let app_ptr = db
        .frontend_application_deployment()
        .c_application_name(deployment);

    let endpoints = endpoints.get(&deployment).unwrap();
    for (idx, ue) in db
        .frontend_application()
        .c_children_frontend_application_external_link(app_ptr)
        .iter()
        .enumerate()
    {
        let is_last = idx
            == db
                .frontend_application()
                .c_children_frontend_application_external_link(app_ptr)
                .len()
                - 1;
        let endpoint_name = db.frontend_application_external_link().c_link_name(*ue);
        let v = endpoints.get(ue).unwrap();
        let tld = db.backend_application_deployment_ingress().c_tld(*v);
        let subdomain = db.backend_application_deployment_ingress().c_subdomain(*v);
        let mountpoint = db.backend_application_deployment_ingress().c_mountpoint(*v);

        res += endpoint_name;
        res += ":\'https://";
        if !subdomain.is_empty() {
            res += subdomain;
            res += ".";
        }
        res += db.tld().c_domain(tld);
        res += mountpoint;
        res += "'";

        if !is_last {
            res += ",";
        }
    }

    res += "}";
    res
}
