use std::fmt::Write;

use crate::{static_analysis::{CheckedDB, networking::{server_region, DcParameters, get_dc_parameters}, server_disks::{pick_disk_id_policy, DiskIdsPolicy}}, codegen::{nixplan::{NixAllServerPlans, ZfsDataset, NixServerPlan, root_secret_config}, secrets::SecretsStorage, preconditions::disk_id_label}, database::{TableRowPointerServer, TableRowPointerServerZpool, TableRowPointerServerZpoolVdev, TableRowPointerServerZpoolSpare, TableRowPointerServerZpoolCache, TableRowPointerServerZpoolLog}};

pub fn provision_disks(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &mut SecretsStorage) {
    provision_zfs_volumes(db, plans, secrets);
    provision_extra_xfs_volumes(db, plans);
}

fn provision_extra_xfs_volumes(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    for dc in db.db.datacenter().rows_iter() {
        let dc_impl = db.db.datacenter().c_implementation(dc);
        let dc_params = get_dc_parameters(dc_impl.as_str());
        let disk_id_policy = pick_disk_id_policy(&db.db, dc);
        for server in db.db.datacenter().c_referrers_server__dc(dc) {
            for disk in db.db.server().c_children_server_disk(*server) {
                if db.db.server_disk().c_xfs_format(*disk) {
                    let plan = plans.fetch_plan(*server);
                    let disk_id = db.db.server_disk().c_disk_id(*disk);
                    let mount_path = format!("/srv/xfs-jbods/{disk_id}");
                    let mut xfs_vols = String::new();
                    for xfs_vol in db.db.server_disk().c_referrers_server_xfs_volume__xfs_disk(*disk) {
                        let vol_name = db.db.server_xfs_volume().c_volume_name(*xfs_vol);
                        writeln!(&mut xfs_vols, "mkdir -p {mount_path}/{vol_name}").unwrap();
                        writeln!(&mut xfs_vols, "chmod 777 {mount_path}/{vol_name}").unwrap();
                    }
                    let disk_label = disk_id_label(&db.db, *disk, &dc_params, &disk_id_policy);
                    let disk_path = if disk_label.starts_with("/dev/") {
                        disk_label
                    } else { format!("/dev/{disk_label}") };
                    plan.add_pre_l1_provisioning_shell_hook(format!(r#"
mkdir -p {mount_path}
chmod 700 {mount_path}

# we don't bother with nixos config or fstab
# because we must run l1 provisioning post boot anyway
CURR_FS_TYPE=$( lsblk -n -o FSTYPE {disk_path} )
if [ -z "$CURR_FS_TYPE" ]
then
    # if disk is empty provision xfs filesystem
    nix-shell -p xfsprogs --command 'mkfs.xfs {disk_path}'
fi

if ! mount | grep '{disk_path}'
then
    mount -o noatime {disk_path} {mount_path}
fi

CURR_FS_SIZE=$( df --output=size -B1 {disk_path} | tail -n-1 )
CURR_DEVICE_SIZE=$( lsblk -n -o SIZE -b {disk_path} )
AT_LEAST_SIZE=$(( CURR_FS_SIZE + 536870912 ))
if [ "$AT_LEAST_SIZE" -le "$CURR_DEVICE_SIZE" ]
then
    echo "Block device {disk_path} expanded, expanding xfs filesystem..."
    nix-shell -p xfsprogs --command 'xfs_growfs {disk_path}'
fi

{xfs_vols}
"#));
                }
            }
        }
    }
}

fn provision_zfs_volumes(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &mut SecretsStorage) {
    for region in db.db.region().rows_iter() {
        let region_name = db.db.region().c_region_name(region);
        let region_passphrase =
            secrets.fetch_secret(
                format!("zfs_dataset_encryption_root_seed_{region_name}"),
                crate::codegen::secrets::SecretKind::StrongPassword42Symbols
            );

        for dc in db.db.region().c_referrers_datacenter__region(region) {
            let dc_impl = db.db.datacenter().c_implementation(*dc);
            let dc_params = get_dc_parameters(dc_impl.as_str());
            let disk_id_policy = pick_disk_id_policy(&db.db, *dc);
            for server in db.db.datacenter().c_referrers_server__dc(*dc) {
                let hostname = db.db.server().c_hostname(*server);
                let plan = plans.fetch_plan(*server);
                add_zfs_exporter(db, *server, plan);

                plan.add_zfs_dataset(
                    "docker".to_string(),
                    ZfsDataset {
                        zpool: "rpool".to_string(),
                        encryption_enabled: false,
                        compression_enabled: true,
                        mountpoint: "/var/lib/docker".to_string(),
                        expose_to_containers: false,
                        record_size: "128k".to_string(),
                    }
                );

                // add server root volumes
                for rv in db.db.server().c_children_server_root_volume(*server) {
                    let dataset_name = db.db.server_root_volume().c_volume_name(*rv);
                    plan.add_zfs_dataset(
                        dataset_name.clone(),
                        ZfsDataset {
                            encryption_enabled: db.db.server_root_volume().c_zfs_encryption(*rv),
                            compression_enabled: db.db.server_root_volume().c_zfs_compression(*rv),
                            expose_to_containers: true,
                            mountpoint: db.db.server_root_volume().c_mountpoint(*rv).clone(),
                            zpool: "rpool".to_string(), // root default zpool
                            record_size: db.db.server_root_volume().c_zfs_recordsize(*rv).clone()
                        }
                    )
                }

                let mut rv_prov = String::new();
                rv_prov += zfs_dataset_provisioning_script();

                // restrict volume access to root only
                rv_prov += "mkdir -m 700 -p /srv/volumes\n";
                // in case some zpools are not mounted
                writeln!(&mut rv_prov, "zpool import -af").unwrap();

                for zpool in db.db.server().c_children_server_zpool(*server) {
                    let zpool_name = db.db.server_zpool().c_zpool_name(*zpool);

                    rv_prov += &zpool_create_command(db, *zpool, &dc_params, &disk_id_policy);

                    let mut vdevs = db.db.server_zpool().c_children_server_zpool_vdev(*zpool).to_vec();
                    vdevs.sort_by_key(|i| db.db.server_zpool_vdev().c_vdev_number(*i));

                    for vdev in &vdevs {
                        // first vdev should always exist after initial creation of zpool
                        if db.db.server_zpool_vdev().c_vdev_number(*vdev) > 1 {
                            rv_prov += &zpool_add_vdev_command(db, *vdev, &dc_params, &disk_id_policy);
                        }
                    }

                    for log in db.db.server_zpool().c_children_server_zpool_log(*zpool) {
                        rv_prov += &zpool_add_log_command(db, *log, &dc_params, &disk_id_policy);
                    }

                    for cache in db.db.server_zpool().c_children_server_zpool_cache(*zpool) {
                        rv_prov += &zpool_add_cache_command(db, *cache, &dc_params, &disk_id_policy);
                    }

                    for spare in db.db.server_zpool().c_children_server_zpool_spare(*zpool) {
                        rv_prov += &zpool_add_spare_command(db, *spare, &dc_params, &disk_id_policy);
                    }

                    rv_prov += &zpool_auto_expand_command(db, *zpool, &dc_params, &disk_id_policy);

                    for dataset in db.db.server_zpool().c_children_server_zfs_dataset(*zpool) {
                        let dataset_name = db.db.server_zfs_dataset().c_dataset_name(*dataset);
                        plan.add_zfs_dataset(
                            dataset_name.clone(),
                            ZfsDataset {
                                encryption_enabled: db.db.server_zfs_dataset().c_zfs_encryption(*dataset),
                                compression_enabled: db.db.server_zfs_dataset().c_zfs_compression(*dataset),
                                expose_to_containers: true,
                                mountpoint: format!("/srv/volumes/{dataset_name}"),
                                zpool: zpool_name.clone(), // root default zpool
                                record_size: db.db.server_zfs_dataset().c_zfs_recordsize(*dataset).clone()
                            }
                        )
                    }
                }

                for (dataset_name, params) in plan.zfs_datasets() {
                    let vol_name = &dataset_name;
                    let rec_size = &params.record_size;
                    let mountpoint = &params.mountpoint;
                    let zpool = &params.zpool;
                    let expose = if params.expose_to_containers {
                        "yes"
                    } else { "no" };
                    let compression =
                        if params.compression_enabled {
                            "on"
                        } else { "off" };
                    let passphrase =
                        if params.encryption_enabled {
                            let password_key = format!("{hostname}.{vol_name}");
                            // whitespace in front ignored
                            let mut the_password = " ".to_string();
                            the_password += &generate_deterministic_password_with_seed(
                                region_passphrase.value().as_str(), &password_key, 42
                            );
                            the_password
                        } else { "".to_string() };

                    writeln!(
                        &mut rv_prov,
                        "provision_zfs_dataset {zpool} {vol_name} {mountpoint} {rec_size} {compression} {expose}{passphrase}"
                    ).unwrap();
                }

                plan.add_pre_l1_provisioning_shell_hook(rv_prov);
            }
        }
    }
}

fn zpool_create_command(db: &CheckedDB, zpool: TableRowPointerServerZpool, dc_params: &DcParameters, id_pol: &DiskIdsPolicy) -> String {
    let mut res = String::new();

    let zpool_name = db.db.server_zpool().c_zpool_name(zpool);
    write!(&mut res, "zpool list -H -o name | grep '{zpool_name}' || zpool create {zpool_name} -m none -o autoexpand=on -o ashift=12").unwrap();
    let mut vdevs = db.db.server_zpool().c_children_server_zpool_vdev(zpool).to_vec();
    vdevs.sort_by_key(|i| db.db.server_zpool_vdev().c_vdev_number(*i));

    for vdev in &vdevs {
        let vdev_type = db.db.server_zpool_vdev().c_vdev_type(*vdev);
        if db.db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*vdev).len() > 1 {
            write!(&mut res, " {vdev_type}").unwrap();
        }
        for disk in db.db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*vdev) {
            let disk = db.db.server_zpool_vdev_disk().c_disk_id(*disk);
            let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
            write!(&mut res, " {disk_label}").unwrap();
        }
    }

    if db.db.server_zpool().c_children_server_zpool_spare(zpool).len() > 0 {
        write!(&mut res, " spare").unwrap();
        for spare in db.db.server_zpool().c_children_server_zpool_spare(zpool) {
            let disk = db.db.server_zpool_spare().c_disk_id(*spare);
            let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
            write!(&mut res, " {disk_label}").unwrap();
        }
    }

    if db.db.server_zpool().c_children_server_zpool_cache(zpool).len() > 0 {
        write!(&mut res, " cache").unwrap();
        for cache in db.db.server_zpool().c_children_server_zpool_cache(zpool) {
            let disk = db.db.server_zpool_cache().c_disk_id(*cache);
            let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
            write!(&mut res, " {disk_label}").unwrap();
        }
    }

    if db.db.server_zpool().c_children_server_zpool_log(zpool).len() > 0 {
        let maybe_mirror =
            if db.db.server_zpool().c_children_server_zpool_log(zpool).len() > 1 {
                " mirror"
            } else { "" };
        write!(&mut res, " log{maybe_mirror}").unwrap();
        for log in db.db.server_zpool().c_children_server_zpool_log(zpool) {
            let disk = db.db.server_zpool_log().c_disk_id(*log);
            let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
            write!(&mut res, " {disk_label}").unwrap();
        }
    }

    res += "\n";

    res
}

