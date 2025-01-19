#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use edendb::checker::errors::DatabaseValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_application_non_existing_query() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              non_existing_query
            '
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
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgShardQueryNotFoundInPgSchema {
            queries_src: "
              non_existing_query
            ".to_string(),
            query_not_found: "non_existing_query".to_string(),
            application_pg_shard: "a".to_string(),
            application_pg_schema: "testdb".to_string(),
            application: "hello-world".to_string(),
        }
    );
}

#[test]
fn test_application_duplicate_query() {
    assert_eq!(
        PlatformValidationError::ApplicationPgShardQueryDefinedTwice {
            used_queries_src: "
              existing_query_a
              existing_query_a
            ".to_string(),
            used_query_defined_twice: "existing_query_a".to_string(),
            application_pg_shard: "a".to_string(),
            application_pg_schema: "testdb".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
              existing_query_a
            '
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
      {
        query_name: existing_query_b,
        query_expression: "INSERT INTO foo(id) VALUES({test_arg:INT}) RETURNING id",
        is_mutating: true,
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 7",
          outputs: "
          - id: 7
          "
        }
      },
    ]
    WITH pg_mutator [
      {
        mutator_name: existing_mutator,
        mutator_expression: "INSERT INTO foo(id) VALUES({test_arg:INT})",
        WITH pg_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: "test_arg: 4",
        }
      },
    ]
    WITH pg_transaction [
      {
        transaction_name: existing_transaction,
        steps: '
          existing_query_b
          existing_query_a[]
          existing_mutator[]
        '
      }
    ]
  }
]
"#,
        ),
    );
}

#[test]
fn test_application_non_existing_mutator() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_mutators: '
              non_existing_mutator
            '
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
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgShardMutatorNotFoundInPgSchema {
            mutators_src: "
              non_existing_mutator
            ".to_string(),
            mutator_not_found: "non_existing_mutator".to_string(),
            application_pg_shard: "a".to_string(),
            application_pg_schema: "testdb".to_string(),
            application: "hello-world".to_string(),
        }
    );
}

#[test]
fn test_application_duplicate_mutator() {
    assert_eq!(
        PlatformValidationError::ApplicationPgShardMutatorDefinedTwice {
            used_mutators_src: "
              existing_mutator
              existing_mutator
            ".to_string(),
            used_mutator_defined_twice: "existing_mutator".to_string(),
            application_pg_shard: "a".to_string(),
            application_pg_schema: "testdb".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_mutators: '
              existing_mutator
              existing_mutator
            '
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
      {
        query_name: existing_query_b,
        query_expression: "INSERT INTO foo(id) VALUES({test_arg:INT}) RETURNING id",
        is_mutating: true,
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 7",
          outputs: "
          - id: 7
          "
        }
      },
    ]
    WITH pg_mutator [
      {
        mutator_name: existing_mutator,
        mutator_expression: "INSERT INTO foo(id) VALUES({test_arg:INT})",
        WITH pg_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: "test_arg: 4",
        }
      },
    ]
    WITH pg_transaction [
      {
        transaction_name: existing_transaction,
        steps: '
          existing_query_b
          existing_query_a[]
          existing_mutator[]
        '
      }
    ]
  }
]
"#,
        ),
    );
}

#[test]
fn test_application_non_existing_transaction() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_transactions: '
              non_existing_transaction
            '
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
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationPgShardTransactionNotFoundInPgSchema {
            transactions_src: "
              non_existing_transaction
            ".to_string(),
            transaction_not_found: "non_existing_transaction".to_string(),
            application_pg_shard: "a".to_string(),
            application_pg_schema: "testdb".to_string(),
            application: "hello-world".to_string(),
        }
    );
}

#[test]
fn test_application_duplicate_transaction() {
    assert_eq!(
        PlatformValidationError::ApplicationPgShardTransactionDefinedTwice {
            used_transactions_src: "
              existing_transaction
              existing_transaction
            ".to_string(),
            used_transaction_defined_twice: "existing_transaction".to_string(),
            application_pg_shard: "a".to_string(),
            application_pg_schema: "testdb".to_string(),
            application: "hello-world".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_transactions: '
              existing_transaction
              existing_transaction
            '
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
      {
        query_name: existing_query_b,
        query_expression: "INSERT INTO foo(id) VALUES({test_arg:INT}) RETURNING id",
        is_mutating: true,
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 7",
          outputs: "
          - id: 7
          "
        }
      },
    ]
    WITH pg_mutator [
      {
        mutator_name: existing_mutator,
        mutator_expression: "INSERT INTO foo(id) VALUES({test_arg:INT})",
        WITH pg_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: "test_arg: 4",
        }
      },
    ]
    WITH pg_transaction [
      {
        transaction_name: existing_transaction,
        steps: '
          existing_query_b
          existing_query_a[]
          existing_mutator[]
        '
      }
    ]
  }
]
"#,
        ),
    );
}

