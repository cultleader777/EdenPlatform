#[cfg(test)]
use crate::static_analysis::PlatformValidationError;
#[cfg(test)]
use crate::tests::common::TestArgs;
#[cfg(test)]
use super::super::common;

#[test]
fn test_into_table_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamIntoTableDoesntExist {
            ch_nats_stream_import_consumer_name: "ch_nats_consumer".to_string(),
            ch_deployment: "test-ch".to_string(),
            ch_database: "testdb_a".to_string(),
            ch_schema: "testdb".to_string(),
            into_table: "non_existant_table".to_string(),
            existing_tables: vec![
                "foo".to_string(),
            ]
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"

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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-west,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    { db_name: testdb_a, ch_schema: testdb,
      WITH ch_nats_stream_import {
        consumer_name: ch_nats_consumer,
        into_table: non_existant_table,
        stream: nats-default=>some_test_stream,
      }
    },
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
        dataset_contents: "
          foo:
          - id: 7
            a: hello
        "
      }
    ]
  }
]


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


DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
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
    ch, exclusive, 4k;
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
    ch, exclusive, 4k;
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
fn test_into_table_column_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamIntoTableColumnDoesntExist {
            ch_nats_stream_import_consumer_name: "ch_nats_consumer".to_string(),
            ch_deployment: "test-ch".to_string(),
            ch_database: "testdb_a".to_string(),
            ch_schema: "testdb".to_string(),
            into_table: "foo".to_string(),
            bw_compat_type: "test_vtype".to_string(),
            bw_compat_non_existing_field_name: "some_field".to_string(),
            existing_columns_in_table: vec![
                "a".to_string(),
                "id".to_string(),
            ]
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"

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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-west,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
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
        dataset_contents: "
          foo:
          - id: 7
            a: hello
        "
      }
    ]
  }
]


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


DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
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
    ch, exclusive, 4k;
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
    ch, exclusive, 4k;
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
fn test_into_table_column_type_unsupported() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamUnsupportedFieldType {
            ch_nats_stream_import_consumer_name: "ch_nats_consumer".to_string(),
            ch_deployment: "test-ch".to_string(),
            ch_database: "testdb_a".to_string(),
            ch_schema: "testdb".to_string(),
            into_table: "foo".to_string(),
            bw_compat_type: "test_vtype".to_string(),
            bw_compat_unsupported_field_name: "id".to_string(),
            bw_compat_unsupported_field_type: "Array".to_string(),
            nats_jetstream_stream: "some_test_stream".to_string(),
            message: "Arrays not supported to be imported to Clickhouse tables".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"

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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-west,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
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
        dataset_contents: "
          foo:
          - id: 7
            a: hello
        "
      }
    ]
  }
]


DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          id @0 :I64[],
        }"
      },
    ]
  },
]

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


DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
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
    ch, exclusive, 4k;
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
    ch, exclusive, 4k;
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
fn test_into_table_column_type_mismatch() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamIntoTableColumnTypeMismatch {
            ch_nats_stream_import_consumer_name: "ch_nats_consumer".to_string(),
            ch_deployment: "test-ch".to_string(),
            ch_database: "testdb_a".to_string(),
            ch_schema: "testdb".to_string(),
            into_table: "foo".to_string(),
            bw_compat_type: "test_vtype".to_string(),
            table_column_type: "Int64".to_string(),
            bw_compat_type_field_name: "id".to_string(),
            bw_compat_type_field_type: "String".to_string(),
            nats_jetstream_stream: "some_test_stream".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"

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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-west,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
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

DATA STRUCT ch_schema [
  {
    schema_name: testdb,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int64,
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
        dataset_contents: "
          foo:
          - id: 7
            a: hello
        "
      }
    ]
  }
]


DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          id @0 :String,
        }"
      },
    ]
  },
]

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


DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
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
    ch, exclusive, 4k;
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
    ch, exclusive, 4k;
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
fn test_into_table_uninsertable_column() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamIntoTableColumnFieldIsNotAllowedToBeInserted {
            ch_nats_stream_import_consumer_name: "ch_nats_consumer".to_string(),
            ch_deployment: "test-ch".to_string(),
            ch_database: "testdb_a".to_string(),
            ch_schema: "testdb".to_string(),
            into_table: "foo".to_string(),
            bw_compat_type: "test_vtype".to_string(),
            table_column_type: "String".to_string(),
            bw_compat_type_field_name: "a".to_string(),
            nats_jetstream_stream: "some_test_stream".to_string(),
            explanation: "Column field is either ALIAS or MATERIALIZED and cannot be inserted, only computed".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"

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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-west,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
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

DATA STRUCT ch_schema [
  {
    schema_name: testdb,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int64,
            a String MATERIALIZED toString(id)
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
        dataset_contents: "
          foo:
          - id: 7
        "
      }
    ]
  }
]


DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          id @0 :I64,
          a @1 :String,
        }"
      },
    ]
  },
]

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


DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
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
    ch, exclusive, 4k;
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
    ch, exclusive, 4k;
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
fn test_into_table_enable_subjects_unsupported() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamHasEnableSubjectsWhichIsUnsupported {
            ch_nats_stream_import_consumer_name: "ch_nats_consumer".to_string(),
            ch_deployment: "test-ch".to_string(),
            ch_database: "testdb_a".to_string(),
            ch_schema: "testdb".to_string(),
            into_table: "foo".to_string(),
            nats_jetstream_stream: "some_test_stream".to_string(),
            nats_jetstream_stream_enable_subjects: true,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"

DATA STRUCT nats_cluster {
  cluster_name: nats-default,
  region: us-west,
  WITH nats_deployment_instance [
    { instance_id: 1, nats_server: server-a=>nats },
    { instance_id: 2, nats_server: server-b=>nats },
    { instance_id: 3, nats_server: server-c=>nats },
  ]
  WITH nats_jetstream_stream [
    { stream_name: some_test_stream, stream_type: test_vtype, enable_subjects: true, },
  ]
}

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-west,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
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

DATA STRUCT ch_schema [
  {
    schema_name: testdb,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int64,
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
        dataset_contents: "
          foo:
          - id: 7
        "
      }
    ]
  }
]


DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          id @0 :I64,
          a @1 :String,
        }"
      },
    ]
  },
]

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


DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
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
    ch, exclusive, 4k;
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
    ch, exclusive, 4k;
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
fn test_into_table_missing_column() {
    assert_eq!(
        PlatformValidationError::ChNatsStreamIntoTableColumnFieldHasNoDefaultAndDoesntExistInBwType {
            ch_nats_stream_import_consumer_name: "ch_nats_consumer".to_string(),
            ch_deployment: "test-ch".to_string(),
            ch_database: "testdb_a".to_string(),
            ch_schema: "testdb".to_string(),
            into_table: "foo".to_string(),
            bw_compat_type: "test_vtype".to_string(),
            table_column_type: "String".to_string(),
            missing_table_column_from_nats: "a".to_string(),
            nats_jetstream_stream: "some_test_stream".to_string(),
            explanation: "Field a exists in table, has no default value but doesn't exist in backwards compatible type test_vtype".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"

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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  region: us-west,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
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

DATA STRUCT ch_schema [
  {
    schema_name: testdb,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int64,
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
        dataset_contents: "
          foo:
          - id: 7
            a: hello
        "
      }
    ]
  }
]


DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          id @0 :I64,
        }"
      },
    ]
  },
]

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


DATA STRUCT ch_keeper_deployment {
  deployment_name: test-chk,
  WITH ch_keeper_deployment_instance [
    { instance_id: 1, keeper_server: server-a=>chk },
    { instance_id: 2, keeper_server: server-b=>chk },
    { instance_id: 3, keeper_server: server-c=>chk },
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
    ch, exclusive, 4k;
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
    ch, exclusive, 4k;
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
