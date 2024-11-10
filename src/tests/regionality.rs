#[cfg(test)]
use crate::static_analysis::PlatformValidationError;
#[cfg(test)]
use crate::tests::common::TestArgs;

#[cfg(test)]
use super::common;

#[test]
fn test_loki_cluster_exists_per_region() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
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
    );

    assert_eq!(
        err,
        PlatformValidationError::NoLoggingClusterInsideRegion {
            region_name: "us-west".to_string(),
        }
    );
}

#[test]
fn test_loki_cluster_unspecified_in_region() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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

DATA STRUCT loki_cluster [
  {
    cluster_name: default-log,
    storage_bucket: us-west=>logging,
  },
  {
    cluster_name: other-log,
    storage_bucket: us-west=>logging2,
  },
]

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
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
    { bucket_name: logging2, },
  ]
}

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::NoRegionDefaultLoggingClusterSpecified {
            region_name: "us-west".to_string(),
            region_clusters: vec!["default-log".to_string(), "other-log".to_string()],
            region_default_clusters: Vec::new(),
        }
    );
}

#[test]
fn test_loki_cluster_more_than_one_default_in_region() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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

DATA STRUCT loki_cluster [
  {
    cluster_name: default-log,
    is_region_default: true,
    storage_bucket: us-west=>logging,
  },
  {
    cluster_name: other-log,
    is_region_default: true,
    storage_bucket: us-west=>logging2,
  },
]

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
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
    { bucket_name: logging2, },
  ]
}

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::MoreThanOneDefaultLoggingClusterFoundInRegion {
            region_name: "us-west".to_string(),
            region_default_clusters: vec!["default-log".to_string(), "other-log".to_string()],
        }
    );
}

#[test]
fn test_loki_cluster_more_than_one_per_region_success() {
    let _ = common::assert_platform_validation_success(
        r#"
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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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

DATA STRUCT loki_cluster [
  {
    cluster_name: default-log,
    is_region_default: true,
    storage_bucket: us-west=>logging,
  },
  {
    cluster_name: other-log,
    is_region_default: false,
    loki_writer_http_port: 3020,
    storage_bucket: us-west=>logging2,
  },
]

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
    { bucket_name: logging2, },
  ]
}

"#,
    );
}

#[test]
fn test_app_using_more_than_one_time_success() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT backend_application {
  application_name: hello-world,
  WITH backend_application_s3_bucket {
    bucket_name: some,
  }
}

DATA STRUCT backend_application_deployment {
  deployment_name: hello-deployment-a,
  application_name: hello-world,
  s3_bucket_wiring: '
    some: us-west=>app
  ',
}

DATA STRUCT backend_application_deployment {
  deployment_name: hello-deployment-b,
  application_name: hello-world,
  http_port: 7357,
  s3_bucket_wiring: '
    some: us-west=>app
  ',
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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

DATA STRUCT loki_cluster [
  {
    cluster_name: default-log,
    is_region_default: true,
    storage_bucket: us-west=>logging,
  },
  {
    cluster_name: other-log,
    is_region_default: false,
    loki_writer_http_port: 3020,
    storage_bucket: us-west=>logging2,
  },
]

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
    { bucket_name: logging2, },
    { bucket_name: app, },
  ]
}
"#,
    );
}

#[test]
fn test_monitoring_cluster_exists_per_region() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::NoMonitoringClusterInsideRegion {
            region_name: "us-west".to_string(),
        }
    );
}

#[test]
fn test_monitoring_cluster_unspecified_in_region() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default-mon,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon },
      { instance_id: 2, monitoring_server: server-b=>mon },
      { instance_id: 3, monitoring_server: server-c=>mon },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am },
      { instance_id: 2, alertmanager_server: server-b=>am },
      { instance_id: 3, alertmanager_server: server-c=>am },
    ]
  },
  {
    cluster_name: other-mon,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon2 },
      { instance_id: 2, monitoring_server: server-b=>mon2 },
      { instance_id: 3, monitoring_server: server-c=>mon2 },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am2 },
      { instance_id: 2, alertmanager_server: server-b=>am2 },
      { instance_id: 3, alertmanager_server: server-c=>am2 },
    ]
  },
]

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
    );

    assert_eq!(
        err,
        PlatformValidationError::NoRegionDefaultMonitoringClusterSpecified {
            region_name: "us-west".to_string(),
            region_clusters: vec!["default-mon".to_string(), "other-mon".to_string()],
            region_default_clusters: Vec::new(),
        }
    );
}

