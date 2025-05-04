use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};

use crate::database::TableRowPointerVersionedType;

use self::parser::{
    MigrationMutatorGeneric, MigrationMutatorUnvalidated, ValidVersionedStructType,
    VersionedStructFieldUnvalidated, VersionedStructGeneric, VersionedTypeUnvalidated,
};

use super::{projections::Projection, PlatformValidationError};

pub mod parser;

pub type MigrationMutator = MigrationMutatorGeneric<ValidVersionedStructType>;
pub type VersionedType = VersionedStructGeneric<ValidVersionedStructType>;

pub struct ComputedType {
    the_type: VersionedType,
    type_version: u16,
    version_hash: u64,
    migrations: Vec<MigrationMutator>,
}

pub fn compute_types(
    db: &crate::database::Database,
) -> Result<Projection<TableRowPointerVersionedType, Vec<ComputedType>>, PlatformValidationError> {
    let result = Projection::maybe_create(db.versioned_type().rows_iter(), |vtype| {
        let vtname = db.versioned_type().c_type_name(vtype);
        let mut version_index = HashSet::new();
        let mut snapshot_index: HashMap<i64, VersionedType> = HashMap::new();
        let mut migration_index: HashMap<i64, Vec<MigrationMutator>> = HashMap::new();
        let mut migration_source_index: HashMap<i64, &str> = HashMap::new();

        for snapshot in db
            .versioned_type()
            .c_children_versioned_type_snapshot(vtype)
        {
            let vtversion = db.versioned_type_snapshot().c_version(*snapshot);
            let _ = version_index.insert(vtversion);
            let source = db.versioned_type_snapshot().c_snapshot_source(*snapshot);
            let (_, mut parsed_snapshot) = parser::parse_migration_snapshot(parser::Span::new(source.as_str()))
                .map_err(|e| PlatformValidationError::BwTypeSnapshotSyntaxError {
                    type_name: vtname.clone(),
                    type_version: vtversion,
                    syntax_error: e.to_string(),
                    snapshot_source: source.clone(),
                })?;

            validate_parsed_snapshot(vtname, vtversion, &mut parsed_snapshot)?;
            let is_none = snapshot_index.insert(vtversion, parsed_snapshot).is_none();
            assert!(is_none);
        }

        for migration in db
            .versioned_type()
            .c_children_versioned_type_migration(vtype)
        {
            let vtversion = db.versioned_type_migration().c_version(*migration);
            let _ = version_index.insert(vtversion);
            let source = db.versioned_type_migration().c_migration_source(*migration);
            let (_, migration) =
                parser::parse_all_migration_lines(parser::Span::new(source.as_str())).map_err(|e| {
                    PlatformValidationError::BwTypeMigrationSyntaxError {
                        type_name: vtname.clone(),
                        type_version: vtversion,
                        syntax_error: e.to_string(),
                        migration_source: source.clone(),
                    }
                })?;

            validate_parsed_migration(vtname, vtversion, &migration, source)?;
            let is_none = migration_index.insert(vtversion, migration).is_none();
            assert!(is_none);
            let _ = migration_source_index.insert(vtversion, source.as_str());
        }

        if version_index.is_empty() {
            return Err(
                PlatformValidationError::BwTypeNoSnapshotsOrMigrationsFound {
                    type_name: vtname.clone(),
                },
            );
        }

        let mut versions_ordered: Vec<i64> = version_index.into_iter().collect();
        versions_ordered.sort();
        let mut result: Vec<ComputedType> = Vec::with_capacity(versions_ordered.len());

        if versions_ordered.len() == 1 {
            let the_version = versions_ordered[0];

            if migration_index.get(&the_version).is_some() {
                return Err(
                    PlatformValidationError::BwTypeWithOneVersionCannotHaveAMigration {
                        type_name: vtname.clone(),
                        type_version: the_version,
                    },
                );
            }

            result.push(ComputedType::new(
                the_version as u16,
                snapshot_index.get(&the_version).unwrap().clone(),
                Vec::new(),
            ));
        } else {
            // more than one version
            let first_version = versions_ordered[0];
            let last_version = *versions_ordered.last().unwrap();

            let first_version_snap = snapshot_index.get(&first_version);
            let first_missing_snapshot = first_version_snap.is_none();
            let last_missing_snapshot = snapshot_index.get(&last_version).is_none();
            if first_missing_snapshot || last_missing_snapshot {
                return Err(
                    PlatformValidationError::BwTypeFirstAndLastVersionMustHaveSnapshots {
                        type_name: vtname.clone(),
                        first_version,
                        last_version,
                        first_missing_snapshot,
                        last_missing_snapshot,
                    },
                );
            }

            if migration_index.get(&first_version).is_some() {
                return Err(
                    PlatformValidationError::BwTypeFirstVersionCantHaveMigration {
                        type_name: vtname.clone(),
                        first_version,
                    },
                );
            }

            let mut current_version = (*first_version_snap.unwrap()).clone();
            result.push(ComputedType::new(
                first_version as u16,
                current_version.clone(),
                Vec::new(),
            ));

            let mut prev_version = first_version;
            for version in &versions_ordered[1..] {
                let copy = current_version.clone();
                let migrations: Vec<MigrationMutator> = match migration_index.get(version) {
                    Some(migration) => {
                        let context = MigrationContext {
                            type_name: vtname.as_str(),
                            from_version: prev_version,
                            to_version: *version,
                            pre_migration: &copy,
                            migration_source: migration_source_index.get(version).unwrap(),
                        };
                        for m in migration {
                            apply_migration_to_version(&context, &mut current_version, m)?;
                        }
                        migration.clone()
                    }
                    None => {
                        return Err(
                            PlatformValidationError::BwTypeMigrationNotFoundForVersions {
                                type_name: vtname.clone(),
                                from_version: prev_version,
                                to_version: *version,
                            },
                        );
                    }
                };

                if let Some(expected_snapshot) = snapshot_index.get(version) {
                    if &current_version != expected_snapshot {
                        let expected_snapshot = format!("{:#?}", expected_snapshot);
                        let actual_snapshot = format!("{:#?}", current_version);
                        return Err(
                            PlatformValidationError::BwTypeSnapshotAndMigrationAreInconsistent {
                                type_name: vtname.clone(),
                                from_version: prev_version,
                                to_version: *version,
                                expected_snapshot,
                                actual_snapshot,
                                pre_migration_snapshot: format!("{:#?}", copy),
                                migration_source: migration_source_index
                                    .get(version)
                                    .unwrap()
                                    .to_string(),
                            },
                        );
                    }
                }

                result.push(ComputedType::new(
                    *version as u16,
                    current_version.clone(),
                    migrations,
                ));

                prev_version = *version;
            }
        }

        Ok(result)
    })?;

    Ok(result)
}