fn zpool_add_vdev_command(db: &CheckedDB, vdev: TableRowPointerServerZpoolVdev, dc_params: &DcParameters, id_pol: &DiskIdsPolicy) -> String {
    let mut res = String::new();

    let zpool = db.db.server_zpool_vdev().c_parent(vdev);
    let zpool_name = db.db.server_zpool().c_zpool_name(zpool);
    let mut vdevs = db.db.server_zpool().c_children_server_zpool_vdev(zpool).to_vec();
    vdevs.sort_by_key(|i| db.db.server_zpool_vdev().c_vdev_number(*i));

    let mut total_disks = 0usize;
    for prev_vdev in &vdevs {
        total_disks += db.db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*prev_vdev).len();
        if *prev_vdev == vdev {
            break;
        }
    }

    // if current disks count are only previous disks add the spare
    writeln!(&mut res, "ZPOOL_DISK_COUNT=$( zpool status -P {zpool_name} | awk '/^config:/{{f=1;next}} /^errors:/{{f=0}} f' | sed -E '/^\\s+logs/q' | sed -E '/^\\s+cache/q' | sed -E '/^\\s+spares/q' | grep -E '^\\s+/dev/' | wc -l )").unwrap();
    writeln!(&mut res, "if [ \"$ZPOOL_DISK_COUNT\" -lt \"{total_disks}\" ];").unwrap();
    res += "then\n";
    write!(&mut res, "    zpool add {zpool_name}").unwrap();
    let vdev_type = db.db.server_zpool_vdev().c_vdev_type(vdev);
    write!(&mut res, " {vdev_type}").unwrap();
    for disk in db.db.server_zpool_vdev().c_children_server_zpool_vdev_disk(vdev) {
        let disk = db.db.server_zpool_vdev_disk().c_disk_id(*disk);
        let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
        write!(&mut res, " {disk_label}").unwrap();
    }
    res += "\n";
    res += "fi\n";

    res
}

