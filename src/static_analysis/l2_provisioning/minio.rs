use std::collections::BTreeSet;

use convert_case::Casing;

use crate::{
    database::{Database, TableRowPointerMinioBucket, TableRowPointerMinioCluster, TableRowPointerMonitoringCluster, TableRowPointerServer},
    static_analysis::{
        server_runtime::{
            AdminService, ConsulServiceHandle, MinIOBucketPermission, NomadJobKind, NomadJobStage,
            ServerRuntime, VaultSecretHandle, VaultSecretRequest, IntegrationTest, epl_architecture_to_nomad_architecture,
        },
        PlatformValidationError, L1Projections, networking::{region_monitoring_clusters, region_loki_clusters, consul_services_exists_integration_test, prometheus_metric_exists_test, admin_service_responds_test, find_zfs_dataset_disk_medium, find_server_root_disk_medium, find_xfs_volume_root_disk_medium, check_servers_regional_distribution}, docker_images::image_handle_from_pin,
    },
};

pub fn deploy_minio_instances(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    let job_kind = NomadJobKind::BoundStateful;

    // no changes to bucket creds are possible after this
    // all bucket user registrations must happen before
    runtime.seal_minio_bucket_credentials();

    let minio_creds_copy = (*runtime.minio_credentials()).clone();

    for minio_cluster in db.minio_cluster().rows_iter() {
        let region = db.minio_cluster().c_region(minio_cluster);
        let workload_architecture = db.minio_cluster().c_workload_architecture(minio_cluster);
        let docker_image_pin_minio = db.minio_cluster().c_docker_image_minio(minio_cluster);
        let docker_image_pin_minio_mc = db.minio_cluster().c_docker_image_minio_mc(minio_cluster);
        let docker_image_pin_nginx = db.minio_cluster().c_docker_image_nginx(minio_cluster);
        let cluster_name = db.minio_cluster().c_cluster_name(minio_cluster);
        let namespace = db.minio_cluster().c_namespace(minio_cluster);
        let service_slug = db.minio_cluster().c_consul_service_name(minio_cluster);
        let service_slug_api = format!("{service_slug}-api");
        let nomad_job_name = format!("minio-{cluster_name}");
        let consul_service_io = runtime.instantiate_and_seal_consul_service(region, service_slug);
        let consul_service_api = runtime.instantiate_and_seal_consul_service(region, &service_slug_api);
        let monitoring_cluster = db.minio_cluster().c_monitoring_cluster(minio_cluster);
        let monitoring_cluster = l1proj.monitoring_clusters.pick(
            region, &monitoring_cluster
        ).ok_or_else(|| PlatformValidationError::MinIOMonitoringClusterDoesntExistInRegion {
            minio_cluster: cluster_name.clone(),
            minio_region: db.region().c_region_name(region).clone(),
            not_found_monitoring_cluster: monitoring_cluster.clone(),
            available_monitoring_clusters: region_monitoring_clusters(db, region),
        })?;
        let loki_cluster = db.minio_cluster().c_loki_cluster(minio_cluster);
        let loki_cluster = l1proj.loki_clusters.pick(
            region, loki_cluster
        ).ok_or_else(|| PlatformValidationError::MinIOLoggingClusterDoesntExistInRegion {
            minio_cluster: cluster_name.clone(),
            minio_region: db.region().c_region_name(region).clone(),
            not_found_loki_cluster: loki_cluster.clone(),
            available_loki_clusters: region_loki_clusters(db, region),
        })?;
        let expected_zfs_recordsize = db.minio_cluster().c_expected_zfs_recordsize(minio_cluster);

        let minio_admin_user = "minio";

        let minio_api_port = db.minio_cluster().c_api_port(minio_cluster);
        let minio_console_port = db.minio_cluster().c_console_port(minio_cluster);
        let minio_lb_port = db.minio_cluster().c_lb_port(minio_cluster);

        runtime.expose_admin_service(
            db,
            AdminService {
                service_title: "MinIO clusters".to_string(),
                service_kind: "minio".to_string(),
                service_instance: cluster_name.clone(),
                service_internal_upstream: consul_service_io.clone(),
                service_internal_port: minio_console_port.try_into().unwrap(),
                is_https: false,
            },
        )?;

        let mut component_secrets_builder = runtime.issue_vault_secret(region, &format!("minio/{cluster_name}"));
        let admin_password = component_secrets_builder
            .request_secret(region, "admin_password", VaultSecretRequest::AwsSecretKey);

        let mut buckets_to_init: Vec<(
            TableRowPointerMinioBucket,
            String,
            VaultSecretHandle,
            MinIOBucketPermission,
        )> = Vec::new();
        for (bucket, users) in &minio_creds_copy {
            if minio_cluster == db.minio_bucket().c_parent(*bucket) {
                for user in users.values() {
                    let handle = component_secrets_builder.request_secret(
                        region,
                        &format!("minio_user_{}_password", user.username()),
                        VaultSecretRequest::ExistingVaultSecret {
                            handle: Box::new(user.password().clone()),
                            sprintf: None,
                        },
                    );

                    buckets_to_init.push((
                        *bucket,
                        user.username().to_string(),
                        handle,
                        user.permission().clone(),
                    ));
                }
            }
        }

        let instance_count = db.minio_cluster().c_children_minio_instance(minio_cluster).len();
        if instance_count < 2 {
            return Err(PlatformValidationError::MinIOMustHaveAtLeastTwoInstances {
                cluster: db.minio_cluster().c_cluster_name(minio_cluster).clone(),
                count: instance_count,
                min_count: 2,
            });
        }

        let vault_policy = component_secrets_builder.finalize();
        let nomad_job =
            runtime.fetch_nomad_job(
                namespace, nomad_job_name.clone(), region, job_kind, NomadJobStage::SystemJob
            );
        nomad_job.set_update_strategy(crate::static_analysis::server_runtime::JobUpdateStrategy::InstantAllAtOnce);
        nomad_job.set_loki_cluster(loki_cluster);
        nomad_job.assign_vault_secrets(vault_policy);

        let mut minio_hosts = Vec::new();
        let mut minio_ips = Vec::new();
        let mut filesystems: BTreeSet<&str> = BTreeSet::new();
        let mut disk_mediums: BTreeSet<&str> = BTreeSet::new();
        for minio_instance in db.minio_cluster().c_children_minio_instance(minio_cluster) {
            let server_volume = db.minio_instance().c_instance_volume(*minio_instance);
            let server = db.server_volume().c_parent(server_volume);
            let volume_source = db.server_volume().c_source(server_volume);
            let volume_name = db.server_volume().c_volume_name(server_volume);
            let dc = db.server().c_dc(server);
            let srv_region = db.datacenter().c_region(dc);

            if region != srv_region {
                return Err(PlatformValidationError::MinIOClusterInstanceIsOutsideSpecifiedRegion {
                    cluster: db.minio_cluster().c_cluster_name(minio_cluster).clone(),
                    cluster_region: db.region().c_region_name(region).clone(),
                    server: db.server().c_hostname(server).clone(),
                    server_region: db.region().c_region_name(srv_region).clone(),
                });
            }

            match volume_source.as_str() {
                "server_root_volume" => {
                    filesystems.insert("zfs");
                    disk_mediums.insert(find_server_root_disk_medium(db, server));
                    let vol =
                        db.server().c_children_server_root_volume(server)
                                   .iter()
                                   .find(|i| {
                                       db.server_root_volume().c_volume_name(**i) == volume_name
                                   })
                                   .unwrap();
                    let found_recordsize = db.server_root_volume().c_zfs_recordsize(*vol);
                    if expected_zfs_recordsize != found_recordsize {
                        return Err(PlatformValidationError::MinIOUnexpectedZfsRecordsizeOnVolume {
                            minio_cluster: db.minio_cluster().c_cluster_name(minio_cluster).clone(),
                            minio_server: db.server().c_hostname(server).clone(),
                            minio_volume: volume_name.clone(),
                            volume_source: volume_source.clone(),
                            expected_recordsize: expected_zfs_recordsize.clone(),
                            found_recordsize: found_recordsize.clone(),
                        });
                    }
                }
                "server_zfs_dataset" => {
                    filesystems.insert("zfs");
                    let vol =
                        db.server().c_children_server_zpool(server)
                                   .iter()
                                   .map(|i| {
                                       db.server_zpool()
                                         .c_children_server_zfs_dataset(*i)
                                   })
                                   .flatten()
                                   .find(|i| {
                                       db.server_zfs_dataset().c_dataset_name(**i) == volume_name
                                   })
                                   .unwrap();
                    disk_mediums.insert(find_zfs_dataset_disk_medium(db, *vol));

                    let found_recordsize = db.server_zfs_dataset().c_zfs_recordsize(*vol);
                    if expected_zfs_recordsize != found_recordsize {
                        return Err(PlatformValidationError::MinIOUnexpectedZfsRecordsizeOnVolume {
                            minio_cluster: db.minio_cluster().c_cluster_name(minio_cluster).clone(),
                            minio_server: db.server().c_hostname(server).clone(),
                            minio_volume: volume_name.clone(),
                            volume_source: volume_source.clone(),
                            expected_recordsize: expected_zfs_recordsize.clone(),
                            found_recordsize: found_recordsize.clone(),
                        });
                    }
                }
                "server_xfs_volume" => {
                    filesystems.insert("xfs");
                    let vol =
                        db.server().c_children_server_xfs_volume(server)
                                   .iter()
                                   .find(|i| {
                                       db.server_xfs_volume().c_volume_name(**i) == volume_name
                                   })
                                   .unwrap();
                    disk_mediums.insert(find_xfs_volume_root_disk_medium(db, *vol));
                }
                _ => {
                    panic!("Unknown volume source {volume_source}")
                }
            }

            if disk_mediums.len() > 1 {
                return Err(PlatformValidationError::MinIOMultipleDiskMediumsDetectedInCluster {
                    minio_cluster: db.minio_cluster().c_cluster_name(minio_cluster).clone(),
                    disk_mediums: disk_mediums.iter().map(|i| i.to_string()).collect(),
                });
            }

            if filesystems.len() > 1 {
                return Err(PlatformValidationError::MinIOMultipleFilesystemsDetectedInCluster {
                    minio_cluster: db.minio_cluster().c_cluster_name(minio_cluster).clone(),
                    filesystems: filesystems.iter().map(|i| i.to_string()).collect(),
                });
            }

            let mut minio_ip = None;
            for ni in db.server().c_children_network_interface(server) {
                if db
                    .network()
                    .c_network_name(db.network_interface().c_if_network(*ni))
                    == "lan"
                {
                    assert!(minio_ip.is_none());
                    minio_ip = Some(db.network_interface().c_if_ip(*ni));
                }
            }
            let minio_ip = minio_ip.unwrap();

            minio_ips.push(minio_ip);
            minio_hosts.push(format!("http://{minio_ip}:{minio_api_port}/var/lib/minio"));
        }

        if db.minio_cluster().c_distribute_over_dcs(minio_cluster) {
            check_servers_regional_distribution(
                db,
                region,
                db.minio_cluster().c_children_minio_instance(minio_cluster).iter().map(|i| {
                    let srv_volume = db.minio_instance().c_instance_volume(*i);
                    db.server_volume().c_parent(srv_volume)
                }),
                format!("minio_cluster=>{cluster_name}")
            )?;
        }

        let mut buckets_provisioned = false;
        for minio_instance in db.minio_cluster().c_children_minio_instance(minio_cluster) {
            let server_volume = db.minio_instance().c_instance_volume(*minio_instance);
            let server = db.server_volume().c_parent(server_volume);
            let hostname = db.server().c_hostname(server);
            let depl_id = db.minio_instance().c_instance_id(*minio_instance);

            let mut server_ip = None;
            for ni in db.server().c_children_network_interface(server) {
                if db
                    .network()
                    .c_network_name(db.network_interface().c_if_network(*ni))
                    == "lan"
                {
                    assert!(server_ip.is_none());
                    server_ip = Some(db.network_interface().c_if_ip(*ni));
                }
            }

            let server_ip = server_ip.unwrap();

            let server_data = runtime.fetch_server_data(db, server);
            let volume_lock = server_data.server_volume_write_lock(
                db,
                server_volume,
                format!("Exclusive epl-minio-{cluster_name} volume lock"),
            )?;

            let api_port_lock = server_data.lock_port(
                db,
                minio_api_port.try_into().unwrap(),
                format!("MinIO API port for {cluster_name}"),
            )?;
            let console_port_lock = server_data.lock_port(
                db,
                minio_console_port.try_into().unwrap(),
                format!("MinIO console port for {cluster_name}"),
            )?;
            let lb_port_lock = server_data.lock_port(
                db,
                minio_lb_port.try_into().unwrap(),
                format!("MinIO lb port for {cluster_name}"),
            )?;

            let locked_instance_mem = server_data.reserve_memory_mb(
                format!("MinIO instance memory {cluster_name}"),
                db.minio_cluster().c_instance_memory_mb(minio_cluster),
            )?;
            let locked_lb_mem = server_data.reserve_memory_mb(
                format!("MinIO lb memory {cluster_name}"),
                db.minio_cluster().c_lb_memory_mb(minio_cluster),
            )?;
            let locked_mc_mem = server_data.reserve_memory_mb(
                format!("MinIO mc bucket provisioning memory {cluster_name}"),
                128,
            )?;

            let server_lock = runtime.lock_server_with_label(
                db,
                format!("epl-minio-{hostname}-{cluster_name}"),
                server,
            )?;

            let nomad_job =
                runtime.fetch_nomad_job(
                    namespace, nomad_job_name.clone(), region, job_kind, NomadJobStage::SystemJob
                );
            let tg_name = format!("epl-minio-{cluster_name}-{depl_id}");

            let tg = nomad_job.fetch_task_group(tg_name.clone());
            tg.constrain_architecture(epl_architecture_to_nomad_architecture(workload_architecture));
            tg.assign_server_lock(server_lock);
            tg.add_locked_port("api", api_port_lock);
            tg.expose_port_as_tcp_service("api", &consul_service_api);
            tg.collect_prometheus_metrics(
                &consul_service_api,
                monitoring_cluster,
                Some("/minio/v2/metrics/cluster"),
            );
            tg.add_locked_port("con", console_port_lock);
            tg.add_locked_port("lb", lb_port_lock);
            tg.expose_port_as_tcp_service("lb", &consul_service_io);

            let docker_image_minio = image_handle_from_pin(db, workload_architecture, docker_image_pin_minio, "minio")?;
            let minio_task =
                tg.fetch_task(
                    format!("minio-{cluster_name}-daemon"),
                    docker_image_minio,
                );

            minio_task.add_memory(locked_instance_mem);
            minio_task.bind_volume(volume_lock, "/var/lib/minio".to_string());
            minio_task.set_env_variable("MINIO_ROOT_USER", minio_admin_user);
            minio_task.set_env_variable("MINIO_PROMETHEUS_AUTH_TYPE", "public");
            minio_task.add_secure_env_variables(
                "admin_password".to_string(),
                &[("MINIO_ROOT_PASSWORD", &admin_password)],
            );
            let mut args_vec = vec![
                "server".to_string(),
                format!("--address=${{meta.private_ip}}:{minio_api_port}"),
                format!("--console-address=${{meta.private_ip}}:{minio_console_port}"),
            ];

            for other in &minio_hosts {
                args_vec.push(other.clone());
            }

            minio_task.set_arguments(args_vec);

            let docker_image_nginx = image_handle_from_pin(db, workload_architecture, docker_image_pin_nginx, "nginx")?;
            let lb_task = tg.fetch_task(
                format!("minio-{cluster_name}-lb"),
                docker_image_nginx,
            );

            lb_task.add_memory(locked_lb_mem);
            lb_task.add_secure_config("nginx.conf".to_string(), generate_nginx_confg());
            lb_task.add_secure_config(
                "site.conf".to_string(),
                generate_minio_lb_config(server_ip, minio_api_port, minio_lb_port, &minio_ips),
            );
            lb_task.set_entrypoint(vec![
                "/usr/sbin/nginx".to_string(),
                "-g".to_string(),
                "daemon off;".to_string(),
                "-c".to_string(),
                "/secrets/nginx.conf".to_string(),
            ]);

            if !buckets_provisioned
                && !db
                    .minio_cluster()
                    .c_children_minio_bucket(minio_cluster)
                    .is_empty()
            {
                buckets_provisioned = true;

                let docker_image_minio_mc = image_handle_from_pin(db, workload_architecture, docker_image_pin_minio_mc, "minio_mc")?;
                let buckets_task = tg.fetch_post_start_ephemeral_task(
                    format!("epl-minio-{cluster_name}-{depl_id}-provision-buckets"),
                    docker_image_minio_mc,
                );

                let provisioning_script = generate_minio_buckets_provisioning_script(
                    db,
                    minio_cluster,
                    minio_admin_user,
                    &admin_password,
                    &consul_service_io,
                    minio_lb_port,
                    &buckets_to_init,
                );

                let entrypoint_abs_path = buckets_task
                    .add_secure_config("entrypoint.sh".to_string(), provisioning_script);
                buckets_task.set_entrypoint(vec!["/bin/bash".to_string(), entrypoint_abs_path]);
                buckets_task.add_memory(locked_mc_mem);
            }
        }

        // tests are added at the end to ensure all errors are validated before
        minio_cluster_tests(db, l1proj, minio_cluster, monitoring_cluster, runtime);
    }

    Ok(())
}

