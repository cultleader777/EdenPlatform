#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_application_ch_wiring_inserter_table_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardInserterTableDoesntExist {
            application: "hello-world".to_string(),
            non_existant_inserter_table: "foozo_baro".to_string(),
            existing_tables_in_schema: vec![
                "bar".to_string(),
                "foo".to_string(),
            ]
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_inserters: '
              foozo_baro
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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

"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_inserter_into_view_table_is_not_allowed() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardInserterIntoViewIsNotAllowed {
            application: "hello-world".to_string(),
            inserter_table: "foo".to_string(),
            inserter_table_type: "view or materialized view".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_inserters: '
              foo
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE VIEW IF NOT EXISTS foo AS
          SELECT 1 AS id;
        ",
        downgrade: "
          DROP VIEW IF EXISTS foo;
        ",
      },
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
        ",
      },
    ]
  }
]

"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_inserter_table_column_type_is_invalid() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardInserterTableTypeNotSupported {
            application: "hello-world".to_string(),
            inserter_table: "foo".to_string(),
            inserter_table_column: "a".to_string(),
            inserter_table_column_type: "UInt256".to_string(),
            supported_inserter_types: vec![
                "String".to_string(),
                "Int32".to_string(),
                "Int64".to_string(),
                "Int128".to_string(),
                "Int256".to_string(),
                "DateTime".to_string(),
                "Date".to_string(),
                "Float32".to_string(),
                "Float64".to_string(),
                "Bool".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_inserters: '
              foo
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a UInt256
          ) ENGINE = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS foo;
        ",
      },
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
        ",
      },
    ]
  }
]