#[test]
fn test_monitoring_cluster_more_than_one_default_in_region() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default-mon,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon },
      { instance_id: 2, monitoring_server: server-b=>mon },
      { instance_id: 3, monitoring_server: server-c=>mon },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am },
      { instance_id: 2, alertmanager_server: server-b=>am },
      { instance_id: 3, alertmanager_server: server-c=>am },
    ]
  },
  {
    cluster_name: other-mon,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon2 },
      { instance_id: 2, monitoring_server: server-b=>mon2 },
      { instance_id: 3, monitoring_server: server-c=>mon2 },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am2 },
      { instance_id: 2, alertmanager_server: server-b=>am2 },
      { instance_id: 3, alertmanager_server: server-c=>am2 },
    ]
  },
]

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
    );

    assert_eq!(
        err,
        PlatformValidationError::MoreThanOneDefaultMonitoringClusterFoundInRegion {
            region_name: "us-west".to_string(),
            region_default_clusters: vec!["default-mon".to_string(), "other-mon".to_string()],
        }
    );
}

#[test]
fn test_monitoring_cluster_more_than_one_per_region_success() {
    let _ = common::assert_platform_validation_success(
        r#"
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
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    mon2, exclusive, 4k;
    am2, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default-mon,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon },
      { instance_id: 2, monitoring_server: server-b=>mon },
      { instance_id: 3, monitoring_server: server-c=>mon },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am },
      { instance_id: 2, alertmanager_server: server-b=>am },
      { instance_id: 3, alertmanager_server: server-c=>am },
    ]
  },
  {
    cluster_name: other-mon,
    is_region_default: false,
    prometheus_port: 9094,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon2 },
      { instance_id: 2, monitoring_server: server-b=>mon2 },
      { instance_id: 3, monitoring_server: server-c=>mon2 },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am2 },
      { instance_id: 2, alertmanager_server: server-b=>am2 },
      { instance_id: 3, alertmanager_server: server-c=>am2 },
    ]
  },
]

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
    );
}

#[test]
fn test_backend_app_no_loki_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  loki_cluster: none,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
  }
]

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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::ApplicationLoggingClusterDoesntExistInRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-west".to_string(),
            available_loki_clusters: vec!["default-log".to_string()],
            not_found_loki_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_backend_app_no_tracing_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  tracing_cluster: none,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
  }
]

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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::ApplicationTracingClusterDoesntExistInRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-west".to_string(),
            available_tempo_clusters: vec!["r1-tempo".to_string()],
            not_found_tempo_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_backend_app_no_monitoring_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  monitoring_cluster: none,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
  }
]

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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::ApplicationMonitoringClusterDoesntExistInRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-west".to_string(),
            available_monitoring_clusters: vec!["default-mon".to_string()],
            not_found_monitoring_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_frontend_app_no_loki_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  loki_cluster: none,
  WITH frontend_application_deployment_ingress {
    tld: epl-infra.net,
  }
}