impl ComputedType {
    pub fn new(
        type_version: u16,
        the_type: VersionedType,
        migrations: Vec<MigrationMutator>,
    ) -> ComputedType {
        let mut hasher = DefaultHasher::new();
        the_type.hash(&mut hasher);
        let initial = hasher.finish() & 0x0000ffffffffffff;
        let version = (type_version as u64).to_le() << 48 & 0xffff000000000000;

        // first two bytes -> version
        // the rest -> hash
        let version_hash = initial | version;

        ComputedType {
            the_type,
            type_version,
            version_hash,
            migrations,
        }
    }

    pub fn version_hash(&self) -> u64 {
        self.version_hash
    }

    pub fn the_type(&self) -> &VersionedType {
        &self.the_type
    }

    pub fn type_version(&self) -> u16 {
        self.type_version
    }

    pub fn migrations(&self) -> &[MigrationMutator] {
        &self.migrations
    }
}

struct MigrationContext<'a> {
    type_name: &'a str,
    from_version: i64,
    to_version: i64,
    pre_migration: &'a VersionedType,
    migration_source: &'a str,
}

fn validate_parsed_migration(
    type_name: &str,
    type_version: i64,
    input: &[MigrationMutatorUnvalidated],
    migration_source: &str,
) -> Result<(), PlatformValidationError> {
    for i in input {
        match i {
            MigrationMutatorGeneric::AddField {
                field_path,
                opt_fields,
                field_type,
                default_value,
                ..
            } => {
                let last_idx = field_path.len() - 1;
                if opt_fields[last_idx] {
                    return Err(PlatformValidationError::BwTypeLastFieldCannotBeOptional {
                        type_name: type_name.to_string(),
                        path: field_path.clone(),
                        to_version: type_version,
                        migration_source: migration_source.to_string(),
                    });
                }

                if field_type.is_option() && default_value.is_some() {
                    return Err(PlatformValidationError::BwTypeMigrationOptionAndDefaultValueAreMutuallyExclusive {
                        type_name: type_name.to_string(),
                        to_version: type_version,
                        migration_source: migration_source.to_string(),
                        field_path: field_path.clone(),
                    });
                }

                if !field_type.is_option() && default_value.is_none() {
                    return Err(PlatformValidationError::BwTypeMigrationNewFieldMustHaveDefaultValueOrBeOptional {
                        type_name: type_name.to_string(),
                        to_version: type_version,
                        migration_source: migration_source.to_string(),
                        field_path: field_path.clone(),
                    });
                }
            }
            MigrationMutatorGeneric::DropField { .. } => {}
            MigrationMutatorGeneric::RenameField { .. } => {}
        };
    }

    Ok(())
}