#[test]
fn test_application_existing_queries() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
        {
            shard_name: a,
            pg_schema: testdb,
            used_queries: '
              existing_query_a
              existing_query_b
            ',
            used_mutators: '
              existing_mutator
            ',
            used_transactions: '
              existing_transaction
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
      {
        query_name: existing_query_b,
        query_expression: "INSERT INTO foo(id) VALUES({test_arg:INT}) RETURNING id",
        is_mutating: true,
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 7",
          outputs: "
          - id: 7
          "
        }
      },
    ]
    WITH pg_mutator [
      {
        mutator_name: existing_mutator,
        mutator_expression: "INSERT INTO foo(id) VALUES({test_arg:INT})",
        WITH pg_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: "test_arg: 4",
        }
      },
    ]
    WITH pg_transaction [
      {
        transaction_name: existing_transaction,
        steps: '
          existing_query_b
          existing_query_a[]
          existing_mutator[]
        '
      }
    ]
  }
]
"#,
    );
}

#[test]
fn test_application_invalid_build_environment_kind() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: wrong-backend,
    build_environment: test_frontend,
  }
]

DATA STRUCT rust_compilation_environment {
    env_name: test_frontend,
    environment_kind: frontend_app,
    WITH rust_crate_version [
        {
            crate_name: yew,
            version: 0.20.0,
            features: csr,
        },
    ]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationInvalidBuildEnvironmentKind {
            application_name: "wrong-backend".to_string(),
            expected_compilation_environment_kind: "backend_app".to_string(),
            found_compilation_environment_kind: "frontend_app".to_string(),
        }
    );
}

#[test]
fn test_frontend_and_backend_deployments_name_clash() {
    let err = common::assert_eden_db_error_wcustom_data(
        r#"
DATA STRUCT backend_application {
  application_name: backend-app,
}

DATA STRUCT frontend_application {
  application_name: frontend-app,
}

DATA STRUCT backend_application_deployment {
    deployment_name: duplicate,
    application_name: backend-app,
}

DATA STRUCT frontend_application_deployment {
    deployment_name: duplicate,
    application_name: frontend-app,
}
"#,
    );
    assert_eq!(
        err,
        DatabaseValidationError::DuplicatePrimaryKey {
            table_name: "unique_deployment_names".to_string(),
            value: "duplicate".to_string(),
        }
    );
}

#[test]
fn test_frontend_and_backend_application_name_clash() {
    let err = common::assert_eden_db_error_wcustom_data(
        r#"
DATA STRUCT backend_application {
  application_name: duplicate-app-name,
}

DATA STRUCT frontend_application {
  application_name: duplicate-app-name,
}
"#,
    );
    assert_eq!(
        err,
        DatabaseValidationError::DuplicatePrimaryKey {
            table_name: "unique_application_names".to_string(),
            value: "duplicate-app-name".to_string(),
        }
    );
}

#[test]
fn test_application_deployment_too_few_servers() {
    assert_eq!(
        PlatformValidationError::RegionWithDeploymentsHasLessThanFourServers {
            region: "us-west".to_string(),
            servers: vec!["server-a".to_string(), "server-b".to_string()],
            minimum: 4,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
}

DEFAULTS {
    region.tld epl-infra.net,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc dc1,
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

DATA STRUCT region {
    region_name: us-west,
    is_dns_master: true,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/',
    subdomain: 'www',
    tld: epl-infra.net,
    endpoint_list: '
      primary
    ',
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_dns_master, is_dns_slave, is_vpn_gateway) {
  server-a, eth0, true, true, false, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
    eth1, internet, 77.77.77.10;
  };
  server-b, eth0, true, false, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
    eth1, internet, 77.77.77.11;
  };
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
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
]

"#,
    ));
}

