#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_bw_snapshot_syntax_error() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo . mozo
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotSyntaxError {
            type_name: "test_type".to_string(),
            type_version: 7,
            syntax_error:
                "Parsing Error: VerboseError { errors: [(LocatedSpan { offset: 19, line: 2, fragment: \". mozo\\n        \", extra: () }, Char('@'))] }"
                    .to_string(),
            snapshot_source: "{
            bozo . mozo
        }"
            .to_string(),
        }
    );
}

#[test]
fn test_bw_migration_syntax_error() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_migration {
        version: 77,
        migration_source: "
            wot is dis milky
        "
    }
}
"#,
    );
    assert_eq!(err, PlatformValidationError::BwTypeMigrationSyntaxError {
        type_name: "test_type".to_string(),
        type_version: 77,
        syntax_error: "Parsing Error: VerboseError { errors: [(LocatedSpan { offset: 13, line: 2, fragment: \"wot is dis milky\\n        \", extra: () }, Nom(Tag)), (LocatedSpan { offset: 13, line: 2, fragment: \"wot is dis milky\\n        \", extra: () }, Nom(Alt))] }".to_string(),
        migration_source: "
            wot is dis milky
        ".to_string(),
    });
}

#[test]
fn test_bw_snapshot_duplicate_field_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            bozo @1 :String,
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotDuplicateFields {
            type_name: "test_type".to_string(),
            type_version: 7,
            duplicate_field_name: "bozo".to_string(),
        }
    );
}

#[test]
fn test_bw_snapshot_no_versions() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeNoSnapshotsOrMigrationsFound {
            type_name: "test_type".to_string(),
        }
    );
}

#[test]
fn test_bw_has_only_migration() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_migration {
        version: 77,
        migration_source: "
            ADD .tossboi @0 String DEFAULT 'thicc'
        "
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeWithOneVersionCannotHaveAMigration {
            type_name: "test_type".to_string(),
            type_version: 77,
        }
    );
}

#[test]
fn test_bw_first_and_last_has_no_snapshot() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    } WITH versioned_type_migration {
        version: 8,
        migration_source: "
            ADD .le_field @1 String DEFAULT 'sum feld'
        "
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFirstAndLastVersionMustHaveSnapshots {
            type_name: "test_type".to_string(),
            first_version: 7,
            last_version: 8,
            first_missing_snapshot: false,
            last_missing_snapshot: true,
        }
    );
}

#[test]
fn test_bw_first_version_cannot_have_migration() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 7,
        migration_source: "
            ADD .le_field @1 String DEFAULT 'le def'
        "
    }, {
        version: 8,
        migration_source: "
            ADD .le_foold @1 String DEFAULT 'le def'
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFirstVersionCantHaveMigration {
            type_name: "test_type".to_string(),
            first_version: 7,
        }
    );
}

#[test]
fn test_bw_cannot_find_migration_for_versions() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationNotFoundForVersions {
            type_name: "test_type".to_string(),
            from_version: 7,
            to_version: 8,
        }
    );
}

#[test]
fn test_bw_duplicate_field() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .bozo @0 String DEFAULT 'mozo'
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeCannotAddDuplicateNameField {
            type_name: "test_type".to_string(),
            migration_source: "
            ADD .bozo @0 String DEFAULT 'mozo'
        "
            .to_string(),
            field_path: vec!["bozo".to_string()],
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
            from_version: 7,
            to_version: 8,
        }
    );
}

#[test]
fn test_bw_inconsistent_migration() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :I64,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .lozo @1 String DEFAULT 'larp'
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotAndMigrationAreInconsistent {
            type_name: "test_type".to_string(),
            actual_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 1,
                default_value: Some(
                    "larp",
                ),
                last_mutation_version: 8,
            },
        ),
    ],
}"#
            .to_string(),
            migration_source: r#"
            ADD .lozo @1 String DEFAULT 'larp'
        "#
            .to_string(),
            expected_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: I64,
                field_index: 1,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
            from_version: 7,
            to_version: 8,
        }
    );
}

#[test]
fn test_bw_trying_add_inside_ground_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :I64,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .bozo.lozo @1 String DEFAULT 'moo'
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeExpectedStructTypeGotGround {
            field_path: vec!["bozo".to_string(), "lozo".to_string()],
            type_name: "test_type".to_string(),
            migration_source: "
            ADD .bozo.lozo @1 String DEFAULT 'moo'
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
            from_version: 7,
            to_version: 8,
        }
    );
}