DATA STRUCT frontend_application [
  {
    application_name: hello-world,
    WITH frontend_page {
      page_name: home,
      path: '/',
    }
  }
]

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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLoggingClusterDoesntExistInRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-west".to_string(),
            available_loki_clusters: vec!["default-log".to_string()],
            not_found_loki_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_minio_no_monitoring_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  monitoring_cluster: none,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MinIOMonitoringClusterDoesntExistInRegion {
            available_monitoring_clusters: vec!["default-mon".to_string()],
            minio_cluster: "us-west".to_string(),
            minio_region: "us-west".to_string(),
            not_found_monitoring_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_minio_no_loki_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  loki_cluster: none,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MinIOLoggingClusterDoesntExistInRegion {
            available_loki_clusters: vec!["default-log".to_string()],
            minio_cluster: "us-west".to_string(),
            minio_region: "us-west".to_string(),
            not_found_loki_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_nats_no_monitoring_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT nats_cluster {
  cluster_name: test-nats,
  monitoring_cluster: none,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats, },
    { instance_id: 2, nats_server: server-b=>nats, },
    { instance_id: 3, nats_server: server-c=>nats, },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::NatsMonitoringClusterDoesntExistInRegion {
            available_monitoring_clusters: vec!["default-mon".to_string()],
            nats_cluster: "test-nats".to_string(),
            nats_region: "us-west".to_string(),
            not_found_monitoring_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_nats_no_loki_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT nats_cluster {
  cluster_name: test-nats,
  loki_cluster: none,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats, },
    { instance_id: 2, nats_server: server-b=>nats, },
    { instance_id: 3, nats_server: server-c=>nats, },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::NatsLoggingClusterDoesntExistInRegion {
            available_loki_clusters: vec!["default-log".to_string()],
            nats_cluster: "test-nats".to_string(),
            nats_region: "us-west".to_string(),
            not_found_loki_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_nats_no_quorum() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT nats_cluster {
  cluster_name: test-nats,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats, },
    { instance_id: 2, nats_server: server-b=>nats, },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::NatsClusterInstancesCountMustBeThreeOrFive {
            nats_cluster: "test-nats".to_string(),
            nats_region: "us-west".to_string(),
            nats_instance_count: 2,
        }
    );
}

#[test]
fn test_loki_no_monitoring_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT nats_cluster {
  cluster_name: test-nats,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats, },
    { instance_id: 2, nats_server: server-b=>nats, },
    { instance_id: 3, nats_server: server-c=>nats, },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::LokiMonitoringClusterDoesntExistInRegion {
            available_monitoring_clusters: vec!["default-mon".to_string()],
            loki_cluster: "default-log".to_string(),
            loki_region: "us-west".to_string(),
            not_found_monitoring_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_loki_no_loki_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
  loki_cluster: none,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT nats_cluster {
  cluster_name: test-nats,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats, },
    { instance_id: 2, nats_server: server-b=>nats, },
    { instance_id: 3, nats_server: server-c=>nats, },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::LokiLoggingClusterDoesntExistInRegion {
            available_loki_clusters: vec!["default-log".to_string()],
            loki_cluster: "default-log".to_string(),
            loki_region: "us-west".to_string(),
            not_found_loki_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_postgres_no_monitoring_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  monitoring_cluster: none,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-a=>pg },
    { instance_id: 2, pg_server: server-b=>pg },
  ]
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
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDeploymentMonitoringClusterDoesntExistInRegion {
            available_monitoring_clusters: vec!["default-mon".to_string()],
            pg_deployment: "test-pg".to_string(),
            db_region: "us-west".to_string(),
            not_found_monitoring_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_postgres_no_loki_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  loki_cluster: none,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-a=>pg },
    { instance_id: 2, pg_server: server-b=>pg },
  ]
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
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDeploymentLoggingClusterDoesntExistInRegion {
            available_loki_clusters: vec!["default-log".to_string()],
            pg_deployment: "test-pg".to_string(),
            db_region: "us-west".to_string(),
            not_found_loki_cluster: "none".to_string(),
        }
    );
}


#[test]
fn test_grafana_no_monitoring_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-a=>pg },
    { instance_id: 2, pg_server: server-b=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  monitoring_cluster: none,
  database: test-pg=>grafana,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::GrafanaMonitoringClusterDoesntExistInRegion {
            available_monitoring_clusters: vec!["default-mon".to_string()],
            grafana_deployment: "test-grafana".to_string(),
            grafana_region: "us-west".to_string(),
            not_found_monitoring_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_grafana_no_loki_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-a=>pg },
    { instance_id: 2, pg_server: server-b=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  loki_cluster: none,
  database: test-pg=>grafana,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::GrafanaLoggingClusterDoesntExistInRegion {
            available_loki_clusters: vec!["default-log".to_string()],
            grafana_deployment: "test-grafana".to_string(),
            grafana_region: "us-west".to_string(),
            not_found_loki_cluster: "none".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_db_instance() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-a=>pg },
    { instance_id: 2, pg_server: server-e=>pg },
  ]
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
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDeploymentInstanceIsOutsideSpecifiedRegion {
            pg_deployment: "test-pg".to_string(),
            db_region: "us-west".to_string(),
            server: "server-e".to_string(),
            server_region: "us-east".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_nats_instance() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
  ]
}

DATA STRUCT nats_cluster {
  cluster_name: nats-default,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats },
    { instance_id: 2, nats_server: server-b=>nats },
    { instance_id: 3, nats_server: server-e=>nats },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::NatsClusterInstanceIsOutsideSpecifiedRegion {
            nats_cluster: "nats-default".to_string(),
            nats_cluster_region: "us-west".to_string(),
            server: "server-e".to_string(),
            server_region: "us-east".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_tempo_instance() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
}

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-east=>tempo2,
}

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r2-tempo,
  storage_bucket: us-east=>tempo,
  http_port: 4320,
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
    { bucket_name: tempo2, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::TempoClusterMinIOBucketIsOutsideSpecifiedRegion {
            tempo_cluster: "r1-tempo".to_string(),
            tempo_cluster_region: "us-west".to_string(),
            minio_cluster: "us-east".to_string(),
            minio_cluster_region: "us-east".to_string(),
        }
    );
}

