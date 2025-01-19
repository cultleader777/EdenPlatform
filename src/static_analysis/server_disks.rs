use std::collections::{BTreeMap, BTreeSet};

use regex::Regex;

use crate::database::{Database, TableRowPointerServerDisk, TableRowPointerDatacenter, TableRowPointerServer};
use super::PlatformValidationError;

// There are only two valuable formats for eden platform:
// ZFS - we should run everything we can as ZFS,
// it is superior filesystem, provides checksumming, snapshots,
// encryption, automatic disk replacement. We should find excuse
// not to run it, that's why root is on zfs
// XFS - the only reason this is needed at all because MinIO recommends
// using it and it already should provide redundancy guarantees.
// Maybe overkill, a bunch of pussies, we could also run ZFS
// and enjoy it with MinIO but whatever
//
// ext4 - no use
// btrfs - no use and not production ready
#[derive(Debug)]
enum DiskUsage {
    Zfs(String),
    Xfs,
}

#[derive(Default)]
struct DiskState {
    usage: Option<DiskUsage>,
}

pub fn server_disk_analysis(db: &Database) -> Result<BTreeMap<TableRowPointerServerDisk, i64>, PlatformValidationError> {
    server_disk_ids_analysis(db)?;

    let minimum_capacity_bytes = 1024 * 1024 * 1024;
    for disk_kind in db.disk_kind().rows_iter() {
        let is_elastic = db.disk_kind().c_is_elastic(disk_kind);
        let specified_capacity_bytes = db.disk_kind().c_capacity_bytes(disk_kind);
        let max_capacity_bytes = db.disk_kind().c_max_capacity_bytes(disk_kind);
        let min_capacity_bytes = db.disk_kind().c_min_capacity_bytes(disk_kind);
        if is_elastic {
            if specified_capacity_bytes > 0 {
                return Err(PlatformValidationError::DiskKindDiskIsElasticButHasSpecifiedCapacity {
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    specified_capacity_bytes,
                    is_elastic,
                });
            }

            if max_capacity_bytes < 0 {
                return Err(PlatformValidationError::DiskKindDiskIsElasticButDoesntSpecifyMaxCapacity {
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    max_capacity_bytes,
                    is_elastic,
                });
            }

            if min_capacity_bytes > max_capacity_bytes {
                return Err(PlatformValidationError::DiskKindMinCapacityBiggerThanMaxCapacity {
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    specified_min_capacity_bytes: min_capacity_bytes,
                    specified_max_capacity_bytes: max_capacity_bytes,
                    is_elastic,
                });
            }
        } else {
            if specified_capacity_bytes < 0 {
                return Err(PlatformValidationError::DiskKindDiskIsNotElasticButHasUnspecifiedCapacity {
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    specified_capacity_bytes,
                    is_elastic,
                });
            }

            // minimum disk size is 1GB
            if specified_capacity_bytes < minimum_capacity_bytes {
                return Err(PlatformValidationError::DiskKindDiskCapacityMustBeAtLeast1GB {
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    specified_capacity_bytes,
                    minimum_capacity_bytes,
                    is_elastic,
                });
            }

            if max_capacity_bytes > 0 {
                return Err(PlatformValidationError::DiskKindDiskIsNotElasticButHasMaxCapacitySpecified {
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    specified_max_capacity_bytes: max_capacity_bytes,
                    is_elastic,
                });
            }

            if min_capacity_bytes > 0 {
                return Err(PlatformValidationError::DiskKindDiskIsNotElasticButHasMinCapacitySpecified {
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    specified_min_capacity_bytes: min_capacity_bytes,
                    is_elastic,
                });
            }
        }
    }

    let mut disk_sizes: BTreeMap<TableRowPointerServerDisk, i64> = BTreeMap::new();
    for server in db.server().rows_iter() {
        let root_disk = db.server().c_root_disk(server);

        if db.server_disk().c_xfs_format(root_disk) {
            return Err(PlatformValidationError::ServerRootDiskCanOnlyBeFormattedAsZfs {
                server: db.server().c_hostname(server).clone(),
                root_disk_id: db.server_disk().c_disk_id(root_disk).clone(),
                wanted_format: "xfs".to_string(),
                only_allowed_format: "zfs".to_string(),
            });
        }

        for xfs_vol in db.server().c_children_server_xfs_volume(server) {
            let disk = db.server_xfs_volume().c_xfs_disk(*xfs_vol);
            if !db.server_disk().c_xfs_format(disk) {
                return Err(PlatformValidationError::ServerXfsVolumeIsNotOnXfsFormattedDisk {
                    server: db.server().c_hostname(server).clone(),
                    is_xfs_disk: false,
                    xfs_disk_id: db.server_disk().c_disk_id(disk).clone(),
                    xfs_volume_name: db.server_xfs_volume().c_volume_name(*xfs_vol).clone(),
                });
            }
        }

        let mut disk_states: BTreeMap<TableRowPointerServerDisk, DiskState> = BTreeMap::new();
        for disk in db.server().c_children_server_disk(server) {
            let disk_kind = db.server_disk().c_disk_kind(*disk);
            let disk_kind_name = db.disk_kind().c_kind(disk_kind);
            let non_eligible_reason = db.disk_kind().c_non_eligible_reason(disk_kind);
            let has_extra_config = db.disk_kind().c_has_extra_config(disk_kind);
            let extra_config = db.server_disk().c_extra_config(*disk);
            let disk_id = db.server_disk().c_disk_id(*disk);
            let kind_capacity_bytes = db.disk_kind().c_capacity_bytes(disk_kind);
            let server_capacity_bytes = db.server_disk().c_capacity_bytes(*disk);
            let maximum_kind_capacity_bytes = db.disk_kind().c_max_capacity_bytes(disk_kind);
            let minimum_kind_capacity_bytes = db.disk_kind().c_min_capacity_bytes(disk_kind);
            let is_elastic = db.disk_kind().c_is_elastic(disk_kind);
            if !non_eligible_reason.is_empty() {
                return Err(PlatformValidationError::ServerDiskWithDiskKindIsNotSupportedForProvisioning {
                    server: db.server().c_hostname(server).clone(),
                    disk_id: disk_id.clone(),
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    non_eligible_reason: non_eligible_reason.clone(),
                });
            }

            if disk_id.starts_with("nvme") && "nvme" != db.disk_kind().c_medium(disk_kind) {
                if !disk_kind_name.starts_with("aws.") { // damn aws, hdd can be nvme
                    return Err(PlatformValidationError::ServerNvmeNamedDiskIsNotNvmeMediumByDiskKind {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: disk_id.clone(),
                        disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                        disk_kind_medium: db.disk_kind().c_medium(disk_kind).clone(),
                    });
                }
            }

            if !has_extra_config && !extra_config.is_empty() {
                return Err(PlatformValidationError::ServerDiskHasExtraConfigNotAllowedForDiskKind {
                    server: db.server().c_hostname(server).clone(),
                    disk_id: disk_id.clone(),
                    disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                    extra_config: extra_config.clone(),
                    is_extra_config_allowed: has_extra_config,
                });
            }

            if is_elastic {
                if server_capacity_bytes < 0 {
                    return Err(PlatformValidationError::ServerDiskKindIsElasticButDiskSizeIsNotSpecified {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: disk_id.clone(),
                        disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                        is_elastic,
                        specified_capacity_bytes: server_capacity_bytes,
                    });
                }

                if server_capacity_bytes < minimum_capacity_bytes {
                    return Err(PlatformValidationError::ServerDiskMinimumCapacityIs1GB {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: disk_id.clone(),
                        disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                        is_elastic,
                        specified_capacity_bytes: server_capacity_bytes,
                    });
                }

                if minimum_kind_capacity_bytes != -1 && server_capacity_bytes < minimum_kind_capacity_bytes {
                    return Err(PlatformValidationError::ServerDiskHasLowerThanMinimumCapacityAllowedByDiskKind {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: disk_id.clone(),
                        disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                        is_elastic,
                        specified_capacity_bytes: server_capacity_bytes,
                        minimum_disk_capacity_bytes: minimum_kind_capacity_bytes,
                    });
                }

                if server_capacity_bytes > maximum_kind_capacity_bytes {
                    return Err(PlatformValidationError::ServerDiskExceedsMaximumCapacityAllowedByDiskKind {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: disk_id.clone(),
                        disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                        is_elastic,
                        specified_capacity_bytes: server_capacity_bytes,
                        maximum_disk_capacity_bytes: maximum_kind_capacity_bytes,
                    });
                }
            } else {
                if server_capacity_bytes > 0 {
                    return Err(PlatformValidationError::ServerDiskKindIsNotElasticButDiskSizeIsSpecified {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: disk_id.clone(),
                        disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                        is_elastic,
                        specified_capacity_bytes: server_capacity_bytes,
                    });
                }
            }

            assert!(server_capacity_bytes != kind_capacity_bytes, "Capacity must be specified in one place, not both");
            assert!(server_capacity_bytes > 0 || kind_capacity_bytes > 0, "User errors should have checked this earlier");
            let final_capacity = server_capacity_bytes.max(kind_capacity_bytes);
            assert!(disk_sizes.insert(*disk, final_capacity).is_none());

            let state = disk_states.entry(*disk).or_default();
            if db.server_disk().c_xfs_format(*disk) {
                assert!(state.usage.is_none());
                state.usage = Some(DiskUsage::Xfs);
            }
            if root_disk == *disk {
                // rough size of NixOS install in our context
                let minimum_root_disk_capacity_bytes = 5 * 1024 * 1024 * 1024;
                if final_capacity < minimum_root_disk_capacity_bytes {
                    return Err(PlatformValidationError::ServerDiskRootDiskMustBeAtLeast5GB {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: disk_id.clone(),
                        disk_kind: db.disk_kind().c_kind(disk_kind).clone(),
                        is_root_disk: true,
                        is_elastic,
                        minimum_root_disk_capacity_bytes,
                        specified_capacity_bytes: server_capacity_bytes,
                    });
                }
                state.usage = Some(DiskUsage::Zfs("ROOT zpool:rpool".to_string()))
            }
        }

        for zpool in db.server().c_children_server_zpool(server) {
            let is_redundant = db.server_zpool().c_is_redundant(*zpool);
            let zpool_name = db.server_zpool().c_zpool_name(*zpool);
            let vdev_count = db.server_zpool().c_children_server_zpool_vdev(*zpool).len();

            if "rpool" == zpool_name {
                return Err(PlatformValidationError::ServerZpoolNameIsReserved {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: zpool_name.clone(),
                    reserved_zpool_name: "rpool".to_string(),
                });
            }

            if db.server_zpool().c_children_server_zpool_vdev(*zpool).is_empty() {
                return Err(PlatformValidationError::ServerZpoolHasNoVdevs {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    vdev_count,
                });
            }

            let mut vdev_types = BTreeSet::new();
            let mut vdev_indexes: BTreeSet<i64> = BTreeSet::new();
            let mut disks_per_vdev_index: BTreeSet<usize> = BTreeSet::new();
            let mut unique_disk_sizes: BTreeSet<i64> = BTreeSet::new();
            let mut unique_disk_mediums: BTreeSet<&str> = BTreeSet::new();
            let mut unique_disk_kinds: BTreeSet<&str> = BTreeSet::new();
            let mut has_elastic_disks = false;
            for vdev in db.server_zpool().c_children_server_zpool_vdev(*zpool) {
                let vdev_type = db.server_zpool_vdev().c_vdev_type(*vdev);
                let _ = vdev_types.insert(vdev_type);
                let vdev_number = db.server_zpool_vdev().c_vdev_number(*vdev);
                assert!(vdev_indexes.insert(vdev_number), "EDB should ensure vdev indexes are unique");

                disks_per_vdev_index.insert(db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*vdev).len());

                let vdev_disk_count = db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*vdev).len();
                // if zpool has only one vdev we risk
                if vdev_disk_count == 0 {
                    return Err(PlatformValidationError::ServerZpoolVdevHasNoDisks {
                        server: db.server().c_hostname(server).clone(),
                        zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                        vdev_index: db.server_zpool_vdev().c_vdev_number(*vdev),
                        disks_found: vdev_disk_count,
                    });
                }

                if is_redundant && 1 == vdev_disk_count {
                    return Err(PlatformValidationError::ServerRedundantZpoolVdevHasOnlyOneDisk {
                        server: db.server().c_hostname(server).clone(),
                        zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                        vdev_index: db.server_zpool_vdev().c_vdev_number(*vdev),
                        disks_found: vdev_disk_count,
                        is_zpool_marked_redundant: true,
                    });
                }

                if vdev_type == "mirror" {
                    // I have heard of people running 3 way mirror
                    // but not about 4
                    let max_allowed_mirror_disks = 3;
                    if vdev_disk_count > max_allowed_mirror_disks {
                        return Err(PlatformValidationError::ServerZpoolMirrorVdevHasMoreThanAllowedDisks {
                            server: db.server().c_hostname(server).clone(),
                            zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                            vdev_index: db.server_zpool_vdev().c_vdev_number(*vdev),
                            disks_found: vdev_disk_count,
                            maximum_allowed_disks: max_allowed_mirror_disks,
                        });
                    }
                } else if vdev_type.starts_with("raidz") {
                    let raidz_level: usize = (&vdev_type["raidz".len()..]).parse().unwrap();
                    assert!(raidz_level >= 1 && raidz_level <= 3);
                    // okay, there could be a case where someone would run
                    // raidz1 with two disks, but I don't get why someone
                    // would do that, so lets forbid it
                    let minimum_disks_required = raidz_level + 2;
                    // someone needs to justify having more disks than
                    // that per vdev?
                    let maximum_disks_allowed = 24;
                    if vdev_disk_count < minimum_disks_required {
                        return Err(PlatformValidationError::ServerZpoolRaidzVdevHasTooFewDisks {
                            server: db.server().c_hostname(server).clone(),
                            zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                            vdev_index: db.server_zpool_vdev().c_vdev_number(*vdev),
                            disks_found: vdev_disk_count,
                            minimum_disks_required,
                            raid_type: format!("raidz{raidz_level}"),
                        });
                    }

                    if vdev_disk_count > maximum_disks_allowed {
                        return Err(PlatformValidationError::ServerZpoolRaidzVdevHasTooManyDisks {
                            server: db.server().c_hostname(server).clone(),
                            zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                            vdev_index: db.server_zpool_vdev().c_vdev_number(*vdev),
                            disks_found: vdev_disk_count,
                            maximum_disks_allowed,
                            raid_type: format!("raidz{raidz_level}"),
                        });
                    }

                } else {
                    panic!("Unknown vdev disk config: {vdev_type}");
                }

                for vdev_child_disk in db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*vdev) {
                    let disk_id = db.server_zpool_vdev_disk().c_disk_id(*vdev_child_disk);
                    let state = disk_states.get_mut(&disk_id).unwrap();
                    let expected_usage = DiskUsage::Zfs(format!("zpool:{zpool_name} vdev:{vdev_number}"));
                    if let Some(usage) = &state.usage {
                        return Err(PlatformValidationError::DoubleUsageOfServerDiskDetected {
                            server: db.server().c_hostname(server).clone(),
                            disk_id: db.server_disk().c_disk_id(disk_id).clone(),
                            previous_usage: format!("{:?}", usage),
                            another_usage: format!("{:?}", expected_usage),
                        });
                    }
                    state.usage = Some(expected_usage);

                    let disk_kind = db.server_disk().c_disk_kind(disk_id);
                    let disk_medium = db.disk_kind().c_medium(disk_kind);
                    let disk_size = disk_sizes.get(&disk_id).unwrap();
                    unique_disk_mediums.insert(disk_medium);
                    unique_disk_sizes.insert(*disk_size);
                    unique_disk_kinds.insert(db.disk_kind().c_kind(disk_kind).as_str());
                    has_elastic_disks |= db.disk_kind().c_is_elastic(disk_kind);
                }
            }

            for spare in db.server_zpool().c_children_server_zpool_spare(*zpool) {
                let disk_id = db.server_zpool_spare().c_disk_id(*spare);
                let state = disk_states.get_mut(&disk_id).unwrap();
                let expected_usage = DiskUsage::Zfs(format!("zpool:{zpool_name} spare"));
                if let Some(usage) = &state.usage {
                    return Err(PlatformValidationError::DoubleUsageOfServerDiskDetected {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: db.server_disk().c_disk_id(disk_id).clone(),
                        previous_usage: format!("{:?}", usage),
                        another_usage: format!("{:?}", expected_usage),
                    });
                }
                state.usage = Some(expected_usage);

                let disk_kind = db.server_disk().c_disk_kind(disk_id);
                let disk_medium = db.disk_kind().c_medium(disk_kind);
                let disk_size = disk_sizes.get(&disk_id).unwrap();
                unique_disk_mediums.insert(disk_medium);
                unique_disk_sizes.insert(*disk_size);
                unique_disk_kinds.insert(db.disk_kind().c_kind(disk_kind).as_str());
            }

            for cache in db.server_zpool().c_children_server_zpool_cache(*zpool) {
                let disk_id = db.server_zpool_cache().c_disk_id(*cache);
                let state = disk_states.get_mut(&disk_id).unwrap();
                let expected_usage = DiskUsage::Zfs(format!("zpool:{zpool_name} cache"));
                if let Some(usage) = &state.usage {
                    return Err(PlatformValidationError::DoubleUsageOfServerDiskDetected {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: db.server_disk().c_disk_id(disk_id).clone(),
                        previous_usage: format!("{:?}", usage),
                        another_usage: format!("{:?}", expected_usage),
                    });
                }
                state.usage = Some(expected_usage);
            }

            if db.server_zpool().c_children_server_zpool_log(*zpool).len() > 2 {
                return Err(PlatformValidationError::ServerZpoolCannotHaveMoreThanTwoLogDisks {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    log_devices_found: db.server_zpool().c_children_server_zpool_log(*zpool).len(),
                    log_devices_maximum: 2,
                });
            }

            let mut unique_log_disk_sizes: BTreeSet<i64> = BTreeSet::new();
            let mut unique_log_disk_mediums: BTreeSet<&str> = BTreeSet::new();
            let mut unique_log_disk_kinds: BTreeSet<&str> = BTreeSet::new();
            for log in db.server_zpool().c_children_server_zpool_log(*zpool) {
                let disk_id = db.server_zpool_log().c_disk_id(*log);
                let disk_size = disk_sizes.get(&disk_id).unwrap();
                let disk_kind = db.server_disk().c_disk_kind(disk_id);
                let disk_medium = db.disk_kind().c_medium(disk_kind);
                let state = disk_states.get_mut(&disk_id).unwrap();
                unique_log_disk_mediums.insert(disk_medium);
                unique_log_disk_sizes.insert(*disk_size);
                unique_log_disk_kinds.insert(db.disk_kind().c_kind(disk_kind).as_str());

                let expected_usage = DiskUsage::Zfs(format!("zpool:{zpool_name} log"));
                if let Some(usage) = &state.usage {
                    return Err(PlatformValidationError::DoubleUsageOfServerDiskDetected {
                        server: db.server().c_hostname(server).clone(),
                        disk_id: db.server_disk().c_disk_id(disk_id).clone(),
                        previous_usage: format!("{:?}", usage),
                        another_usage: format!("{:?}", expected_usage),
                    });
                }
                state.usage = Some(expected_usage);
            }

            if disks_per_vdev_index.len() > 1 {
                return Err(PlatformValidationError::ServerZpoolVdevsHaveUnequalAmountOfDisks {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    disk_counts_per_vdev_found: disks_per_vdev_index.iter().cloned().collect(),
                });
            }

            if vdev_types.len() > 1 {
                return Err(PlatformValidationError::ServerZpoolHasMoreThanOneVdevType {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    found_vdev_types:
                        vdev_types
                        .iter()
                        .map(|i| (*i).clone())
                        .collect(),
                });
            }

            let mut idx_iter = vdev_indexes.iter();
            let mut prev: i64 = *idx_iter.next().unwrap();
            if prev != 1 {
                return Err(PlatformValidationError::ServerZpoolVdevsIdSequenceDoesntStartWith1 {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    minimum_vdev_id: prev,
                    only_allowed_minimum_vdev_id: 1,
                });
            }

            for next in idx_iter {
                if next - prev != 1 {
                    return Err(PlatformValidationError::ServerZpoolVdevsIdsAreNotSequential {
                        server: db.server().c_hostname(server).clone(),
                        zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                        current_vdev_ids: vdev_indexes.iter().cloned().collect(),
                        vdev_id_a: prev,
                        vdev_id_b: *next,
                        vdev_id_b_expected: prev + 1,
                    });
                }
                prev = *next;
            }

            if unique_disk_sizes.len() > 1 {
                return Err(PlatformValidationError::ServerZpoolDifferentDiskSizesDetected {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    different_disk_sizes: unique_disk_sizes.iter().cloned().collect(),
                    different_disk_kinds_involved: unique_disk_kinds.iter().map(|i| i.to_string()).collect(),
                });
            }

            if unique_disk_mediums.len() > 1 {
                return Err(PlatformValidationError::ServerZpoolDifferentDiskMediumsDetected {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    different_disk_mediums: unique_disk_mediums.iter().map(|i| i.to_string()).collect(),
                    different_disk_kinds_involved: unique_disk_kinds.iter().map(|i| i.to_string()).collect(),
                });
            }

            if unique_log_disk_sizes.len() > 1 {
                return Err(PlatformValidationError::ServerZpoolLogDifferentDiskSizesDetected {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    different_disk_sizes: unique_log_disk_sizes.iter().cloned().collect(),
                    different_disk_kinds_involved: unique_log_disk_kinds.iter().map(|i| i.to_string()).collect(),
                });
            }

            if unique_log_disk_mediums.len() > 1 {
                return Err(PlatformValidationError::ServerZpoolLogDifferentDiskMediumsDetected {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    different_disk_mediums: unique_log_disk_mediums.iter().map(|i| i.to_string()).collect(),
                    different_disk_kinds_involved: unique_log_disk_kinds.iter().map(|i| i.to_string()).collect(),
                });
            }

            if has_elastic_disks && vdev_count > 1 {
                // I don't think this ever make sense to ever have it like that?
                return Err(PlatformValidationError::ServerZpoolHasMoreThanOneVdevButHasElasticDisk {
                    server: db.server().c_hostname(server).clone(),
                    zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                    explanation: "You have elastic disks in your zpool but have more than one vdev. In such case you should use only one vdev and increase elastic disk sizes in that vdev instead.".to_string(),
                    different_disk_kinds_involved: unique_disk_kinds.iter().map(|i| i.to_string()).collect(),
                });
            }

            let fastest_medium_speed =
                unique_disk_mediums.iter().map(|i| disk_medium_speed(*i)).max().unwrap();

            for cache in db.server_zpool().c_children_server_zpool_cache(*zpool) {
                let cache_disk = db.server_zpool_cache().c_disk_id(*cache);
                let disk_kind = db.server_disk().c_disk_kind(cache_disk);
                let disk_medium = db.disk_kind().c_medium(disk_kind).as_str();
                let medium_speed = disk_medium_speed(disk_medium);

                if medium_speed < fastest_medium_speed {
                    let vdev_medium = disk_medium_speed_to_medium(fastest_medium_speed);
                    return Err(PlatformValidationError::ServerZpoolCacheDeviceIsSlowerThanVdevDisks {
                        server: db.server().c_hostname(server).clone(),
                        zpool_name: db.server_zpool().c_zpool_name(*zpool).clone(),
                        cache_disk_id: db.server_disk().c_disk_id(cache_disk).clone(),
                        cache_disk_medium: disk_medium.to_string(),
                        fastest_vdev_medium: vdev_medium.to_string(),
                    });
                }
            }
        }
    }

    Ok(disk_sizes)
}

