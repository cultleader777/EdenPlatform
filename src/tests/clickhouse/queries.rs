#[cfg(test)]
use crate::static_analysis::{PlatformValidationError};

#[cfg(test)]
use super::super::common;

#[test]
fn test_basic_success() {
    common::assert_platform_validation_success(
        r#"
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
"#,
    );
}

#[test]
fn test_nullable_columns_not_allowed_in_schema() {
    assert_eq!(
        PlatformValidationError::ChColumnNullableValuesNotAllowed {
            column_name: "a".to_string(),
            column_type: "Nullable(String)".to_string(),
            table_name: "foo".to_string(),
            migration_sql: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a Nullable(String)
          ) ENGINE = MergeTree() ORDER BY id;
        ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a Nullable(String)
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
        ),
    )
}

#[test]
fn test_forbidden_table_prefix() {
    assert_eq!(
        PlatformValidationError::ChTableForbiddenPrefix {
            table_name: "nats_foo".to_string(),
            forbidden_prefix: "nats_".to_string(),
            migration_sql: "
          CREATE TABLE IF NOT EXISTS nats_foo (
            id Int32,
            a Nullable(String)
          ) ENGINE = MergeTree() ORDER BY id;
        ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS nats_foo (
            id Int32,
            a Nullable(String)
          ) ENGINE = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS nats_foo;
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
        ),
    )
}

#[test]
fn test_whitespace_not_allowed_in_arguments() {
    assert_eq!(
        PlatformValidationError::ChWhitespaceForbiddenInQueryArguments {
            bad_argument: "test_arg".to_string(),
            query: "SELECT max(id) AS max_id FROM foo WHERE id > { test_arg : INT }".to_string(),
            expected_whitespace: 0,
            found_whitespace: 4,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > { test_arg : INT }",
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
"#,
        ),
    )
}

#[test]
fn test_migration_order_check() {
    assert_eq!(
        PlatformValidationError::ChMigrationsAreNotOrdered {
            previous_migration_time: 2,
            current_migration_time: 1,
            previous_migration: "\n          CREATE TABLE IF NOT EXISTS foo (\n            id Int32,\n            a String\n          ) ENGINE = MergeTree() ORDER BY id;\n        ".to_string(),
            current_migration: "\n          CREATE TABLE IF NOT EXISTS bar (\n            id Int64,\n            b Bool\n          ) engine = MergeTree() ORDER BY id;\n        ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 2,
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
        time: 1,
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
        ),
    )
}

