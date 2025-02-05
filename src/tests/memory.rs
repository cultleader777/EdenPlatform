#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_stateful_memory_exhaust() {
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

DATA STRUCT server_kind {
    kind: testvm.cpu4ram6144,
    cores: 4,
    memory_bytes: 6442450944,
    architecture: x86_64,
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, testvm.cpu4ram6144, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    pgtest1;
    minio;
    mon;
    am;
  };
  server-b, testvm.cpu4ram6144, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    minio;
    mon;
    am;
  };
  server-c, testvm.cpu4ram6144, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    minio;
    mon;
    am;
  };
  server-d, testvm.cpu4ram6144, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume {
    minio;
  };
}

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    storage_bucket: us-west=>loki,
    loki_reader_memory_mb: 9999999,
  }
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    shared_buffers_mb: 9999999
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
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
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
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: tempo,
    },
  ]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ServerCannotReserveMoreMemory {
            server_hostname: "server-a".to_string(),
            memory_reservation_log: vec![
                ("System reserved".to_string(), 1073741824),
                ("Docker registry".to_string(), 134217728),
                (
                    "postgres shared_buffers testdb-1".to_string(),
                    10485758951424
                ),
            ],
            total_sum: 10486966910976,
            server_memory: 1024 * 1024 * 1024 * 6,
        }
    );
}

#[test]
fn test_system_stateless_memory_exhaust() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs {
            add_default_global_flags: false,
            add_default_data: true,
        },
        r#"
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
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
  memory_mb: 9999999,
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
      instance_volume: server-a=>minio-global,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-global,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-global,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-global,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram6144,
    cores: 4,
    memory_bytes: 6442450944,
    architecture: x86_64,
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, testvm.cpu4ram6144, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
  server-b, testvm.cpu4ram6144, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
  server-c, testvm.cpu4ram6144, eth0, true, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
  server-d, testvm.cpu4ram6144, eth0, false, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ServerCannotReserveMoreMemory {
            server_hostname: "server-a".to_string(),
            memory_reservation_log: vec![
                ("System reserved".to_string(), 1073741824),
                ("Docker registry".to_string(), 10485758951424),
            ],
            total_sum: 10486832693248,
            server_memory: 1024 * 1024 * 1024 * 6,
        }
    );
}

#[test]
fn test_stateless_memory_exhaust() {
    // highest server memory is server-c
    // and it should be exhausted first by the algorithm
    // because we first assign biggest chunks to biggest servers
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    storage_bucket: global=>loki,
    loki_reader_memory_mb: 9999999,
  }
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
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-global,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-global,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-global,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-global,
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
      bucket_name: tempo,
    },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram6144,
    cores: 4,
    memory_bytes: 6442450944,
    architecture: x86_64,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram16g,
    cores: 4,
    memory_bytes: 17179869184,
    architecture: x86_64,
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, testvm.cpu4ram6144, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, testvm.cpu4ram6144, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, testvm.cpu4ram16g, eth0, true, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, testvm.cpu4ram6144, eth0, false, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
}

"#,
    );
    assert_eq!(
        PlatformValidationError::ServerCannotReserveMoreStatelessMemory {
            server_hostname: "server-c".to_string(),
            memory_reservation_log: vec![
                ("System reserved".to_string(), 1073741824),
                ("Docker registry".to_string(), 134217728),
                ("Prometheus memory default-mon".to_string(), 805306368),
                ("VictoriaMetrics memory default-mon".to_string(), 268435456),
                ("Alertmanager memory default-mon".to_string(), 67108864),
                ("MinIO instance memory global".to_string(), 1073741824),
                ("MinIO lb memory global".to_string(), 67108864),
                (
                    "MinIO mc bucket provisioning memory global".to_string(),
                    134217728
                ),
                ("External load balancer".to_string(), 67108864),
                (
                    "EPL Loki cluster main reader".to_string(),
                    10485758951424
                ),
            ],
            total_sum: 10489449938944,
            server_memory: 1024 * 1024 * 1024 * 16,
            servers_already_hosting_this_workload: Vec::new(),
        },
        err,
    );
}

#[test]
fn test_stateless_placed_memory_exhaust() {
    // highest server memory is server-c
    // and it should be exhausted first by the algorithm
    // because we first assign biggest chunks to biggest servers
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA valid_server_labels {
  loki_reader;
}

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    storage_bucket: global=>loki,
    loki_reader_memory_mb: 20480,
    reader_placement: '
      match_keys_and_values:
        loki_reader: true
    ',
  }
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
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-global,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-global,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-global,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-global,
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
      bucket_name: tempo,
    },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram6144,
    cores: 4,
    memory_bytes: 6442450944,
    architecture: x86_64,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram16g,
    cores: 4,
    memory_bytes: 17179869184,
    architecture: x86_64,
}