#[test]
fn test_bw_adding_deep_nested_type() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :{
                gozo @0 :{
                    ozo @0 :F64?
                }
            },
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .lozo.gozo.ozo @0 F64?
        "
    }]
}
"#,
    );
}

#[test]
fn test_bw_adding_field_with_default_optional_value() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :I64,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .lozo @1 String? DEFAULT 'boi'
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationOptionAndDefaultValueAreMutuallyExclusive {
            type_name: "test_type".to_string(),
            migration_source: r#"
            ADD .lozo @1 String? DEFAULT 'boi'
        "#
            .to_string(),
            to_version: 8,
            field_path: vec!["lozo".to_string()],
        }
    );
}

#[test]
fn test_bw_adding_field_without_default_value() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :I64,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .lozo @1 String
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationNewFieldMustHaveDefaultValueOrBeOptional {
            type_name: "test_type".to_string(),
            migration_source: "
            ADD .lozo @1 String
        "
            .to_string(),
            to_version: 8,
            field_path: vec!["lozo".to_string()],
        }
    );
}

#[test]
fn test_bw_non_0_field_index() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @1 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFieldIndexesMustBeZeroBasedSequential {
            type_name: "test_type".to_string(),
            type_version: 7,
            expected_field_index: 0,
            actual_field_index: 1,
            actual_field_index_name: "bozo".to_string(),
        }
    );
}

#[test]
fn test_bw_non_gap_in_field_index() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @2 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFieldIndexesMustBeZeroBasedSequential {
            type_name: "test_type".to_string(),
            type_version: 7,
            expected_field_index: 1,
            actual_field_index: 2,
            actual_field_index_name: "lozo".to_string(),
        }
    );
}

#[test]
fn test_bw_duplicate_field_index() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFieldIndexesMustBeZeroBasedSequential {
            type_name: "test_type".to_string(),
            type_version: 7,
            expected_field_index: 1,
            actual_field_index: 0,
            actual_field_index_name: "lozo".to_string(),
        }
    );
}

#[test]
fn test_bw_adding_field_with_wrong_index() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .lozo @2 String DEFAULT 'hey'
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationAddsInvalidFieldIndex {
            type_name: "test_type".to_string(),
            type_version: 8,
            migration_source: "
            ADD .lozo @2 String DEFAULT 'hey'
        "
            .to_string(),
            expected_field_index: 1,
            actual_field_index: 2,
            actual_field_index_name: "lozo".to_string(),
        }
    );
}

#[test]
fn test_bw_removing_field_empty() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            DROP .bozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationStructBecomesEmptyAfterFieldDrop {
            type_name: "test_type".to_string(),
            field_path: vec!["bozo".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            DROP .bozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_removing_field_not_found() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            DROP .lozo.bozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationCannotFindFieldToDrop {
            type_name: "test_type".to_string(),
            field_path: vec!["lozo".to_string(), "bozo".to_string()],
            field_not_found: "lozo".to_string(),
            from_version: 7,
            to_version: 8,
            migration_source: "
            DROP .lozo.bozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_removing_nested_field_from_ground() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            DROP .bozo.lozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationExpectedNestedStructFieldToDropGotGround {
            type_name: "test_type".to_string(),
            field_path: vec!["bozo".to_string(), "lozo".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            DROP .bozo.lozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_dropping_nested_field() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :{
                pozo @0 :String,
                rozo @1 :I64,
            }
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :{
                rozo @0 :I64,
            }
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            DROP .lozo.pozo
        "
    }]
}
"#,
    );
}

