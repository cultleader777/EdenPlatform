#[cfg(test)]
use crate::static_analysis::databases::postgres::QueryArg;
#[cfg(test)]
use crate::static_analysis::databases::postgres::OutputSignatureField;
#[cfg(test)]
use crate::static_analysis::{databases::postgres::ValidDbType, PlatformValidationError};

#[cfg(test)]
use super::common;

mod mutator_dataset;

#[test]
fn basic_database_test() {
    common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
        {
            time: 5,
            upgrade: "CREATE TABLE moo(id INT NOT NULL);",
            downgrade: "DROP TABLE moo;",
        },
    ]
}
"#,
    );
}

#[test]
fn basic_database_test_multiple_statements_per_migration() {
    common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "
                CREATE TABLE foo(id INT NOT NULL);
                CREATE TABLE moo(id INT NOT NULL);
            ",
            downgrade: "
                DROP TABLE foo;
                DROP TABLE moo;
            ",
        },
    ]
}
"#,
    );
}

#[test]
fn migration_order_check_test() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 3,
            upgrade: "CREATE TABLE foo(id INT);",
            downgrade: "DROP TABLE foo;",
        },
        {
            time: 2,
            upgrade: "CREATE TABLE bar(id INT);",
            downgrade: "DROP TABLE bar;",
        },
    ]
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMigrationsAreNotOrdered {
            previous_migration_time: 3,
            current_migration_time: 2,
            previous_migration: "CREATE TABLE foo(id INT);".to_string(),
            current_migration: "CREATE TABLE bar(id INT);".to_string(),
        }
    )
}

#[test]
fn test_non_snake_case_table() {
    assert_eq!(
        PlatformValidationError::PgTableNameIsNotSnakeCase {
            bad_table_name: "Foo".to_string(),
            migration_sql: "CREATE TABLE \"Foo\"(id INT NOT NULL);".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 1,
            upgrade: 'CREATE TABLE "Foo"(id INT NOT NULL);',
            downgrade: 'DROP TABLE "Foo";',
        },
        {
            time: 2,
            upgrade: "CREATE TABLE bar(id INT NOT NULL);",
            downgrade: "DROP TABLE bar;",
        },
    ]
}
"#,
        ),
    );
}

#[test]
fn test_non_snake_case_field() {
    assert_eq!(
        PlatformValidationError::PgColumnNameIsNotSnakeCase {
            bad_column_name: "Id".to_string(),
            migration_sql: "CREATE TABLE foo(\"Id\" INT NOT NULL);".to_string(),
            table_name: "foo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 1,
            upgrade: 'CREATE TABLE foo("Id" INT NOT NULL);',
            downgrade: "DROP TABLE foo;",
        },
        {
            time: 2,
            upgrade: "CREATE TABLE bar(id INT NOT NULL);",
            downgrade: "DROP TABLE bar;",
        },
    ]
}
"#,
        ),
    );
}

#[test]
fn migration_syntax_error_test() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLEZ foo(id INT);",
            downgrade: "DROP TABLE foo;",
        },
    ]
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMigrationUpgradeError {
            pg_schema: "testy".to_string(),
            upgrade_sql: "CREATE TABLEZ foo(id INT);".to_string(),
            upgrade_time: 4,
            error: "db error: ERROR: syntax error at or near \"TABLEZ\"".to_string(),
        }
    )
}

#[test]
fn migration_syntax_inconsistent_upgrades() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
        {
            time: 5,
            upgrade: "CREATE TABLE moo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ]
}
"#,
    );

    if let PlatformValidationError::PgMigrationInconsistentDowngrade {
        upgrade_sql,
        downgrade_sql,
        upgrade_time,
        ..
    } = err
    {
        assert_eq!(upgrade_sql, "CREATE TABLE moo(id INT NOT NULL);");
        assert_eq!(downgrade_sql, "DROP TABLE foo;");
        assert_eq!(upgrade_time, 5);
    } else {
        panic!("Unexpected exception: {:?}", err)
    }
}