fn apply_migration_to_version<'a>(
    ctx: &MigrationContext<'a>,
    to_mutate: &mut VersionedType,
    migration: &MigrationMutator,
) -> Result<(), PlatformValidationError> {
    match migration {
        MigrationMutatorGeneric::AddField {
            field_path,
            opt_fields,
            field_type,
            default_value,
            field_index,
        } => {
            assert!(!field_path.is_empty());
            let path = field_path.as_slice();
            let remaining_opt = opt_fields.as_slice();
            apply_add_field(
                ctx,
                path,
                path,
                remaining_opt,
                to_mutate,
                field_type,
                *field_index,
                default_value,
            )?;
        }
        MigrationMutatorGeneric::DropField { field_path } => {
            assert!(!field_path.is_empty());
            let path = field_path.as_slice();
            apply_drop_field(ctx, path, path, to_mutate)?;
        }
        MigrationMutatorGeneric::RenameField { from_path, to_path } => {
            assert!(!from_path.is_empty());
            assert!(!to_path.is_empty());

            let from_prefix = &from_path[0..from_path.len() - 1];
            let to_prefix = &to_path[0..to_path.len() - 1];
            if from_prefix != to_prefix {
                return Err(PlatformValidationError::BwTypeMigrationRenamingFieldIsOnlyAllowedAtTheSameStructLevel {
                    type_name: ctx.type_name.to_string(),
                    from_prefix: from_prefix.to_vec(),
                    to_prefix: to_prefix.to_vec(),
                    from_version: ctx.from_version,
                    to_version: ctx.to_version,
                    migration_source: ctx.migration_source.to_string(),
                    pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                });
            }

            if from_path[from_path.len() - 1] == to_path[to_path.len() - 1] {
                return Err(
                    PlatformValidationError::BwTypeMigrationRenamingFromAndToFieldsAreTheSame {
                        type_name: ctx.type_name.to_string(),
                        from_path: from_path.to_vec(),
                        to_path: to_path.to_vec(),
                        from_version: ctx.from_version,
                        to_version: ctx.to_version,
                        migration_source: ctx.migration_source.to_string(),
                        pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                    },
                );
            }

            apply_rename_field(ctx, from_path, to_path, from_path, to_path, to_mutate)?;
        }
    }

    Ok(())
}