#[test]
fn test_tempo_disallow_cross_region_monitoring_instance() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  monitoring_cluster: r2-mon,
  storage_bucket: us-east=>logging,
  loki_writer_http_port: 3020,
}

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  monitoring_cluster: r2-mon,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r2-tempo,
  storage_bucket: us-east=>tempo,
  http_port: 4320,
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
    { bucket_name: tempo2, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::TempoMonitoringClusterDoesntExistInRegion {
            tempo_cluster: "r1-tempo".to_string(),
            tempo_region: "us-west".to_string(),
            not_found_monitoring_cluster: "r2-mon".to_string(),
            available_monitoring_clusters: vec!["default-mon".to_string()],
        }
    );
}

#[test]
fn test_tempo_disallow_cross_region_logging_instance() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  monitoring_cluster: r2-mon,
  storage_bucket: us-east=>logging,
  loki_writer_http_port: 3020,
}

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  loki_cluster: r2-log,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r2-tempo,
  storage_bucket: us-east=>tempo,
  http_port: 4320,
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
    { bucket_name: tempo2, },
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::TempoLoggingClusterDoesntExistInRegion {
            tempo_cluster: "r1-tempo".to_string(),
            tempo_region: "us-west".to_string(),
            not_found_loki_cluster: "r2-log".to_string(),
            available_loki_clusters: vec!["default-log".to_string()],
        }
    );
}

#[test]
fn test_disallow_cross_region_monitoring_instance() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT monitoring_cluster {
  region: us-east,
  cluster_name: r2-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-e=>mon },
    { instance_id: 2, monitoring_server: server-f=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-d=>am },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MonitoringInstanceIsOutsideSpecifiedRegion {
            monitoring_cluster: "r2-mon".to_string(),
            monitoring_cluster_region: "us-east".to_string(),
            server: "server-d".to_string(),
            server_region: "us-west".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_logging_minio_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
  storage_bucket: us-west=>logging2,
}

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
    { bucket_name: logging2, },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::LokiClusterMinIOBucketIsOutsideSpecifiedRegion {
            loki_cluster: "r2-log".to_string(),
            loki_cluster_region: "us-east".to_string(),
            minio_cluster: "us-west".to_string(),
            minio_cluster_region: "us-west".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_docker_registry_and_minio_cluster() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface(if_name, if_network, if_ip, if_prefix) {
    eth0, lan, 10.17.0.12, 24;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  minio_bucket: 'us-west=>docker2',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT loki_cluster {
  region: us-east,
  cluster_name: r2-log,
  loki_writer_http_port: 3020,
  storage_bucket: us-east=>logging,
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

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
  ]
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
    { bucket_name: docker2, },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::DockerRegistryAndMinioClusterAreInDifferentRegions {
            docker_registry_region: "us-east".to_string(),
            minio_cluster: "us-west".to_string(),
            minio_cluster_region: "us-west".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_grafana_and_db() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_data: true,
            add_default_global_flags: false,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-west,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
    );

    assert_eq!(
        err,
        PlatformValidationError::GrafanaDatabaseIsOutsideSpecifiedRegion {
            grafana_deployment: "test-grafana".to_string(),
            grafana_region: "us-west".to_string(),
            pg_deployment: "test-pg".to_string(),
            pg_deployment_region: "us-east".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_application_pg_access() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  pg_shard_wiring: '
    a : test-pg=>testdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: ''
        },
    ]
  }
]

DATA STRUCT pg_schema [
  {
    schema_name: testdb,
    WITH pg_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE foo (
            id INT PRIMARY KEY
          );
        ",
        downgrade: "DROP TABLE foo;",
      }
    ]
    WITH pg_test_dataset [
      {
        dataset_name: default,
        dataset_contents: "
        foo:
        - id: 1
        - id: 2
        - id: 3
        "
      }
    ]
    WITH pg_query [
      {
        query_name: existing_query_a,
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE <test_arg:INT> > 0",
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 3",
          outputs: "
          - max_id: 3
          "
        }
      },
    ]
  }
]

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_schemas [
    { db_name: testdb_a, pg_schema: testdb, },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-east,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringDatabaseIsInDifferentRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-west".to_string(),
            application_db_name: "a".to_string(),
            application_db_wired_deployment: "test-pg".to_string(),
            application_db_wired_region: "us-east".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_application_ch_access() {
    assert_eq!(
        PlatformValidationError::ApplicationChWiringDatabaseIsInDifferentRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-west".to_string(),
            application_db_name: "a".to_string(),
            application_db_wired_deployment: "test-ch".to_string(),
            application_db_wired_region: "us-east".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a : test-ch=>testdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testdb,
            used_queries: ''
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testdb,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS foo;
        ",
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: "
          foo:
          - id: 7
            a: hello
          bar:
          - id: 1
            b: y
        "
      }
    ]
  }
]

DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
  ]
}

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-east,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-e=>ch },
    { instance_id: 2, ch_server: server-f=>ch },
  ]
  WITH ch_deployment_schemas [
    { db_name: testdb_a, ch_schema: testdb, },
  ]
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    chk, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    chk, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    ch, exclusive, 4k;
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
    am, exclusive, 4k;
    ch, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
        ),
    );
}

#[test]
fn test_disallow_cross_region_nats_to_ch_stream() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamAndChDeploymentAreInDifferentRegions {
            ch_deployment: "test-ch".to_string(),
            ch_deployment_region: "us-east".to_string(),
            nats_cluster: "nats-default".to_string(),
            nats_cluster_region: "us-west".to_string(),
            nats_jetstream_stream: "some_test_stream".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}


DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          some_field @0 :I64,
        }"
      },
    ]
  },
]

DATA STRUCT nats_cluster {
  cluster_name: nats-default,
  region: us-west,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats },
    { instance_id: 2, nats_server: server-b=>nats },
    { instance_id: 3, nats_server: server-c=>nats },
  ]
  WITH nats_jetstream_stream [
    { stream_name: some_test_stream, stream_type: test_vtype, },
  ]
}

DATA STRUCT ch_schema [
  {
    schema_name: testdb,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS foo;
        ",
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: "
          foo:
          - id: 7
            a: hello
          bar:
          - id: 1
            b: y
        "
      }
    ]
  }
]

DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
  ]
}

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-east,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-e=>ch },
    { instance_id: 2, ch_server: server-f=>ch },
  ]
  WITH ch_deployment_schemas [
    { db_name: testdb_a, ch_schema: testdb,
      WITH ch_nats_stream_import {
        consumer_name: ch_nats_consumer,
        into_table: foo,
        stream: nats-default=>some_test_stream,
      }
    },
  ]
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
    am, exclusive, 4k;
    chk, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
    chk, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    chk, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    ch, exclusive, 4k;
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
    am, exclusive, 4k;
    ch, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
        ),
    );
}

#[test]
fn test_disallow_cross_region_application_s3_access() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  s3_bucket_wiring: '
    some : us-east=>app
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_s3_bucket [
      { bucket_name: some }
    ]
  }
]

DATA STRUCT pg_schema [
  {
    schema_name: testdb,
    WITH pg_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE foo (
            id INT PRIMARY KEY
          );
        ",
        downgrade: "DROP TABLE foo;",
      }
    ]
    WITH pg_test_dataset [
      {
        dataset_name: default,
        dataset_contents: "
        foo:
        - id: 1
        - id: 2
        - id: 3
        "
      }
    ]
    WITH pg_query [
      {
        query_name: existing_query_a,
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE <test_arg:INT> > 0",
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 3",
          outputs: "
          - max_id: 3
          "
        }
      },
    ]
  }
]

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_schemas [
    { db_name: testdb_a, pg_schema: testdb, },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-east,
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
    am, exclusive, 4k;
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
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
    { bucket_name: app, },
  ]
}

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::ApplicationS3BucketWiringIsInDifferentRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-west".to_string(),
            application_bucket_name: "some".to_string(),
            application_bucket_wired_minio_region: "us-east".to_string(),
            application_bucket_wired_minio_cluster: "us-east".to_string(),
            application_bucket_wired_minio_bucket: "app".to_string(),
        }
    );
}

#[test]
fn test_disallow_cross_region_application_queue_access() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  region: us-east,
  nats_stream_wiring: '
    a : nats-default=>some_test_stream
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_nats_stream [
      { stream_name: a, stream_type: test_vtype, enable_producer: true, },
    ]
  }
]