#[test]
fn test_basic_database_with_test_data_test_yml_fail() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT);",
            downgrade: "DROP TABLE foo;",
        },
        {
            time: 5,
            upgrade: "CREATE TABLE moo(id INT);",
            downgrade: "DROP TABLE moo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "- invalid: schema"
        }
    ],
}
"#,
    );

    if let PlatformValidationError::PgCantDeserializeTestDataset {
        pg_schema,
        error,
        input_data,
        input_dataset_name,
    } = err
    {
        assert_eq!(pg_schema, "testy");
        assert_eq!(error, "invalid type: sequence, expected a map");
        assert_eq!(input_data, "- invalid: schema".to_string());
        assert_eq!(input_dataset_name, "test1".to_string());
    } else {
        panic!("Unexpected test failure: {:#?}", err)
    };
}

#[test]
fn test_basic_database_with_test_data() {
    common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT NOT NULL, f32 REAL NOT NULL, b BOOLEAN NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
        {
            time: 5,
            upgrade: "CREATE TABLE moo(id BIGINT NOT NULL, f64 DOUBLE PRECISION NOT NULL);",
            downgrade: "DROP TABLE moo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            min_time: 5,
            dataset_contents: "
                foo:
                - id: 7
                  f32: 0.7
                  b: yes
                moo:
                - id: 12
                  f64: 1.7
            "
        }
    ],
}
"#,
    );
}

#[test]
fn test_nullable_types_not_allowed() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT);",
            downgrade: "DROP TABLE foo;",
        }
    ]
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgNullableColumnsAreNotAllowed {
            table_name: "foo".to_string(),
            table_column_name: "id".to_string(),
            table_column_type: "integer".to_string(),
            migration_sql: "CREATE TABLE foo(id INT);".to_string(),
        }
    );
}

#[test]
fn test_table_in_dataset_doesnt_exist() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                doesnt_exist:
                - id: 7
            "
        }
    ],
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDatasetTableNotFoundInSchema {
            pg_schema: "testy".to_string(),
            table_tried_to_insert: "doesnt_exist".to_string(),
            input_dataset_name: "test1".to_string(),
        }
    );
}

#[test]
fn test_table_column_in_dataset_doesnt_exist() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - doesnt_exist: 7
            "
        }
    ],
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDatasetTableColumnNotFoundInSchema {
            pg_schema: "testy".to_string(),
            table: "foo".to_string(),
            table_column_tried_to_insert: "doesnt_exist".to_string(),
            input_dataset_name: "test1".to_string(),
        }
    );
}

#[test]
fn test_table_column_unparseable_int() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: henlo
            "
        }
    ],
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDatasetColumnValueCannotBeParsedToExpectedType {
            pg_schema: "testy".to_string(),
            table: "foo".to_string(),
            column: "id".to_string(),
            column_value: "henlo".to_string(),
            type_tried_to_parse_to: "integer".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
            input_dataset_name: "test1".to_string(),
        }
    );
}

#[test]
fn test_dataset_is_never_tested() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 4,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            min_time: 100,
            dataset_contents: "
                foo:
                - id: henlo
            "
        }
    ],
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDatasetIsNeverTested {
            pg_schema: "testy".to_string(),
            input_dataset_name: "test1".to_string(),
            minimum_dataset_time: 100,
            maximum_migration_time: 4,
        }
    );
}

#[test]
fn test_dataset_invalid_boolean_value() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(b BOOLEAN NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - b: huh
            "
        }
    ],
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgDatasetColumnValueInvalidBoolean {
            pg_schema: "testy".to_string(),
            table: "foo".to_string(),
            column: "b".to_string(),
            column_value: "huh".to_string(),
            accepted_true_values: "true, yes, on, y, 1",
            accepted_false_values: "false, no, off, n, 0",
        }
    );
}

#[test]
fn test_dataset_invalid_query() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(b BOOLEAN NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - b: true
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT invalid_field FROM foo
        "
        WITH pg_query_test {
            arguments: "",
            test_dataset: test1,
            outputs: "[]",
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryError {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT invalid_field FROM foo
        "
            .to_string(),
            interpolated_query: "
            SELECT invalid_field FROM foo
        "
            .to_string(),
            query_arguments: "".to_string(),
            error: "db error: ERROR: column \"invalid_field\" does not exist".to_string(),
        }
    );
}