fn apply_rename_field<'a>(
    ctx: &MigrationContext<'a>,
    from_full_path: &[String],
    to_full_path: &[String],
    remaining_from_path: &[String],
    remaining_to_path: &[String],
    to_mutate: &mut VersionedType,
) -> Result<(), PlatformValidationError> {
    assert_eq!(remaining_from_path.len(), remaining_to_path.len());

    let from_field = &remaining_from_path[0];
    let to_field = &remaining_to_path[0];
    let from_tail = &remaining_from_path[1..];
    let to_tail = &remaining_to_path[1..];

    let mut from_found = false;
    for (fname, fval) in &mut to_mutate.fields {
        // target field with such name found
        if fname == to_field && from_tail.is_empty() {
            return Err(
                PlatformValidationError::BwTypeMigrationRenamingFieldWithSuchNameAlreadyExists {
                    type_name: ctx.type_name.to_string(),
                    from_path: from_full_path.to_vec(),
                    to_path: to_full_path.to_vec(),
                    from_version: ctx.from_version,
                    to_version: ctx.to_version,
                    migration_source: ctx.migration_source.to_string(),
                    pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                },
            );
        }

        if fname == from_field {
            from_found = true;
            if !from_tail.is_empty() {
                // recurse
                match &mut fval.field_type {
                    ValidVersionedStructType::Struct(str) => {
                        apply_rename_field(
                            ctx,
                            from_full_path,
                            to_full_path,
                            from_tail,
                            to_tail,
                            str,
                        )?;
                    }
                    ValidVersionedStructType::Option(opt) if opt.is_struct() => {
                        if let ValidVersionedStructType::Struct(str) = opt.as_mut() {
                            apply_rename_field(
                                ctx,
                                from_full_path,
                                to_full_path,
                                from_tail,
                                to_tail,
                                str,
                            )?;
                        } else {
                            panic!("We check in if")
                        }
                    }
                    _ => {
                        return Err(PlatformValidationError::BwTypeMigrationRenamingFieldExpectedInnerStructFoundGroundType {
                            type_name: ctx.type_name.to_string(),
                            from_path: from_full_path.to_vec(),
                            to_path: to_full_path.to_vec(),
                            from_version: ctx.from_version,
                            to_version: ctx.to_version,
                            migration_source: ctx.migration_source.to_string(),
                            pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                        });
                    }
                }
            } else {
                // already mutated/added in this version
                if fval.last_mutation_version == ctx.to_version {
                    return Err(
                        PlatformValidationError::BwTypeFieldMutatedMoreThanOnceDuringMigration {
                            type_name: ctx.type_name.to_string(),
                            from_path: from_full_path.to_vec(),
                            to_path: to_full_path.to_vec(),
                            from_version: ctx.from_version,
                            to_version: ctx.to_version,
                            migration_source: ctx.migration_source.to_string(),
                            pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                        },
                    );
                }
                fval.last_mutation_version = ctx.to_version;
                *fname = to_field.clone();
            }
        }
    }

    if !from_found {
        return Err(
            PlatformValidationError::BwTypeMigrationRenamingFieldNotFound {
                type_name: ctx.type_name.to_string(),
                from_path: from_full_path.to_vec(),
                to_path: to_full_path.to_vec(),
                from_version: ctx.from_version,
                to_version: ctx.to_version,
                migration_source: ctx.migration_source.to_string(),
                pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
            },
        );
    }

    Ok(())
}