#[test]
fn test_bw_renaming_different_prefixes() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            lozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .p1.bozo .p2.lozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationRenamingFieldIsOnlyAllowedAtTheSameStructLevel {
            type_name: "test_type".to_string(),
            from_prefix: vec!["p1".to_string()],
            to_prefix: vec!["p2".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            RENAME .p1.bozo .p2.lozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 1,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_renaming_different_levels() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            lozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .bozo .p2.lozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationRenamingFieldIsOnlyAllowedAtTheSameStructLevel {
            type_name: "test_type".to_string(),
            from_prefix: vec![],
            to_prefix: vec!["p2".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            RENAME .bozo .p2.lozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 1,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_renaming_same_field() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            lozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .bozo .bozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationRenamingFromAndToFieldsAreTheSame {
            type_name: "test_type".to_string(),
            from_path: vec!["bozo".to_string()],
            to_path: vec!["bozo".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            RENAME .bozo .bozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 1,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_renaming_to_existing_field() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            lozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .bozo .lozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationRenamingFieldWithSuchNameAlreadyExists {
            type_name: "test_type".to_string(),
            from_path: vec!["bozo".to_string()],
            to_path: vec!["lozo".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            RENAME .bozo .lozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 1,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_renaming_expected_struct_found_ground() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            lozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .bozo.mozo .bozo.hozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationRenamingFieldExpectedInnerStructFoundGroundType {
            type_name: "test_type".to_string(),
            from_path: vec!["bozo".to_string(), "mozo".to_string()],
            to_path: vec!["bozo".to_string(), "hozo".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            RENAME .bozo.mozo .bozo.hozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 1,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_renaming_field_not_found() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
            lozo @1 :String,
        }"
    }, {
        version: 8,
        snapshot_source: "{
            lozo @0 :String,
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .fozo .hozo
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeMigrationRenamingFieldNotFound {
            type_name: "test_type".to_string(),
            from_path: vec!["fozo".to_string()],
            to_path: vec!["hozo".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            RENAME .fozo .hozo
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
        (
            "lozo",
            VersionedStructFieldGeneric {
                field_type: String,
                field_index: 1,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_renaming_nested_field_success() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :{
                foo @0 :I64
            },
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :{
                bar @0 :I64
            },
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .bozo.foo .bozo.bar
        "
    }]
}
"#,
    );
}

#[test]
fn test_bw_default_value_parsing_error_i64() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :I64 DEFAULT 'mookie'
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
            type_name: "test_type".to_string(),
            version: 7,
            the_type: "I64".to_string(),
            default_value: "mookie".to_string(),
            field_name: "bozo".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
        }
    );
}

#[test]
fn test_bw_default_value_parsing_error_f64() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :F64 DEFAULT 'mookie'
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
            type_name: "test_type".to_string(),
            version: 7,
            the_type: "F64".to_string(),
            default_value: "mookie".to_string(),
            field_name: "bozo".to_string(),
            parsing_error: "invalid float literal".to_string(),
        }
    );
}

#[test]
fn test_bw_default_value_parsing_error_bool() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :Bool DEFAULT 'mookie'
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
            type_name: "test_type".to_string(),
            version: 7,
            the_type: "Bool".to_string(),
            default_value: "mookie".to_string(),
            field_name: "bozo".to_string(),
            parsing_error: "provided string was not `true` or `false`".to_string(),
        }
    );
}

#[test]
fn test_bw_default_value_cannot_be_opt() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String? DEFAULT 'mookie'
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotOptionAndDefaultValueAreMutuallyExclusive {
            type_name: "test_type".to_string(),
            version: 7,
            field_name: "bozo".to_string(),
        }
    );
}

#[test]
fn test_bw_default_value_for_struct() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :{
                gozo @0 :I64
            } DEFAULT 'mookie'
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotDefaultValuesForTypeAreNotSupported {
            type_name: "test_type".to_string(),
            version: 7,
            field_name: "bozo".to_string(),
            the_type: "Struct".to_string(),
        }
    );
}

#[test]
fn test_bw_default_value_for_array() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String[] DEFAULT 'mookie'
        }"
    }
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeSnapshotDefaultValuesForTypeAreNotSupported {
            type_name: "test_type".to_string(),
            version: 7,
            field_name: "bozo".to_string(),
            the_type: "Array".to_string(),
        }
    );
}

#[test]
fn test_bw_default_values() {
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String DEFAULT 'mookie',
            lozo @1 :I64 DEFAULT '777',
            hozo @2 :F64 DEFAULT '7.77',
            rozo @3 :Bool DEFAULT 'true',
        }"
    }
}
"#,
    );
}

