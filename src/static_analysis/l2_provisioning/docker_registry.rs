use std::collections::HashMap;

use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerRegion, TableRowPointerDockerRegistryInstance},
    static_analysis::{
        server_runtime::{
            MinIOBucketPermission, NomadJobKind, NomadJobStage, ServerRuntime, VaultSecretHandle, IntegrationTest, epl_architecture_to_nomad_architecture,
        },
        PlatformValidationError, L1Projections, networking::server_region, get_global_settings, docker_images::image_handle_from_pin,
    },
};

pub fn deploy_docker_registry(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections
) -> Result<(), PlatformValidationError> {
    let settings = get_global_settings(db);

    let mut regions_with_registries: HashMap<TableRowPointerRegion, TableRowPointerDockerRegistryInstance>
        = HashMap::new();

    let locked_port = runtime
        .lock_port_all_servers(settings.docker_registry_port.try_into().unwrap(), "Docker registry".to_string())?;

    for dri in db.docker_registry_instance().rows_iter() {
        let region = db.docker_registry_instance().c_region(dri);
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &settings.docker_registry_service_name);
        let docker_image_pin = db.docker_registry_instance().c_docker_image(dri);
        // its impossible that duplicate regions are specified because region is also
        // primary key for docker_registry_instance table
        assert!(regions_with_registries.insert(region, dri).is_none());

        let minio_bucket = db.docker_registry_instance().c_minio_bucket(dri);
        let minio_cluster = db.minio_bucket().c_parent(minio_bucket);
        let minio_region = db.minio_cluster().c_region(minio_cluster);

        if region != minio_region {
            return Err(
                PlatformValidationError::DockerRegistryAndMinioClusterAreInDifferentRegions {
                    docker_registry_region: db.region().c_region_name(region).clone(),
                    minio_cluster: db.minio_cluster().c_cluster_name(minio_cluster).clone(),
                    minio_cluster_region: db.region().c_region_name(minio_region).clone(),
                },
            );
        }

        let region_used_architectures = l1proj.used_architectures_per_region.get(&region).unwrap();

        let minio_bucket_name = db.minio_bucket().c_bucket_name(minio_bucket);
        let minio_consul_service_fqdn = format!(
            "{}.service.consul",
            db.minio_cluster()
                .c_consul_service_name(db.minio_bucket().c_parent(minio_bucket))
        );
        let minio_lb_port = db.minio_cluster().c_lb_port(minio_cluster);
        let minio_user = "docker_registry";
        let reserved_mem = runtime.reserve_memory_every_server_mb(
            db,
            "Docker registry".to_string(),
            db.docker_registry_instance().c_memory_mb(dri),
        )?;

        let mut secrets_builder = runtime.issue_vault_secret(region, "docker-registry");

        let minio_password = secrets_builder.fetch_minio_bucket_credentials(
            db,
            "minio_bucket_password",
            minio_user,
            minio_bucket,
            MinIOBucketPermission::ReadWrite,
        );
        let registry_http_secret = secrets_builder.request_secret(
            region,
            "registry_http_secret",
            crate::static_analysis::server_runtime::VaultSecretRequest::AlphanumericPassword42Symbols
        );
        let finalized_secrets = secrets_builder.finalize();

        let job = runtime.fetch_nomad_job(
            l1proj.epl_nomad_namespace,
            "docker-registry".to_string(),
            region,
            NomadJobKind::SystemStateless,
            NomadJobStage::SystemJob,
        );

        job.assign_vault_secrets(finalized_secrets);

        for ua in region_used_architectures {
            let tg = job.fetch_task_group("registry".to_string());

            tg.add_locked_port("reg", locked_port.clone());
            tg.expose_port_as_tcp_service("reg", &consul_service);
            tg.constrain_architecture(epl_architecture_to_nomad_architecture(&ua));
            let docker_image = image_handle_from_pin(db, ua, docker_image_pin, "docker_registry")?;

            let task = tg.fetch_task(
                "docker-registry".to_string(),
                docker_image,
            );

            task.add_memory(reserved_mem.clone());
            task.add_secure_env_variables(
                "env".to_string(),
                &[("REGISTRY_HTTP_SECRET", &registry_http_secret)],
            );

            let abs_path = task.add_secure_config(
                "config.yml".to_string(),
                generate_docker_registry_config(
                    settings.docker_registry_port,
                    minio_user,
                    &minio_password,
                    &minio_consul_service_fqdn,
                    minio_lb_port,
                    minio_bucket_name,
                    &settings.docker_registry_service_name,
                ),
            );

            task.set_arguments(vec![abs_path]);
        }
    }

    for region in db.region().rows_iter() {
        if !settings.disable_region_docker_registry_tests && regions_with_registries.get(&region).is_none() {
            // throw this error only if region has servers
            for dc in db.region().c_referrers_datacenter__region(region) {
                for _server in db.datacenter().c_referrers_server__dc(*dc) {
                    return Err(
                        PlatformValidationError::RegionDoesntHaveDockerRegistryInstanceSpecified {
                            region: db.region().c_region_name(region).clone(),
                        },
                    );
                }
            }
        }
    }

    for dri in db.docker_registry_instance().rows_iter() {
        docker_registry_tests(db, l1proj, runtime, dri);
    }

    Ok(())
}