#[test]
fn test_dataset_empty_dataset() {
    assert_eq!(
        PlatformValidationError::PgQueryErrorEmptyRowSet {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT id FROM foo WHERE id > {id:INT}
        "
            .to_string(),
            interpolated_query: "
            SELECT id FROM foo WHERE id > $1
        "
            .to_string(),
            query_arguments: "{ id: 8 }".to_string(),
            test_dataset_name: "test1".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT id FROM foo WHERE id > {id:INT}
        ",
        WITH pg_query_test {
            arguments: "{ id: 8 }",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
        ),
    );
}

#[test]
fn test_query_args_with_whitespace() {
    assert_eq!(
        PlatformValidationError::PgWhitespaceForbiddenInQueryArguments {
            query: "
            SELECT id FROM foo WHERE id > { id : INT }
        "
            .to_string(),
            bad_argument: "id".to_string(),
            expected_whitespace: 0,
            found_whitespace: 4,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT id FROM foo WHERE id > { id : INT }
        ",
        WITH pg_query_test {
            arguments: "{ id: 1 }",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
        ),
    );
}

#[test]
fn test_dataset_query_timeout() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT pg_sleep(0.11)
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryTimeoutError {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT pg_sleep(0.11)
        "
            .to_string(),
            interpolated_query: "
            SELECT pg_sleep(0.11)
        "
            .to_string(),
            query_arguments: "".to_string(),
            limit_ms: 100,
        }
    );
}

#[test]
fn test_dataset_query_unsupported_field_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT 123
        "
        WITH pg_query_test {
            arguments: "",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryInvalidOutputFieldNameFormat {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT 123
        "
            .to_string(),
            interpolated_query: "
            SELECT 123
        "
            .to_string(),
            query_arguments: "".to_string(),
            output_field_index: 1,
            output_field_name: "?column?".to_string(),
            output_field_type: "int4".to_string(),
            expectation: "Field should be snake case",
        }
    );
}

#[test]
fn test_dataset_query_unsupported_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT 123 AS o1, ARRAY[1, 2, 3]::int[] AS o2
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryUnsupportedTypeError {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT 123 AS o1, ARRAY[1, 2, 3]::int[] AS o2
        "
            .to_string(),
            interpolated_query: "
            SELECT 123 AS o1, ARRAY[1, 2, 3]::int[] AS o2
        "
            .to_string(),
            query_arguments: "".to_string(),
            output_field_index: 2,
            output_field_name: "o2".to_string(),
            output_field_type: "_int4".to_string(),
        }
    );
}

#[test]
fn test_dataset_query_duplicate_output_field_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT 123 AS o1, 321 AS o1
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryDuplicateOutputFieldNames {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT 123 AS o1, 321 AS o1
        "
            .to_string(),
            interpolated_query: "
            SELECT 123 AS o1, 321 AS o1
        "
            .to_string(),
            query_arguments: "".to_string(),
            output_field_name: "o1".to_string(),
        }
    );
}

#[test]
fn test_dataset_query_unexpected_outputs() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT 123 AS o1
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{o1: 124}]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryUnexpectedOutputs {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT 123 AS o1
        "
            .to_string(),
            interpolated_query: "
            SELECT 123 AS o1
        "
            .to_string(),
            query_arguments: "".to_string(),
            expected_outputs: "- o1: '124'\n".to_string(),
            actual_outputs: "- o1: '123'\n".to_string(),
        }
    );
}

#[test]
fn test_dataset_query_mutating_transaction() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            DELETE FROM foo
            RETURNING id
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryCannotMutateDatabase {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            DELETE FROM foo
            RETURNING id
        "
            .to_string(),
        }
    );
}

#[test]
fn test_dataset_query_unused_argument_field() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT 123 AS o1
        ",
        WITH pg_query_test {
            arguments: "{ unused_field: 777 }",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryArgumentNotUsedInQuery {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            query_expression: "
            SELECT 123 AS o1
        "
            .to_string(),
            arguments: "{ unused_field: 777 }".to_string(),
            argument_not_used: "unused_field".to_string(),
        }
    );
}

#[test]
fn test_dataset_mutating_query_no_mutation() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        is_mutating: true,
        query_expression: "
            SELECT 123 AS o1 -- UPDATE INSERT DELETE comments should be stripped
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMutatingQueryDoesNotHaveMutationKeywords {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT 123 AS o1 -- UPDATE INSERT DELETE comments should be stripped
        "
            .to_string(),
            expected_keywords: vec!["INSERT", "UPDATE", "DELETE"]
        }
    );
}