fn zpool_add_log_command(db: &CheckedDB, log: TableRowPointerServerZpoolLog, dc_params: &DcParameters, id_pol: &DiskIdsPolicy) -> String {
    let mut res = String::new();

    let zpool = db.db.server_zpool_log().c_parent(log);
    let zpool_name = db.db.server_zpool().c_zpool_name(zpool);
    let mut vdevs = db.db.server_zpool().c_children_server_zpool_vdev(zpool).to_vec();
    vdevs.sort_by_key(|i| db.db.server_zpool_vdev().c_vdev_number(*i));

    let mut total_disks = 0usize;
    for prev_log in db.db.server_zpool().c_children_server_zpool_log(zpool) {
        total_disks += 1;
        if *prev_log == log {
            break;
        }
    }

    // if current disks count are only previous disks add the spare
    writeln!(&mut res, "ZPOOL_DISK_COUNT=$( zpool status -P {zpool_name} | awk '/^config:/{{f=1;next}} /^errors:/{{f=0}} f' | sed -E '1,/^\\s+logs/ d' | sed -E '/^\\s+cache/q' | sed -E '/^\\s+spares/q' | grep -E '\\s+/dev/' | wc -l )").unwrap();
    writeln!(&mut res, "if [ \"$ZPOOL_DISK_COUNT\" -lt \"{total_disks}\" ];").unwrap();
    res += "then\n";
    let disk = db.db.server_zpool_log().c_disk_id(log);
    let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
    writeln!(&mut res, "    zpool add {zpool_name} log {disk_label}").unwrap();
    res += "fi\n";

    res
}