DATA STRUCT nats_cluster {
  cluster_name: nats-default,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats },
    { instance_id: 2, nats_server: server-b=>nats },
    { instance_id: 3, nats_server: server-c=>nats },
  ]
  WITH nats_jetstream_stream [
    { stream_name: some_test_stream, stream_type: test_vtype, },
  ]
}

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          some_field @0 :I64,
        }"
      },
    ]
  },
]

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r2-tempo,
  storage_bucket: us-east=>tempo,
  http_port: 4320,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-east,
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
  '10.18.0.2/24';
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
  ]
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
    );

    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamWiringNatsClusterIsInDifferentRegion {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            application_region: "us-east".to_string(),
            application_nats_stream_name: "a".to_string(),
            application_nats_wired_cluster: "nats-default".to_string(),
            application_nats_wired_region: "us-west".to_string(),
        }
    );
}

#[test]
fn test_disallow_double_usage_of_bucket_logging() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-east,
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  loki_writer_http_port: 3020,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
            minio_cluster: "us-west".to_string(),
            minio_bucket: "logging".to_string(),
            previous_usage: "loki_cluster, cluster name: default-log".to_string(),
            clashing_usage: "loki_cluster, cluster name: r2-log".to_string(),
        }
    );
}

#[test]
fn test_disallow_double_usage_of_bucket_docker() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-east,
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  minio_bucket: us-east=>docker,
}

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: us-east=>docker,
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT loki_cluster {
  region: us-east,
  cluster_name: r2-log,
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
            minio_cluster: "us-east".to_string(),
            minio_bucket: "docker".to_string(),
            previous_usage: "docker_registry_instance, region: us-west".to_string(),
            clashing_usage: "docker_registry_instance, region: us-east".to_string(),
        }
    );
}

#[test]
fn test_disallow_double_usage_of_bucket_loki_and_docker() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-east,
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  minio_bucket: us-west=>docker,
}

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: us-east=>docker,
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT loki_cluster {
  region: us-east,
  cluster_name: r2-log,
  loki_writer_http_port: 3020,
  storage_bucket: us-east=>docker,
}

DATA STRUCT monitoring_cluster {
  region: us-west,
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
            minio_cluster: "us-east".to_string(),
            minio_bucket: "docker".to_string(),
            previous_usage: "docker_registry_instance, region: us-east".to_string(),
            clashing_usage: "loki_cluster, cluster name: r2-log".to_string(),
        }
    );
}

#[test]
fn test_disallow_double_usage_of_bucket_loki_and_backend_app() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
}

DATA STRUCT pg_deployment {
  docker_image_pg: pg_15.1,
  deployment_name: test-pg,
  region: us-east,
  WITH pg_deployment_instance [
    { instance_id: 1, pg_server: server-e=>pg },
    { instance_id: 2, pg_server: server-f=>pg },
  ]
  WITH pg_deployment_unmanaged_db {
    db_name: grafana
  }
}

DATA STRUCT grafana {
  deployment_name: test-grafana,
  database: test-pg=>grafana,
  region: us-east,
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
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
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
    nats, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
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
    am, exclusive, 4k;
    pg, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-h, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
  };
}

DATA STRUCT backend_application {
  application_name: hello-world,
  WITH backend_application_s3_bucket {
    bucket_name: some,
  }
}

DATA STRUCT backend_application_deployment {
  deployment_name: hello-deployment,
  application_name: hello-world,
  s3_bucket_wiring: '
    some: us-west=>logging
  ',
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: us-west=>docker,
}

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: us-east=>docker,
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT loki_cluster {
  region: us-east,
  cluster_name: r2-log,
  loki_writer_http_port: 3020,
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-g=>am },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
            minio_cluster: "us-west".to_string(),
            minio_bucket: "logging".to_string(),
            previous_usage: "loki_cluster, cluster name: default-log".to_string(),
            clashing_usage: "backend_application_deployment, app s3 bucket: some".to_string(),
        }
    );
}

