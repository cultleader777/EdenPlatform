#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_non_sequential_ports() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentPortsAreNotSequential {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            deployment_ports: vec![1200, 1202],
            port_a: 1200,
            non_sequential_port_b: 1202,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
    WITH blackbox_deployment_port [
      {
        port: 1200,
        port_description: "port a",
        protocol: tcp,
      },
      {
        port: 1202,
        port_description: "port b",
        protocol: tcp,
      },
    ]
  }
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
  '10.17.0.2/24';
}

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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

"#,
        ),
    )
}

#[test]
fn test_deployment_has_no_groups() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentHasNoGroups {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            groups_count: 0,
            groups_minimum: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
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
  '10.17.0.2/24';
}

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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

"#,
        ),
    )
}

#[test]
fn test_deployment_group_has_no_tasks() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentGroupHasNoTasks {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            tasks_count: 0,
            tasks_minimum: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
  }
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
  '10.17.0.2/24';
}

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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

"#,
        ),
    )
}

#[test]
fn test_deployment_server_mountpoint_is_in_different_region() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentGroupBoundToOtherRegion {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            group_bound_server: "server-e".to_string(),
            group_bound_server_region: "us-east".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
    WITH blackbox_deployment_task {
      task_name: main-node,
      docker_image: geth_stable,
      docker_image_set: geth,
      memory_mb: 32,
      WITH blackbox_deployment_task_mount {
        target_path: '/data',
        server_volume: server-e=>geth,
      }
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}


// OTHER REGION
DATA STRUCT datacenter {
  dc_name: dc2,
  network_cidr: '10.18.0.0/16',
  region: us-east,
}

DATA region {
  us-east;
}

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

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
    geth, exclusive, 4k;
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
  region: us-east,
  minio_bucket: 'us-east=>docker',
}

DATA STRUCT loki_cluster {
  region: us-east,
  cluster_name: r2-log,
  loki_writer_http_port: 3020,
  storage_bucket: us-east=>logging,
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
  region: us-east,
  cluster_name: r2-tempo,
  storage_bucket: us-east=>tempo,
  http_port: 4320,
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
    )
}

#[test]
fn test_deployment_group_task_mountpoint_is_in_multiple_servers() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentGroupBoundToMultipleServers {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            group_bound_server_a: "server-a".to_string(),
            group_bound_server_b: "server-b".to_string(),
            maximum_servers: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
    WITH blackbox_deployment_task {
      task_name: main-node,
      docker_image: geth_stable,
      docker_image_set: geth,
      memory_mb: 32,
      WITH blackbox_deployment_task_mount [
        {
          target_path: '/data-a',
          server_volume: server-a=>geth,
        },
        {
          target_path: '/data-b',
          server_volume: server-b=>geth,
        },
      ]
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
    geth, exclusive, 4k;
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
    geth, exclusive, 4k;
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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}
"#,
        ),
    )
}

#[test]
fn test_deployment_stateful_group_has_multiple_counts() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentStatefulGroupMustHaveCountOfOne {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            current_count: 2,
            expected_count: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
    count: 2,
    WITH blackbox_deployment_task {
      task_name: main-node,
      memory_mb: 32,
      docker_image: geth_stable,
      docker_image_set: geth,
      WITH blackbox_deployment_task_mount [
        {
          target_path: '/data-a',
          server_volume: server-a=>geth,
        },
      ]
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
    geth, exclusive, 4k;
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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}
"#,
        ),
    )
}

#[test]
fn test_deployment_no_service_instances() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentServiceRegistrationHasNotEnoughInstances {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            service_name: "epl-bb-prom-scrape".to_string(),
            current_service_instances: 0,
            min_service_instances: 2,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_service_registration {
    service_name: epl-bb-prom-scrape,
    scrape_prometheus_metrics: true,
  }
  WITH blackbox_deployment_group {
    group_name: some-group,
    WITH blackbox_deployment_task {
      task_name: main-node,
      memory_mb: 32,
      docker_image: geth_stable,
      docker_image_set: geth,
      WITH blackbox_deployment_task_mount [
        {
          target_path: '/data-a',
          server_volume: server-a=>geth,
        },
      ]
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
    geth, exclusive, 4k;
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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}
"#,
        ),
    )
}

