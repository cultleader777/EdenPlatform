use std::collections::HashMap;

use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerPgDeployment, TableRowPointerPgSchema, TableRowPointerRegion},
    static_analysis::{
        server_runtime::{
            CustomConsulPolicyBuilder, PgAccessKind, NomadJobKind, NomadJobStage,
            PostgresDbCredentials, ProvisioningResourcePath, ProvisioningScriptTag, ServerRuntime,
            VaultSecretHandle, VaultSecretRequest, IntegrationTest, epl_architecture_to_nomad_architecture,
        },
        PlatformValidationError, L1Projections, networking::{region_monitoring_clusters, region_loki_clusters, server_region, prometheus_metric_exists_test}, docker_images::image_handle_from_pin,
    },
};

pub fn deploy_postgres_instances(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    let pg_deployment_subnet = db
        .network()
        .rows_iter()
        .find(|n| db.network().c_network_name(*n) == "lan")
        .map(|i| db.network().c_cidr(i).clone());
    let job_kind = NomadJobKind::BoundStateful;

    // schema migrations
    for region in db.region().rows_iter() {
        let mut schema_prov_scripts: HashMap<TableRowPointerPgSchema, ProvisioningResourcePath> =
            HashMap::new();
        for schema in db.pg_schema().rows_iter() {
            if l1proj.pg_schemas_in_region.value(region).contains(&schema) {
                let db_name = db.pg_schema().c_schema_name(schema);
                let fname = format!("up_{db_name}.sh");
                let migration = generate_pg_migration_script(db, schema);
                schema_prov_scripts.insert(
                    schema,
                    runtime.add_provisioning_resource(region, "pg-migrations", fname, migration, true, Vec::default()),
                );
            }
        }
        runtime.add_provisioning_script(
            region,
            ProvisioningScriptTag::SystemResourceProvision,
            "provision-pg-instances.sh",
            generate_pg_init_script(db, region, &schema_prov_scripts),
        );
    }

    for depl in db.pg_deployment().rows_iter() {
        let pg_deployment_subnet = pg_deployment_subnet
            .as_deref()
            .unwrap_or("no network to deploy found boi!");
        let region = db.pg_deployment().c_region(depl);
        let depl_name = db.pg_deployment().c_deployment_name(depl);
        let namespace = db.pg_deployment().c_namespace(depl);
        let workload_architecture = db.pg_deployment().c_workload_architecture(depl);
        let docker_image_postgres = db.pg_deployment().c_docker_image_pg(depl);
        let docker_image_haproxy = db.pg_deployment().c_docker_image_haproxy(depl);
        let docker_image_pg_exporter = db.pg_deployment().c_docker_image_pg_exporter(depl);
        let synchronous_replication = db.pg_deployment().c_synchronous_replication(depl);
        let replica_rolling_update_delay_seconds = db.pg_deployment().c_replica_rolling_update_delay_seconds(depl);

        let monitoring_cluster = db.pg_deployment().c_monitoring_cluster(depl);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::PgDeploymentMonitoringClusterDoesntExistInRegion {
            pg_deployment: depl_name.clone(),
            db_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = db.pg_deployment().c_loki_cluster(depl);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::PgDeploymentLoggingClusterDoesntExistInRegion {
            pg_deployment: depl_name.clone(),
            db_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;

        let service_slug = format!("epl-pg-{depl_name}");
        let job_name = format!("pg-{depl_name}");
        let haproxy_prom_exp_service_slug = format!("epl-pg-{depl_name}-hap-exp");
        let pg_prom_exp_service_slug = format!("epl-pg-{depl_name}-pg-exp");
        let consul_service = runtime.instantiate_and_seal_consul_service(region, &service_slug);
        let haproxy_prom_exp_consul_service =
            runtime.instantiate_and_seal_consul_service(region, &haproxy_prom_exp_service_slug);
        let pg_prom_exp_consul_service =
            runtime.instantiate_and_seal_consul_service(region, &pg_prom_exp_service_slug);
        let mut component_secrets_builder = runtime.issue_vault_secret(region, &format!("pg/{depl_name}"));
        // how to say this job needs secrets without parsing text?
        let mut consul_policy_builder = CustomConsulPolicyBuilder::new(service_slug.clone());
        consul_policy_builder.add_kw_read_write(&format!("epl-patroni/{service_slug}"));
        consul_policy_builder.allow_service_write(&consul_service);
        consul_policy_builder.allow_session_write();
        let consul_policy = consul_policy_builder.build();

        let consul_token = component_secrets_builder.request_secret(
            region,
            "consul_token",
            VaultSecretRequest::ConsulTokenWithPolicy {
                policy: consul_policy,
            },
        );
        let pg_superuser_password = component_secrets_builder.request_secret(
            region,
            "pg_superuser_password",
            VaultSecretRequest::AlphanumericPassword42Symbols,
        );
        let pg_admin_password = component_secrets_builder.request_secret(
            region,
            "pg_admin_password",
            VaultSecretRequest::AlphanumericPassword42Symbols,
        );
        let pg_replicator_password = component_secrets_builder.request_secret(
            region,
            "pg_replicator_password",
            VaultSecretRequest::AlphanumericPassword42Symbols,
        );
        let pg_rewind_password = component_secrets_builder.request_secret(
            region,
            "pg_rewind_password",
            VaultSecretRequest::AlphanumericPassword42Symbols,
        );
        let pg_exporter_password = component_secrets_builder.request_secret(
            region,
            "pg_exporter_password",
            VaultSecretRequest::AlphanumericPassword42Symbols,
        );

        let mut to_add_pg_access: Vec<(PgAccessKind, PostgresDbCredentials)> = Vec::new();
        for pg_schema in db.pg_deployment().c_children_pg_deployment_schemas(depl) {
            let db_name = db.pg_deployment_schemas().c_db_name(*pg_schema);
            let db_pw_handle = component_secrets_builder.request_secret(
                region,
                &format!("pg_db_{db_name}_password"),
                VaultSecretRequest::AlphanumericPassword42Symbols,
            );

            to_add_pg_access.push((
                PgAccessKind::Managed(*pg_schema),
                PostgresDbCredentials {
                    db_host: consul_service.service_fqdn(),
                    db_master_port: db
                        .pg_deployment()
                        .c_instance_pg_master_port(depl)
                        .try_into()
                        .unwrap(),
                    db_user: db_name.clone(),
                    db_password: db_pw_handle,
                    db_database: db_name.clone(),
                },
            ));
        }

        for pg_schema in db
            .pg_deployment()
            .c_children_pg_deployment_unmanaged_db(depl)
        {
            let db_name = db.pg_deployment_unmanaged_db().c_db_name(*pg_schema);
            let db_pw_handle = component_secrets_builder.request_secret(
                region,
                &format!("pg_db_{db_name}_password"),
                VaultSecretRequest::AlphanumericPassword42Symbols,
            );

            to_add_pg_access.push((
                PgAccessKind::Unmanaged(*pg_schema),
                PostgresDbCredentials {
                    db_host: consul_service.service_fqdn(),
                    db_master_port: db
                        .pg_deployment()
                        .c_instance_pg_master_port(depl)
                        .try_into()
                        .unwrap(),
                    db_user: db_name.clone(),
                    db_password: db_pw_handle,
                    db_database: db_name.clone(),
                },
            ));
        }

        let secrets = PostgresSecretsSet {
            consul_token,
            pg_superuser_password,
            pg_admin_password,
            pg_replicator_password,
            pg_rewind_password,
        };

        let fin_secrets = component_secrets_builder.finalize();

        for (k, v) in to_add_pg_access {
            runtime.add_pg_access(k, v);
        }
        let nomad_job =
            runtime.fetch_nomad_job(namespace, job_name.clone(), region, job_kind, NomadJobStage::SystemJob);
        if nomad_job.loki_cluster().is_none() {
            nomad_job.set_loki_cluster(loki_cluster);
        }
        nomad_job.assign_vault_secrets(fin_secrets);

        let mut update_delay = 0;
        for depl_child in db.pg_deployment().c_children_pg_deployment_instance(depl) {
            let target_mount_path = "/data";

            let instance_id = db.pg_deployment_instance().c_instance_id(*depl_child);
            let server_volume = db.pg_deployment_instance().c_pg_server(*depl_child);
            let server = db.server_volume().c_parent(server_volume);
            let hostname = db.server().c_hostname(server);

            let server_region = server_region(db, server);
            if region != server_region {
                return Err(PlatformValidationError::PgDeploymentInstanceIsOutsideSpecifiedRegion {
                    pg_deployment: depl_name.clone(),
                    db_region: db.region().c_region_name(region).clone(),
                    server: hostname.clone(),
                    server_region: db.region().c_region_name(server_region).clone(),
                });
            }

            let server_lock = runtime.lock_server_with_label(
                db,
                format!("epl-pg-{hostname}-{depl_name}"),
                server,
            )?;
            let server_data = runtime.fetch_server_data(db, server);
            let volume_lock = server_data.server_volume_write_lock(
                db,
                server_volume,
                format!("Exclusive epl-postgres-{depl_name} database volume lock"),
            )?;

            let mut mem_blocks = Vec::new();
            {
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("postgres shared_buffers {depl_name}-{instance_id}"),
                    db.pg_deployment().c_shared_buffers_mb(depl),
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("postgres work_mem {depl_name}-{instance_id}"),
                    db.pg_deployment().c_work_mem_mb(depl),
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("postgres maintenance_work_mem {depl_name}-{instance_id}"),
                    db.pg_deployment().c_maintenance_work_mem_mb(depl),
                )?);
                mem_blocks.push(server_data.reserve_memory_mb(
                    format!("postgres overhead mem {depl_name}-{instance_id}"),
                    db.pg_deployment().c_overhead_mem_mb(depl),
                )?);
            }
            let mem_for_haproxy = server_data
                .reserve_memory_mb(format!("postgres haproxy {depl_name}-{instance_id}"), 32)?;
            let mem_for_postgres_exporter = server_data
                .reserve_memory_mb(format!("postgres exporter {depl_name}-{instance_id}"), 32)?;

            let pg_main_port = db
                .pg_deployment()
                .c_instance_pg_port(depl)
                .try_into()
                .unwrap();
            let pg_master_port = db
                .pg_deployment()
                .c_instance_pg_master_port(depl)
                .try_into()
                .unwrap();
            let pg_slave_port = db
                .pg_deployment()
                .c_instance_pg_slave_port(depl)
                .try_into()
                .unwrap();
            let pg_patroni_port = db
                .pg_deployment()
                .c_instance_patroni_port(depl)
                .try_into()
                .unwrap();
            let pg_haproxy_metrics_port = db
                .pg_deployment()
                .c_instance_haproxy_metrics_port(depl)
                .try_into()
                .unwrap();
            let pg_exporter_port = db
                .pg_deployment()
                .c_instance_pg_exporter_port(depl)
                .try_into()
                .unwrap();
            let mut port_locks = Vec::new();
            {
                port_locks.push((
                    "pg_main",
                    server_data.lock_port(
                        db,
                        pg_main_port,
                        format!("Postgres main port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "pg_ha_master",
                    server_data.lock_port(
                        db,
                        pg_master_port,
                        format!("Postgres master haproxy port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "pg_ha_slave",
                    server_data.lock_port(
                        db,
                        pg_slave_port,
                        format!("Postgres slave haproxy port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "patroni",
                    server_data.lock_port(
                        db,
                        pg_patroni_port,
                        format!("Postgres patroni port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "haproxy_metrics",
                    server_data.lock_port(
                        db,
                        pg_haproxy_metrics_port,
                        format!("Postgres haproxy port for {depl_name}"),
                    )?,
                ));
                port_locks.push((
                    "pg_exporter",
                    server_data.lock_port(
                        db,
                        pg_exporter_port,
                        format!("Postgres exporter port for {depl_name}"),
                    )?,
                ));
            }

            let nomad_job = runtime.fetch_nomad_job(
                namespace,
                job_name.clone(),
                region,
                job_kind,
                NomadJobStage::SystemJob,
            );
            let depl_tg = nomad_job.fetch_task_group(format!("pg-{instance_id}"));
            // don't restart all nodes at once, we do this because
            // nomad doesn't react to update stanza between different groups
            // all groups are separate
            depl_tg.set_shutdown_delay_seconds(update_delay);
            update_delay += replica_rolling_update_delay_seconds;
            depl_tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
            depl_tg.assign_server_lock(server_lock);
            for (pm, lp) in port_locks {
                depl_tg.add_locked_port(pm, lp);
            }
            depl_tg.expose_port_as_tcp_service("haproxy_metrics", &haproxy_prom_exp_consul_service);
            depl_tg.expose_port_as_tcp_service("pg_exporter", &pg_prom_exp_consul_service);
            depl_tg.collect_prometheus_metrics(
                &haproxy_prom_exp_consul_service,
                monitoring_cluster,
                None,
            );
            depl_tg.collect_prometheus_metrics(&pg_prom_exp_consul_service, monitoring_cluster, None);
            let docker_image_postgres = image_handle_from_pin(
                db, &workload_architecture, docker_image_postgres, "postgres_wpatroni_wconsul"
            )?;

            let task =
                    depl_tg.fetch_task(
                        format!("pg-{depl_name}-{instance_id}-patroni"),
                        docker_image_postgres,
                    );

            for mb in mem_blocks {
                task.add_memory(mb);
            }

            task.bind_volume(volume_lock, target_mount_path.to_string());

            let depl_config =
                generate_deployment_config(
                    db, depl, instance_id, pg_deployment_subnet,
                    &secrets, synchronous_replication,
                );

            let config_path = task.add_secure_config("patroni.yml".to_string(), depl_config);
            task.set_env_variable("PATRONICTL_CONFIG_FILE", &config_path);

            task.set_entrypoint(vec!["/usr/local/bin/patroni".to_string()]);
            task.set_arguments(vec![config_path]);

            let docker_image_haproxy = image_handle_from_pin(
                db, &workload_architecture, docker_image_haproxy, "haproxy"
            )?;
            let hp_task = depl_tg.fetch_task(
                format!("pg-{depl_name}-{instance_id}-haproxy"),
                docker_image_haproxy,
            );

            let haproxy_cfg = hp_task.add_secure_config(
                "haproxy.cfg".to_string(),
                generate_haproxy_config(
                    pg_main_port,
                    pg_master_port,
                    pg_slave_port,
                    pg_haproxy_metrics_port,
                    consul_service.service_name(),
                ),
            );

            hp_task.set_entrypoint(vec!["/usr/local/sbin/haproxy".to_string()]);
            hp_task.set_arguments(vec!["-W".to_string(), "-f".to_string(), haproxy_cfg]);

            hp_task.add_memory(mem_for_haproxy);

            let docker_image_pg_exporter = image_handle_from_pin(
                db, &workload_architecture, docker_image_pg_exporter, "postgres_exporter"
            )?;
            let prom_exp_task = depl_tg.fetch_task(
                format!("pg-{depl_name}-{instance_id}-prom-exp"),
                docker_image_pg_exporter,
            );
            prom_exp_task.add_memory(mem_for_postgres_exporter);
            prom_exp_task.add_secure_env_variables(
                "exporter.env".to_string(),
                &[("DATA_SOURCE_PASS", &pg_exporter_password)],
            );
            prom_exp_task.set_env_variable(
                "DATA_SOURCE_URI",
                &format!("${{meta.private_ip}}:{pg_main_port}/postgres?sslmode=disable"),
            );
            prom_exp_task.set_env_variable("DATA_SOURCE_USER", "postgres_exporter");

            let arguments = vec![
                format!("--web.listen-address=${{meta.private_ip}}:{pg_exporter_port}"),
                "--config.file=/dev/null".to_string(),
                "--collector.database".to_string(),
                "--collector.bgwriter".to_string(),
                "--collector.replication_slot".to_string(),
                "--auto-discover-databases".to_string(),
            ];
            prom_exp_task.set_arguments(arguments);
        }

        pg_deployment_tests(db, l1proj, depl, runtime);
    }

    Ok(())
}

fn pg_deployment_tests(
    db: &Database,
    l1proj: &L1Projections,
    depl: TableRowPointerPgDeployment,
    runtime: &mut ServerRuntime,
) {
    let mut inst_ips = Vec::new();
    let mut open_sockets = Vec::new();
    let port_start = db.pg_deployment().c_instance_pg_port(depl);
    let region = db.pg_deployment().c_region(depl);
    let mon_cluster = l1proj.monitoring_clusters.pick(region, &db.pg_deployment().c_monitoring_cluster(depl));
    for inst in db.pg_deployment().c_children_pg_deployment_instance(depl) {
        let vol = db.pg_deployment_instance().c_pg_server(*inst);
        let srv = db.server_volume().c_parent(vol);
        let iface = l1proj.consul_network_iface.value(srv);
        let ip = db.network_interface().c_if_ip(*iface);
        inst_ips.push(ip.clone());
        for port in port_start..port_start + 4 {
            open_sockets.push(format!("{ip}:{port}"))
        }
    }
    let depl_name = db.pg_deployment().c_deployment_name(depl);
    let depl_snake = depl_name.to_case(convert_case::Case::Snake);

    runtime.add_integration_test(
        format!("pg_deployment_{depl_snake}_dns_exists"),
        IntegrationTest::DnsResolutionWorksARecords {
            target_servers: vec![format!("{}:53", inst_ips[0].clone())],
            queries: vec![
                (format!("epl-pg-{depl_name}.service.consul"), inst_ips.clone())
            ]
        },
    );

    runtime.add_integration_test(
        format!("pg_deployment_{depl_snake}_sockets_open"),
        IntegrationTest::TcpSocketsOpen {
            target_sockets: open_sockets,
        },
    );

    if let Some(mon_cluster) = mon_cluster {
        runtime.add_integration_test(
            format!("pg_deployment_{depl_snake}_prometheus_metrics_gathered"),
            prometheus_metric_exists_test(db, l1proj, mon_cluster, &format!("pg_locks_count{{job='epl-pg-{depl_name}-pg-exp'}}"))
        );

        let mut db_exists = |db_name: &str| {
            runtime.add_integration_test(
                format!("pg_deployment_{depl_snake}_db_exists_{db_name}"),
                prometheus_metric_exists_test(
                    db,
                    l1proj,
                    mon_cluster,
                    &format!("pg_locks_count{{job='epl-pg-{depl_name}-pg-exp',datname='{db_name}'}}")
                )
            );
        };

        db_exists("postgres");
        for schema in db.pg_deployment().c_children_pg_deployment_schemas(depl) {
            db_exists(db.pg_deployment_schemas().c_db_name(*schema));
        }
        for unm_db in db.pg_deployment().c_children_pg_deployment_unmanaged_db(depl) {
            db_exists(db.pg_deployment_unmanaged_db().c_db_name(*unm_db));
        }
    }
}

struct PostgresSecretsSet {
    consul_token: VaultSecretHandle,
    pg_admin_password: VaultSecretHandle,
    pg_replicator_password: VaultSecretHandle,
    pg_superuser_password: VaultSecretHandle,
    pg_rewind_password: VaultSecretHandle,
}

fn generate_deployment_config(
    db: &Database,
    depl: TableRowPointerPgDeployment,
    instance_id: i64,
    subnet_access: &str,
    secrets: &PostgresSecretsSet,
    synchronous_replication: bool,
) -> String {
    let depl_name = db.pg_deployment().c_deployment_name(depl);
    let pg_port = db.pg_deployment().c_instance_pg_port(depl);
    let patroni_port = db.pg_deployment().c_instance_patroni_port(depl);
    let shared_buffers_mb = db.pg_deployment().c_shared_buffers_mb(depl);
    let work_mem_mb = db.pg_deployment().c_work_mem_mb(depl);
    let maintenance_work_mem_mb = db.pg_deployment().c_maintenance_work_mem_mb(depl);
    let max_connections = db.pg_deployment().c_max_connections(depl);
    // need to have access to consul secret here for all pg instances?
    let consul_token = secrets.consul_token.template_expression();
    let pg_superuser_password = secrets.pg_superuser_password.template_expression();
    let pg_replicator_password = secrets.pg_replicator_password.template_expression();
    let pg_rewind_password = secrets.pg_rewind_password.template_expression();
    let pg_admin_password = secrets.pg_admin_password.template_expression();
    let maybe_synchronous_replication =
        if synchronous_replication {
            r#"
    synchronous_commit: "on"
    synchronous_standby_names: "*""#
        } else { "" };
    format!(
        r#"
scope: epl-pg-{depl_name}
name: instance-{instance_id}
namespace: /epl-patroni

restapi:
  listen: {{{{ env "meta.private_ip" }}}}:{patroni_port}
  connect_address: {{{{ env "meta.private_ip" }}}}:{patroni_port}

consul:
  host: 127.0.0.1
  port: 8500
  scheme: http
  register_service: true
  token: {consul_token}

bootstrap:
  # this section will be written into Etcd:/<namespace>/<scope>/config after initializing new cluster
  # and all other cluster members will use it as a `global configuration`
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576
    postgresql:
      use_pg_rewind: true

  initdb:
  - encoding: UTF8
  - data-checksums

  pg_hba:
  - host replication replicator {subnet_access} md5
  - host all all {subnet_access} md5

  users:
    admin:
      password: "{pg_admin_password}"
      options:
        - createrole
        - createdb
    replicator:
      password: "{pg_replicator_password}"
      options:
        - replication

postgresql:
  parameters:{maybe_synchronous_replication}
    work_mem: {work_mem_mb}MB
    shared_buffers: {shared_buffers_mb}MB
    maintenance_work_mem: {maintenance_work_mem_mb}MB
    max_connections: {max_connections}
  listen: {{{{ env "meta.private_ip" }}}}:{pg_port}
  connect_address: {{{{ env "meta.private_ip" }}}}:{pg_port}
  data_dir: /data/postgresql
  pgpass: /secrets/pgpass
  authentication:
    replication:
      username: replicator
      password: "{pg_replicator_password}"
    superuser:
      username: postgres
      password: "{pg_superuser_password}"
    rewind:
      username: rewind_user
      password: "{pg_rewind_password}"
  pg_hba:
  - local all all trust
  - host replication replicator {subnet_access} md5
  - host all all {subnet_access} md5

tags:
    nofailover: false
    noloadbalance: false
    clonefrom: false
    nosync: false
"#
    )
}

fn generate_haproxy_config(
    pg_port: u16,
    master_port: u16,
    slave_port: u16,
    metrics_port: u16,
    consul_service_name: &str,
) -> String {
    format!(
        r#"
global
  maxconn         100
  ulimit-n        300
  nbthread        4

defaults
    log global
    retries 2
    timeout client 30m
    timeout connect 4s
    timeout server 30m
    timeout check 5s

frontend stats
  mode http
  bind {{{{ env "meta.private_ip" }}}}:{metrics_port}
  http-request use-service prometheus-exporter if {{ path /metrics }}
  http-request return status 200 content-type text/plain string ok if {{ path /health }}
  stats enable
  stats uri /stats
  stats refresh 10s

resolvers consul
  nameserver consul 127.0.0.1:8600
  accepted_payload_size 8192
  hold valid 5s

listen postgres_write
    bind {{{{ env "meta.private_ip" }}}}:{master_port}
    mode tcp
    default-server inter 3s fall 3 rise 2 on-marked-down shutdown-sessions
    server-template pg-master 1 master.{consul_service_name}.service.consul:{pg_port} resolvers consul resolve-prefer ipv4 check

listen postgres_read
    bind {{{{ env "meta.private_ip" }}}}:{slave_port}
    mode tcp
    default-server inter 3s fall 4 rise 2 on-marked-down shutdown-sessions
    server-template pg-master 4 replica.{consul_service_name}.service.consul:{pg_port} resolvers consul resolve-prefer ipv4 check
"#
    )
}

pub fn generate_pg_init_script(
    db: &Database,
    region: TableRowPointerRegion,
    schema_map: &HashMap<TableRowPointerPgSchema, ProvisioningResourcePath>,
) -> String {
    let mut res = String::new();

    res += r#"
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
"#;

    for pg_deployment in db.pg_deployment().rows_iter() {
        let depl_region = db.pg_deployment().c_region(pg_deployment);
        if region != depl_region {
            continue;
        }
        let deployment_name = db.pg_deployment().c_deployment_name(pg_deployment);
        let master_port = db.pg_deployment().c_instance_pg_master_port(pg_deployment);
        res += &format!("export PGHOST=master.epl-pg-{deployment_name}.service.consul\n");
        res += &format!("export PGPORT={master_port}\n");
        res += "export PGUSER=postgres\n";
        res += &format!("export PGPASSWORD=$( vault kv get -field=pg_superuser_password epl/pg/{deployment_name} )\n");
        res += "export PGDATABASE=postgres\n";
        res += "while ! psql -c 'SELECT 1'\n";
        res += "do\n";
        res += &format!("    echo Waiting for database deployment {deployment_name} to be up...\n");
        res += "    sleep 5\n";
        res += "done\n";

        res += &format!("export PG_EXPORTER_PASSWORD=$( vault kv get -field=pg_exporter_password epl/pg/{deployment_name} )\n");
        res += "cat <<EOF | psql -f -\n";
        res += r#"

CREATE OR REPLACE FUNCTION __tmp_create_user() returns void as \$\$
BEGIN
  IF NOT EXISTS (
          SELECT                       -- SELECT list can stay empty for this
          FROM   pg_catalog.pg_user
          WHERE  usename = 'postgres_exporter') THEN
    CREATE USER postgres_exporter;
  END IF;
END;
\$\$ language plpgsql;

SELECT __tmp_create_user();
DROP FUNCTION __tmp_create_user();

ALTER USER postgres_exporter WITH PASSWORD '$PG_EXPORTER_PASSWORD';
ALTER USER postgres_exporter SET SEARCH_PATH TO postgres_exporter,pg_catalog;

GRANT CONNECT ON DATABASE postgres TO postgres_exporter;
GRANT pg_monitor to postgres_exporter;

EOF
"#;

        for schema in db
            .pg_deployment()
            .c_children_pg_deployment_unmanaged_db(pg_deployment)
        {
            let db_name = db.pg_deployment_unmanaged_db().c_db_name(*schema);
            res += &format!("export THIS_DB_PW=$( vault kv get -field=pg_db_{db_name}_password epl/pg/{deployment_name} )\n");
            res += &format!("echo \"SELECT 'CREATE DATABASE {db_name}' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '{db_name}')\\gexec\" | psql\n");
            res += "cat <<EOF | psql -f -\n";
            res += &format!(
                r#"
DO
\$\$
BEGIN
  IF NOT EXISTS (SELECT * FROM pg_user WHERE usename = '{db_name}') THEN
     CREATE USER {db_name} password '$THIS_DB_PW';
  END IF;
  GRANT ALL PRIVILEGES ON DATABASE {db_name} TO {db_name};
  ALTER DATABASE {db_name} OWNER TO {db_name};
END
\$\$
;
"#
            );
            res += "\nEOF\n\n";
        }

        for schema in db
            .pg_deployment()
            .c_children_pg_deployment_schemas(pg_deployment)
        {
            let db_name = db.pg_deployment_schemas().c_db_name(*schema);
            let schema = db.pg_deployment_schemas().c_pg_schema(*schema);
            let migration_path = schema_map.get(&schema).unwrap();
            let mp = migration_path.path();

            res += &format!("export THIS_DB_PW=$( vault kv get -field=pg_db_{db_name}_password epl/pg/{deployment_name} )\n");

            res += &format!("echo \"SELECT 'CREATE DATABASE {db_name}' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '{db_name}')\\gexec\" | psql\n");
            res += "cat <<EOF | psql -f -\n";
            res += &format!(
                r#"
DO
\$\$
BEGIN
  IF NOT EXISTS (SELECT * FROM pg_user WHERE usename = '{db_name}') THEN
     CREATE USER {db_name} password '$THIS_DB_PW';
  END IF;
  GRANT ALL PRIVILEGES ON DATABASE {db_name} TO {db_name};
  ALTER DATABASE {db_name} OWNER TO {db_name};
END
\$\$
;
"#
            );
            res += "\nEOF\n\n";
            res +=
                &format!("PGUSER={db_name} PGPASSWORD=$THIS_DB_PW PGDATABASE={db_name} {mp} &\n");
            res += "\n";
        }
        res += "echo Migrations scheduled, waiting for finish...\n";
        res += "wait\n";
        res += "echo All migrations ran successfully\n";
    }

    res
}

pub fn generate_pg_migration_script(db: &Database, schema: TableRowPointerPgSchema) -> String {
    let mut res = String::new();

    res += "#!/bin/sh\n";
    res += "set -e\n";
    res += "\n";
    res += "[ -n \"$PGHOST\" ] || { echo \"PGHOST environment variable must be provided\"; exit 7; }\n";
    res += "[ -n \"$PGPORT\" ] || { echo \"PGPORT environment variable must be provided\"; exit 7; }\n";
    res += "[ -n \"$PGDATABASE\" ] || { echo \"PGDATABASE environment variable must be provided\"; exit 7; }\n";
    res += "[ -n \"$PGUSER\" ] || { echo \"PGUSER environment variable must be provided\"; exit 7; }\n";
    res += "[ -n \"$PGPASSWORD\" ] || { echo \"PGPASSWORD environment variable must be provided\"; exit 7; }\n";
    res += "\n";
    res += "while ! psql -c 'SELECT 1'\n";
    res += "do\n";
    res += "    echo Waiting for database to be up...\n";
    res += "    sleep 5\n";
    res += "done\n";
    res += "\n";
    res += "psql -c 'CREATE TABLE IF NOT EXISTS epl_schema_migrations(logical_time INT PRIMARY KEY, time_started TIMESTAMP, time_ended TIMESTAMP);'\n";

    for mig in db.pg_schema().c_children_pg_migration(schema) {
        let log_time = db.pg_migration().c_time(*mig);
        res += &format!("if ! psql -c \"SELECT 'MIG_FOUND' FROM epl_schema_migrations WHERE logical_time = {log_time}\" | grep MIG_FOUND\n");
        res += "then\n";
        res += "    cat <<EOF | psql -f -\n";
        res += "    BEGIN;\n";
        res += &format!("    INSERT INTO epl_schema_migrations(logical_time, time_started) VALUES ({log_time}, NOW());\n");
        res += db.pg_migration().c_upgrade(*mig);
        res += &format!("\n    UPDATE epl_schema_migrations SET time_ended = clock_timestamp() WHERE logical_time = {log_time};\n");
        res += "    COMMIT;\n";
        res += "EOF\n";
        res += "fi\n";
    }

    res
}