#[test]
fn test_query_has_no_tests() {
    assert_eq!(
        PlatformValidationError::ChQueryHasNoTests {
            ch_schema: "testch".to_string(),
            query_name: "max_id_from_foo".to_string(),
            original_query: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32}".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_create_table_no_repicated_merge_tree() {
    assert_eq!(
        PlatformValidationError::ChMigrationUseUnreplicatedMergeTreesInEpl {
            database: "testch".to_string(),
            migration_sql: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String
          ) ENGINE = ReplicatedMergeTree() ORDER BY id;
        ".to_string(),
            table_engine: "ReplicatedMergeTree".to_string(),
            expected_table_engine: "MergeTree".to_string(),
            explanation: "In EPL Clickhouse schema migrations use unreplicated *MergeTree table engines. All these are converted to Replicated* engines automatically in production.".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          ) ENGINE = ReplicatedMergeTree() ORDER BY id;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_create_table_unsupported_engine() {
    assert_eq!(
        PlatformValidationError::ChMigrationUnsupportedTableEngine {
            database: "testch".to_string(),
            migration_sql: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String
          ) ENGINE = Memory() ORDER BY id;
        ".to_string(),
            table_engine: "Memory".to_string(),
            supported_table_engines: vec![
                "MergeTree".to_string(),
                "ReplacingMergeTree".to_string(),
                "SummingMergeTree".to_string(),
                "AggregatingMergeTree".to_string(),
                "CollapsingMergeTree".to_string(),
                "VersionedCollapsingMergeTree".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          ) ENGINE = Memory() ORDER BY id;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_create_table_doesnt_have_if_not_exists() {
    assert_eq!(
        PlatformValidationError::ChMigrationCreateTableMustHaveIfNotExistsStatement {
            database: "testch".to_string(),
            migration_sql: "
          CREATE TABLE foo (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id;
        ".to_string(),
            expected_create_table_statement: "CREATE TABLE IF NOT EXISTS ...".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE foo (
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_drop_table_doesnt_have_if_exists() {
    assert_eq!(
        PlatformValidationError::ChMigrationDropTableMustHaveIfExistsStatement {
            database: "testch".to_string(),
            migration_sql: "
          DROP TABLE foo;
        ".to_string(),
            expected_drop_table_statement: "DROP TABLE IF EXISTS ...".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          DROP TABLE foo;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_create_view_doesnt_have_if_not_exists() {
    assert_eq!(
        PlatformValidationError::ChMigrationCreateViewMustHaveIfNotExistsStatement {
            database: "testch".to_string(),
            migration_sql: "
          CREATE VIEW foo AS SELECT 1;
        ".to_string(),
            expected_create_view_statement: "CREATE VIEW IF NOT EXISTS ...".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE VIEW foo AS SELECT 1;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_create_mat_view_doesnt_have_if_not_exists() {
    assert_eq!(
        PlatformValidationError::ChMigrationCreateMaterializedViewMustHaveIfNotExistsStatement {
            database: "testch".to_string(),
            migration_sql: "
          CREATE MATERIALIZED VIEW foo AS SELECT 1;
        ".to_string(),
            expected_create_materialized_view_statement: "CREATE MATERIALIZED VIEW IF NOT EXISTS ...".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE MATERIALIZED VIEW foo AS SELECT 1;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_drop_view_doesnt_have_if_not_exists() {
    assert_eq!(
        PlatformValidationError::ChMigrationDropViewMustHaveIfExistsStatement {
            database: "testch".to_string(),
            migration_sql: "
          DROP VIEW foo;
        ".to_string(),
            expected_drop_view_statement: "DROP VIEW IF EXISTS ...".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE VIEW IF NOT EXISTS foo AS SELECT 1;
        ",
        downgrade: "
          DROP VIEW foo;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_renames_unsupported() {
    assert_eq!(
        PlatformValidationError::ChMigrationDoesntSupportRenamesOrExchanges {
            database: "testch".to_string(),
            migration_sql: "
          RENAME foo TO moo;
        ".to_string(),
            unsupported_statements: vec![
                "RENAME".to_string(),
                "EXCHANGE".to_string(),
            ]
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          RENAME foo TO moo;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_exchange_unsupported() {
    assert_eq!(
        PlatformValidationError::ChMigrationDoesntSupportRenamesOrExchanges {
            database: "testch".to_string(),
            migration_sql: "
          EXCHANGE TABLES foo AND moo;
        ".to_string(),
            unsupported_statements: vec![
                "RENAME".to_string(),
                "EXCHANGE".to_string(),
            ]
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          EXCHANGE TABLES foo AND moo;
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
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_cant_parse_test_dataset() {
    assert_eq!(
        PlatformValidationError::ChCantDeserializeTestDataset {
            ch_schema: "testch".to_string(),
            error: "invalid type: sequence, expected a map at line 2 column 11".to_string(),
            input_dataset_name: "default".to_string(),
            input_data: "
          - id: 7
            a: hello
        ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          - id: 7
            a: hello
        "
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_cant_perform_migration_upgrade() {
    let err =
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE dookie;
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
        );

    match err {
        PlatformValidationError::ChMigrationUpgradeError { ch_schema, upgrade_sql, upgrade_time, error } => {
            assert_eq!(ch_schema, "testch");
            assert_eq!(upgrade_sql, "
          CREATE dookie;
        ");
            assert_eq!(upgrade_time, 1);
            // string includes clickhouse version which we don't want to exactly compare with
            assert!(error.contains("Syntax error: failed at position 8 ('dookie')"))
        }
        err => panic!("Unexpected error: {:?}", err)
    }
}

#[test]
fn test_reserved_table_name() {
    assert_eq!(
        PlatformValidationError::ChReservedTableName {
            ch_schema: "testch".to_string(),
            table_name: "epl_schema_migrations".to_string(),
            upgrade_sql: "
          CREATE TABLE IF NOT EXISTS epl_schema_migrations (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id;
        ".to_string(),
            upgrade_time: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS epl_schema_migrations (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id;
        ",
        downgrade: "
          DROP TABLE IF EXISTS epl_schema_migrations;
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
        ),
    )
}

#[test]
fn test_dataset_never_tested() {
    assert_eq!(
        PlatformValidationError::ChDatasetIsNeverTested {
            ch_schema: "testch".to_string(),
            input_dataset_name: "default".to_string(),
            minimum_dataset_time: 3,
            maximum_migration_time: 2,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        min_time: 3,
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
        ),
    )
}

#[test]
fn test_dataset_contains_non_existing_table() {
    assert_eq!(
        PlatformValidationError::ChDatasetTableNotFoundInSchema {
            ch_schema: "testch".to_string(),
            input_dataset_name: "default".to_string(),
            table_tried_to_insert: "non_existant".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          non_existant:
          - field: 123
        "
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_dataset_non_existing_column() {
    assert_eq!(
        PlatformValidationError::ChDatasetTableColumnNotFoundInSchema {
            ch_schema: "testch".to_string(),
            input_dataset_name: "default".to_string(),
            table: "foo".to_string(),
            table_column_tried_to_insert: "non_existant".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
            non_existant: true
          bar:
          - id: 1
            b: y
        "
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_dataset_column_is_not_insertable() {
    assert_eq!(
        PlatformValidationError::ChDatasetTableColumnIsNotAllowedToBeInserted {
            ch_schema: "testch".to_string(),
            input_dataset_name: "default".to_string(),
            table: "foo".to_string(),
            table_column_tried_to_insert: "a".to_string(),
            explanation: "Column is either MATERIALIZED or ALIAS and is computed and cannot be inserted by a dataset".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String MATERIALIZED toString(id)
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
        ),
    )
}

#[test]
fn test_dataset_bad_column_value() {
    assert_eq!(
        PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
            ch_schema: "testch".to_string(),
            input_dataset_name: "default".to_string(),
            table: "foo".to_string(),
            column: "id".to_string(),
            column_value: "bad_value".to_string(),
            type_tried_to_parse_to: "Int32".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          - id: bad_value
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
        ),
    )
}

#[test]
fn test_dataset_unsupported_col_type() {
    assert_eq!(
        PlatformValidationError::ChDatasetUnsupportedColumnType {
            database: "testch".to_string(),
            input_dataset_name: "default".to_string(),
            table: "foo".to_string(),
            column: "b".to_string(),
            column_value: "bad_value".to_string(),
            column_type: "Map(String, UInt64)".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            id Int32,
            a String,
            b Map(String,UInt64)
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
            b: bad_value
          bar:
          - id: 1
            b: y
        "
      }
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_cant_parse_test_arguments() {
    assert_eq!(
        PlatformValidationError::ChCantParseTestArguments {
            input_data: "bad value".to_string(),
            error: "invalid type: string \"bad value\", expected a map".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          arguments: 'bad value',
          outputs: "
          - max_id: 7
          "
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_cant_parse_test_outputs() {
    assert_eq!(
        PlatformValidationError::ChCantParseTestOutputs {
            input_data: "bad value".to_string(),
            error: "invalid type: string \"bad value\", expected a sequence".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          arguments: 'test_arg: 5',
          outputs: 'bad value',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_bad_original_arguments_curly_bracket() {
    assert_eq!(
        PlatformValidationError::ChUnsupportedArgumentType {
            allowed_types: vec![
                "Int32",
                "Int64",
                "Int128",
                "Int256",
                "Float32",
                "Float64",
                "Bool",
                "String",
                "DateTime",
            ],
            unsupported_type: "Map".to_string(),
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Map}".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Map}",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 5',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_bad_original_arguments_question_mark() {
    assert_eq!(
        PlatformValidationError::ChOriginalQueryParametersAreNotAllowed {
            found_forbidden_value: "?".to_string(),
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > ?".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > ?",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 5',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_bad_query_arg_type() {
    assert_eq!(
        PlatformValidationError::ChUnsupportedArgumentType {
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:HUMPTY}".to_string(),
            unsupported_type: "HUMPTY".to_string(),
            allowed_types: vec!["Int32", "Int64", "Int128", "Int256", "Float32", "Float64", "Bool", "String", "DateTime", "Date"],
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:HUMPTY}",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 5',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_query_diverging_type() {
    assert_eq!(
        PlatformValidationError::ChDivergingTypesForSameArgument {
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32} AND id < {test_arg:Bool}".to_string(),
            argument_name: "test_arg".to_string(),
            type_a: "Int32".to_string(),
            type_b: "Bool".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32} AND id < {test_arg:Bool}",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 5',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_query_unspecified_type() {
    assert_eq!(
        PlatformValidationError::ChArgumentTypeUnspecifiedAtLeastOnce {
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg}".to_string(),
            argument_name: "test_arg".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg}",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 5',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_query_cant_parse_opt_fields() {
    assert_eq!(
        PlatformValidationError::ChQueryOptFieldMustBeSnakeCase {
            ch_schema: "testch".to_string(),
            query_name: "max_id_from_foo".to_string(),
            bad_optional_field: "@".to_string(),
            optional_fields: "@".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        opt_fields: '@',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 5',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_query_duplicate_opt_fields() {
    assert_eq!(
        PlatformValidationError::ChQueryOptFieldDuplicate {
            ch_schema: "testch".to_string(),
            query_name: "max_id_from_foo".to_string(),
            duplicate_optional_field: "max_id".to_string(),
            optional_fields: "max_id max_id".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        opt_fields: 'max_id max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 5',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_query_cant_parse_arg_to_type() {
    assert_eq!(
        PlatformValidationError::ChQueryCannotParseArgumentToType {
            database: "testch".to_string(),
            query_name: "max_id_from_foo".to_string(),
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            argument_expected_type: "i32".to_string(),
            argument_name: "test_arg".to_string(),
            argument_value: "abc".to_string(),
            arguments: "test_arg: abc".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          arguments: 'test_arg: abc',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_query_cant_find_arg_in_set() {
    assert_eq!(
        PlatformValidationError::ChQueryArgumentNotFoundInTest {
            database: "testch".to_string(),
            query_name: "max_id_from_foo".to_string(),
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            arguments: "test_argz: 123".to_string(),
            argument_not_found: "test_arg".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          arguments: 'test_argz: 123',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_query_unused_argument() {
    assert_eq!(
        PlatformValidationError::ChQueryArgumentNotUsedInQuery {
            ch_schema: "testch".to_string(),
            query_name: "max_id_from_foo".to_string(),
            query_expression: "SELECT max(id) AS max_id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            arguments: "
            test_arg: 123
            unused: 0
          ".to_string(),
            argument_not_used: "unused".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
          arguments: '
            test_arg: 123
            unused: 0
          ',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        ),
    )
}

#[test]
fn test_bad_query() {
    let err =
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_expression: "SELECT max(id) AS max_id FROM who WHERE id > {test_arg:Int32}",
        opt_fields: 'max_id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 123',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        );

    match err {
        PlatformValidationError::ChQueryError { ch_schema, query_name, original_query, interpolated_query, query_arguments, error } => {
            assert_eq!(ch_schema, "testch");
            assert_eq!(query_name, "max_id_from_foo");
            assert_eq!(interpolated_query, "SELECT max(id) AS max_id FROM who WHERE id > {test_arg:Int32}");
            assert_eq!(original_query, "SELECT max(id) AS max_id FROM who WHERE id > {test_arg:Int32}");
            assert_eq!(query_arguments, "test_arg: 123");
            assert!(error.contains("Unknown table expression identifier 'who'"));
        }
        err => panic!("Unexpected error: {err}")
    }
}

#[test]
fn test_query_empty_rows() {
    assert_eq!(
        PlatformValidationError::ChQueryErrorEmptyRowSet {
            ch_schema: "testch".to_string(),
            query_name: "some_id_from_foo".to_string(),
            original_query: "SELECT id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            interpolated_query: "SELECT id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            query_arguments: "test_arg: 123".to_string(),
            test_dataset_name: "default".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_name: "some_id_from_foo",
        query_expression: "SELECT id FROM foo WHERE id > {test_arg:Int32}",
        opt_fields: 'id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 123',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_query_no_opt_field() {
    assert_eq!(
        PlatformValidationError::ChQueryOptFieldDoesntExistInQueryResults {
            ch_schema: "testch".to_string(),
            query_name: "some_id_from_foo".to_string(),
            original_query: "SELECT id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            bad_optional_field: "bad_field".to_string(),
            optional_fields: "bad_field id".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_name: "some_id_from_foo",
        query_expression: "SELECT id FROM foo WHERE id > {test_arg:Int32}",
        opt_fields: 'bad_field id',
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 1',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_query_non_snake_case_field() {
    assert_eq!(
        PlatformValidationError::ChQueryInvalidOutputFieldNameFormat {
            ch_schema: "testch".to_string(),
            query_name: "some_id_from_foo".to_string(),
            original_query: "SELECT id AS `@` FROM foo WHERE id > {test_arg:Int32}".to_string(),
            interpolated_query: "SELECT id AS `@` FROM foo WHERE id > {test_arg:Int32}".to_string(),
            output_field_name: "@".to_string(),
            output_field_type: "Int32".to_string(),
            output_field_index: 1,
            expectation: "Field should be snake case",
            query_arguments: "test_arg: 1".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_name: "some_id_from_foo",
        query_expression: "SELECT id AS `@` FROM foo WHERE id > {test_arg:Int32}",
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 1',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_query_unsupported_type() {
    assert_eq!(
        PlatformValidationError::ChQueryUnsupportedTypeError {
            ch_schema: "testch".to_string(),
            query_name: "bad_type".to_string(),
            original_query: "SELECT CAST(([1], ['Ready']), 'Map(UInt8, String)') AS bad_type".to_string(),
            interpolated_query: "SELECT CAST(([1], ['Ready']), 'Map(UInt8, String)') AS bad_type".to_string(),
            output_field_name: "bad_type".to_string(),
            output_field_type: "Map(UInt8, String)".to_string(),
            output_field_index: 1,
            query_arguments: "".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_name: "bad_type",
        query_expression: "SELECT CAST(([1], ['Ready']), 'Map(UInt8, String)') AS bad_type",
        WITH ch_query_test {
          test_dataset: default,
          arguments: '',
          outputs: '- max_id: 7',
        }
      },
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_query_unexpected_ouputs() {
    assert_eq!(
        PlatformValidationError::ChQueryUnexpectedOutputs {
            ch_schema: "testch".to_string(),
            query_name: "some_id_from_foo".to_string(),
            original_query: "SELECT id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            interpolated_query: "SELECT id FROM foo WHERE id > {test_arg:Int32}".to_string(),
            query_arguments: "test_arg: 1".to_string(),
            actual_outputs: "- id: '7'\n".to_string(),
            expected_outputs: "- id: '6'\n".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        query_name: "some_id_from_foo",
        query_expression: "SELECT id FROM foo WHERE id > {test_arg:Int32}",
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 1',
          outputs: '- id: 6',
        }
      },
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_bad_sql_downgrade() {
    let err =
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        downgrade: "bad sql",
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
        query_name: "some_id_from_foo",
        query_expression: "SELECT id FROM foo WHERE id > {test_arg:Int32}",
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 1',
          outputs: '- id: 7',
        }
      },
    ]
  }
]
"#,
        );

    match err {
        PlatformValidationError::ChMigrationDowngradeError { database, downgrade_sql, upgrade_time, error } => {
            assert_eq!(database, "testch");
            assert_eq!(downgrade_sql, "bad sql");
            assert_eq!(upgrade_time, 2);
            assert!(error.contains("Syntax error"))
        }
        err => panic!("Unexpected error: {err}")
    }
}

#[test]
fn test_inconsistent_sql_downgrade() {
    assert_eq!(
        PlatformValidationError::ChMigrationInconsistentDowngrade {
            database: "testch".to_string(),
            downgrade_sql: "DROP TABLE IF EXISTS boor;".to_string(),
            schema_diff: "+-----+----+--------+---+--+--+--+---+---+---+---+--+------+----------+---------+---------+------+\n \u{1b}[32m| bar | b  | Bool   | 2 |  |  |  | 0 | 0 | 0 | 0 |  | None | Some(8)  | Some(2) | Some(0) | None |\u{1b}[0m\n \u{1b}[32m+-----+----+--------+---+--+--+--+---+---+---+---+--+------+----------+---------+---------+------+\u{1b}[0m\n \u{1b}[32m| bar | id | Int64  | 1 |  |  |  | 0 | 1 | 1 | 0 |  | None | Some(64) | Some(2) | Some(0) | None |\u{1b}[0m\n \u{1b}[32m+-----+----+--------+---+--+--+--+---+---+---+---+--+------+----------+---------+---------+------+\u{1b}[0m\n| foo | a  | String | 2 |  |  |  | 0 | 0 | 0 | 0 |  | None | None     | None    | None    | None |\n+-----+----+--------+---+--+--+--+---+---+---+---+--+------+----------+---------+---------+------+\n| foo | id | Int32  | 1 |  |  |  | 0 | 1 | 1 | 0 |  | None | Some(32) | Some(2) | Some(0) | None |\n+-----+----+--------+---+--+--+--+---+---+---+---+--+------+----------+---------+---------+------+".to_string(),
            upgrade_sql: "
          CREATE TABLE IF NOT EXISTS bar (
            id Int64,
            b Bool
          ) engine = MergeTree() ORDER BY id;
        ".to_string(),
            upgrade_time: 2,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
        downgrade: "DROP TABLE IF EXISTS boor;",
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
        query_name: "some_id_from_foo",
        query_expression: "SELECT id FROM foo WHERE id > {test_arg:Int32}",
        WITH ch_query_test {
          test_dataset: default,
          arguments: 'test_arg: 1',
          outputs: '- id: 7',
        }
      },
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_migration_non_snake_case_column() {
    assert_eq!(
        PlatformValidationError::ChColumnNameIsNotSnakeCase {
            table_name: "foo".to_string(),
            bad_column_name: "Id".to_string(),
            migration_sql: "
          CREATE TABLE IF NOT EXISTS foo (
            Id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY Id;
        ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            Id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY Id;
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
        ),
    )
}

#[test]
fn test_migration_non_snake_case_table() {
    assert_eq!(
        PlatformValidationError::ChTableNameIsNotSnakeCase {
            bad_table_name: "Foo".to_string(),
            migration_sql: "
          CREATE TABLE IF NOT EXISTS Foo (
            id Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id;
        ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS Foo (
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
  }
]
"#,
        ),
    )
}

#[test]
fn test_migration_no_backticks_allowed_in_migration() {
    assert_eq!(
        PlatformValidationError::ChMigrationContainsBacktick {
            forbidden_character: "`".to_string(),
            input_sql: "CREATE TABLE IF NOT EXISTS foo (
            `id` Int32,
            a String
          ) ENGINE = MergeTree() ORDER BY id".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE IF NOT EXISTS foo (
            `id` Int32,
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
  }
]
"#,
        ),
    )
}