#[test]
fn test_deployment_bad_prometheus_scrape_protocol() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentPrometheusMetricsCanBeScrapedOnlyFromHttpPorts {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            expected_protocol: "http".to_string(),
            port_protocol: "tcp".to_string(),
            service_name: "epl-bb-prom-scrape".to_string(),
            group_port: 1200,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_service_registration {
    service_name: epl-bb-prom-scrape,
    scrape_prometheus_metrics: true,
    min_instances: 1,
  }
  WITH blackbox_deployment_group {
    group_name: some-group,
    WITH blackbox_deployment_service_instance {
      service_registration: epl-bb-prom-scrape,
      port: 1200,
    }
    WITH blackbox_deployment_port [
      {
        port: 1200,
        port_description: "port a",
        protocol: tcp,
      },
    ]
    WITH blackbox_deployment_task {
      task_name: main-node,
      memory_mb: 32,
      docker_image: geth_stable,
      docker_image_set: geth,
      WITH blackbox_deployment_task_mount [
        {
          target_path: '/data-a',
          server_volume: server-a=>geth,
        },
      ]
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
    geth, exclusive, 4k;
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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}
"#,
        ),
    )
}

#[test]
fn test_deployment_bad_arguments_parse() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentCantParseTaskArguments {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            task_name: "main-node".to_string(),
            task_arguments: "
        wooo
      ".to_string(),
            example_arguments_yaml: "
- /bin/sleep
- '123'
".to_string(),
            parsing_error: "invalid type: string \"wooo\", expected a sequence at line 2 column 9".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
    WITH blackbox_deployment_port [
      {
        port: 1200,
        port_description: "port a",
        protocol: tcp,
      },
    ]
    WITH blackbox_deployment_task {
      task_name: main-node,
      memory_mb: 32,
      args: '
        wooo
      ',
      docker_image: geth_stable,
      docker_image_set: geth,
      WITH blackbox_deployment_task_mount [
        {
          target_path: '/data-a',
          server_volume: server-a=>geth,
        },
      ]
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
    geth, exclusive, 4k;
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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}
"#,
        ),
    )
}

#[test]
fn test_deployment_bad_entrypoint_parse() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentCantParseTaskEntrypoint {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            task_name: "main-node".to_string(),
            task_entrypoint: "
        fooo
      ".to_string(),
            example_entrypoint_yaml: "
- /bin/sleep
- '123'
".to_string(),
            parsing_error: "invalid type: string \"fooo\", expected a sequence at line 2 column 9".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
    WITH blackbox_deployment_port [
      {
        port: 1200,
        port_description: "port a",
        protocol: tcp,
      },
    ]
    WITH blackbox_deployment_task {
      task_name: main-node,
      memory_mb: 32,
      entrypoint: '
        fooo
      ',
      docker_image: geth_stable,
      docker_image_set: geth,
      WITH blackbox_deployment_task_mount [
        {
          target_path: '/data-a',
          server_volume: server-a=>geth,
        },
      ]
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
    geth, exclusive, 4k;
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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}
"#,
        ),
    )
}

#[test]
fn test_deployment_placement_not_allowed_for_stateful_workload() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentPlacementsAreValidOnlyForStatelessWorkloads {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            group_placement: "wookie: true".to_string(),
            only_valid_placement: "".to_string(),
            already_bound_server: "server-a".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT blackbox_deployment {
  deployment_name: geth-mainnet,
  WITH blackbox_deployment_group {
    group_name: some-group,
    placement: 'wookie: true',
    WITH blackbox_deployment_port [
      {
        port: 1200,
        port_description: "port a",
        protocol: tcp,
      },
    ]
    WITH blackbox_deployment_task {
      task_name: main-node,
      memory_mb: 32,
      entrypoint: '
        fooo
      ',
      docker_image: geth_stable,
      docker_image_set: geth,
      WITH blackbox_deployment_task_mount [
        {
          target_path: '/data-a',
          server_volume: server-a=>geth,
        },
      ]
    }
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA STRUCT docker_image [
  {
    image_set: geth,
    checksum: 'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05',
    repository: 'ethereum/client-go',
    tag: v1.14.3,
    architecture: x86_64,
  },
]

DATA docker_image_set(set_name) {
  geth;
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
  '10.17.0.2/24';
}

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
    geth, exclusive, 4k;
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
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  region: us-west,
  cluster_name: default-log,
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

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}
"#,
        ),
    )
}
