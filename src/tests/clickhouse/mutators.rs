#[cfg(test)]
use crate::static_analysis::{PlatformValidationError};

#[cfg(test)]
use super::super::common;

#[test]
fn test_mutator_basic_success() {
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
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
    ]
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: '
            bar:
            - id: 7
          '
        }
      },
    ]
  }
]
"#,
    );
}

#[test]
fn test_mutator_has_no_tests() {
    assert_eq!(
        PlatformValidationError::ChMutatorHasNoTests {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
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
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_mutator_undefined_resulting_dataset() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetForMutatorTestIsUndefined {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "".to_string(),
            ch_mutator_test_arguments: "min_id: 0".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          resulting_data: '',
          arguments: 'min_id: 0',
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
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
    ]
  }
]
"#,
        )
    );
}

#[test]
fn test_mutator_bad_argument_type() {
    assert_eq!(
        PlatformValidationError::ChQueryCannotParseArgumentToType {
            database: "testch".to_string(),
            query_name: "put_ids_from_foo_to_bar".to_string(),
            query_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}".to_string(),
            argument_expected_type: "i32".to_string(),
            argument_name: "min_id".to_string(),
            arguments: "min_id: abc".to_string(),
            argument_value: "abc".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: abc',
          resulting_data: '
            bar:
            - id: 7
          '
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_diverging_types_same_argument() {
    assert_eq!(
        PlatformValidationError::ChDivergingTypesForSameArgument {
            query_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32} AND id > {min_id:Int64}".to_string(),
            argument_name: "min_id".to_string(),
            type_a: "Int32".to_string(),
            type_b: "Int64".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32} AND id > {min_id:Int64}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: '
            bar:
            - id: 7
          '
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_arguments_filter_out_rows() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetRowsAreNotFoundInTableAfterMutatorExecution {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            rows_not_found_after_mutator_execution: "- id: '7'\n".to_string(),
            resulting_data_table: "bar".to_string(),
            resulting_data: "
            bar:
            - id: 7
          ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 8',
          resulting_data: '
            bar:
            - id: 7
          '
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_invalid_result_data() {
    assert_eq!(
        PlatformValidationError::ChCantDeserializeMutatorResultingData {
            ch_schema: "testch".to_string(),
            error: "invalid type: string \"hoo\", expected a map".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            mutator_test_data: "hoo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'hoo'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_result_data_no_such_table() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetTableDoesntExist {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "moo: [{id: 123}]".to_string(),
            resulting_data_non_existing_table: "moo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'moo: [{id: 123}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_result_data_row_is_empty() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetTableRowIsEmpty {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "foo: [{}]".to_string(),
            resulting_data_table: "foo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'foo: [{}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_result_data_column_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetTableColumnDoesntExist {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "foo: [{woo: 123}]".to_string(),
            resulting_data_table: "foo".to_string(),
            resulting_data_non_existing_column: "woo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'foo: [{woo: 123}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_result_data_column_bad_type_conversion() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "foo: [{id: abc}]".to_string(),
            resulting_data_table: "foo".to_string(),
            resulting_data_column: "id".to_string(),
            resulting_data_column_value: "abc".to_string(),
            type_tried_to_parse_to: "Int32".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'foo: [{id: abc}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_cant_parse_test_arguments() {
    assert_eq!(
        PlatformValidationError::ChCantParseTestArguments {
            input_data: "huh".to_string(),
            error: "invalid type: string \"huh\", expected a map".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'huh',
          resulting_data: 'foo: [{id: 1}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_result_data_exists_before_mutation() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetRowIsFoundInTestDatasetBeforeMutatorIsExecuted {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "foo: [{id: 7}]".to_string(),
            resulting_data_table: "foo".to_string(),
            row_found_before_mutator_execution: "id: '7'\n".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'foo: [{id: 7}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
        '
      }
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
fn test_mutator_result_data_found_more_than_once_in_result() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetRowFoundMoreThanOnceInTable {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "bar: [{a: hello}]".to_string(),
            resulting_data_table: "bar".to_string(),
            resulting_data_ambigous_row: "a: hello\n".to_string(),
            output_dataset_matching_rows: "- a: hello\n  id: '7'\n- a: hello\n  id: '8'\n".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id, a FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'bar: [{a: hello}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
          - id: 8
            a: hello
        '
      }
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
            a String
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
fn test_mutator_result_data_not_found_after_execution() {
    assert_eq!(
        PlatformValidationError::ChResultingDatasetRowsAreNotFoundInTableAfterMutatorExecution {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            resulting_data: "bar: [{id: 9}, {id: 10}]".to_string(),
            resulting_data_table: "bar".to_string(),
            rows_not_found_after_mutator_execution: "- id: '9'\n- id: '10'\n".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id, a FROM foo WHERE id > {min_id:Int32}",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'bar: [{id: 9}, {id: 10}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
          - id: 8
            a: hello
        '
      }
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
            a String
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
fn test_mutator_query_returns_rows() {
    assert_eq!(
        PlatformValidationError::ChMutatorCannotReturnAnyRows {
            ch_schema: "testch".to_string(),
            mutator_name: "put_ids_from_foo_to_bar".to_string(),
            original_query: "SELECT 123".to_string(),
            interpolated_query: "SELECT 123".to_string(),
            query_arguments: "".to_string(),
            returned_rows_count: 1,
            returned_rows: vec![vec!["123".to_string()]],
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "SELECT 123",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: '',
          resulting_data: 'bar: [{id: 9}, {id: 10}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
          - id: 8
            a: hello
        '
      }
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
            a String
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
fn test_mutator_unused_argument_in_query() {
    assert_eq!(
        PlatformValidationError::ChQueryArgumentNotUsedInQuery {
            ch_schema: "testch".to_string(),
            query_expression: "INSERT INTO bar SELECT id, a FROM foo".to_string(),
            query_name: "put_ids_from_foo_to_bar".to_string(),
            argument_not_used: "min_id".to_string(),
            arguments: "min_id: 0".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id, a FROM foo",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'bar: [{id: 9}, {id: 10}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
          - id: 8
            a: hello
        '
      }
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
            a String
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
fn test_mutator_cannot_alter_schema() {
    let err =
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "DROP TABLE foo",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: '',
          resulting_data: 'bar: [{id: 9}, {id: 10}]'
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
          - id: 8
            a: hello
        '
      }
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
            a String
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
        );
    match err {
        PlatformValidationError::ChMutatorError { ch_schema, mutator_name, original_query, interpolated_query, query_arguments, error } => {
            assert_eq!(ch_schema, "testch");
            assert_eq!(mutator_name, "put_ids_from_foo_to_bar");
            assert_eq!(original_query, "DROP TABLE foo");
            assert_eq!(interpolated_query, "DROP TABLE foo");
            assert_eq!(query_arguments, "");
            assert!(error.contains("ACCESS_DENIED"), "");
        }
        e => panic!("Unexpected error: {e}"),
    }
}

#[test]
fn test_mutator_and_query_cant_share_name() {
    assert_eq!(
        PlatformValidationError::ChSchemaHasDuplicateQueriesOrMutators {
            ch_schema: "testch".to_string(),
            query_name: "put_ids_from_foo_to_bar".to_string(),
            mutator_with_same_name: "put_ids_from_foo_to_bar".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT ch_schema [
  {
    schema_name: testch,
    WITH ch_mutator [
      {
        mutator_name: "put_ids_from_foo_to_bar",
        mutator_expression: "INSERT INTO bar SELECT id, a FROM foo",
        WITH ch_mutator_test {
          test_dataset: default,
          arguments: 'min_id: 0',
          resulting_data: 'bar: [{id: 9}, {id: 10}]'
        }
      },
    ]
    WITH ch_query [
      {
        query_name: "put_ids_from_foo_to_bar",
        query_expression: "SELECT 123 AS num",
        WITH ch_query_test {
          test_dataset: default,
          arguments: '',
          outputs: '
            - num: 123
          ',
        }
      },
    ]
    WITH ch_test_dataset [
      {
        dataset_name: default,
        min_time: 2,
        dataset_contents: '
          foo:
          - id: 7
            a: hello
          - id: 8
            a: hello
        '
      }
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
            a String
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