#[test]
fn test_bw_renaming_field_twice_fails() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :{
                foo @0 :I64
            },
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :{
                baz @0 :I64
            },
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            RENAME .bozo.foo .bozo.bar
            RENAME .bozo.bar .bozo.baz
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFieldMutatedMoreThanOnceDuringMigration {
            type_name: "test_type".to_string(),
            from_path: vec!["bozo".to_string(), "bar".to_string()],
            to_path: vec!["bozo".to_string(), "baz".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            RENAME .bozo.foo .bozo.bar
            RENAME .bozo.bar .bozo.baz
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: Struct(
                    VersionedStructGeneric {
                        fields: [
                            (
                                "foo",
                                VersionedStructFieldGeneric {
                                    field_type: I64,
                                    field_index: 0,
                                    default_value: None,
                                    last_mutation_version: -1,
                                },
                            ),
                        ],
                    },
                ),
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_adding_and_renaming_field() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :{
                foo @0 :I64
            },
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :{
                baz @0 :I64
            },
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .bozo.bar @1 String DEFAULT 'thicc'
            RENAME .bozo.bar .bozo.baz
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFieldMutatedMoreThanOnceDuringMigration {
            type_name: "test_type".to_string(),
            from_path: vec!["bozo".to_string(), "bar".to_string()],
            to_path: vec!["bozo".to_string(), "baz".to_string()],
            from_version: 7,
            to_version: 8,
            migration_source: "
            ADD .bozo.bar @1 String DEFAULT 'thicc'
            RENAME .bozo.bar .bozo.baz
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: Struct(
                    VersionedStructGeneric {
                        fields: [
                            (
                                "foo",
                                VersionedStructFieldGeneric {
                                    field_type: I64,
                                    field_index: 0,
                                    default_value: None,
                                    last_mutation_version: -1,
                                },
                            ),
                        ],
                    },
                ),
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}

#[test]
fn test_bw_adding_and_dropping_same_field() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :{
                foo @0 :I64
            },
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :{
                baz @0 :I64
            },
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .bozo.bar @1 String DEFAULT 'thicc'
            DROP .bozo.bar
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeFieldMutatedMoreThanOnceDuringMigration {
            type_name: "test_type".to_string(),
            from_path: vec!["bozo".to_string(), "bar".to_string()],
            to_path: vec![],
            from_version: 7,
            to_version: 8,
            migration_source: "
            ADD .bozo.bar @1 String DEFAULT 'thicc'
            DROP .bozo.bar
        "
            .to_string(),
            pre_migration_snapshot: r#"VersionedStructGeneric {
    fields: [
        (
            "bozo",
            VersionedStructFieldGeneric {
                field_type: Struct(
                    VersionedStructGeneric {
                        fields: [
                            (
                                "foo",
                                VersionedStructFieldGeneric {
                                    field_type: I64,
                                    field_index: 0,
                                    default_value: None,
                                    last_mutation_version: -1,
                                },
                            ),
                        ],
                    },
                ),
                field_index: 0,
                default_value: None,
                last_mutation_version: -1,
            },
        ),
    ],
}"#
            .to_string(),
        }
    );
}


#[test]
fn test_bw_last_nested_field_cannot_be_optional() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 7,
        snapshot_source: "{
            bozo @0 :{
                foo @0 :I64
            },
        }"
    }, {
        version: 8,
        snapshot_source: "{
            bozo @0 :{
                foo @0 :I64,
                bozo @1 :I64?
            },
        }"
    }] WITH versioned_type_migration [{
        version: 8,
        migration_source: "
            ADD .bozo? @1 String
        "
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::BwTypeLastFieldCannotBeOptional {
            type_name: "test_type".to_string(),
            path: vec!["bozo".to_string()],
            to_version: 8,
            migration_source: "
            ADD .bozo? @1 String
        "
            .to_string(),
        }
    );
}

#[test]
fn test_bw_nested_optional_fields() {
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
        version: 4,
        snapshot_source: "{
          some_field @0 :I64,
          other_field @1 :F64 DEFAULT '1.23',
          coordinates @2 :{
            x @0 :F64 DEFAULT '0.0',
            yy @1 :F64 DEFAULT '0.0',
          }?,
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
          ADD .coordinates?.x @0 F64 DEFAULT '0.0'
          ADD .coordinates.y @1 F64 DEFAULT '0.0'
          ADD .coordinates.z @2 F64 DEFAULT '0.0'
        "
      },
      {
        version: 4,
        migration_source: "
          RENAME .coordinates.y .coordinates.yy
          DROP .coordinates.z
        "
      }
    ]
  }
]
"#,
    );
}
