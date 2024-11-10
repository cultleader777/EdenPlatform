#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[cfg(test)]
use super::common;

#[test]
fn test_root_disk_only_allowed_as_zfs() {
    assert_eq!(
        PlatformValidationError::ServerRootDiskCanOnlyBeFormattedAsZfs {
            server: "server-a".to_string(),
            root_disk_id: "vda".to_string(),
            wanted_format: "xfs".to_string(),
            only_allowed_format: "zfs".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk {
        disk_id: 'vda',
        xfs_format: true,
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_reserved_name() {
    assert_eq!(
        PlatformValidationError::ServerZpoolNameIsReserved {
            server: "server-a".to_string(),
            reserved_zpool_name: "rpool".to_string(),
            zpool_name: "rpool".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk {
        disk_id: 'vda',
    }
    WITH server_zpool {
        zpool_name: rpool,
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_no_vdevs() {
    assert_eq!(
        PlatformValidationError::ServerZpoolHasNoVdevs {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            vdev_count: 0,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk {
        disk_id: 'vda',
    }
    WITH server_zpool {
        zpool_name: pool2,
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_vdev_no_disks() {
    assert_eq!(
        PlatformValidationError::ServerZpoolVdevHasNoDisks {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            vdev_index: 1,
            disks_found: 0,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk {
        disk_id: 'vda',
    }
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev {
          vdev_number: 1,
          vdev_type: mirror,
        }
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_redundant_zpool_vdev_only_one_disk() {
    assert_eq!(
        PlatformValidationError::ServerRedundantZpoolVdevHasOnlyOneDisk {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            vdev_index: 1,
            disks_found: 1,
            is_zpool_marked_redundant: true,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev {
          vdev_number: 1,
          vdev_type: mirror,
          WITH server_zpool_vdev_disk {
            disk_id: vdb
          }
        }
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_mirror_more_than_allowed_disks() {
    assert_eq!(
        PlatformValidationError::ServerZpoolMirrorVdevHasMoreThanAllowedDisks {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            vdev_index: 1,
            disks_found: 4,
            maximum_allowed_disks: 3,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
        { disk_id: 'vdc' },
        { disk_id: 'vdd' },
        { disk_id: 'vde' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev {
          vdev_number: 1,
          vdev_type: mirror,
          WITH server_zpool_vdev_disk [
            { disk_id: vdb },
            { disk_id: vdc },
            { disk_id: vdd },
            { disk_id: vde },
          ]
        }
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_raidz_vdev_has_too_few_disks() {
    assert_eq!(
        PlatformValidationError::ServerZpoolRaidzVdevHasTooFewDisks {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            vdev_index: 1,
            disks_found: 2,
            minimum_disks_required: 3,
            raid_type: "raidz1".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
        { disk_id: 'vdc' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev {
          vdev_number: 1,
          vdev_type: raidz1,
          WITH server_zpool_vdev_disk [
            { disk_id: vdb },
            { disk_id: vdc },
          ]
        }
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_raidz_vdev_has_too_many_disks() {
    assert_eq!(
        PlatformValidationError::ServerZpoolRaidzVdevHasTooManyDisks {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            vdev_index: 1,
            disks_found: 25,
            maximum_disks_allowed: 24,
            raid_type: "raidz1".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
        { disk_id: 'vdc' },
        { disk_id: 'vdd' },
        { disk_id: 'vde' },
        { disk_id: 'vdf' },
        { disk_id: 'vdg' },
        { disk_id: 'vdh' },
        { disk_id: 'vdi' },
        { disk_id: 'vdj' },
        { disk_id: 'vdk' },
        { disk_id: 'vdl' },
        { disk_id: 'vdm' },
        { disk_id: 'vdn' },
        { disk_id: 'vdo' },
        { disk_id: 'vdp' },
        { disk_id: 'vdr' },
        { disk_id: 'vds' },
        { disk_id: 'vdt' },
        { disk_id: 'vdu' },
        { disk_id: 'vdv' },
        { disk_id: 'vdx' },
        { disk_id: 'vdy' },
        { disk_id: 'vdz' },
        { disk_id: 'sda' },
        { disk_id: 'sdb' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev {
          vdev_number: 1,
          vdev_type: raidz1,
          WITH server_zpool_vdev_disk [
            { disk_id: 'vdb' },
            { disk_id: 'vdc' },
            { disk_id: 'vdd' },
            { disk_id: 'vde' },
            { disk_id: 'vdf' },
            { disk_id: 'vdg' },
            { disk_id: 'vdh' },
            { disk_id: 'vdi' },
            { disk_id: 'vdj' },
            { disk_id: 'vdk' },
            { disk_id: 'vdl' },
            { disk_id: 'vdm' },
            { disk_id: 'vdn' },
            { disk_id: 'vdo' },
            { disk_id: 'vdp' },
            { disk_id: 'vdr' },
            { disk_id: 'vds' },
            { disk_id: 'vdt' },
            { disk_id: 'vdu' },
            { disk_id: 'vdv' },
            { disk_id: 'vdx' },
            { disk_id: 'vdy' },
            { disk_id: 'vdz' },
            { disk_id: 'sda' },
            { disk_id: 'sdb' },
          ]
        }
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_double_usage_of_root_disk() {
    assert_eq!(
        PlatformValidationError::DoubleUsageOfServerDiskDetected {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            previous_usage: "Zfs(\"ROOT zpool:rpool\")".to_string(),
            another_usage: "Zfs(\"zpool:pool2 vdev:1\")".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        is_redundant: false,
        WITH server_zpool_vdev {
          vdev_number: 1,
          vdev_type: mirror,
          WITH server_zpool_vdev_disk [
            { disk_id: vda },
          ]
        }
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_double_usage_of_disk() {
    assert_eq!(
        PlatformValidationError::DoubleUsageOfServerDiskDetected {
            server: "server-a".to_string(),
            disk_id: "vdb".to_string(),
            previous_usage: "Zfs(\"zpool:pool2 vdev:1\")".to_string(),
            another_usage: "Zfs(\"zpool:pool2 vdev:2\")".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        is_redundant: false,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_double_usage_of_disk_with_xfs() {
    assert_eq!(
        PlatformValidationError::DoubleUsageOfServerDiskDetected {
            server: "server-a".to_string(),
            disk_id: "vdb".to_string(),
            previous_usage: "Xfs".to_string(),
            another_usage: "Zfs(\"zpool:pool2 vdev:1\")".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', xfs_format: true, },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        is_redundant: false,
        WITH server_zpool_vdev {
          vdev_number: 1,
          vdev_type: mirror,
          WITH server_zpool_vdev_disk [
            { disk_id: vdb },
          ]
        }
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_vdev_different_disk_count() {
    assert_eq!(
        PlatformValidationError::ServerZpoolVdevsHaveUnequalAmountOfDisks {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            disk_counts_per_vdev_found: vec![2, 3],
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
        { disk_id: 'vdc' },
        { disk_id: 'vdd' },
        { disk_id: 'vde' },
        { disk_id: 'vdf' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdd },
                    { disk_id: vde },
                    { disk_id: vdf },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_vdev_different_types() {
    assert_eq!(
        PlatformValidationError::ServerZpoolHasMoreThanOneVdevType {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            found_vdev_types: vec![
                "mirror".to_string(),
                "raidz1".to_string(),
            ]
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
        { disk_id: 'vdc' },
        { disk_id: 'vdd' },
        { disk_id: 'vde' },
        { disk_id: 'vdf' },
        { disk_id: 'vdg' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                    { disk_id: vdd },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: raidz1,
                WITH server_zpool_vdev_disk [
                    { disk_id: vde },
                    { disk_id: vdf },
                    { disk_id: vdg },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_vdev_bad_sequence_start() {
    assert_eq!(
        PlatformValidationError::ServerZpoolVdevsIdSequenceDoesntStartWith1 {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            minimum_vdev_id: 3,
            only_allowed_minimum_vdev_id: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
        { disk_id: 'vdc' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 3,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_vdev_sequence_has_gaps() {
    assert_eq!(
        PlatformValidationError::ServerZpoolVdevsIdsAreNotSequential {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            current_vdev_ids: vec![1, 7],
            vdev_id_a: 1,
            vdev_id_b: 7,
            vdev_id_b_expected: 2,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb' },
        { disk_id: 'vdc' },
        { disk_id: 'vdd' },
        { disk_id: 'vde' },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
            {
                vdev_number: 7,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdd },
                    { disk_id: vde },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_vdev_different_disk_sizes() {
    assert_eq!(
        PlatformValidationError::ServerZpoolDifferentDiskSizesDetected {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            different_disk_sizes: vec![21474836480, 22474836480],
            different_disk_kinds_involved: vec![
                "another-ssd".to_string(),
                "default-ssd".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: another-ssd,
    medium: ssd,
    capacity_bytes: 22474836480,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', disk_kind: default-ssd },
        { disk_id: 'vdc', disk_kind: another-ssd },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_vdev_different_disk_mediums() {
    assert_eq!(
        PlatformValidationError::ServerZpoolDifferentDiskMediumsDetected {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            different_disk_mediums: vec![
                "hdd".to_string(),
                "ssd".to_string(),
            ],
            different_disk_kinds_involved: vec![
                "another-hdd".to_string(),
                "default-ssd".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: another-hdd,
    medium: hdd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', disk_kind: default-ssd },
        { disk_id: 'vdc', disk_kind: another-hdd },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_more_than_one_vdev_with_elastic_disks() {
    assert_eq!(
        PlatformValidationError::ServerZpoolHasMoreThanOneVdevButHasElasticDisk {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            explanation: "You have elastic disks in your zpool but have more than one vdev. In such case you should use only one vdev and increase elastic disk sizes in that vdev instead.".to_string(),
            different_disk_kinds_involved: vec![
                "default-ssd".to_string(),
                "elastic-ssd".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: elastic-ssd,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
    max_capacity_bytes: 21474836480,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', disk_kind: default-ssd },
        { disk_id: 'vdc', disk_kind: elastic-ssd, capacity_bytes: 21474836480 },
        { disk_id: 'vdd', disk_kind: default-ssd },
        { disk_id: 'vde', disk_kind: default-ssd },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdd },
                    { disk_id: vde },
                ]
            },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_cache_device_slower_than_vdev() {
    assert_eq!(
        PlatformValidationError::ServerZpoolCacheDeviceIsSlowerThanVdevDisks {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            cache_disk_id: "vdf".to_string(),
            cache_disk_medium: "hdd".to_string(),
            fastest_vdev_medium: "ssd".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: some-hdd,
    medium: hdd,
    is_elastic: false,
    capacity_bytes: 21474836480,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', disk_kind: default-ssd },
        { disk_id: 'vdc', disk_kind: default-ssd },
        { disk_id: 'vdd', disk_kind: default-ssd },
        { disk_id: 'vde', disk_kind: default-ssd },
        { disk_id: 'vdf', disk_kind: some-hdd },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdd },
                    { disk_id: vde },
                ]
            },
        ]
        WITH server_zpool_cache [
            { disk_id: vdf },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_more_than_two_log_devices() {
    assert_eq!(
        PlatformValidationError::ServerZpoolCannotHaveMoreThanTwoLogDisks {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            log_devices_found: 3,
            log_devices_maximum: 2,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', disk_kind: default-ssd },
        { disk_id: 'vdc', disk_kind: default-ssd },
        { disk_id: 'vdd', disk_kind: default-ssd },
        { disk_id: 'vde', disk_kind: default-ssd },
        { disk_id: 'vdf', disk_kind: default-ssd },
        { disk_id: 'vdg', disk_kind: default-ssd },
        { disk_id: 'vdh', disk_kind: default-ssd },
        { disk_id: 'vdi', disk_kind: default-ssd },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdd },
                    { disk_id: vde },
                ]
            },
        ]
        WITH server_zpool_cache [
            { disk_id: vdf },
        ]
        WITH server_zpool_log [
            { disk_id: vdg },
            { disk_id: vdh },
            { disk_id: vdi },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_different_size_log_devices() {
    assert_eq!(
        PlatformValidationError::ServerZpoolLogDifferentDiskSizesDetected {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            different_disk_kinds_involved: vec![
                "default-ssd".to_string(),
                "other-ssd".to_string(),
            ],
            different_disk_sizes: vec![
                21474836480,
                22474836480,
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: other-ssd,
    medium: ssd,
    is_elastic: false,
    capacity_bytes: 22474836480,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', disk_kind: default-ssd },
        { disk_id: 'vdc', disk_kind: default-ssd },
        { disk_id: 'vdd', disk_kind: default-ssd },
        { disk_id: 'vde', disk_kind: default-ssd },
        { disk_id: 'vdf', disk_kind: default-ssd },
        { disk_id: 'vdg', disk_kind: default-ssd },
        { disk_id: 'vdh', disk_kind: other-ssd },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdd },
                    { disk_id: vde },
                ]
            },
        ]
        WITH server_zpool_cache [
            { disk_id: vdf },
        ]
        WITH server_zpool_log [
            { disk_id: vdg },
            { disk_id: vdh },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_zpool_different_disk_mediums() {
    assert_eq!(
        PlatformValidationError::ServerZpoolLogDifferentDiskMediumsDetected {
            server: "server-a".to_string(),
            zpool_name: "pool2".to_string(),
            different_disk_kinds_involved: vec![
                "default-ssd".to_string(),
                "other-hdd".to_string(),
            ],
            different_disk_mediums: vec![
                "hdd".to_string(),
                "ssd".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: other-hdd,
    medium: hdd,
    is_elastic: false,
    capacity_bytes: 21474836480,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'vdb', disk_kind: default-ssd },
        { disk_id: 'vdc', disk_kind: default-ssd },
        { disk_id: 'vdd', disk_kind: default-ssd },
        { disk_id: 'vde', disk_kind: default-ssd },
        { disk_id: 'vdf', disk_kind: default-ssd },
        { disk_id: 'vdg', disk_kind: default-ssd },
        { disk_id: 'vdh', disk_kind: other-hdd },
    ]
    WITH server_zpool {
        zpool_name: pool2,
        WITH server_zpool_vdev [
            {
                vdev_number: 1,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdb },
                    { disk_id: vdc },
                ]
            },
            {
                vdev_number: 2,
                vdev_type: mirror,
                WITH server_zpool_vdev_disk [
                    { disk_id: vdd },
                    { disk_id: vde },
                ]
            },
        ]
        WITH server_zpool_cache [
            { disk_id: vdf },
        ]
        WITH server_zpool_log [
            { disk_id: vdg },
            { disk_id: vdh },
        ]
    }
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_nvme_disk_is_not_nvme() {
    assert_eq!(
        PlatformValidationError::ServerNvmeNamedDiskIsNotNvmeMediumByDiskKind {
            server: "server-a".to_string(),
            disk_id: "nvme0n0".to_string(),
            disk_kind: "default-ssd".to_string(),
            disk_kind_medium: "ssd".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: another-hdd,
    medium: hdd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
        { disk_id: 'nvme0n0', disk_kind: default-ssd },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_xfs_volume_not_on_xfs_disk() {
    assert_eq!(
        PlatformValidationError::ServerXfsVolumeIsNotOnXfsFormattedDisk {
            server: "server-a".to_string(),
            xfs_disk_id: "vda".to_string(),
            xfs_volume_name: "some-xfs".to_string(),
            is_xfs_disk: false,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda' },
    ]
    WITH server_xfs_volume [
        { volume_name: some-xfs, xfs_disk: vda }
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_disk_kind_is_elastic_but_has_specified_capacity() {
    assert_eq!(
        PlatformValidationError::DiskKindDiskIsElasticButHasSpecifiedCapacity {
            disk_kind: "bad-kind".to_string(),
            specified_capacity_bytes: 1073741824,
            is_elastic: true,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: bad-kind,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: 1073741824,
}
"#,
    ));
}

#[test]
fn test_disk_kind_is_elastic_doesnt_specify_max_capacity() {
    assert_eq!(
        PlatformValidationError::DiskKindDiskIsElasticButDoesntSpecifyMaxCapacity {
            disk_kind: "bad-kind".to_string(),
            max_capacity_bytes: -1,
            is_elastic: true,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: bad-kind,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
}
"#,
    ));
}

#[test]
fn test_disk_kind_is_not_elastic_but_has_not_specified_capacity() {
    assert_eq!(
        PlatformValidationError::DiskKindDiskIsNotElasticButHasUnspecifiedCapacity {
            disk_kind: "bad-kind".to_string(),
            specified_capacity_bytes: -1,
            is_elastic: false,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: bad-kind,
    medium: ssd,
    is_elastic: false,
    capacity_bytes: -1,
}
"#,
    ));
}

#[test]
fn test_disk_kind_is_not_elastic_but_has_max_capacity_specified() {
    assert_eq!(
        PlatformValidationError::DiskKindDiskIsNotElasticButHasMaxCapacitySpecified {
            disk_kind: "bad-kind".to_string(),
            specified_max_capacity_bytes: 100,
            is_elastic: false,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: bad-kind,
    medium: ssd,
    is_elastic: false,
    capacity_bytes: 2147483648,
    max_capacity_bytes: 100,
}
"#,
    ));
}

#[test]
fn test_disk_kind_capacity_for_not_elastic_disk_must_be_at_least_1gb() {
    assert_eq!(
        PlatformValidationError::DiskKindDiskCapacityMustBeAtLeast1GB {
            disk_kind: "bad-kind".to_string(),
            specified_capacity_bytes: 100,
            minimum_capacity_bytes: 1024 * 1024 * 1024,
            is_elastic: false,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: bad-kind,
    medium: ssd,
    is_elastic: false,
    capacity_bytes: 100,
}
"#,
    ));
}

#[test]
fn test_server_disk_elastic_disk_has_unspecified_size_in_bytes() {
    assert_eq!(
        PlatformValidationError::ServerDiskKindIsElasticButDiskSizeIsNotSpecified {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            disk_kind: "elastic-ssd".to_string(),
            is_elastic: true,
            specified_capacity_bytes: -1,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: elastic-ssd,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
    max_capacity_bytes: 107374182400,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda', disk_kind: elastic-ssd },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_server_disk_elastic_disk_is_too_small() {
    assert_eq!(
        PlatformValidationError::ServerDiskMinimumCapacityIs1GB {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            disk_kind: "elastic-ssd".to_string(),
            is_elastic: true,
            specified_capacity_bytes: 100,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: elastic-ssd,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
    max_capacity_bytes: 107374182400,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda', disk_kind: elastic-ssd, capacity_bytes: 100 },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_server_disk_root_disk_must_be_at_least_5gb() {
    assert_eq!(
        PlatformValidationError::ServerDiskRootDiskMustBeAtLeast5GB {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            disk_kind: "elastic-ssd".to_string(),
            is_elastic: true,
            specified_capacity_bytes: 2147483648,
            minimum_root_disk_capacity_bytes: 5 * 1024 * 1024 * 1024,
            is_root_disk: true,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: elastic-ssd,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
    max_capacity_bytes: 107374182400,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda', disk_kind: elastic-ssd, capacity_bytes: 2147483648 },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_server_disk_is_not_elastic_but_has_speficied_capacity() {
    assert_eq!(
        PlatformValidationError::ServerDiskKindIsNotElasticButDiskSizeIsSpecified {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            disk_kind: "static-ssd".to_string(),
            is_elastic: false,
            specified_capacity_bytes: 2147483648,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: static-ssd,
    medium: ssd,
    is_elastic: false,
    capacity_bytes: 107374182400,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda', disk_kind: static-ssd, capacity_bytes: 2147483648 },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_server_elastic_disk_exceeds_maximum_capacity() {
    assert_eq!(
        PlatformValidationError::ServerDiskExceedsMaximumCapacityAllowedByDiskKind {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            disk_kind: "elastic-ssd".to_string(),
            is_elastic: true,
            specified_capacity_bytes: 21474836480,
            maximum_disk_capacity_bytes: 10737418240,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: elastic-ssd,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
    max_capacity_bytes: 10737418240,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda', disk_kind: elastic-ssd, capacity_bytes: 21474836480 },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_disk_kind_is_not_elastic_but_has_min_capacity_specified() {
    assert_eq!(
        PlatformValidationError::DiskKindDiskIsNotElasticButHasMinCapacitySpecified {
            disk_kind: "bad-kind".to_string(),
            specified_min_capacity_bytes: 100,
            is_elastic: false,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: bad-kind,
    medium: ssd,
    is_elastic: false,
    capacity_bytes: 2147483648,
    min_capacity_bytes: 100,
}
"#,
    ));
}

#[test]
fn test_disk_kind_min_capacity_more_than_max_capacity() {
    assert_eq!(
        PlatformValidationError::DiskKindMinCapacityBiggerThanMaxCapacity {
            disk_kind: "bad-kind".to_string(),
            is_elastic: true,
            specified_max_capacity_bytes: 1073741824,
            specified_min_capacity_bytes: 1073741825,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: bad-kind,
    medium: ssd,
    is_elastic: true,
    min_capacity_bytes: 1073741825,
    max_capacity_bytes: 1073741824,
}
"#,
    ));
}

#[test]
fn test_server_disk_has_less_than_minimum_capacity() {
    assert_eq!(
        PlatformValidationError::ServerDiskHasLowerThanMinimumCapacityAllowedByDiskKind {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            disk_kind: "elastic-ssd".to_string(),
            is_elastic: true,
            specified_capacity_bytes: 5368709120,
            minimum_disk_capacity_bytes: 10737418240,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: elastic-ssd,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
    max_capacity_bytes: 10737418241,
    min_capacity_bytes: 10737418240,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda', disk_kind: elastic-ssd, capacity_bytes: 5368709120 },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_server_disk_has_extra_config_when_none_allowed() {
    assert_eq!(
        PlatformValidationError::ServerDiskHasExtraConfigNotAllowedForDiskKind {
            server: "server-a".to_string(),
            disk_id: "vda".to_string(),
            disk_kind: "elastic-ssd".to_string(),
            extra_config: "salookie".to_string(),
            is_extra_config_allowed: false,
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT disk_kind {
    kind: elastic-ssd,
    medium: ssd,
    is_elastic: true,
    capacity_bytes: -1,
    max_capacity_bytes: 10737418241,
    has_extra_config: false,
}

DATA STRUCT server {
    hostname: server-a,
    dc: dc1,
    is_vault_instance: true,
    ssh_interface: eth0
    WITH server_disk [
        { disk_id: 'vda', disk_kind: elastic-ssd, capacity_bytes: 5368709120, extra_config: 'salookie' },
    ]
    WITH network_interface {
        if_name: eth0,
        if_network: internet,
        if_ip: 123.123.123.123,
        if_prefix: 24,
    }
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_server_disk_unsupported() {
    assert_eq!(
        PlatformValidationError::ServerDiskWithDiskKindIsNotSupportedForProvisioning {
            disk_id: "vda".to_string(),
            disk_kind: "gcloud.hyperdisk-extreme".to_string(),
            non_eligible_reason: "google cloud faggots only allow attaching this disk to very large machines and in the spirit of eden platform buying big machines in cloud is a waste of time and money anyway".to_string(),
            server: "server-a".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk [
      {
        disk_id: sda,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 21474836480,
      },
      {
        disk_id: vda,
        disk_kind: gcloud.hyperdisk-extreme,
        capacity_bytes: 21474836480,
      },
    ]
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: gcloud-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.11,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
    ]
  },
]

"#,
        ),
    );
}

#[test]
fn test_server_disk_no_serial_assigned() {
    assert_eq!(
        PlatformValidationError::ServerDiskIdUnexpectedFormatInDatacenter {
            disk_id: "sda".to_string(),
            actual_disk_id_kind: "require_devname".to_string(),
            server: "server-a".to_string(),
            datacenter: "bm-1".to_string(),
            datacenter_disk_ids_policy: "require_serial".to_string(),
            datacenter_implementation: "bm_simple".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.17.2/24';
}

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: bm-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: bm_simple,
        implementation_settings: '
          gateway_ip: 10.18.0.1
        ',
    },
]

DATA STRUCT server [
  {
    dc: bm-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk [
      {
        disk_id: sda,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 21474836480,
      },
      {
        disk_id: vda,
        disk_kind: gcloud.hyperdisk-extreme,
        capacity_bytes: 21474836480,
      },
    ]
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: bm-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.11,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
    ]
  },
]

"#,
        ),
    );
}

#[test]
fn test_server_disk_duplicate_serials() {
    assert_eq!(
        PlatformValidationError::DetectedDuplicateDiskSerials {
            server_a: "server-a".to_string(),
            server_a_disk_id: "mcdookie123".to_string(),
            server_b: "server-b".to_string(),
            server_b_disk_id: "mcdookie123".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.17.2/24';
}

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: bm-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: bm_simple,
        implementation_settings: '
          gateway_ip: 10.18.0.1
        ',
    },
]

DATA STRUCT server [
  {
    dc: bm-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: mcdookie123,
    WITH server_disk [
      {
        disk_id: mcdookie123,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 21474836480,
      },
    ]
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: bm-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: mcdookie123,
    WITH server_disk {
      disk_id: mcdookie123,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.11,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
    ]
  },
]

"#,
        ),
    );
}