#[test]
fn test_dataset_mutating_query_no_mutation_real() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        is_mutating: true,
        query_expression: "
            UPDATE foo SET id = {to_set:INT}
            WHERE id = {where:INT}
            RETURNING id
        ",
        WITH pg_query_test {
            arguments: "{ to_set: 123, where: 8 }",
            outputs: "[]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMutatingQueryDidNotModifyDatabase {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            UPDATE foo SET id = {to_set:INT}
            WHERE id = {where:INT}
            RETURNING id
        "
            .to_string(),
            interpolated_query: "
            UPDATE foo SET id = $1
            WHERE id = $2
            RETURNING id
        "
            .to_string(),
            query_arguments: "{ to_set: 123, where: 8 }".to_string(),
            test_dataset_name: "test1".to_string(),
        }
    );
}

#[test]
fn test_dataset_mutating_query_success() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        is_mutating: true,
        seqscan_ok: true,
        query_expression: "
            UPDATE foo SET id = {to_set:INT}
            WHERE id = {where:INT}
            RETURNING id
        ",
        WITH pg_query_test {
            arguments: "{ to_set: 123, where: 7 }",
            outputs: "[{ id: 123 }]",
            test_dataset: test1,
        }
    }
]
"#,
    );
}

#[test]
fn test_dataset_query_no_tests() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ]
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT 123 AS o1, ARRAY[1, 2, 3]::int[] AS o2
        ",
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryHasNoTests {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            original_query: "
            SELECT 123 AS o1, ARRAY[1, 2, 3]::int[] AS o2
        "
            .to_string(),
        }
    );
}

#[test]
fn test_dataset_mutator_has_no_tests() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ]
}

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            DELETE FROM foo
        ",
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMutatorHasNoTests {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            original_query: "
            DELETE FROM foo
        "
            .to_string(),
        }
    );
}

#[test]
fn test_materialized_view_has_no_tests() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE MATERIALIZED VIEW foo AS (SELECT 123 AS col);",
            downgrade: "DROP MATERIALIZED VIEW foo;",
        },
    ]
}

DATA STRUCT pg_mat_view [
    {
        schema_name: testy,
        mview_name: foo,
        update_frequency: hourly,
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMaterializedViewHasNoTests {
            pg_schema: "testy".to_string(),
            materialized_view_name: "foo".to_string(),
        }
    );
}

#[test]
fn test_materialized_view_incorrect_columns() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE MATERIALIZED VIEW foo AS (SELECT 123 AS col);",
            downgrade: "DROP MATERIALIZED VIEW foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "",
        }
    ]
}

DATA STRUCT pg_mat_view [
    {
        schema_name: testy,
        mview_name: foo,
        update_frequency: hourly,
        WITH pg_mat_view_test {
            expected_data: "
                - incorrect: column
            ",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMaterializedViewExpectedOutputColumnNotFound {
            pg_schema: "testy".to_string(),
            materialized_view: "foo".to_string(),
            expected_dataset_column: "incorrect".to_string(),
            actual_materialized_view_columns: vec!["col".to_string()],
            expected_results: "
                - incorrect: column
            "
            .to_string(),
        }
    );
}

#[test]
fn test_materialized_view_incorrect_columns_rev() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE MATERIALIZED VIEW foo AS (SELECT 123 AS col, 321 AS other);",
            downgrade: "DROP MATERIALIZED VIEW foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "",
        }
    ]
}

DATA STRUCT pg_mat_view [
    {
        schema_name: testy,
        mview_name: foo,
        update_frequency: hourly,
        WITH pg_mat_view_test {
            expected_data: "
                - col: 123
            ",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMaterializedViewExpectedOutputIsMissingActualColumn {
            pg_schema: "testy".to_string(),
            materialized_view: "foo".to_string(),
            missing_column_in_expected_row: "other".to_string(),
            actual_materialized_view_columns: vec!["col".to_string(), "other".to_string()],
            expected_results: "
                - col: 123
            "
            .to_string(),
        }
    );
}

#[test]
fn test_materialized_view_incorrect_output_rows_count() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE MATERIALIZED VIEW foo AS (SELECT 123 AS col);",
            downgrade: "DROP MATERIALIZED VIEW foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "",
        }
    ]
}

