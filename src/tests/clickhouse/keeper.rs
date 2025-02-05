#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_ch_keeper_monitoring_cluster_diff_region() {
    assert_eq!(
        PlatformValidationError::ChKeeperDeploymentMonitoringClusterDoesntExistInRegion {
            available_monitoring_clusters: vec!["default-mon".to_string()],
            ch_keeper_deployment: "test-chk".to_string(),
            ch_keeper_region: "us-west".to_string(),
            not_found_monitoring_cluster: "none".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(r#"
DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  monitoring_cluster: none,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  monitoring_cluster: none,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: tempo, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

"#,
    ));
}

#[test]
fn test_ch_keeper_logging_cluster_diff_region() {
    assert_eq!(
        PlatformValidationError::ChKeeperDeploymentLoggingClusterDoesntExistInRegion {
            available_loki_clusters: vec!["default-log".to_string()],
            ch_keeper_deployment: "test-chk".to_string(),
            ch_keeper_region: "us-west".to_string(),
            not_found_loki_cluster: "none".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(r#"
DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  loki_cluster: none,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  monitoring_cluster: none,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: tempo, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

"#,
    ));
}

#[test]
fn test_ch_keeper_must_be_3_or_5() {
    assert_eq!(
        PlatformValidationError::ChKeeperClusterInstancesCountMustBeThreeOrFive {
            ch_keeper_deployment: "test-chk".to_string(),
            ch_keeper_region: "us-west".to_string(),
            ch_keeper_instance_count: 0,
        },
        common::assert_platform_validation_error_wcustom_data(r#"
DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  monitoring_cluster: none,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: tempo, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

"#,
    ));
}

#[test]
fn test_ch_keeper_instance_outside_region() {
    assert_eq!(
        PlatformValidationError::ChKeeperDeploymentInstanceIsOutsideSpecifiedRegion {
            ch_keeper_deployment: "test-chk".to_string(),
            ch_keeper_region: "us-west".to_string(),
            server: "server-e".to_string(),
            server_region: "us-east".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
        r#"
DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-e=>chk },
  ]
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
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

// additional to us-west
DATA region {
  us-east;
}

// additional to dc2
DATA STRUCT datacenter {
  dc_name: dc2,
  network_cidr: '10.18.0.0/16',
  region: us-east,
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
  '10.18.0.2/24';
}

// dc1
DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface(if_name, if_network, if_ip, if_prefix) {
    eth0, lan, 10.17.0.10, 24;
    eth1, internet, 77.77.77.10, 24;
    wg0, vpn, 172.21.7.10, 16;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
    chk, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface(if_name, if_network, if_ip, if_prefix) {
    eth0, lan, 10.17.0.11, 24;
    eth1, internet, 77.77.77.11, 24;
    wg0, vpn, 172.21.7.11, 16;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
    chk, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
  };
}

// dc2
DATA server(hostname, dc, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-e, dc2, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface(if_name, if_network, if_ip, if_prefix) {
    eth0, lan, 10.18.0.10, 24;
    eth1, internet, 77.77.77.12, 24;
    wg0, vpn, 172.21.7.12, 16;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
    chk, exclusive, 4k;
  };
  server-f, dc2, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface(if_name, if_network, if_ip, if_prefix) {
    eth0, lan, 10.18.0.11, 24;
    eth1, internet, 77.77.77.13, 24;
    wg0, vpn, 172.21.7.13, 16;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
  };
  server-h, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: 'us-east=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT loki_cluster {
  region: us-east,
  cluster_name: r2-log,
  storage_bucket: us-east=>logging,
}

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
}

DATA STRUCT monitoring_cluster {
  region: us-east,
  cluster_name: r2-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-e=>mon },
    { instance_id: 2, monitoring_server: server-f=>mon },
    { instance_id: 3, monitoring_server: server-g=>mon },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r2-tempo,
  storage_bucket: us-east=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: tempo, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

DATA STRUCT minio_cluster {
  cluster_name: us-east,
  region: us-east,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-e=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-f=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-g=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-h=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: tempo, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

"#,
    ));
}
