#[cfg(test)]
use crate::static_analysis::PlatformValidationError;
#[cfg(test)]
use super::common;

#[test]
fn test_regression_duplicate_internet_ips() {
    assert_eq!(
        PlatformValidationError::DuplicateIpFoundOnTheNetwork {
            server_a_name: "server-c".to_string(),
            interface_a_ip: "77.77.77.12/24".to_string(),
            interface_a_name: "eth1".to_string(),
            server_b_name: "server-e".to_string(),
            interface_b_ip: "77.77.77.12/24".to_string(),
            interface_b_name: "eth1".to_string(),
            subnet_name: "internet".to_string(),
            subnet_range: "0.0.0.0/0".to_string(),
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
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
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
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: us-west=>docker,
}

DATA STRUCT EXCLUSIVE tld {
    domain: epl-infra.net,
    expose_admin: false,
}

DATA STRUCT EXCLUSIVE datacenter [
  {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
  },
  {
    dc_name: dc2,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    network_cidr: '10.19.0.0/16',
  },
]

DATA STRUCT EXCLUSIVE region [
  { region_name: us-west, is_dns_master: true, availability_mode: multi_dc }
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
]

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
    { bucket_name: docker, },
    { bucket_name: logging, },
  ]
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-c=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-c=>am },
    { instance_id: 3, alertmanager_server: server-d=>am },
  ]
}

DATA STRUCT server [
  {
    dc: dc1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 24,
      },
    ]

    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
      {
        volume_name: am,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.11,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
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
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
      {
        volume_name: am,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
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
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
      {
        volume_name: am,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-e,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
      {
        volume_name: am,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
      {
        volume_name: am,
      },
    ]
  }
]
"#,
    ));
}


#[test]
fn test_bw_big_regression_1() {
    let _ = common::assert_platform_validation_success(
        r#"
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
      {
        version: 5,
        snapshot_source: "{
          some_field @0 :I64,
          other_field @1 :F64 DEFAULT '1.23',
          coordinates @2 :{
            x @0 :F64?,
            y @1 :F64?,
          },
          is_nice @3 :Bool DEFAULT 'true',
        }"
      }
    ] WITH versioned_type_migration [
      {
        version: 2,
        migration_source: "
          ADD .other_field @1 F64 DEFAULT '1.23'
        "
      },
      {
        version: 3,
        migration_source: "
          ADD .coordinates.x @0 F64?
          ADD .coordinates.y @1 F64?
        "
      },
      {
        version: 4,
        migration_source: "
          ADD .is_good @3 Bool DEFAULT 'true'
          ADD .nickname @4 String DEFAULT 'who knows'
        "
      },
      {
        version: 5,
        migration_source: "
          RENAME .is_good .is_nice
          DROP .nickname
        "
      }
    ]
  }
]
"#,
    );
}

#[test]
fn test_regression_for_dcrouter_for_internet_routing() {
    assert_eq!(
        PlatformValidationError::IntraDcRoutingNeededButNoDcrouterNetworkExists {
            dc: "dc1".to_string(),
            subnet_count: 2,
            missing_network: "dcrouter".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE datacenter {
  dc_name: dc1,
  allow_small_subnets: true,
  network_cidr: '10.17.0.0/16',
  implementation: aws,
  implementation_settings: '
    availability_zone: us-west-1b
  ',
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

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
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
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
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
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
    ssh_interface: eth1,
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
        if_ip: 10.17.1.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
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
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: eth1,
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
        if_ip: 10.17.1.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
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
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-e,
    ssh_interface: eth0,
    WITH server_disk {
      disk_id: 'vda'
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
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.1.12,
        if_prefix: 24,
      },
    ]
  }
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
        pg_server: server-b=>pgtest1,
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
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
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
fn test_regression_for_dcrouter_for_internet_routing_interface_needed() {
    assert_eq!(
        PlatformValidationError::DcrouterServerMustHaveDcrouterInterface {
            server: "server-a".to_string(),
            server_dc: "dc1".to_string(),
            network_interfaces: vec!["lan".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"

DATA STRUCT EXCLUSIVE datacenter {
  dc_name: dc1,
  allow_small_subnets: true,
  network_cidr: '10.17.0.0/16',
  implementation: aws,
  implementation_settings: '
    availability_zone: us-west-1b
  ',
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

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
    network_name: dcrouter,
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
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
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
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
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
    ssh_interface: eth1,
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
        if_ip: 10.17.1.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
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
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: eth1,
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
        if_ip: 10.17.1.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
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
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-e,
    ssh_interface: eth0,
    WITH server_disk {
      disk_id: 'vda'
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
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.1.12,
        if_prefix: 24,
      },
    ]
  }
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
        pg_server: server-b=>pgtest1,
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
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
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