fn minio_cluster_tests(
    db: &Database,
    l1proj: &L1Projections,
    cluster: TableRowPointerMinioCluster,
    mon_cluster: TableRowPointerMonitoringCluster,
    runtime: &mut ServerRuntime
) {
    let cluster_name_kebab = db.minio_cluster().c_cluster_name(cluster);
    let cluster_name = cluster_name_kebab.to_case(convert_case::Case::Snake);
    let region = db.minio_cluster().c_region(cluster);
    let consul_service_name = db.minio_cluster().c_consul_service_name(cluster);
    let mut servers: Vec<TableRowPointerServer> = Vec::new();
    let mut server_ips: Vec<String> = Vec::new();
    for inst in db.minio_cluster().c_children_minio_instance(cluster) {
        let vol = db.minio_instance().c_instance_volume(*inst);
        let server = db.server_volume().c_parent(vol);
        servers.push(server);

        let iface = l1proj.consul_network_iface.value(server);
        let ip = db.network_interface().c_if_ip(*iface);
        server_ips.push(ip.clone());
    }

    runtime.add_integration_test(
        format!("minio_cluster_{cluster_name}_instances_available_in_dns"),
        consul_services_exists_integration_test(db, l1proj, region, format!("{consul_service_name}.service.consul"), &servers)
    );

    runtime.add_integration_test(
        format!("minio_cluster_{cluster_name}_prometheus_metrics_gathered"),
        prometheus_metric_exists_test(db, l1proj, mon_cluster, "minio_s3_requests_total")
    );

    runtime.add_integration_test(
        format!("minio_cluster_{cluster_name}_healthcheck_responds_ok"),
        IntegrationTest::HttpGetRespondsOk {
            server_ips,
            http_server_port: db.minio_cluster().c_api_port(cluster),
            path: "/minio/health/live".to_string(),
        }
    );

    runtime.add_integration_test(
        format!("minio_external_admin_panel_responds_{cluster_name}"),
        admin_service_responds_test(
            db,
            l1proj,
            format!("adm-minio-{cluster_name_kebab}"),
            "/",
            "<title>MinIO Console</title>"
        )
    );
}

