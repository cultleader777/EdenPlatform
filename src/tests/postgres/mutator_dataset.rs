#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_mutator_resulting_dataset_is_undefined() {
    assert_eq!(
        PlatformValidationError::PgResultingDatasetForMutatorTestIsUndefined {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            resulting_data: "".to_string(),
            pg_mutator_test_arguments: "{ id: 123 }".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
            resulting_data: '',
            arguments: "{ id: 123 }",
        }
    }
]

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
"#,
        )
    );
}

#[test]
fn test_mutator_resulting_dataset_has_empty_rows() {
    assert_eq!(
        PlatformValidationError::PgResultingDatasetTableRowIsEmpty {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            resulting_data: "foo: [{}]".to_string(),
            resulting_data_table: "foo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
            resulting_data: 'foo: [{}]',
            arguments: "{ id: 123 }",
        }
    }
]

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
"#,
        )
    );
}

#[test]
fn test_mutator_resulting_dataset_column_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::PgResultingDatasetTableColumnDoesntExist {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            resulting_data: "foo: [{woo: 123}]".to_string(),
            resulting_data_table: "foo".to_string(),
            resulting_data_non_existing_column: "woo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
            resulting_data: 'foo: [{woo: 123}]',
            arguments: "{ id: 123 }",
        }
    }
]

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
"#,
        )
    );
}

#[test]
fn test_mutator_resulting_dataset_column_cant_be_parsed() {
    assert_eq!(
        PlatformValidationError::PgResultingDatasetColumnValueCannotBeParsedToExpectedType {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            resulting_data: "foo: [{id: abc}]".to_string(),
            resulting_data_table: "foo".to_string(),
            resulting_data_column: "id".to_string(),
            resulting_data_column_value: "abc".to_string(),
            type_tried_to_parse_to: "integer".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
            resulting_data: 'foo: [{id: abc}]',
            arguments: "{ id: 123 }",
        }
    }
]

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
"#,
        )
    );
}

#[test]
fn test_mutator_resulting_dataset_row_found_before_mutator_execution() {
    assert_eq!(
        PlatformValidationError::PgResultingDatasetRowIsFoundInTestDatasetBeforeMutatorIsExecuted {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            resulting_data: "foo: [{id: 7}]".to_string(),
            resulting_data_table: "foo".to_string(),
            row_found_before_mutator_execution: "id: '7'\n".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
            resulting_data: 'foo: [{id: 7}]',
            arguments: "{ id: 123 }",
        }
    }
]

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
"#,
        )
    );
}

#[test]
fn test_mutator_resulting_dataset_row_found_multiple_times() {
    assert_eq!(
        PlatformValidationError::PgResultingDatasetRowFoundMoreThanOnceInTable {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            resulting_data: "foo: [{attr: a}]".to_string(),
            resulting_data_table: "foo".to_string(),
            resulting_data_ambigous_row: "attr: a\n".to_string(),
            output_dataset_matching_rows: "- attr: a\n  id: '123'\n- attr: a\n  id: '7'\n".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT pg_mutator [
    {
        schema_name: testy,
        mutator_name: test_mutator_1,
        mutator_expression: "
            INSERT INTO foo(id, attr)
            VALUES ({id:INT}, {attr:TEXT}), (7, 'a')
        ",
        WITH pg_mutator_test {
            test_dataset: test1,
            resulting_data: 'foo: [{attr: a}]',
            arguments: "{ id: 123, attr: a }",
        }
    }
]

DATA STRUCT pg_schema {
    schema_name: testy WITH pg_migration [
        {
            time: 0,
            upgrade: "
              CREATE TABLE foo(id INT NOT NULL, attr TEXT NOT NULL);
            ",
            downgrade: "
              DROP TABLE foo;
            ",
        },
    ] WITH pg_test_dataset [
        {
            dataset_name: test1,
            dataset_contents: ""
        }
    ],
}
"#,
        )
    );
}

#[test]
fn test_mutator_resulting_dataset_row_not_found_in_output() {
    assert_eq!(
        PlatformValidationError::PgResultingDatasetRowsAreNotFoundInTableAfterMutatorExecution {
            pg_schema: "testy".to_string(),
            mutator_name: "test_mutator_1".to_string(),
            resulting_data: "foo: [{id: 124}]".to_string(),
            resulting_data_table: "foo".to_string(),
            rows_not_found_after_mutator_execution: "- id: '124'\n".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
            resulting_data: 'foo: [{id: 124}]',
            arguments: "{ id: 123 }",
        }
    }
]

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
"#,
        )
    );
}