#[test]
fn test_minio_too_few_dcs() {
    use std::collections::BTreeMap;

    assert_eq!(
        PlatformValidationError::ApplicationInsideMultiDcRegionIsNotDistributedAcrossEnoughDatacenters {
            region: "us-west".to_string(),
            context: "minio_cluster=>global".to_string(),
            region_availability_mode: "multi_dc".to_string(),
            application_servers_buckets: BTreeMap::from([
                ("dc1".to_string(), vec![
                    "server-a".to_string(),
                    "server-b".to_string(),
                ]),
                ("dc2".to_string(), vec![
                    "server-c".to_string(),
                    "server-d".to_string(),
                ]),
            ]),
            found_dcs: 2,
            min_dcs: 3,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_data: false,
                add_default_global_flags: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-project,
    google_cloud_artefacts_bucket_name: henlo-bois,
    aws_artefacts_s3_bucket_name: henlo-bois,
}

DATA STRUCT EXCLUSIVE datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-a
        ',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-b
        ',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
            availability_zone: us-west-2c
        ',
    },
]

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
  loki_cluster.region us-west,
  tempo_cluster.region us-west,
  ch_deployment.region us-west,
  ch_keeper_deployment.region us-west,
  blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
  availability_mode: multi_dc,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.12,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: pgtest1,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.13,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-e,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.14,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-f,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.15,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-g,
    dc: dc1,
    ssh_interface: eth0,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
    ]
  },
  {
    hostname: server-h,
    dc: dc2,
    ssh_interface: eth0,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
        if_prefix: 24,
      },
    ]
  },
  {
    hostname: server-i,
    dc: dc3,
    ssh_interface: eth0,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.12,
        if_prefix: 24,
      },
    ]
  },
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-c=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-e=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am },
      { instance_id: 2, alertmanager_server: server-c=>mon-am },
      { instance_id: 3, alertmanager_server: server-e=>mon-am },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-docker-d,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#,
    ));
}

#[test]
fn test_minio_unequal_distribution_across_dcs() {
    use std::collections::BTreeMap;

    assert_eq!(
        PlatformValidationError::ApplicationInsideMultiDcRegionIsDistributedDistributedNonEqually {
            region: "us-west".to_string(),
            context: "minio_cluster=>global".to_string(),
            region_availability_mode: "multi_dc".to_string(),
            application_servers_buckets: BTreeMap::from([
                ("dc1".to_string(), vec![
                    "server-a".to_string(),
                    "server-b".to_string(),
                    "server-g".to_string(),
                ]),
                ("dc2".to_string(), vec![
                    "server-c".to_string(),
                    "server-h".to_string(),
                ]),
                ("dc3".to_string(), vec![
                    "server-e".to_string(),
                ]),
            ]),
            found_dcs: 3,
            dc_with_lowest_nodes_count: 1,
            dc_with_most_nodes_count: 3,
            difference: 2,
            maximum_allowed_difference: 1,
            dc_with_lowest_nodes: "dc3".to_string(),
            dc_with_most_nodes: "dc1".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_data: false,
                add_default_global_flags: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-project,
    google_cloud_artefacts_bucket_name: henlo-bois,
    aws_artefacts_s3_bucket_name: henlo-bois,
}

DATA STRUCT EXCLUSIVE datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-a
        ',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-b
        ',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
            availability_zone: us-west-2c
        ',
    },
]

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
  loki_cluster.region us-west,
  tempo_cluster.region us-west,
  ch_deployment.region us-west,
  ch_keeper_deployment.region us-west,
  blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
  availability_mode: multi_dc,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.12,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: pgtest1,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.13,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
  {
    hostname: server-e,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.14,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: minio-docker-e,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-f,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.15,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-g,
    dc: dc1,
    ssh_interface: eth0,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: minio-docker-g,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-h,
    dc: dc2,
    ssh_interface: eth0,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: minio-docker-h,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-i,
    dc: dc3,
    ssh_interface: eth0,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.12,
        if_prefix: 24,
      },
    ]
  },
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-c=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-e=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am },
      { instance_id: 2, alertmanager_server: server-c=>mon-am },
      { instance_id: 3, alertmanager_server: server-e=>mon-am },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-e=>minio-docker-e,
    },
    {
      instance_id: 5,
      instance_volume: server-g=>minio-docker-g,
    },
    {
      instance_id: 6,
      instance_volume: server-h=>minio-docker-h,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#,
    ));
}

#[test]
fn test_minio_unequal_distribution_allow_across_dcs() {
    let _res = common::assert_platform_validation_success_wargs(
        common::TestArgs {
            add_default_data: false,
            add_default_global_flags: false,
        },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-project,
    google_cloud_artefacts_bucket_name: henlo-bois,
    aws_artefacts_s3_bucket_name: henlo-bois,
}

DATA STRUCT EXCLUSIVE datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-a
        ',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-b
        ',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
            availability_zone: us-west-2c
        ',
    },
]

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
  loki_cluster.region us-west,
  tempo_cluster.region us-west,
  ch_deployment.region us-west,
  ch_keeper_deployment.region us-west,
  blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
  availability_mode: multi_dc,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.12,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: pgtest1,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.13,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
  {
    hostname: server-e,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.14,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: minio-docker-e,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-f,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.15,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-g,
    dc: dc1,
    ssh_interface: eth0,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: minio-docker-g,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-h,
    dc: dc2,
    ssh_interface: eth0,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: minio-docker-h,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-i,
    dc: dc3,
    ssh_interface: eth0,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.12,
        if_prefix: 24,
      },
    ]
  },
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-c=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-e=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am },
      { instance_id: 2, alertmanager_server: server-c=>mon-am },
      { instance_id: 3, alertmanager_server: server-e=>mon-am },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  distribute_over_dcs: false,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-e=>minio-docker-e,
    },
    {
      instance_id: 5,
      instance_volume: server-g=>minio-docker-g,
    },
    {
      instance_id: 6,
      instance_volume: server-h=>minio-docker-h,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#,
    );
}