fn apply_drop_field<'a>(
    ctx: &MigrationContext<'a>,
    full_path: &[String],
    remaining_path: &[String],
    to_mutate: &mut VersionedType,
) -> Result<(), PlatformValidationError> {
    let my_field = &remaining_path[0];
    let tail = &remaining_path[1..];

    let res = to_mutate
        .fields
        .iter()
        .enumerate()
        .find_map(|(idx, (k, _))| if k == my_field { Some(idx) } else { None });

    match res {
        Some(found) => {
            if !tail.is_empty() {
                let (_fname, fval) = &mut to_mutate.fields[found];
                match &mut fval.field_type {
                    ValidVersionedStructType::Struct(str) => {
                        apply_drop_field(ctx, full_path, tail, str)?;
                        Ok(())
                    }
                    ValidVersionedStructType::Option(opt) if opt.is_struct() => {
                        if let ValidVersionedStructType::Struct(str) = opt.as_mut() {
                            apply_drop_field(ctx, full_path, tail, str)?;
                            Ok(())
                        } else {
                            panic!("We check in if")
                        }
                    }
                    _ => {
                        Err(PlatformValidationError::BwTypeMigrationExpectedNestedStructFieldToDropGotGround {
                            type_name: ctx.type_name.to_string(),
                            field_path: full_path.to_vec(),
                            from_version: ctx.from_version,
                            to_version: ctx.to_version,
                            migration_source: ctx.migration_source.to_string(),
                            pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                        })
                    }
                }
            } else {
                if to_mutate.fields[found].1.last_mutation_version == ctx.to_version {
                    return Err(
                        PlatformValidationError::BwTypeFieldMutatedMoreThanOnceDuringMigration {
                            type_name: ctx.type_name.to_string(),
                            from_path: full_path.to_vec(),
                            to_path: Vec::new(),
                            from_version: ctx.from_version,
                            to_version: ctx.to_version,
                            migration_source: ctx.migration_source.to_string(),
                            pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                        },
                    );
                }

                let fidx = to_mutate.fields[found].1.field_index;
                to_mutate.fields.remove(found);

                for (_, fval) in to_mutate.fields.iter_mut() {
                    if fval.field_index > fidx {
                        fval.field_index -= 1;
                    }
                }

                if to_mutate.fields.is_empty() {
                    return Err(
                        PlatformValidationError::BwTypeMigrationStructBecomesEmptyAfterFieldDrop {
                            type_name: ctx.type_name.to_string(),
                            field_path: full_path.to_vec(),
                            from_version: ctx.from_version,
                            to_version: ctx.to_version,
                            migration_source: ctx.migration_source.to_string(),
                            pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                        },
                    );
                }

                Ok(())
            }
        }
        None => Err(
            PlatformValidationError::BwTypeMigrationCannotFindFieldToDrop {
                type_name: ctx.type_name.to_string(),
                field_path: full_path.to_vec(),
                field_not_found: my_field.clone(),
                from_version: ctx.from_version,
                to_version: ctx.to_version,
                migration_source: ctx.migration_source.to_string(),
                pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
            },
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_add_field<'a>(
    ctx: &MigrationContext<'a>,
    full_path: &[String],
    remaining_path: &[String],
    remaining_opt: &[bool],
    to_mutate: &mut VersionedType,
    new_ftype: &ValidVersionedStructType,
    field_index: u32,
    default_value: &Option<String>,
) -> Result<(), PlatformValidationError> {
    let my_field = &remaining_path[0];
    let is_this_opt = remaining_opt[0];
    let tail = &remaining_path[1..];
    let tail_opt = &remaining_opt[1..];

    let mut nested_mutation = false;
    for (fname, ftype) in &mut to_mutate.fields {
        if fname == my_field {
            if tail.is_empty() {
                return Err(PlatformValidationError::BwTypeCannotAddDuplicateNameField {
                    type_name: ctx.type_name.to_string(),
                    field_path: full_path.to_vec(),
                    from_version: ctx.from_version,
                    to_version: ctx.to_version,
                    migration_source: ctx.migration_source.to_string(),
                    pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                });
            } else {
                match &mut ftype.field_type {
                    ValidVersionedStructType::Struct(str) => {
                        apply_add_field(
                            ctx,
                            full_path,
                            tail,
                            tail_opt,
                            str,
                            new_ftype,
                            field_index,
                            default_value,
                        )?;
                        nested_mutation = true;
                    }
                    ValidVersionedStructType::Option(opt) if opt.is_struct() => {
                        if let ValidVersionedStructType::Struct(str) = opt.as_mut() {
                            apply_add_field(
                                ctx,
                                full_path,
                                tail,
                                tail_opt,
                                str,
                                new_ftype,
                                field_index,
                                default_value,
                            )?;
                            nested_mutation = true;
                        } else {
                            panic!("We check in if")
                        }
                    }
                    _ => {
                        return Err(PlatformValidationError::BwTypeExpectedStructTypeGotGround {
                            type_name: ctx.type_name.to_string(),
                            field_path: full_path.to_vec(),
                            from_version: ctx.from_version,
                            to_version: ctx.to_version,
                            migration_source: ctx.migration_source.to_string(),
                            pre_migration_snapshot: format!("{:#?}", ctx.pre_migration),
                        });
                    }
                }
            }
        }
    }

    if !nested_mutation {
        if tail.is_empty() {
            let expected_field_index = to_mutate
                .fields
                .iter()
                .map(|(_, f)| f.field_index + 1)
                .max()
                .unwrap_or(0);
            if field_index != expected_field_index {
                return Err(
                    PlatformValidationError::BwTypeMigrationAddsInvalidFieldIndex {
                        type_name: ctx.type_name.to_string(),
                        type_version: ctx.to_version,
                        expected_field_index,
                        actual_field_index: field_index,
                        actual_field_index_name: my_field.clone(),
                        migration_source: ctx.migration_source.to_string(),
                    },
                );
            }

            let field = VersionedStructFieldUnvalidated {
                field_type: new_ftype.clone(),
                default_value: default_value.clone(),
                field_index,
                last_mutation_version: ctx.to_version,
            };
            to_mutate.fields.push((my_field.clone(), field));
        } else {
            let max_field = to_mutate
                .fields
                .iter()
                .map(|(_, f)| f.field_index + 1)
                .max();
            let mut this_struct = VersionedStructGeneric { fields: vec![] };
            apply_add_field(
                ctx,
                full_path,
                tail,
                tail_opt,
                &mut this_struct,
                new_ftype,
                field_index,
                default_value,
            )?;
            let mut final_type = ValidVersionedStructType::Struct(this_struct);
            if is_this_opt {
                final_type = ValidVersionedStructType::Option(Box::new(final_type));
            }
            let field = VersionedStructFieldUnvalidated {
                field_type: final_type,
                default_value: None,
                field_index: max_field.unwrap_or(0),
                last_mutation_version: ctx.to_version,
            };
            to_mutate.fields.push((my_field.clone(), field));
        }
    }

    to_mutate.fields.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    Ok(())
}

fn validate_parsed_snapshot(
    type_name: &str,
    type_version: i64,
    input: &mut VersionedTypeUnvalidated,
) -> Result<(), PlatformValidationError> {
    let mut fields_set = HashSet::new();

    input.fields.sort_by_key(|(_, v)| v.field_index);

    for (idx_count, (fname, field)) in input.fields.iter_mut().enumerate() {
        let idx_count: u32 = idx_count as u32;
        if field.field_index != idx_count {
            return Err(
                PlatformValidationError::BwTypeFieldIndexesMustBeZeroBasedSequential {
                    type_name: type_name.to_string(),
                    type_version,
                    expected_field_index: idx_count,
                    actual_field_index: field.field_index,
                    actual_field_index_name: fname.clone(),
                },
            );
        }

        match (&mut field.field_type, &field.default_value) {
            (ValidVersionedStructType::String, _) => {}
            (ValidVersionedStructType::DateTime, Some(dv)) => {
                if let Err(e) = chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(dv.as_str()) {
                    return Err(
                        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
                            type_name: type_name.to_string(),
                            version: type_version,
                            field_name: fname.clone(),
                            the_type: "DateTime".to_string(),
                            default_value: dv.clone(),
                            parsing_error: e.to_string(),
                        },
                    );
                }
            }
            (ValidVersionedStructType::UUID, Some(dv)) => {
                if let Err(e) = uuid::Uuid::parse_str(dv) {
                    return Err(
                        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
                            type_name: type_name.to_string(),
                            version: type_version,
                            field_name: fname.clone(),
                            the_type: "UUID".to_string(),
                            default_value: dv.clone(),
                            parsing_error: e.to_string(),
                        },
                    );
                }
            }
            (ValidVersionedStructType::I64, Some(dv)) => {
                if let Err(e) = dv.parse::<i64>() {
                    return Err(
                        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
                            type_name: type_name.to_string(),
                            version: type_version,
                            field_name: fname.clone(),
                            the_type: "I64".to_string(),
                            default_value: dv.clone(),
                            parsing_error: e.to_string(),
                        },
                    );
                }
            }
            (ValidVersionedStructType::F64, Some(dv)) => {
                if let Err(e) = dv.parse::<f64>() {
                    return Err(
                        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
                            type_name: type_name.to_string(),
                            version: type_version,
                            field_name: fname.clone(),
                            the_type: "F64".to_string(),
                            default_value: dv.clone(),
                            parsing_error: e.to_string(),
                        },
                    );
                }
            }
            (ValidVersionedStructType::Bool, Some(dv)) => {
                if let Err(e) = dv.parse::<bool>() {
                    return Err(
                        PlatformValidationError::BwTypeSnapshotCannotParseDefaultValueForType {
                            type_name: type_name.to_string(),
                            version: type_version,
                            field_name: fname.clone(),
                            the_type: "Bool".to_string(),
                            default_value: dv.clone(),
                            parsing_error: e.to_string(),
                        },
                    );
                }
            }
            (ValidVersionedStructType::Option(_), Some(_)) => {
                return Err(PlatformValidationError::BwTypeSnapshotOptionAndDefaultValueAreMutuallyExclusive {
                    type_name: type_name.to_string(),
                    version: type_version,
                    field_name: fname.clone(),
                });
            }
            (ValidVersionedStructType::Array(_), Some(_)) => {
                return Err(
                    PlatformValidationError::BwTypeSnapshotDefaultValuesForTypeAreNotSupported {
                        type_name: type_name.to_string(),
                        version: type_version,
                        field_name: fname.clone(),
                        the_type: "Array".to_string(),
                    },
                );
            }
            (ValidVersionedStructType::Struct(it), dv) => {
                if dv.is_some() {
                    return Err(PlatformValidationError::BwTypeSnapshotDefaultValuesForTypeAreNotSupported {
                        type_name: type_name.to_string(),
                        version: type_version,
                        field_name: fname.clone(),
                        the_type: "Struct".to_string(),
                    });
                }

                validate_parsed_snapshot(type_name, type_version, it)?;
            }
            _ => {}
        }

        if !fields_set.insert(fname.clone()) {
            return Err(PlatformValidationError::BwTypeSnapshotDuplicateFields {
                type_name: type_name.to_string(),
                type_version,
                duplicate_field_name: fname.clone(),
            });
        }
    }

    input.fields.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    Ok(())
}
