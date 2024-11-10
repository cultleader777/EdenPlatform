
#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_docker_image_doesnt_start_with_sha256() {
    assert_eq!(
        PlatformValidationError::DockerImageChecksumDoesntStartWithSha256 {
            bad_image_checksum: "sha256z:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0c".to_string(),
            expected_prefix: "sha256:".to_string(),
            image_set: "postgres_wpatroni_wconsul".to_string(),
            repository: "cultleader777/patroni-pg".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT docker_image [
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256z:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0c',
    repository: 'cultleader777/patroni-pg',
    architecture: x86_64,
  }
]

"#,
        )
    );
}

#[test]
fn test_docker_image_checksum_too_long() {
    assert_eq!(
        PlatformValidationError::DockerImageChecksumBadLength {
            bad_image_checksum: "sha256:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0ca".to_string(),
            expected_length: 71,
            actual_length: 72,
            image_set: "postgres_wpatroni_wconsul".to_string(),
            repository: "cultleader777/patroni-pg".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT docker_image [
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0ca',
    repository: 'cultleader777/patroni-pg',
    architecture: x86_64,
  }
]

"#,
        )
    );
}

#[test]
fn test_docker_image_checksum_invalid_characters() {
    assert_eq!(
        PlatformValidationError::DockerImageChecksumBadSymbols {
            bad_image_checksum: "sha256:9bddbbbe30e2eb6030158zcc1e9375e26105f557aa45df4ce66c7abde698db0c".to_string(),
            only_allowed_checksum_characters: "0123456789abcdef".to_string(),
            image_set: "postgres_wpatroni_wconsul".to_string(),
            repository: "cultleader777/patroni-pg".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT docker_image [
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256:9bddbbbe30e2eb6030158zcc1e9375e26105f557aa45df4ce66c7abde698db0c',
    repository: 'cultleader777/patroni-pg',
    architecture: x86_64,
  }
]

"#,
        )
    );
}

#[test]
fn test_docker_image_pin_multiple_images_for_same_arch() {
    assert_eq!(
        PlatformValidationError::DockerImagePinContainsMultipleImagesForSameArchitecture {
            previous_docker_image: "cultleader777/patroni-pg@sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0c".to_string(),
            duplicate_docker_image: "cultleader777/patroni-pg@sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0d".to_string(),
            architecture: "x86_64".to_string(),
            image_pin_name: "some_pin".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"

DATA docker_image_pin {
  some_pin WITH docker_image_pin_images {
    'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0c';
    'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0d';
  }
}

DATA STRUCT docker_image [
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0c',
    repository: 'cultleader777/patroni-pg',
    architecture: x86_64,
  },
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0d',
    repository: 'cultleader777/patroni-pg',
    architecture: x86_64,
  }
]

"#,
        )
    );
}

#[test]
fn test_docker_image_invalid_arch() {
    let e = common::assert_eden_db_error_wcustom_data(
            r#"

DATA docker_image_pin {
  some_pin WITH docker_image_pin_images {
    'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0c';
  }
}

DATA STRUCT docker_image [
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0c',
    repository: 'cultleader777/patroni-pg',
    architecture: lolwat,
  },
]

"#,
    );
    assert!(matches!(e, edendb::checker::errors::DatabaseValidationError::LuaCheckEvaluationFailed { .. }));
}

#[test]
fn test_docker_image_pin_multiple_images_for_multiple_arch_works() {
    let _ =
        common::assert_platform_validation_success(
            r#"
DATA docker_image_pin {
  some_pin WITH docker_image_pin_images {
    'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0c';
    'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0d';
  }
}

DATA STRUCT docker_image [
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0c',
    repository: 'cultleader777/patroni-pg',
    architecture: x86_64,
  },
  {
    image_set: postgres_wpatroni_wconsul,
    tag: whatever,
    checksum: 'sha256:9bddbbbe30e2eb6030158acc1e9375e26105f557aa45df4ce66c7abde698db0d',
    repository: 'cultleader777/patroni-pg',
    architecture: arm64,
  }
]

"#,
        );
}

#[test]
fn test_docker_image_bad_type_for_component() {
    assert_eq!(
        PlatformValidationError::DockerImageDoesNotBelongToTheExpectedSet {
            image_pin_name: "some_minio_pin".to_string(),
            image_architecture: "x86_64".to_string(),
            image_checksum: "sha256:68622c3e49dd98fbbcb8200729297207759d52e3b02d2ed908c1a7ff3b83f3f7".to_string(),
            image_repository: "minio/minio".to_string(),
            expected_docker_image_set: "openresty".to_string(),
            found_docker_image_set: "minio".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
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
fn test_docker_image_not_found_for_architecture() {
    assert_eq!(
        PlatformValidationError::DockerImageNotFoundForArchitectureForPin {
            image_pin_name: "some_openresty_pin".to_string(),
            architecture_image_not_found: "x86_64".to_string(),
            found_architecture_images: vec!["arm64".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA docker_image_pin {
  some_openresty_pin WITH docker_image_pin_images {
    'sha256:6bea73e8fe3848abd7c228458d7fde9417f509cebe2ebb46ee9e47363b5ff846';
  }
}

DATA STRUCT docker_image [
  {
    image_set: openresty,
    checksum: 'sha256:6bea73e8fe3848abd7c228458d7fde9417f509cebe2ebb46ee9e47363b5ff846',
    repository: 'openresty/openresty',
    tag: 1.25.3.1-2-buster-fat,
    architecture: arm64,
  },
]

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
  docker_image_external_lb: some_openresty_pin,
}

DATA STRUCT datacenter {
  dc_name: dc2,
  region: us-east,
  allow_small_subnets: true,
  network_cidr: '10.18.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: manual,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
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
fn test_docker_image_arm64_builds_not_supported_yet_for_backend_apps() {
    assert_eq!(
        PlatformValidationError::NonAmd64BuildsNotSupportedYetForApplications {
            application_deployment: "test-depl".to_string(),
            workload_architecture: "arm64".to_string(),
            only_allowed_workload_architecture: "x86_64".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
  }
]

DATA STRUCT backend_application_deployment {
  region: us-east,
  deployment_name: test-depl,
  application_name: hello-world,
  workload_architecture: arm64,
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
}

DATA STRUCT datacenter {
  dc_name: dc2,
  region: us-east,
  allow_small_subnets: true,
  network_cidr: '10.18.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: manual,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
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
fn test_docker_image_arm64_builds_not_supported_yet_for_frontend_apps() {
    assert_eq!(
        PlatformValidationError::NonAmd64BuildsNotSupportedYetForApplications {
            application_deployment: "test-depl".to_string(),
            workload_architecture: "arm64".to_string(),
            only_allowed_workload_architecture: "x86_64".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-world,
    WITH frontend_page {
      page_name: home,
      path: '/',
    }
  }
]

DATA STRUCT frontend_application_deployment {
  region: us-east,
  deployment_name: test-depl,
  application_name: hello-world,
  workload_backend_architecture: arm64,
}

DATA STRUCT frontend_application_deployment_ingress {
  deployment: test-depl,
  subdomain: www,
  tld: epl-infra.net,
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
}

DATA STRUCT datacenter {
  dc_name: dc2,
  region: us-east,
  allow_small_subnets: true,
  network_cidr: '10.18.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: manual,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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
        zfs_recordsize: 1M,
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

DATA STRUCT tempo_cluster {
  region: us-east,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  region: us-east,
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
