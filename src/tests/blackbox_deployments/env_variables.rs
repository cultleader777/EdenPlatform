#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_invalid_variable_name() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentEnvironmentVariableNameIsInvalid {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            task_name: "main".to_string(),
            env_variable_name: "1ead".to_string(),
            must_match_regex: "^[a-zA-Z_][a-zA-Z0-9_]*$".to_string(),
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
    WITH blackbox_deployment_task [
      {
        task_name: main,
        docker_image: geth_stable,
        docker_image_set: geth,
        memory_mb: 32,
        WITH blackbox_deployment_env_variable [
          {
             var_name: 1ead,
             raw_value: moo,
          }
        ]
      }
    ]
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA docker_image_set(set_name) {
  geth;
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
fn test_source_and_raw_value_empty() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentRawValueOrValueSourceMustBeNotEmpty {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            task_name: "main".to_string(),
            env_variable_name: "HELLO_WORLD".to_string(),
            raw_value: "".to_string(),
            value_source: "".to_string(),
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
    WITH blackbox_deployment_task [
      {
        task_name: main,
        docker_image: geth_stable,
        docker_image_set: geth,
        memory_mb: 32,
        WITH blackbox_deployment_env_variable [
          {
             var_name: HELLO_WORLD,
          }
        ]
      }
    ]
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA docker_image_set(set_name) {
  geth;
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
fn test_source_and_raw_value_mutually_exclusive() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentRawValueAndValueSourceAreMutuallyExclusive {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            task_name: "main".to_string(),
            env_variable_name: "HELLO_WORLD".to_string(),
            raw_value: "value".to_string(),
            value_source: "something".to_string(),
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
    WITH blackbox_deployment_task [
      {
        task_name: main,
        docker_image: geth_stable,
        docker_image_set: geth,
        memory_mb: 32,
        WITH blackbox_deployment_env_variable [
          {
             var_name: HELLO_WORLD,
             raw_value: value,
             value_source: something,
          }
        ]
      }
    ]
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA docker_image_set(set_name) {
  geth;
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
fn test_value_source_not_found_in_region() {
    assert_eq!(
        PlatformValidationError::BlackboxDeploymentValueSourceNotFoundInRegion {
            bb_deployment: "geth-mainnet".to_string(),
            bb_region: "us-west".to_string(),
            group_name: "some-group".to_string(),
            task_name: "main".to_string(),
            env_variable_name: "HELLO_WORLD".to_string(),
            value_source: "something".to_string(),
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
    WITH blackbox_deployment_task [
      {
        task_name: main,
        docker_image: geth_stable,
        docker_image_set: geth,
        memory_mb: 32,
        WITH blackbox_deployment_env_variable [
          {
             var_name: HELLO_WORLD,
             value_source: something,
          }
        ]
      }
    ]
  }
}

DATA docker_image_pin {
  geth_stable WITH docker_image_pin_images {
    'sha256:01d80da9635e3fbbaac04056ff8c9887e972838775790c7636996d5caeaa2b05';
  };
}

DATA docker_image_set(set_name) {
  geth;
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