DATA STRUCT pg_mat_view [
    {
        schema_name: testy,
        mview_name: foo,
        update_frequency: hourly,
        WITH pg_mat_view_test {
            expected_data: "
                - col: 123
                - col: 321
            ",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMaterializedViewTestOutputRowCountMistmatch {
            pg_schema: "testy".to_string(),
            materialized_view: "foo".to_string(),
            expected_materialized_view_rows_count: 2,
            actual_materialized_view_rows_count: 1,
            expected_results: "
                - col: 123
                - col: 321
            "
            .to_string(),
        }
    );
}

#[test]
fn test_materialized_view_different_outputs() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE MATERIALIZED VIEW foo AS (SELECT 123 AS col);",
            downgrade: "DROP MATERIALIZED VIEW foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "",
        }
    ]
}

DATA STRUCT pg_mat_view [
    {
        schema_name: testy,
        mview_name: foo,
        update_frequency: hourly,
        WITH pg_mat_view_test {
            expected_data: "
                - col: 321
            ",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(err, PlatformValidationError::PgMaterializedViewSortedOutputRowsMismatch {
        pg_schema: "testy".to_string(),
        materialized_view: "foo".to_string(),
        diff: "[\n    [\n \u{1b}[9;31m        \"321\",\u{1b}[0m\n \u{1b}[32m        \"123\",\u{1b}[0m\n    ],\n]".to_string(),
    });
}

#[test]
fn test_query_with_all_types() {
    let cdb = common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT
              123::INT AS o1,
              1234::BIGINT AS o2,
              12.3::REAL AS o3,
              123.4::DOUBLE PRECISION AS o4,
              true AS o5,
              'some text' AS o6
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o2: 1234, o3: 12.3, o4: 123.4, o5: true, o6: 'some text', o1: 123 }]",
            test_dataset: test1,
        }
    }
]

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            INSERT INTO foo(id)
            VALUES({id:INT})
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: '{}',
            arguments: "{ id: 123 }",
        }
    }
]
"#,
    );

    assert_eq!(cdb.async_res.checked_pg_dbs.len(), 1);
    let the_db = cdb.async_res.checked_pg_dbs.iter().collect::<Vec<_>>();
    assert_eq!(the_db.len(), 1);
    let queries = the_db[0].1.queries.iter().collect::<Vec<_>>();
    let mutators = the_db[0].1.mutators.iter().collect::<Vec<_>>();
    assert_eq!(
        queries[0].1.full_query.original_expression,
        "
            SELECT
              123::INT AS o1,
              1234::BIGINT AS o2,
              12.3::REAL AS o3,
              123.4::DOUBLE PRECISION AS o4,
              true AS o5,
              'some text' AS o6
        "
    );
    assert_eq!(
        queries[0].1.full_query.interpolated_expression,
        "
            SELECT
              123::INT AS o1,
              1234::BIGINT AS o2,
              12.3::REAL AS o3,
              123.4::DOUBLE PRECISION AS o4,
              true AS o5,
              'some text' AS o6
        "
    );

    assert_eq!(
        queries[0].1.output_signature,
        vec![
            OutputSignatureField {
                name: "o1".to_string(),
                the_type: ValidDbType::INT,
                optional: false
            },
            OutputSignatureField {
                name: "o2".to_string(),
                the_type: ValidDbType::BIGINT,
                optional: false
            },
            OutputSignatureField {
                name: "o3".to_string(),
                the_type: ValidDbType::FLOAT,
                optional: false
            },
            OutputSignatureField {
                name: "o4".to_string(),
                the_type: ValidDbType::DOUBLE,
                optional: false
            },
            OutputSignatureField {
                name: "o5".to_string(),
                the_type: ValidDbType::BOOL,
                optional: false
            },
            OutputSignatureField {
                name: "o6".to_string(),
                the_type: ValidDbType::TEXT,
                optional: false
            },
        ]
    );
    assert_eq!(mutators.len(), 1);
    let the_mutator = &mutators[0].1;
    assert_eq!(
        the_mutator.full_query.args,
        vec![QueryArg {
            the_type: ValidDbType::INT,
            name: "id".to_string(),
        }]
    );
}