#[test]
fn test_disallow_multiple_dcs_without_vpn_network() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
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

// dc1
DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
}

// dc2
DATA server(hostname, dc, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-e, dc2, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-f, dc2, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT monitoring_cluster {
  region: us-east,
  cluster_name: r2-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-e=>mon },
    { instance_id: 2, monitoring_server: server-f=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-d=>am },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::MoreThanOneDatacenterButNoVpnNetworkToConnectThem {
            datacenter_count: 2,
            missing_network: "vpn".to_string(),
        }
    );
}

#[test]
fn test_disallow_multiple_subnets_without_router_network() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  }
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
  allow_small_subnets: true,
}

// dc1
DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
}

// dc2
DATA server(hostname, dc, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-e, dc2, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-f, dc2, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-g, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.1.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-h, dc2, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.18.1.13;
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
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT monitoring_cluster {
  region: us-east,
  cluster_name: r2-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-e=>mon },
    { instance_id: 2, monitoring_server: server-f=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-e=>am },
    { instance_id: 2, alertmanager_server: server-f=>am },
    { instance_id: 3, alertmanager_server: server-d=>am },
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
  http_port: 4320,
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
    );

    assert_eq!(
        err,
        PlatformValidationError::IntraDcRoutingNeededButNoDcrouterNetworkExists {
            dc: "dc2".to_string(),
            subnet_count: 2,
            missing_network: "dcrouter".to_string(),
        }
    );
}

#[test]
fn test_only_one_monitoring_instance() {
    assert_eq!(
        PlatformValidationError::MonitoringInstancesMustAtLeastTwoToThree {
            cluster_name: "default-mon".to_string(),
            minimum_count: 2,
            maximum_count: 3,
            current_count: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
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

DATA STRUCT backend_application {
  application_name: hello-world,
  WITH backend_application_s3_bucket {
    bucket_name: some,
  }
}

DATA STRUCT backend_application_deployment {
  deployment_name: hello-deployment-a,
  application_name: hello-world,
  s3_bucket_wiring: '
    some: us-west=>app
  ',
}

DATA STRUCT backend_application_deployment {
  deployment_name: hello-deployment-b,
  application_name: hello-world,
  http_port: 7357,
  s3_bucket_wiring: '
    some: us-west=>app
  ',
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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

DATA STRUCT loki_cluster [
  {
    cluster_name: default-log,
    is_region_default: true,
    storage_bucket: us-west=>logging,
  },
  {
    cluster_name: other-log,
    is_region_default: false,
    loki_writer_http_port: 3020,
    storage_bucket: us-west=>logging2,
  },
]

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
    { bucket_name: logging2, },
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_too_few_alertmanager_instances() {
    assert_eq!(
        PlatformValidationError::AlertmanagerInstancesMustBeThreeOrFive {
            cluster_name: "default-mon".to_string(),
            valid_counts: vec![3, 5],
            current_count: 2,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
  ]
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

DATA STRUCT backend_application {
  application_name: hello-world,
  WITH backend_application_s3_bucket {
    bucket_name: some,
  }
}

DATA STRUCT backend_application_deployment {
  deployment_name: hello-deployment-a,
  application_name: hello-world,
  s3_bucket_wiring: '
    some: us-west=>app
  ',
}

DATA STRUCT backend_application_deployment {
  deployment_name: hello-deployment-b,
  application_name: hello-world,
  http_port: 7357,
  s3_bucket_wiring: '
    some: us-west=>app
  ',
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
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

DATA STRUCT loki_cluster [
  {
    cluster_name: default-log,
    is_region_default: true,
    storage_bucket: us-west=>logging,
  },
  {
    cluster_name: other-log,
    is_region_default: false,
    loki_writer_http_port: 3020,
    storage_bucket: us-west=>logging2,
  },
]

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
    { bucket_name: logging2, },
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}