#[test]
fn test_application_deployment_too_few_ingresses() {
    assert_eq!(
        PlatformValidationError::RegionWithIngressesHasLessThanTwoIngressServers {
            region: "us-west".to_string(),
            ingress_servers: vec!["server-b".to_string()],
            minimum: 2,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
}

DEFAULTS {
    region.tld epl-infra.net,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc dc1,
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

DATA STRUCT region {
    region_name: us-west,
    is_dns_master: true,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/',
    subdomain: 'www',
    tld: epl-infra.net,
    endpoint_list: '
      primary
    ',
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_dns_master, is_dns_slave, is_ingress, is_vpn_gateway) {
  server-a, eth0, true, true, false, true, false, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
    eth1, internet, 77.77.77.10;
  };
  server-b, eth0, true, false, true, false, true, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
    eth1, internet, 77.77.77.11;
  };
  server-c, eth0, true, false, true, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
    eth1, internet, 77.77.77.12;
  };
  server-d, eth0, true, false, true, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
    eth1, internet, 77.77.77.13;
  };
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
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

"#,
    ));
}

#[test]
fn test_application_deployment_with_forced_ipv6_has_no_ipv6_ingress() {
    assert_eq!(
        PlatformValidationError::RegionWithIngressesHasInconsistentIpV6IngressCount {
            region: "us-west".to_string(),
            ipv4_ingress_servers: vec![
                "server-a".to_string(),
                "server-b".to_string(),
            ],
            ipv6_ingress_servers: vec![],
            expected_ipv6_ingress_count: 2,
            is_ipv6_support_enabled: true,
            is_ipv6_support_forced: true,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    force_ipv6: true,
}

DEFAULTS {
    region.tld epl-infra.net,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc dc1,
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

DATA STRUCT region {
    region_name: us-west,
    is_dns_master: true,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/',
    subdomain: 'www',
    tld: epl-infra.net,
    endpoint_list: '
      primary
    ',
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_dns_master, is_dns_slave, is_ingress, is_vpn_gateway) {
  server-a, eth0, true, true, false, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
    eth1, internet, 77.77.77.10;
  };
  server-b, eth0, true, false, true, false, true, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
    eth1, internet, 77.77.77.11;
  };
  server-c, eth0, true, false, true, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
    eth1, internet, 77.77.77.12;
  };
  server-d, eth0, true, false, true, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
    eth1, internet, 77.77.77.13;
  };
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
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

"#,
    ));
}

#[test]
fn test_application_deployment_has_too_few_ipv6_ingress() {
    assert_eq!(
        PlatformValidationError::RegionWithIngressesHasInconsistentIpV6IngressCount {
            region: "us-west".to_string(),
            ipv4_ingress_servers: vec![
                "server-a".to_string(),
                "server-b".to_string(),
            ],
            ipv6_ingress_servers: vec![
                "server-a".to_string(),
            ],
            expected_ipv6_ingress_count: 2,
            is_ipv6_support_enabled: true,
            is_ipv6_support_forced: false,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    force_ipv6: false,
}

DEFAULTS {
    region.tld epl-infra.net,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc dc1,
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

DATA STRUCT region {
    region_name: us-west,
    is_dns_master: true,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/',
    subdomain: 'www',
    tld: epl-infra.net,
    endpoint_list: '
      primary
    ',
}

DATA server(hostname, ssh_interface, public_ipv6_address, is_consul_master, is_nomad_master, is_vault_instance, is_dns_master, is_dns_slave, is_ingress, is_vpn_gateway) {
  server-a, eth0, '2a03:2880:f32e:3:face:b00c:0:1', true, true, false, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
    eth1, internet, 77.77.77.10;
  };
  server-b, eth0, '', true, false, true, false, true, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
    eth1, internet, 77.77.77.11;
  };
  server-c, eth0, '', true, false, true, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
    eth1, internet, 77.77.77.12;
  };
  server-d, eth0, '', true, false, true, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
    eth1, internet, 77.77.77.13;
  };
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
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

"#,
    ));
}

#[test]
fn test_application_deployment_ingress_no_internet() {
    assert_eq!(
        PlatformValidationError::ServerMarkedAsIngressHasNoPublicIpInterface {
            datacenter: "dc1".to_string(),
            server: "server-d".to_string(),
            missing_network_interface: "internet".to_string(),
            existing_network_interfaces: vec!["lan".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
    region.tld epl-infra.net,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc dc1,
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

DATA STRUCT region {
    region_name: us-west,
    is_dns_master: true,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_dns_master, is_dns_slave, is_ingress, is_vpn_gateway) {
  server-a, eth0, true, true, false, true, false, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
    eth1, internet, 77.77.77.10;
  };
  server-b, eth0, true, false, true, false, true, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
    eth1, internet, 77.77.77.11;
  };
  server-c, eth0, true, false, true, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
    eth1, internet, 77.77.77.12;
  };
  server-d, eth0, true, false, true, false, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  };
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
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

"#,
    ));
}