#[test]
fn test_pg_mutator_syntax_error() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            INSORT INTO foo(id) VALUES({id:INT})
        ",
        WITH pg_mutator_test {
            arguments: "{ id: 123 }",
            resulting_data: '{}',
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMutatorError {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            original_query: "
            INSORT INTO foo(id) VALUES({id:INT})
        "
            .to_string(),
            interpolated_query: "
            INSORT INTO foo(id) VALUES($1)
        "
            .to_string(),
            query_arguments: "{ id: 123 }".to_string(),
            error: "db error: ERROR: syntax error at or near \"INSORT\"".to_string(),
        }
    );
}

#[test]
fn test_pg_mutator_no_modifications() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            DELETE FROM foo WHERE id = {id:INT}
        "
        WITH pg_mutator_test {
            arguments: "{ id: 123 }",
            resulting_data: '{}',
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMutatorDidNotModifyDatabase {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            original_query: "
            DELETE FROM foo WHERE id = {id:INT}
        "
            .to_string(),
            interpolated_query: "
            DELETE FROM foo WHERE id = $1
        "
            .to_string(),
            query_arguments: "{ id: 123 }".to_string(),
            test_dataset_name: "test1".to_string(),
        }
    );
}

#[test]
fn test_pg_mutator_no_schema_modifications() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            DROP TABLE foo
        "
        WITH pg_mutator_test {
            arguments: "",
            resulting_data: '{}',
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMutatorCannotChangeDbSchema {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            original_query: "
            DROP TABLE foo
        "
            .to_string(),
        }
    );
}

#[test]
fn test_pg_mutator_timeout() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            INSERT INTO foo(id)
            VALUES({id:INT})
            RETURNING pg_sleep(0.11)
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: '{}',
            arguments: "{ id: 123 }",
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgMutatorTimeoutError {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            original_query: "
            INSERT INTO foo(id)
            VALUES({id:INT})
            RETURNING pg_sleep(0.11)
        "
            .to_string(),
            interpolated_query: "
            INSERT INTO foo(id)
            VALUES($1)
            RETURNING pg_sleep(0.11)
        "
            .to_string(),
            query_arguments: "{ id: 123 }".to_string(),
            limit_ms: 100,
        }
    );
}

#[test]
fn test_db_duplicate_mutator_and_query_names() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_q_1,
        mutator_expression: "
            INSERT INTO foo(id)
            VALUES({id:INT})
            RETURNING pg_sleep(0.0)
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: '{}',
            arguments: "{ id: 123 }",
        }
    }
]

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_q_1,
        query_expression: "
            SELECT
              123::INT AS o1,
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryAndMutatorShareSameName {
            pg_schema: "testy".to_string(),
            query_or_mutator_name: "test_q_1".to_string(),
        }
    );
}

#[test]
fn test_pg_transaction_step_not_found() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_transaction [
    {
        schema_name: testy,
        transaction_name: some_path,
        steps: "
            henlo boi
        ",
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgTransactionStepNotFound {
            pg_schema: "testy".to_string(),
            transaction_name: "some_path".to_string(),
            step_not_found: "henlo boi".to_string(),
        }
    );
}

#[test]
fn test_pg_transaction_at_least_two_steps() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_transaction [
    {
        schema_name: testy,
        transaction_name: some_path,
        steps: "
            test_mutator_1
        ",
    }
]

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            INSERT INTO foo(id)
            VALUES({id:INT})
            RETURNING pg_sleep(0.0)
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: '{}',
            arguments: "{ id: 123 }",
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgTransactionMustHaveAtLeastTwoSteps {
            pg_schema: "testy".to_string(),
            transaction_name: "some_path".to_string(),
            step_count: 1,
        }
    );
}

#[test]
fn test_db_non_explicit_read_only_transaction() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_transaction [
    {
        schema_name: testy,
        transaction_name: some_path,
        steps: "
            test_query_1
            test_query_2
        ",
    }
]

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT
              123::INT AS o1,
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    },
    {
        schema_name: testy,
        query_name: test_query_2,
        query_expression: "
            SELECT
              123::INT AS o1,
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    },
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgTransactionWithoutMutatorsMustBeMarkedAsReadOnly {
            pg_schema: "testy".to_string(),
            transaction_name: "some_path".to_string(),
        }
    );
}

