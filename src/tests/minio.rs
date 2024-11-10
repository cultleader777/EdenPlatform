#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[test]
fn test_minio_too_few_instances() {
    assert_eq!(
        PlatformValidationError::MinIOMustHaveAtLeastTwoInstances {
            cluster: "global".to_string(),
            count: 1,
            min_count: 2,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
  expected_zfs_recordsize: 512k,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
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

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 8k,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
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
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.13,
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
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
]


DATA docker_image_pin {
  some_minio_pin WITH docker_image_pin_images {
    'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7';
  }
}

DATA subnet_router_floating_ip {
  '10.18.0.7/24';
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

DATA STRUCT region {
  region_name: us-east,
  docker_image_external_lb: some_minio_pin,
}

DATA STRUCT datacenter {
  dc_name: dc2,
  region: us-east,
  allow_small_subnets: true,
  network_cidr: '10.18.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: manual,
}

DATA STRUCT docker_image [
  {
    image_set: minio,
    checksum: 'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7',
    repository: 'minio/minio',
    tag: 'latest',
    architecture: x86_64,
  },
]

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    region: us-east,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    region: us-east,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am-default, },
      { instance_id: 2, alertmanager_server: server-c=>am-default, },
      { instance_id: 3, alertmanager_server: server-d=>am-default, },
    ]
  }
]

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: global=>docker,
}
"#,
        )
    );
}

#[test]
fn test_minio_zfs_recordsize_mismatch() {
    assert_eq!(
        PlatformValidationError::MinIOUnexpectedZfsRecordsizeOnVolume {
            minio_cluster: "global".to_string(),
            minio_server: "server-a".to_string(),
            minio_volume: "minio-docker-a".to_string(),
            volume_source: "server_root_volume".to_string(),
            expected_recordsize: "512k".to_string(),
            found_recordsize: "8k".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
  expected_zfs_recordsize: 512k,
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

DATA docker_image_pin {
  some_minio_pin WITH docker_image_pin_images {
    'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7';
  }
}

DATA subnet_router_floating_ip {
  '10.18.0.7/24';
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

DATA STRUCT region {
  region_name: us-east,
  docker_image_external_lb: some_minio_pin,
}

DATA STRUCT datacenter {
  dc_name: dc2,
  region: us-east,
  allow_small_subnets: true,
  network_cidr: '10.18.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: manual,
}

DATA STRUCT docker_image [
  {
    image_set: minio,
    checksum: 'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7',
    repository: 'minio/minio',
    tag: 'latest',
    architecture: x86_64,
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 8k,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
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
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.13,
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
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
]

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    region: us-east,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    region: us-east,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am-default, },
      { instance_id: 2, alertmanager_server: server-c=>am-default, },
      { instance_id: 3, alertmanager_server: server-d=>am-default, },
    ]
  }
]

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: global=>docker,
}
"#,
        )
    );
}

#[test]
fn test_minio_different_disk_mediums() {
    assert_eq!(
        PlatformValidationError::MinIOMultipleDiskMediumsDetectedInCluster {
            minio_cluster: "global".to_string(),
            disk_mediums: vec![
                "hdd".to_string(),
                "ssd".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
  expected_zfs_recordsize: 512k,
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

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA docker_image_pin {
  some_minio_pin WITH docker_image_pin_images {
    'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7';
  }
}

DATA subnet_router_floating_ip {
  '10.18.0.7/24';
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

DATA STRUCT region {
  region_name: us-east,
  docker_image_external_lb: some_minio_pin,
}

DATA STRUCT datacenter {
  dc_name: dc2,
  region: us-east,
  allow_small_subnets: true,
  network_cidr: '10.18.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: manual,
}

DATA STRUCT docker_image [
  {
    image_set: minio,
    checksum: 'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7',
    repository: 'minio/minio',
    tag: 'latest',
    architecture: x86_64,
  },
]

DATA STRUCT disk_kind {
  kind: some-hdd,
  medium: hdd,
  capacity_bytes: 21474836480,
}

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    WITH server_disk [
      { disk_id: vda },
      { disk_id: vdb, xfs_format: true, disk_kind: some-hdd },
    ]
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
    ]
    WITH server_xfs_volume [
      {
        volume_name: minio-docker-a,
        xfs_disk: vdb,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 512k,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
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
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 512k,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.13,
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
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 512k,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    region: us-east,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    region: us-east,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am-default, },
      { instance_id: 2, alertmanager_server: server-c=>am-default, },
      { instance_id: 3, alertmanager_server: server-d=>am-default, },
    ]
  }
]

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: global=>docker,
}
"#,
        )
    );
}

#[test]
fn test_minio_different_filesystems() {
    assert_eq!(
        PlatformValidationError::MinIOMultipleFilesystemsDetectedInCluster {
            minio_cluster: "global".to_string(),
            filesystems: vec![
                "xfs".to_string(),
                "zfs".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
  expected_zfs_recordsize: 512k,
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

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA docker_image_pin {
  some_minio_pin WITH docker_image_pin_images {
    'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7';
  }
}

DATA subnet_router_floating_ip {
  '10.18.0.7/24';
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

DATA STRUCT region {
  region_name: us-east,
  docker_image_external_lb: some_minio_pin,
}

DATA STRUCT datacenter {
  dc_name: dc2,
  region: us-east,
  allow_small_subnets: true,
  network_cidr: '10.18.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: manual,
}

DATA STRUCT docker_image [
  {
    image_set: minio,
    checksum: 'sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7',
    repository: 'minio/minio',
    tag: 'latest',
    architecture: x86_64,
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    WITH server_disk [
      { disk_id: vda },
      { disk_id: vdb, xfs_format: true },
    ]
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
    ]
    WITH server_xfs_volume [
      {
        volume_name: minio-docker-a,
        xfs_disk: vdb,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc2,
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 512k,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
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
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 512k,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.13,
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
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 512k,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: am-default,
      },
    ]
  },
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    region: us-east,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    region: us-east,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>am-default, },
      { instance_id: 2, alertmanager_server: server-c=>am-default, },
      { instance_id: 3, alertmanager_server: server-d=>am-default, },
    ]
  }
]

DATA STRUCT docker_registry_instance {
  region: us-east,
  minio_bucket: global=>docker,
}
"#,
        )
    );
}
