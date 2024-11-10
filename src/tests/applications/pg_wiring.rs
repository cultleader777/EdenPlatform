#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_application_db_wiring_bad_syntax() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
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
  pg_shard_wiring: '
    bad corrupt syntax
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringInvalidFormat {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "bad corrupt syntax".to_string(),
            explanation: "Valid example of db wiring \"pg_shard_a: pg_deployment_b=>db_name_c\"",
        }
    );
}

#[test]
fn test_application_db_wiring_no_db_shards() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
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
  pg_shard_wiring: '
    a : testdb=>testdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringApplicationHasNoDbShards {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "a : testdb=>testdb_a".to_string(),
            explanation: "This application has no Postgres shards that need to be wired",
        }
    );
}

#[test]
fn test_application_db_wiring_invalid_app_shard() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
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
  pg_shard_wiring: '
    non_existing_shard : testdb=>testdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringApplicationHasNoDbShard {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "non_existing_shard : testdb=>testdb_a".to_string(),
            missing_application_db_shard: "non_existing_shard".to_string(),
            valid_app_db_shards: vec!["a".to_string()],
            explanation: "Specified application Postgres shard is missing",
        }
    );
}

#[test]
fn test_application_db_wiring_invalid_target_deployment_shard() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
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
  pg_shard_wiring: '
    a: non=>existing
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringTargetDbDeploymentDoesntExist {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "a: non=>existing".to_string(),
            missing_pg_deployment: "non=>existing".to_string(),
            explanation: "Specified Postgres deployment with schema doesn't exist",
        }
    );
}

#[test]
fn test_application_pg_wiring_schema_mismatch() {
    assert_eq!(
        PlatformValidationError::ApplicationPgWiringSchemaMismatch {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "a: testdb=>other_schema_a".to_string(),
            application_expected_pg_schema: "testdb".to_string(),
            target_deployment_pg_schema: "other_schema".to_string(),
            explanation: "Application expected Postgres schema mismatches wired shard actual shard schema",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
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
  pg_shard_wiring: '
    a: testdb=>other_schema_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
        },
    ]
  }
]

DATA STRUCT pg_schema [
  {
    schema_name: other_schema,
    WITH pg_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE bar (
            id INT PRIMARY KEY
          );
        ",
        downgrade: "DROP TABLE bar;",
      }
    ]
  },
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: other_schema_a,
        pg_schema: other_schema,
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
        ),
    );
}

#[test]
fn test_application_db_wiring_shard_defined_multiple_times() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
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
  pg_shard_wiring: '
    a: testdb=>testdb_a
    a: testdb=>testdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringApplicationShardDefinedMultipleTimes {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "a: testdb=>testdb_a".to_string(),
            redefined_app_db_shard_name: "a".to_string(),
            explanation: "Specified application Postgres shard redefined multiple times",
        }
    );
}

#[test]
fn test_application_db_wiring_shard_undefined() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
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
  pg_shard_wiring: '',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringUndefinedAppDbShard {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            undefined_application_db_shard: "a".to_string(),
            explanation: "Specified application db shard was not defined in wiring",
        }
    );
}

#[test]
fn test_application_db_wiring_shards_point_to_duplicate_db() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
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
  pg_shard_wiring: '
    a: testdb=>testdb_a
    b: testdb=>testdb_a
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
        },
        {
            shard_name: b,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgWiringDifferentAppShardsPointToSameDatabase {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            app_shard_a_name: "a".to_string(),
            app_shard_b_name: "b".to_string(),
            target_physical_db_a: "testdb=>testdb_a".to_string(),
            target_physical_db_b: "testdb=>testdb_a".to_string(),
            explanation:
                "Two different db connections in application point to the same physical database",
        }
    );
}

#[test]
fn test_application_db_wiring_success() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  pg_shard_wiring: '
    a: testdb=>testdb_a
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
            ',
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
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
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    pgtest1, exclusive, 4k;
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    pgtest1, exclusive, 4k;
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
  cluster_name: default,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default,
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
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}

"#,
    );
}