#[test]
fn test_pg_transaction_duplicate_steps() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_transaction [
    {
        schema_name: testy,
        transaction_name: some_path,
        is_read_only: true,
        steps: "
            test_query_1
            test_query_1
        ",
    }
]

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT
              123::INT AS o1,
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    },
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgTransactionDuplicateStepsDetected {
            pg_schema: "testy".to_string(),
            transaction_name: "some_path".to_string(),
            duplicate_step_name: "test_query_1".to_string(),
        }
    );
}

#[test]
fn test_db_ro_transaction_has_mutators() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_transaction [
    {
        schema_name: testy,
        transaction_name: some_path,
        is_read_only: true,
        steps: "
            test_mutator_1
            test_query_1
        ",
    }
]

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT
              123::INT AS o1
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    },
]

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            INSERT INTO foo(id)
            VALUES({id:INT})
            RETURNING pg_sleep(0.0)
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: '{}',
            arguments: "{ id: 123 }",
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgTransactionReadOnlyTransactionHasMutators {
            pg_schema: "testy".to_string(),
            transaction_name: "some_path".to_string(),
        }
    );
}

#[test]
fn test_db_read_only_transaction_success() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_transaction [
    {
        schema_name: testy,
        transaction_name: some_path,
        is_read_only: true,
        steps: "
            test_query_1
            test_query_2
        ",
    }
]

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT
              123::INT AS o1
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    },
    {
        schema_name: testy,
        query_name: test_query_2,
        query_expression: "
            SELECT
              123::INT AS o1
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    },
]
"#,
    );
}

#[test]
fn test_db_read_write_transaction_success() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_transaction [
    {
        schema_name: testy,
        transaction_name: some_path,
        steps: "
            test_mutator_1
            test_query_1
        ",
    }
]

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        query_expression: "
            SELECT
              123::INT AS o1
        ",
        WITH pg_query_test {
            arguments: "",
            outputs: "[{ o1: 123 }]",
            test_dataset: test1,
        }
    },
]

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            INSERT INTO foo(id)
            VALUES({id:INT})
            RETURNING pg_sleep(0.0)
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: '
              foo:
              - id: 123
            ',
            arguments: "{ id: 123 }",
        }
    }
]
"#,
    );
}

#[test]
fn test_reserved_table_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE epl_schema_migrations (foo INT NOT NULL);",
            downgrade: "DROP TABLE epl_schema_migrations;",
        },
    ]
}
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgReservedTableName {
            pg_schema: "testy".to_string(),
            table_name: "epl_schema_migrations".to_string(),
            upgrade_sql: "CREATE TABLE epl_schema_migrations (foo INT NOT NULL);".to_string(),
            upgrade_time: 0,
        }
    );
}

#[test]
fn test_dataset_query_seq_scan_error() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        seqscan_ok: false,
        query_expression: "
            SELECT id
            FROM foo
            WHERE id = {where:INT}
        ",
        WITH pg_query_test {
            arguments: "{ where: 7 }",
            outputs: "[{ id: 7 }]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    match err {
        PlatformValidationError::PgQuerySequentialScansFound {
            seq_scan_table,
            query_name,
            ..
        } => {
            assert_eq!(seq_scan_table, "foo");
            assert_eq!(query_name, "test_query_1");
        }
        e => {
            panic!("Unexpected error: {:#?}", e)
        }
    }
}

#[test]
fn test_dataset_mutator_seq_scan_error() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        seqscan_ok: false,
        mutator_expression: "
            UPDATE foo
            SET id = {id:INT}
            WHERE id = {where:INT}
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: '{}',
            arguments: "{ where: 7, id: 123 }",
        }
    }
]
"#,
    );

    match err {
        PlatformValidationError::PgQuerySequentialScansFound {
            seq_scan_table,
            query_name,
            ..
        } => {
            assert_eq!(seq_scan_table, "foo");
            assert_eq!(query_name, "test_mutator_1");
        }
        e => {
            panic!("Unexpected error: {:#?}", e)
        }
    }
}

#[test]
fn test_query_opt_fields_bad_value() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        opt_fields: ' #b@d_f#*()# good_field ',
        query_expression: "
            SELECT id
            FROM foo
            WHERE id = {where:INT}
        ",
        WITH pg_query_test {
            arguments: "{ where: 7 }",
            outputs: "[{ id: 7 }]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryOptFieldMustBeSnakeCase {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            bad_optional_field: "#b@d_f#*()#".to_string(),
            optional_fields: " #b@d_f#*()# good_field ".to_string(),
        }
    );
}