DATA STRUCT server_kind {
    kind: testvm.bigone,
    cores: 4,
    memory_bytes: 34359738368,
    architecture: x86_64,
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, testvm.cpu4ram6144, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH server_label {
    loki_reader, true;
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, testvm.bigone, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH server_label {
    loki_reader, true;
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, testvm.cpu4ram6144, eth0, true, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, testvm.cpu4ram6144, eth0, false, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
}

"#,
    );
    assert_eq!(
        PlatformValidationError::ServerCannotReserveMoreStatelessPlacedMemory {
            server_hostname: "server-a".to_string(),
            placement_labels: "match_keys_and_values:\n  loki_reader: 'true'\n".to_string(),
            all_placement_servers: vec!["server-b".to_string(), "server-a".to_string()],
            memory_reservation_log: vec![
                ("System reserved".to_string(), 1073741824),
                ("Docker registry".to_string(), 134217728),
                ("Prometheus memory default-mon".to_string(), 805306368),
                ("VictoriaMetrics memory default-mon".to_string(), 268435456),
                ("Alertmanager memory default-mon".to_string(), 67108864),
                ("MinIO instance memory global".to_string(), 1073741824),
                ("MinIO lb memory global".to_string(), 67108864),
                (
                    "MinIO mc bucket provisioning memory global".to_string(),
                    134217728
                ),
                ("External load balancer".to_string(), 67108864),
                (
                    "EPL Loki cluster main reader".to_string(),
                    21474836480
                ),
            ],
            total_sum: 25165824000,
            server_memory: 1024 * 1024 * 1024 * 6,
        },
        err,
    );
}

#[test]
fn test_stateless_forbidden_memory_exhaust() {
    // highest server memory is server-c
    // and it should be exhausted first by the algorithm
    // because we first assign biggest chunks to biggest servers
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    storage_bucket: global=>loki,
    loki_reader_memory_mb: 20480,
    loki_readers: 2,
  }
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
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-global,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-global,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-global,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-global,
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
      bucket_name: tempo,
    },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram6144,
    cores: 4,
    memory_bytes: 6442450944,
    architecture: x86_64,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram16g,
    cores: 4,
    memory_bytes: 17179869184,
    architecture: x86_64,
}

DATA STRUCT server_kind {
    kind: testvm.bigone,
    cores: 4,
    memory_bytes: 34359738368,
    architecture: x86_64,
}

DATA server(hostname, run_unassigned_workloads, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, false, testvm.cpu4ram6144, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, false, testvm.bigone, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, false, testvm.cpu4ram6144, eth0, true, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, false, testvm.cpu4ram6144, eth0, false, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
}

"#,
    );
    assert_eq!(
        PlatformValidationError::ServerCannotPlaceStatelessWorkload {
            needed_bytes: vec![
                ("EPL Loki cluster main reader".to_string(), 21474836480)
            ],
            servers_disallowed_to_run_unassigned_workloads: vec![
                "server-a".to_string(),
                "server-c".to_string(),
                "server-d".to_string(),
                "server-b".to_string(),
            ],
        },
        err,
    );
}

#[test]
fn test_stateless_forbidden_memory_exhaust_one_node() {
    // highest server memory is server-c
    // and it should be exhausted first by the algorithm
    // because we first assign biggest chunks to biggest servers
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    storage_bucket: global=>loki,
    loki_reader_memory_mb: 20480,
    loki_readers: 2,
  }
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
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-global,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-global,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-global,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-global,
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
      bucket_name: tempo,
    },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram6144,
    cores: 4,
    memory_bytes: 6442450944,
    architecture: x86_64,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram16g,
    cores: 4,
    memory_bytes: 17179869184,
    architecture: x86_64,
}

DATA STRUCT server_kind {
    kind: testvm.bigone,
    cores: 4,
    memory_bytes: 53687091200,
    architecture: x86_64,
}

DATA server(hostname, run_unassigned_workloads, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, true, testvm.cpu4ram6144, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, true, testvm.bigone, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, false, testvm.bigone, eth0, true, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, true, testvm.cpu4ram6144, eth0, false, true, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio-global, exclusive, 1M;
  };
}

"#,
    );
    assert_eq!(
        PlatformValidationError::ServerCannotReserveMoreStatelessMemory {
            server_hostname: "server-d".to_string(),
            memory_reservation_log: vec![
                (
                    "System reserved".to_string(),
                    1073741824,
                ),
                (
                    "Docker registry".to_string(),
                    134217728,
                ),
                (
                    "MinIO instance memory global".to_string(),
                    1073741824,
                ),
                (
                    "MinIO lb memory global".to_string(),
                    67108864,
                ),
                (
                    "MinIO mc bucket provisioning memory global".to_string(),
                    134217728,
                ),
                (
                    "External load balancer".to_string(),
                    67108864,
                ),
                (
                    "EPL Loki cluster main reader".to_string(),
                    21474836480,
                ),
            ],
            total_sum: 24024973312,
            server_memory: 6442450944,
            servers_already_hosting_this_workload: vec![
                "server-b".to_string(),
            ],
        },
        err,
    );
}