fn zpool_add_cache_command(db: &CheckedDB, cache: TableRowPointerServerZpoolCache, dc_params: &DcParameters, id_pol: &DiskIdsPolicy) -> String {
    let mut res = String::new();

    let zpool = db.db.server_zpool_cache().c_parent(cache);
    let zpool_name = db.db.server_zpool().c_zpool_name(zpool);
    let mut vdevs = db.db.server_zpool().c_children_server_zpool_vdev(zpool).to_vec();
    vdevs.sort_by_key(|i| db.db.server_zpool_vdev().c_vdev_number(*i));

    let mut total_disks = 0usize;
    for prev_cache in db.db.server_zpool().c_children_server_zpool_cache(zpool) {
        total_disks += 1;
        if *prev_cache == cache {
            break;
        }
    }

    // if current disks count are only previous disks add the spare
    writeln!(&mut res, "ZPOOL_DISK_COUNT=$( zpool status -P {zpool_name} | awk '/^config:/{{f=1;next}} /^errors:/{{f=0}} f' | sed -E '1,/^\\s+cache/ d' | sed -E '/^\\s+spares/q' | grep -E '\\s+/dev/' | wc -l )").unwrap();
    writeln!(&mut res, "if [ \"$ZPOOL_DISK_COUNT\" -lt \"{total_disks}\" ];").unwrap();
    res += "then\n";
    let disk = db.db.server_zpool_cache().c_disk_id(cache);
    let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
    writeln!(&mut res, "    zpool add {zpool_name} cache {disk_label}").unwrap();
    res += "fi\n";

    res
}