fn determine_disk_id_kind(inp: &str) -> DiskIdsPolicy {
    lazy_static! {
        pub static ref DISK_DEV_REGEX: Regex = Regex::new(r#"^x?[svh]d[a-z]$"#).unwrap();
        pub static ref NVME_DEV_REGEX: Regex = Regex::new(r#"^nvme[0-9][0-9]?n[0-9][0-9]?$"#).unwrap();
    }

    // is there anything longer? I'm not sure yet
    let longest_possible_disk_id = "nvme99n99";

    if DISK_DEV_REGEX.is_match(inp) || NVME_DEV_REGEX.is_match(inp)
        || inp.len() <= longest_possible_disk_id.len()
    {
        DiskIdsPolicy::ByDevName
    } else {
        // anything longer than longest possible id assume it is serial
        DiskIdsPolicy::ByDiskSerial
    }
}

fn server_disk_ids_analysis(db: &Database) -> Result<(), PlatformValidationError> {

    let mut disk_serials: BTreeMap<&str, TableRowPointerServer> = BTreeMap::new();

    for dc in db.datacenter().rows_iter() {
        let policy = pick_disk_id_policy(db, dc);
        for server in db.datacenter().c_referrers_server__dc(dc) {
            for disk in db.server().c_children_server_disk(*server) {
                let disk_id = db.server_disk().c_disk_id(*disk);
                let disk_name_kind = determine_disk_id_kind(disk_id.as_str());
                if policy != disk_name_kind {
                    return Err(PlatformValidationError::ServerDiskIdUnexpectedFormatInDatacenter {
                        server: db.server().c_hostname(*server).clone(),
                        datacenter: db.datacenter().c_dc_name(dc).clone(),
                        datacenter_implementation: db.datacenter().c_implementation(dc).clone(),
                        datacenter_disk_ids_policy: disk_id_policy_to_str(&policy).to_string(),
                        actual_disk_id_kind: disk_id_policy_to_str(&disk_name_kind).to_string(),
                        disk_id: db.server_disk().c_disk_id(*disk).clone(),
                    });
                }
            }
        }

        if policy == DiskIdsPolicy::ByDiskSerial {
            for server in db.datacenter().c_referrers_server__dc(dc) {
                for disk in db.server().c_children_server_disk(*server) {
                    if let Some(prev) = disk_serials.insert(db.server_disk().c_disk_id(*disk).as_str(), *server) {
                        return Err(PlatformValidationError::DetectedDuplicateDiskSerials {
                            server_a: db.server().c_hostname(prev).clone(),
                            server_a_disk_id: db.server_disk().c_disk_id(*disk).clone(),
                            server_b: db.server().c_hostname(*server).clone(),
                            server_b_disk_id: db.server_disk().c_disk_id(*disk).clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn disk_medium_speed(medium: &str) -> i64 {
    // relative
    match medium {
        "nvme" => 100,
        "ssd" => 50,
        "hdd" => 10,
        _ => panic!("Unknown disk medium {medium}")
    }
}

fn disk_medium_speed_to_medium(input: i64) -> &'static str {
    match input {
        100 => "nvme",
        50 => "ssd",
        10 => "hdd",
        _ => panic!("Unknown disk medium speed")
    }
}

#[derive(PartialEq, Eq)]
pub enum DiskIdsPolicy {
    ByDevName,
    ByDiskSerial,
}

pub fn disk_id_policy_to_str(pol: &DiskIdsPolicy) -> &'static str {
    match pol {
        DiskIdsPolicy::ByDevName => "require_devname",
        DiskIdsPolicy::ByDiskSerial => "require_serial",
    }
}

pub fn pick_disk_id_policy(db: &Database, dc: TableRowPointerDatacenter) -> DiskIdsPolicy {
    let policy = db.datacenter().c_disk_ids_policy(dc);
    let implementation = db.datacenter().c_implementation(dc);
    match policy.as_str() {
        "auto" => {
            // only bm dcs require disk ids specified
            if implementation == "bm_simple" || implementation == "hetzner" {
                DiskIdsPolicy::ByDiskSerial
            } else {
                DiskIdsPolicy::ByDevName
            }
        }
        "require_serial" => {
            DiskIdsPolicy::ByDiskSerial
        }
        "require_devname" => {
            DiskIdsPolicy::ByDevName
        }
        other => {
            panic!("Unknown disk ids policy: {other}")
        }
    }
}

pub fn pick_absolute_disk_path_by_policy(db: &Database, disk: TableRowPointerServerDisk, pol: &DiskIdsPolicy) -> String {
    match pol {
        DiskIdsPolicy::ByDevName => db.server_disk().c_disk_id(disk).clone(),
        DiskIdsPolicy::ByDiskSerial => format!("/dev/disk/by-id/{}", db.server_disk().c_disk_id(disk))
    }
}