#[test]
fn test_query_opt_fields_duplicate_value() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        opt_fields: ' duplicate_field duplicate_field ',
        query_expression: "
            SELECT id
            FROM foo
            WHERE id = {where:INT}
        ",
        WITH pg_query_test {
            arguments: "{ where: 7 }",
            outputs: "[{ id: 7 }]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryOptFieldDuplicate {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            duplicate_optional_field: "duplicate_field".to_string(),
            optional_fields: " duplicate_field duplicate_field ".to_string(),
        }
    );
}

#[test]
fn test_query_opt_fields_non_existing_field() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        opt_fields: 'non_existing',
        query_expression: "
            SELECT id
            FROM foo
            WHERE id = {where:INT}
        ",
        WITH pg_query_test {
            arguments: "{ where: 7 }",
            outputs: "[{ id: 7 }]",
            test_dataset: test1,
        }
    }
]
"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::PgQueryOptFieldDoesntExistInQueryResults {
            pg_schema: "testy".to_string(),
            query_name: "test_query_1".to_string(),
            bad_optional_field: "non_existing".to_string(),
            optional_fields: "non_existing".to_string(),
            original_query: r#"
            SELECT id
            FROM foo
            WHERE id = {where:INT}
        "#
            .to_string(),
        }
    );
}

#[test]
fn test_query_opt_fields_success() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "CREATE TABLE foo(id INT PRIMARY KEY NOT NULL);",
            downgrade: "DROP TABLE foo;",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: "
                foo:
                - id: 7
            "
        }
    ],
}

DATA STRUCT pg_query [
    {
        schema_name: testy,
        query_name: test_query_1,
        opt_fields: 'id',
        query_expression: "
            SELECT NULL as id
            FROM foo
            WHERE id = {where:INT}
        ",
        WITH pg_query_test {
            arguments: "{ where: 7 }",
            outputs: "[{ id: None }]",
            test_dataset: test1,
        }
    }
]
"#,
    );
}

#[test]
fn test_pg_deployment_duplicate_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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
    ] WITH pg_deployment_unmanaged_db [
      {
        db_name: testdb_a,
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
        PlatformValidationError::PgDeploymentDuplicateDatabases {
            pg_deployment: "testdb".to_string(),
            db_name_a: "testdb_a".to_string(),
            db_name_b: "testdb_a".to_string(),
        }
    );
}

#[test]
fn test_pg_deployment_only_one_instance() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    synchronous_replication: false,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
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
        PlatformValidationError::PgDeploymentMustHaveAtLeastTwoNodes {
            pg_deployment: "testdb".to_string(),
            db_region: "us-west".to_string(),
            minimum_instances: 2,
            found_instances: 1,
        }
    );
}

#[test]
fn test_pg_deployment_sync_replication_needs_at_least_3_instances() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    synchronous_replication: true,
    WITH pg_deployment_instance [
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
        PlatformValidationError::PgDeploymentForSynchronousReplicationYouMustRunAtLeastThreeNodes {
            pg_deployment: "testdb".to_string(),
            db_region: "us-west".to_string(),
            minimum_instances: 3,
            found_instances: 2,
            synchronous_replication_enabled: true,
        }
    );
}

#[test]
fn test_pg_deployment_no_more_than_5_instances() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
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

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    synchronous_replication: true,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
      {
        instance_id: 3,
        pg_server: server-c=>pgtest1,
      },
      {
        instance_id: 4,
        pg_server: server-d=>pgtest1,
      },
      {
        instance_id: 5,
        pg_server: server-e=>pgtest1,
      },
      {
        instance_id: 6,
        pg_server: server-f=>pgtest1,
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
  server-c, eth0, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    pgtest1;
  };
  server-d, eth0, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume {
    pgtest1;
  };
  server-e, eth0, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.14;
  } WITH server_root_volume {
    pgtest1;
  };
  server-f, eth0, false, false, false, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.15;
  } WITH server_root_volume {
    pgtest1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::PgDeploymentHasMoreThanMaximumChildInstancesAllowed {
            pg_deployment: "testdb".to_string(),
            db_region: "us-west".to_string(),
            maximum_instances: 5,
            found_instances: 6,
        }
    );
}