fn zpool_add_spare_command(db: &CheckedDB, spare: TableRowPointerServerZpoolSpare, dc_params: &DcParameters, id_pol: &DiskIdsPolicy) -> String {
    let mut res = String::new();

    let zpool = db.db.server_zpool_spare().c_parent(spare);
    let zpool_name = db.db.server_zpool().c_zpool_name(zpool);
    let mut vdevs = db.db.server_zpool().c_children_server_zpool_vdev(zpool).to_vec();
    vdevs.sort_by_key(|i| db.db.server_zpool_vdev().c_vdev_number(*i));

    let mut total_disks = 0usize;
    for prev_spare in db.db.server_zpool().c_children_server_zpool_spare(zpool) {
        total_disks += 1;
        if spare == *prev_spare {
            break;
        }
    }

    // if current disks count are only previous disks add the spare
    writeln!(&mut res, "ZPOOL_DISK_COUNT=$( zpool status -P {zpool_name} | awk '/^config:/{{f=1;next}} /^errors:/{{f=0}} f' | sed -E '1,/^\\s+spares/ d' | grep -E '^\\s+/dev/' | wc -l )").unwrap();
    writeln!(&mut res, "if [ \"$ZPOOL_DISK_COUNT\" -lt \"{total_disks}\" ];").unwrap();
    res += "then\n";
    let disk = db.db.server_zpool_spare().c_disk_id(spare);
    let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
    writeln!(&mut res, "    zpool add {zpool_name} spare {disk_label}").unwrap();
    res += "fi\n";

    res
}

fn zpool_auto_expand_command(db: &CheckedDB, zpool: TableRowPointerServerZpool, dc_params: &DcParameters, id_pol: &DiskIdsPolicy) -> String {
    let mut res = String::new();

    let zpool_name = db.db.server_zpool().c_zpool_name(zpool);

    for vdev in db.db.server_zpool().c_children_server_zpool_vdev(zpool) {
        for disk in db.db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*vdev) {
            let disk = db.db.server_zpool_vdev_disk().c_disk_id(*disk);
            let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
            writeln!(&mut res, "zpool online -e {zpool_name} {disk_label}").unwrap();
        }
    }

    for log in db.db.server_zpool().c_children_server_zpool_log(zpool) {
        let disk = db.db.server_zpool_log().c_disk_id(*log);
        let disk_label = disk_id_label(&db.db, disk, dc_params, id_pol);
        writeln!(&mut res, "zpool online -e {zpool_name} {disk_label}").unwrap();
    }

    res
}

fn zfs_dataset_provisioning_script() -> &'static str {
    r#"
function provision_zfs_dataset() {
  VOLUME_ZPOOL=$1
  VOLUME_NAME=$2
  MOUNTPOINT=$3
  RECORDSIZE=$4
  COMPRESSION=$5
  EXPOSE_TO_CONTAINERS=$6
  ENCRYPTION_PASSPHRASE=$7
  EXPECTED_DATASET_NAME="$VOLUME_ZPOOL/epl-$VOLUME_NAME"
  KEY_LOCATION="/run/keys/zfs-volume-$VOLUME_NAME-passphrase"

  CREATION_ARGS="-o compression=$COMPRESSION -o mountpoint=$MOUNTPOINT -o recordsize=$RECORDSIZE"

  if ! zfs list -H -o name $EXPECTED_DATASET_NAME
  then
    # dataset doesnt exist, create

    ENCRYPTION_ARGS=""
    if [ -n "$ENCRYPTION_PASSPHRASE" ];
    then
        # we rely on umask created for l1 provisioning script to create file with 0600 permissions
        echo -n "$ENCRYPTION_PASSPHRASE" > $KEY_LOCATION
        ENCRYPTION_ARGS="-o encryption=on -o keylocation=file://$KEY_LOCATION -o keyformat=passphrase"
    fi

    zfs create $CREATION_ARGS $ENCRYPTION_ARGS $EXPECTED_DATASET_NAME

    if [ -n "$ENCRYPTION_PASSPHRASE" ];
    then
        shred $KEY_LOCATION
        rm -f $KEY_LOCATION
    fi
  else
    # dataset exists, check if not mounted then mount
    IS_MOUNTED=$( zfs list -H -o mounted $EXPECTED_DATASET_NAME )
    if [ "$IS_MOUNTED" == "no" ]
    then
        if [ -n "$ENCRYPTION_PASSPHRASE" ];
        then
            echo -n "$ENCRYPTION_PASSPHRASE" > $KEY_LOCATION
            zfs load-key $EXPECTED_DATASET_NAME || true
        fi

        zfs mount $EXPECTED_DATASET_NAME

        if [ -n "$ENCRYPTION_PASSPHRASE" ];
        then
            shred $KEY_LOCATION
            rm -f $KEY_LOCATION
        fi
    fi
  fi

  if [ "$EXPOSE_TO_CONTAINERS" == "yes" ]
  then
    chmod 777 $MOUNTPOINT
  else
    chmod 700 $MOUNTPOINT
  fi
}
"#
}