fn docker_registry_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime, dri: TableRowPointerDockerRegistryInstance) {
    let settings = get_global_settings(db);
    let reg = db.docker_registry_instance().c_region(dri);
    let reg_snake = db.region().c_region_name(reg).to_case(convert_case::Case::Snake);
    let mut server_ips = Vec::new();
    for server in db.server().rows_iter() {
        if reg == server_region(db, server) {
            let iface = l1proj.consul_network_iface.value(server);
            server_ips.push(db.network_interface().c_if_ip(*iface).clone());
        }
    }
    // don't go over max results returned by dns service
    if !server_ips.is_empty() && server_ips.len() <= 7 {
        runtime.add_integration_test(
            format!("docker_registry_{reg_snake}_dns_exists"),
            crate::static_analysis::server_runtime::IntegrationTest::DnsResolutionWorksARecords {
                target_servers: vec![format!("{}:53", server_ips[0].clone())],
                queries: vec![
                    ("epl-docker-registry.service.consul".to_string(), server_ips.clone())
                ]
            },
        );
    }

    runtime.add_integration_test(
        format!("docker_registry_{reg_snake}_healthcheck_responds_ok"),
        IntegrationTest::HttpGetRespondsOk {
            server_ips,
            http_server_port: settings.docker_registry_port,
            path: "/".to_string(),
        }
    );
}

fn generate_docker_registry_config(
    registry_port: i64,
    minio_user: &str,
    minio_password: &VaultSecretHandle,
    minio_consul_service_fqdn: &str,
    minio_lb_port: i64,
    minio_bucket_name: &str,
    docker_registry_service: &str,
) -> String {
    let minio_password = minio_password.template_expression();
    format!(
        r#"
version: 0.1
log:
  level: info
  formatter: text
  fields:
    service: registry
    environment: staging
loglevel: info
storage:
  s3:
    accesskey: {minio_user}
    secretkey: {minio_password}
    region: us-east-1
    regionendpoint: http://{minio_consul_service_fqdn}:{minio_lb_port}
    bucket: {minio_bucket_name}
    encrypt: false
    secure: false
    v4auth: true
    chunksize: 5242880
    rootdirectory: /
  delete:
    enabled: true
  maintenance:
    uploadpurging:
      enabled: false
      age: 5040h
      interval: 24h
      dryrun: false
    readonly:
      enabled: false
  cache:
    blobdescriptor: inmemory
    blobdescriptorsize: 10000
http:
  addr: {{{{ env "meta.private_ip" }}}}:{registry_port}
  host: http://{docker_registry_service}.service.consul:{registry_port}
"#
    )
}