"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_invalid_format() {
    assert_eq!(
        PlatformValidationError::ApplicationChWiringInvalidFormat {
            application_name: "hello-world".to_string(),
            application_deployment: "test-depl".to_string(),
            bad_line: "bad corrupt syntax".to_string(),
            explanation: "Valid example of db wiring \"ch_shard_a: ch_deployment_b=>db_name_c\"",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    bad corrupt syntax
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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
"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_app_has_no_specific_ch_shard() {
    assert_eq!(
        PlatformValidationError::ApplicationChWiringApplicationHasNoDbShard {
            application_name: "hello-world".to_string(),
            application_deployment: "test-depl".to_string(),
            bad_line: "non_existant: testch=>chdb_a".to_string(),
            missing_application_db_shard: "non_existant".to_string(),
            valid_app_db_shards: vec![
                "a".to_string(),
            ],
            explanation: "Specified application Clickhouse shard is missing",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    non_existant: testch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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
  } WITH server_root_volume {
    pgtest1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_app_has_no_ch_shards() {
    assert_eq!(
        PlatformValidationError::ApplicationChWiringApplicationHasNoDbShards {
            application_name: "hello-world".to_string(),
            application_deployment: "test-depl".to_string(),
            bad_line: "non_existant: testch=>chdb_a".to_string(),
            explanation: "This application has no Clickhouse shards that need to be wired",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    non_existant: testch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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
  } WITH server_root_volume {
    pgtest1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_target_ch_deployment_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ApplicationChWiringTargetChDeploymentDoesntExist {
            application_name: "hello-world".to_string(),
            application_deployment: "test-depl".to_string(),
            bad_line: "a: non_existant=>chdb_a".to_string(),
            missing_ch_deployment: "non_existant=>chdb_a".to_string(),
            explanation: "Specified Clickhouse deployment with schema doesn't exist",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: non_existant=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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
  } WITH server_root_volume {
    pgtest1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_shard_defined_multiple_times() {
    assert_eq!(
        PlatformValidationError::ApplicationChWiringApplicationShardDefinedMultipleTimes {
            application_name: "hello-world".to_string(),
            application_deployment: "test-depl".to_string(),
            bad_line: "a: test-ch=>chdb_a".to_string(),
            explanation: "Specified application Clickhouse shard redefined multiple times",
            redefined_app_db_shard_name: "a".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: test-ch=>chdb_a
    a: test-ch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  loki_cluster: none,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    {
      db_name: chdb_a,
      ch_schema: testch,
    }
  ]
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
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
    chk;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_wiring_schema_mismatch() {
    assert_eq!(
        PlatformValidationError::ApplicationChWiringSchemaMismatch {
            application_name: "hello-world".to_string(),
            application_deployment: "test-depl".to_string(),
            bad_line: "a: test-ch=>chdb_b".to_string(),
            explanation: "Application expected Clickhouse schema mismatches wired shard schema",
            application_expected_ch_schema: "testch".to_string(),
            target_deployment_ch_schema: "otherschema".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: test-ch=>chdb_b
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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

DATA STRUCT ch_schema [
  {
    schema_name: otherschema,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS moo (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS moo;
        ",
      },
    ]
  }
]

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  loki_cluster: none,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    {
      db_name: chdb_a,
      ch_schema: testch,
    },
    {
      db_name: chdb_b,
      ch_schema: otherschema,
    },
  ]
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
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
    chk;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_query_not_found_in_schema() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardQueryNotFoundInChSchema {
            query_not_found: "wookie".to_string(),
            application_ch_schema: "testch".to_string(),
            application_ch_shard: "a".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: test-ch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_queries: '
              wookie
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  loki_cluster: none,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    {
      db_name: chdb_a,
      ch_schema: testch,
    },
  ]
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
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
    chk;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_query_defined_twice() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardQueryDefinedTwice {
            used_query_defined_twice: "max_id_from_foo".to_string(),
            used_queries_src: "
              max_id_from_foo
              max_id_from_foo
            ".to_string(),
            application_ch_schema: "testch".to_string(),
            application_ch_shard: "a".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: test-ch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_queries: '
              max_id_from_foo
              max_id_from_foo
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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
    WITH ch_query [
      {
        query_name: "max_id_from_foo",
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32}",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 2',
          outputs: "
          - max_id: 7
          "
        }
      },
    ]
  }
]

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  loki_cluster: none,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    {
      db_name: chdb_a,
      ch_schema: testch,
    },
  ]
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
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
    chk;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_inserter_defined_twice() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardInserterDefinedTwice {
            inserter_defined_twice: "foo".to_string(),
            inserters_src: "
              foo
              foo
            ".to_string(),
            application_ch_schema: "testch".to_string(),
            application_ch_shard: "a".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: test-ch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_queries: '
              max_id_from_foo
            ',
            used_inserters: '
              foo
              foo
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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
    WITH ch_query [
      {
        query_name: "max_id_from_foo",
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32}",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 2',
          outputs: "
          - max_id: 7
          "
        }
      },
    ]
  }
]

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  loki_cluster: none,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    {
      db_name: chdb_a,
      ch_schema: testch,
    },
  ]
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
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
    chk;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_mutator_defined_twice() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardMutatorDefinedTwice {
            used_mutator_defined_twice: "foo".to_string(),
            used_mutators_src: "
              foo
              foo
            ".to_string(),
            application_ch_schema: "testch".to_string(),
            application_ch_shard: "a".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: test-ch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_mutators: '
              foo
              foo
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "foo",
        mutator_expression: "TRUNCATE foo",
        WITH ch_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: 'test_arg: 2',
        }
      },
    ]
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  loki_cluster: none,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    {
      db_name: chdb_a,
      ch_schema: testch,
    },
  ]
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
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
    chk;
  };
}
"#,
        )
    );
}

#[test]
fn test_application_ch_mutator_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ApplicationChShardMutatorNotFoundInChSchema {
            mutator_not_found: "non_existing".to_string(),
            application_ch_schema: "testch".to_string(),
            application_ch_shard: "a".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: true,
            },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  ch_shard_wiring: '
    a: test-ch=>chdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_ch_shard [
        {
            shard_name: a,
            ch_schema: testch,
            used_mutators: '
              non_existing
            ',
        },
    ]
  }
]

DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "foo",
        mutator_expression: "TRUNCATE foo",
        WITH ch_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: 'test_arg: 2',
        }
      },
    ]
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
      {
        time: 2,
        upgrade: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS bar;
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

DATA STRUCT ch_deployment {
  deployment_name: test-ch,
  keeper: test-chk,
  loki_cluster: none,
  WITH ch_deployment_instance [
    { instance_id: 1, ch_server: server-a=>ch },
    { instance_id: 2, ch_server: server-b=>ch },
  ]
  WITH ch_deployment_schemas [
    {
      db_name: chdb_a,
      ch_schema: testch,
    },
  ]
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
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    pgtest1;
    chk;
    ch;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
    chk;
  };
}
"#,
        )
    );
}