fn generate_minio_buckets_provisioning_script(
    db: &Database,
    cluster: TableRowPointerMinioCluster,
    root_user: &str,
    root_user_password: &VaultSecretHandle,
    consul_service: &ConsulServiceHandle,
    minio_port: i64,
    extra_users: &[(
        TableRowPointerMinioBucket,
        String,
        VaultSecretHandle,
        MinIOBucketPermission,
    )],
) -> String {
    let mut res = String::new();

    let service_fqdn = consul_service.service_fqdn();
    let root_pwd = root_user_password.template_expression();

    res += &format!(
        r#"
set -e

mkdir -p /secrets/.mc
ln -s /secrets/.mc /root/.mc

while ! curl -f http://{service_fqdn}:{minio_port}/minio/health/cluster
do
  echo minio healthcheck failed, retrying in one second...
  sleep 1
done

mc alias set thisminio http://{service_fqdn}:{minio_port} {root_user} {root_pwd}

while ! mc ls thisminio/
do
  echo minio list buckets failed, retrying in one second...
  sleep 1
done

# provision buckets
"#
    );

    for bucket in db.minio_cluster().c_children_minio_bucket(cluster) {
        res += "mc mb --ignore-existing ";
        if db.minio_bucket().c_locking_enabled(*bucket) {
            res += "--with-lock ";
        }
        res += "thisminio/";
        res += db.minio_bucket().c_bucket_name(*bucket);
        res += "\n";
    }

    res += "\n\n";

    res += "# privision policies\n";
    for bucket in db.minio_cluster().c_children_minio_bucket(cluster) {
        let bucket_name = db.minio_bucket().c_bucket_name(*bucket);
        res += &format!(
            r#"
cat <<EOF > /secrets/policy.json
{{
  "Version": "2012-10-17",
  "Statement": [
    {{
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket",
        "s3:GetBucketLocation",
        "s3:GetObject"
      ],
      "Resource": ["arn:aws:s3:::{bucket_name}"]
    }},
    {{
      "Effect": "Allow",
      "Action": [
        "s3:*"
      ],
      "Resource": [
        "arn:aws:s3:::{bucket_name}/*"
      ]
    }}
  ]
}}
EOF
"#
        );
        res += &format!("mc admin policy add thisminio rw-{bucket_name} /secrets/policy.json\n");

        res += &format!(
            r#"
cat <<EOF > /secrets/policy.json
{{
  "Version": "2012-10-17",
  "Statement": [
    {{
      "Effect": "Allow",
      "Action": [
        "s3:GetBucketLocation",
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::{bucket_name}/*",
        "arn:aws:s3:::{bucket_name}"
      ]
    }}
  ]
}}
EOF
"#
        );
        res += &format!("mc admin policy add thisminio ro-{bucket_name} /secrets/policy.json\n");
    }

    if !extra_users.is_empty() {
        res += "\n\n# provision extra bucket users\n";
    }

    for (e_bucket, e_username, e_vault_secret, e_perm) in extra_users {
        let bucket_name = db.minio_bucket().c_bucket_name(*e_bucket);
        res += &format!(
            "mc admin user add thisminio {} {}\n",
            e_username,
            e_vault_secret.template_expression()
        );
        let policy = match e_perm {
            // TODO: add read only policy when needed
            MinIOBucketPermission::ReadWrite => format!("rw-{bucket_name}"),
        };
        res += &format!("mc admin policy set thisminio {policy} user={e_username}\n");
    }

    res
}

fn generate_minio_lb_config(
    server_ip: &str,
    minio_port: i64,
    lb_port: i64,
    upstreams: &[&String],
) -> String {
    let mut upstream_str = String::new();

    for upstream in upstreams {
        upstream_str.push_str(&format!("    server {upstream}:{minio_port};\n"));
    }

    format!(
        r#"
upstream minio {{
    least_conn;
{upstream_str}
}}

server {{
    listen {server_ip}:{lb_port};

    ignore_invalid_headers off;
    # Allow any size file to be uploaded.
    # Set to a value such as 1000m; to restrict file size to a specific value
    client_max_body_size 0;
    # To disable buffering
    proxy_buffering off;


    location / {{
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Host $http_host;

        proxy_connect_timeout 300;
        # Default is HTTP/1, keepalive is only enabled in HTTP/1.1
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        chunked_transfer_encoding off;

        proxy_pass http://minio/;
    }}
}}
"#
    )
}

fn generate_nginx_confg() -> String {
    r#"
pcre_jit on;

worker_processes auto;
worker_rlimit_nofile 12288;

events {
    worker_connections  1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Log in JSON Format
    log_format nginxlog_json escape=json '{ "@timestamp": "$time_iso8601", '
         '"remote_addr": "$remote_addr", '
         '"body_bytes_sent": $body_bytes_sent, '
         '"request_time": $request_time, '
         '"response_status": $status, '
         '"request": "$request", '
         '"request_method": "$request_method", '
         '"host": "$host",'
         '"upstream_addr": "$upstream_addr",'
         '"http_x_forwarded_for": "$http_x_forwarded_for",'
         '"http_referrer": "$http_referer", '
         '"http_user_agent": "$http_user_agent", '
         '"http_version": "$server_protocol", '
         '"server_port": "$server_port"}';
    access_log /dev/stdout nginxlog_json;

    sendfile        on;

    keepalive_timeout  65;

    include /secrets/site.conf;
}
"#
    .to_string()
}