pub fn generate_deterministic_password_with_seed_impl(master_seed: &str, password_name: &str, password_length: usize) -> String {
    use rand::{SeedableRng, seq::SliceRandom};

    let seed_str = format!("{master_seed}.{password_name}");
    let hash = hmac_sha256::Hash::hash(seed_str.as_bytes());
    let mut rng = rand::rngs::StdRng::from_seed(hash);

    let char_range = [
        'A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
        'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z',
        '0','1','2','3','4','5','6','7','8','9',
    ];
    let mut res = String::with_capacity(password_length);
    while res.len() < password_length {
        let this_char = char_range.choose(&mut rng).unwrap();
        res.push(*this_char);
    }

    res
}

#[cfg(not(test))]
pub fn generate_deterministic_password_with_seed(master_seed: &str, password_name: &str, password_length: usize) -> String {
    generate_deterministic_password_with_seed_impl(master_seed, password_name, password_length)
}

#[cfg(test)]
pub fn generate_deterministic_password_with_seed(master_seed: &str, password_name: &str, _password_length: usize) -> String {
    format!("DETERMINISTIC_PW_{master_seed}.{password_name}")
}

fn add_zfs_exporter(db: &CheckedDB, server: TableRowPointerServer, plan: &mut NixServerPlan) {
    let monitoring_cluster = db.projections.monitoring_clusters.region_default(server_region(&db.db, server));
    if let Some(monitoring_cluster) = &monitoring_cluster {
        let monitoring_cluster = db.db.monitoring_cluster().c_cluster_name(*monitoring_cluster);
        let exp_iface = db.projections.consul_network_iface.value(server);
        let exp_service_ip = db.db.network_interface().c_if_ip(*exp_iface);
        let port = 9134;
        let sec_conf = plan.add_secret_config(
            root_secret_config(
                "epl-zfs-exporter-service.hcl".to_string(),
                format!(r#"
service {{
  name = "epl-zfs-exporter"
  id   = "epl-zfs-exporter"
  port = {port}
  tags = ["epl-mon-{monitoring_cluster}"]

  meta = {{
    metrics_path = "/metrics"
  }}

  tagged_addresses = {{
    lan = {{
      address = "{exp_service_ip}"
      port    = {port}
    }}
  }}

  checks = [
    {{
        id       = "home"
        name     = "/"
        http     = "http://{exp_service_ip}:{port}/"
        interval = "15s"
    }},
  ]
}}
"#)));
        let abs_service_path = sec_conf.absolute_path();
        // add zfs exporter
        plan.add_post_second_round_secrets_shell_hook(format!(r#"
# wait for consul to be up and running if restarted for up to 10 seconds
for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

export CONSUL_HTTP_TOKEN=$(cat /run/keys/consul-agent-token.txt)
for I in $(seq 1 5); do
  consul services register {abs_service_path} && break || true
  # try a few times if consul is down
  sleep 1
done
"#));
        plan.add_custom_nix_block(format!(r#"
    services.prometheus.exporters.zfs.enable = true;
    services.prometheus.exporters.zfs.port = {port};
"#));

    }
}

#[test]
fn test_deterministic_password_generator_fixation() {
    let pwd_one = generate_deterministic_password_with_seed_impl("dizzle-wit-wizzle", "password_one", 42);
    let pwd_two = generate_deterministic_password_with_seed_impl("dizzle-wit-wizzle", "password_two", 42);
    let pwd_one_2 = generate_deterministic_password_with_seed_impl("dizzle-wit-wizzle", "password_one", 42);
    let pwd_two_2 = generate_deterministic_password_with_seed_impl("dizzle-wit-wizzle", "password_two", 42);

    assert_eq!(pwd_one.len(), 42);
    assert_eq!(pwd_two.len(), 42);
    assert_ne!(pwd_one, pwd_two);
    assert_eq!(pwd_one, pwd_one_2);
    assert_eq!(pwd_two, pwd_two_2);

    assert_eq!(pwd_one, "7lXxsL9b5Vqt079gzDeb5DGUJOPFInNrJzx8kdifue");
    assert_eq!(pwd_two, "FRisDy0AUnAvrRw7R8vPLIIJ9rFYPsKBlLXttjuBo4");
}
