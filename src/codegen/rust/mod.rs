pub mod backend;
pub mod frontend;

use std::collections::{BTreeMap, HashMap};

use crate::{
    database::{
        TableRowPointerPgQuery, TableRowPointerRustCompilationEnvironment,
        TableRowPointerVersionedType, TableRowPointerChQuery, TableRowPointerChSchema,
    },
    static_analysis::{
        bw_compat_types::{parser::ValidVersionedStructType, ComputedType, MigrationMutator},
        databases::{postgres, clickhouse},
        http_endpoints::ValidHttpPrimitiveType,
        projections::Projection,
        CheckedDB,
    },
};
use convert_case::{Case, Casing};

use super::Directory;

pub fn generate_compile_envs(checked: &CheckedDB, dir: &mut Directory) {
    let db = &checked.db;
    for comp_env in checked.db.rust_compilation_environment().rows_iter() {
        let dir = dir.create_directory(
            checked
                .db
                .rust_compilation_environment()
                .c_env_name(comp_env),
        );
        let edition = db.rust_compilation_environment().c_rust_edition(comp_env);
        let src_dir = dir.create_directory("src");
        src_dir.create_file(
            "main.rs",
            "fn main() { println!(\"hello world\") }".to_string(),
        );
        dir.create_file(
            "Cargo.toml",
            rust_cargo_toml(edition.as_str(), checked, comp_env),
        );
    }
}

struct UsedLibraries {
    postgres: bool,
    nats: bool,
    http: bool,
    json_ser: bool,
    binary_ser: bool,
}

impl UsedLibraries {
    fn new() -> Self {
        Self::init(false)
    }

    fn init(v: bool) -> Self {
        Self {
            postgres: v,
            nats: v,
            http: v,
            json_ser: v,
            binary_ser: v,
        }
    }
}

pub struct GenratedPromVariable {
    pub initialization_body: String,
}

struct RustCodegenContext {
    prom_variables: BTreeMap<String, GenratedPromVariable>,
    transaction_script_types: Vec<String>,
    db_aux_types: BTreeMap<String, String>,
    used_libraries: UsedLibraries,
}

impl RustCodegenContext {
    fn new() -> RustCodegenContext {
        RustCodegenContext {
            prom_variables: BTreeMap::new(),
            transaction_script_types: Vec::new(),
            db_aux_types: BTreeMap::new(),
            used_libraries: UsedLibraries::new(),
        }
    }

    fn register_prom_variable(&mut self, name: String, initialization_body: String) {
        let res = self.prom_variables.insert(
            name,
            GenratedPromVariable {
                initialization_body: initialization_body.clone(),
            },
        );
        if let Some(res) = &res {
            assert_eq!(res.initialization_body, initialization_body);
        }
    }

    fn pg_query_output_type_name(&mut self, db: &CheckedDB, query: TableRowPointerPgQuery) -> String {
        let the_db = db.db.pg_query().c_parent(query);
        let checked_db = db.async_res.checked_pg_dbs.get(&the_db).unwrap();
        let db_name = db.db.pg_schema().c_schema_name(the_db);
        let checked_query = checked_db.queries.get(&query).unwrap();
        let qname = db.db.pg_query().c_query_name(query);
        let aux_struct_name = format!("Pgq{}Row{}", db_name.to_case(Case::Pascal), qname.to_case(Case::Pascal));
        let _ = self
            .db_aux_types
            .entry(aux_struct_name.clone())
            .or_insert_with(|| {
                let mut aux_type_def = String::new();
                aux_type_def += "pub struct ";
                aux_type_def += &aux_struct_name;
                aux_type_def += " {\n";

                for of in &checked_query.output_signature {
                    aux_type_def += "    pub ";
                    aux_type_def += &of.name;
                    aux_type_def += ": ";
                    if of.optional {
                        aux_type_def += "Option<";
                    }
                    aux_type_def += pg_db_type_to_rust_type(&of.the_type);
                    if of.optional {
                        aux_type_def += ">";
                    }
                    aux_type_def += ",\n";
                }

                aux_type_def += "}";
                aux_type_def
            });

        aux_struct_name
    }

    fn ch_query_output_type_name(&mut self, db: &CheckedDB, query: TableRowPointerChQuery) -> String {
        let the_db = db.db.ch_query().c_parent(query);
        let checked_db = db.async_res.checked_ch_dbs.get(&the_db).unwrap();
        let db_name = db.db.ch_schema().c_schema_name(the_db);
        let checked_query = checked_db.queries.get(&query).unwrap();
        let qname = db.db.ch_query().c_query_name(query);
        let aux_struct_name = format!("Chq{}Row{}", db_name.to_case(Case::Pascal), qname.to_case(Case::Pascal));
        let _ = self
            .db_aux_types
            .entry(aux_struct_name.clone())
            .or_insert_with(|| {
                let mut aux_type_def = String::new();
                aux_type_def += "pub struct ";
                aux_type_def += &aux_struct_name;
                aux_type_def += " {\n";

                for of in &checked_query.output_signature {
                    aux_type_def += "    pub ";
                    aux_type_def += &of.name;
                    aux_type_def += ": ";
                    if of.optional {
                        aux_type_def += "Option<";
                    }
                    aux_type_def += ch_db_type_to_rust_type(&of.the_type);
                    if of.optional {
                        aux_type_def += ">";
                    }
                    aux_type_def += ",\n";
                }

                aux_type_def += "}";
                aux_type_def
            });

        aux_struct_name
    }

    fn ch_inserter_input_type_name(&mut self, db: &CheckedDB, db_schema: TableRowPointerChSchema, table_name: &str) -> String {
        let db_name = db.db.ch_schema().c_schema_name(db_schema);
        let checked_schema = db.async_res.checked_ch_dbs.get(&db_schema).unwrap();
        let latest_schema = checked_schema.schema_snapshots.values().rev().next().unwrap();
        let columns = latest_schema.field_type_index.get(table_name).unwrap();
        let aux_struct_name = format!("ChIns{}Row{}", db_name.to_case(Case::Pascal), table_name.to_case(Case::Pascal));
        let _ = self
            .db_aux_types
            .entry(aux_struct_name.clone())
            .or_insert_with(|| {
                let mut aux_type_def = String::new();
                aux_type_def += "pub struct ";
                aux_type_def += &aux_struct_name;
                aux_type_def += " {\n";

                for (cname, ctype) in &columns.fields {
                    if !ctype.insertion_allowed {
                        continue;
                    }

                    aux_type_def += "    pub ";
                    aux_type_def += &cname;
                    aux_type_def += ": ";
                    if ctype.has_default {
                        aux_type_def += "Option<";
                    }
                    aux_type_def += ch_str_type_to_rust_type(&ctype.col_type);
                    if ctype.has_default {
                        aux_type_def += ">";
                    }
                    aux_type_def += ",\n";
                }

                aux_type_def += "}";
                aux_type_def
            });

        aux_struct_name
    }
}

pub struct RustGeneratedFunction {
    pub function_body: String,
    pub function_name: String,
}

pub struct RustVersionedTypeSnippets {
    pub struct_definitions: String,
    pub version_struct_names: HashMap<u16, String>,
    pub binary_deserialization_function: RustGeneratedFunction,
    pub binary_serialization_function: RustGeneratedFunction,
    pub json_deserialization_function: RustGeneratedFunction,
    pub json_serialization_function: RustGeneratedFunction,
    pub migration_functions: String,
    pub nominal_type_name: String,
}

pub struct GeneratedRustSourceForHttpEndpoint {
    pub rust_endpoint_declaration: String,
    pub rust_args_struct_name: String,
    pub rust_args_struct_definition: String,
    pub rust_output_struct_name: String,
    prometheus_variables: Vec<(String, GenratedPromVariable)>,
}

pub fn compute_snippets(
    db: &crate::database::Database,
    ct: &Projection<TableRowPointerVersionedType, Vec<ComputedType>>,
) -> Projection<TableRowPointerVersionedType, RustVersionedTypeSnippets> {
    ct.derive_another(
        |ptr: TableRowPointerVersionedType, ct: &Vec<ComputedType>| -> RustVersionedTypeSnippets {
            let mut v_struct_idx = HashMap::new();
            let mut aux_types_idx: HashMap<u16, HashMap<Vec<String>, String>> = HashMap::new();
            let nominal_type = format!(
                "BwType{}",
                db.versioned_type().c_type_name(ptr).to_case(Case::Pascal)
            );
            let struct_definitions = generate_struct_for_versions(
                db,
                ptr,
                ct,
                &mut v_struct_idx,
                &mut aux_types_idx,
                &nominal_type,
            );
            RustVersionedTypeSnippets {
                struct_definitions,
                binary_serialization_function: generate_binary_serialization_function(
                    db,
                    ptr,
                    ct,
                    &nominal_type,
                ),
                binary_deserialization_function: generate_binary_deserialization_function(
                    db,
                    ptr,
                    ct,
                    &v_struct_idx,
                    &nominal_type,
                ),
                json_serialization_function: generate_json_serialization_function(
                    db,
                    ptr,
                    &nominal_type,
                ),
                json_deserialization_function: generate_json_deserialization_function(
                    db,
                    ptr,
                    ct,
                    &v_struct_idx,
                    &nominal_type,
                ),
                migration_functions: generate_migration_functions(
                    db,
                    ptr,
                    ct,
                    &v_struct_idx,
                    &aux_types_idx,
                ),
                version_struct_names: v_struct_idx,
                nominal_type_name: nominal_type,
            }
        },
    )
}

fn generate_binary_deserialization_function(
    db: &crate::database::Database,
    ptr: TableRowPointerVersionedType,
    ct: &Vec<ComputedType>,
    index: &HashMap<u16, String>,
    nominal_type: &str,
) -> RustGeneratedFunction {
    let mut ser_func = String::with_capacity(256);

    let last_version = ct.last().unwrap();
    let fname = format!(
        "bw_type_{}_deserialize_bin",
        db.versioned_type().c_type_name(ptr).to_case(Case::Snake)
    );
    ser_func += "fn ";
    ser_func += &fname;
    ser_func += "(input: &[u8]) -> Result<";
    ser_func += nominal_type;
    ser_func += ", BinaryDeserializationError> {\n";
    ser_func += "    let cursor_size = input.len();\n";
    ser_func += "    let mut cursor = ::std::io::Cursor::new(input);\n";
    ser_func += "    let header: u64 = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::MessageTooShort)?;\n";
    ser_func += "    let version: u16 = ((header & 0xffff0000000000)>>48).try_into().unwrap();\n";
    ser_func += "    match version {\n";
    for version in ct.iter().rev() {
        // iterate from latest version to last, optimistic
        let this_v_name = index.get(&version.type_version()).unwrap();
        ser_func += "        ";
        ser_func += &version.type_version().to_string();
        ser_func += " => {\n";
        ser_func += "            if header != ";
        ser_func += &version.version_hash().to_string();
        ser_func += &format!(" {{ return Err(BinaryDeserializationError::VersionHashMismatch {{ expected: {}, actual: header }}) }}\n", version.version_hash());
        ser_func += "            let the_val: ";
        ser_func += this_v_name;
        // this migration needs to be generated
        ser_func += " = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::CorruptedData)?;\n";
        ser_func += "            if (cursor.position() as usize) < cursor_size { return Err(BinaryDeserializationError::ExtraBytesLeft) }\n";
        for later_version in ct.as_slice().windows(2) {
            let from_v = later_version[0].type_version();
            let to_v = later_version[1].type_version();
            if from_v >= version.type_version() {
                ser_func += &format!(
                    "            let the_val = bw_type_{}_v{}_to_v{}(the_val);\n",
                    db.versioned_type().c_type_name(ptr).to_case(Case::Snake),
                    from_v,
                    to_v
                );
            }
        }
        ser_func += "            Ok(the_val)\n";
        ser_func += "        }\n";
    }

    ser_func += "        unknown_version => {\n";
    ser_func += "            if unknown_version > ";
    ser_func += &last_version.type_version().to_string();
    ser_func +=
        " { return Err(BinaryDeserializationError::UnsupportedVersionYet(unknown_version)) }\n";
    ser_func += "            Err(BinaryDeserializationError::UnknownVersion(unknown_version))\n";
    ser_func += "        }\n";

    ser_func += "    }\n";
    ser_func += "}\n";
    ser_func += "\n";

    RustGeneratedFunction {
        function_body: ser_func,
        function_name: fname,
    }
}

fn generate_json_deserialization_function(
    db: &crate::database::Database,
    ptr: TableRowPointerVersionedType,
    ct: &Vec<ComputedType>,
    index: &HashMap<u16, String>,
    nominal_type: &str,
) -> RustGeneratedFunction {
    let mut ser_func = String::with_capacity(256);

    let fname = format!(
        "bw_type_{}_deserialize_json",
        db.versioned_type().c_type_name(ptr).to_case(Case::Snake)
    );
    ser_func += "fn ";
    ser_func += &fname;
    ser_func += "(input: &[u8]) -> Result<";
    ser_func += nominal_type;
    ser_func += ", JsonDeserializationError> {\n";
    for version in ct.iter().rev() {
        // iterate from latest version to last, optimistic
        let this_v_name = index.get(&version.type_version()).unwrap();
        ser_func += "    if let Ok(the_val) = ::serde_json::from_slice::<";
        ser_func += this_v_name;
        ser_func += ">(input) {\n";
        for later_version in ct.as_slice().windows(2) {
            let from_v = later_version[0].type_version();
            let to_v = later_version[1].type_version();
            if from_v >= version.type_version() {
                ser_func += &format!(
                    "        let the_val = bw_type_{}_v{}_to_v{}(the_val);\n",
                    db.versioned_type().c_type_name(ptr).to_case(Case::Snake),
                    from_v,
                    to_v
                );
            }
        }
        ser_func += "        return Ok(the_val);\n";
        ser_func += "    }\n";
    }

    ser_func += "    return Err(JsonDeserializationError::UnknownType);\n";

    ser_func += "}\n";
    ser_func += "\n";

    RustGeneratedFunction {
        function_body: ser_func,
        function_name: fname,
    }
}

fn generate_migration_functions(
    db: &crate::database::Database,
    ptr: TableRowPointerVersionedType,
    ct: &Vec<ComputedType>,
    index: &HashMap<u16, String>,
    aux_index: &HashMap<u16, HashMap<Vec<String>, String>>,
) -> String {
    let mut ser_func = String::with_capacity(64);
    for later_version in ct.as_slice().windows(2) {
        let from_v = later_version[0].type_version();
        let to_v = later_version[1].type_version();
        let aux_index = aux_index.get(&to_v).unwrap();
        let later_version = &later_version[1];
        ser_func += &format!(
            "fn bw_type_{}_v{}_to_v{}(input: {}) -> {} {{\n",
            db.versioned_type().c_type_name(ptr).to_case(Case::Snake),
            from_v,
            to_v,
            index.get(&from_v).unwrap(),
            index.get(&to_v).unwrap(),
        );

        ser_func += &format!("    {} {{\n", index.get(&to_v).unwrap());
        for (fname, fval) in &later_version.the_type().fields {
            generate_inner_fields_from_migration(
                &mut ser_func,
                8,
                &vec![fname.clone()],
                &fval.field_type,
                later_version.migrations(),
                aux_index,
            );
        }
        ser_func += "    }\n";

        ser_func += "}\n";
    }
    ser_func
}

fn generate_inner_fields_from_migration(
    output: &mut String,
    padding: u16,
    path: &Vec<String>,
    fval: &ValidVersionedStructType,
    migrations: &[MigrationMutator],
    aux_index: &HashMap<Vec<String>, String>,
) {
    for _ in 0..padding {
        *output += " ";
    }
    let last_seg = path.last().unwrap();
    *output += last_seg;
    *output += ": ";
    let found_mig = migrations.iter().find(|i| match i {
        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::AddField {
            field_path,
            ..
        } => field_path.starts_with(path),
        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::DropField {
            field_path,
        } => field_path == path,
        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::RenameField {
            to_path,
            ..
        } => to_path == path,
    });

    let needs_quotes = fval == &ValidVersionedStructType::String;
    match fval {
        ValidVersionedStructType::String
        | ValidVersionedStructType::I64
        | ValidVersionedStructType::F64
        | ValidVersionedStructType::Bool => {
            match found_mig {
                Some(mig) => {
                    match mig {
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::AddField { default_value, .. } => {
                            if needs_quotes { *output += "r#\""};
                            *output += default_value.as_ref().unwrap();
                            if needs_quotes { *output += "\"#.to_string()"};
                            *output += ",\n";
                        },
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::RenameField { from_path, .. } => {
                            *output += "input.";
                            *output += &from_path.join(".");
                            *output += ",\n";
                        },
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::DropField { .. } => {
                            panic!("Drop field should never be found in new struct")
                        },
                    }
                },
                None => {
                    // just assign the same field
                    *output += "input.";
                    *output += &path.join(".");
                    *output += ",\n";
                },
            }
        }
        ValidVersionedStructType::Struct(inner) => {
            let aux_type_name = aux_index.get(path).unwrap();
            *output += aux_type_name;
            *output += " {\n";

            for (key, fval) in &inner.fields {
                let mut path = path.to_vec();
                path.push(key.clone());
                generate_inner_fields_from_migration(
                    output,
                    padding + 4,
                    &path,
                    &fval.field_type,
                    migrations,
                    aux_index,
                )
            }

            for _ in 0..padding {
                *output += " ";
            }
            *output += "},\n";
        }
        ValidVersionedStructType::Option(inner_t) => {
            match found_mig {
                Some(mig) => {
                    match mig {
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::AddField { .. } => {
                            *output += "None,\n";
                        },
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::RenameField { from_path, .. } => {
                            if !inner_t.is_struct() {
                                *output += "input.";
                                *output += &from_path.join(".");
                            } else {
                                panic!("Implement inner value remap for structs")
                            }
                            *output += ",\n";
                        },
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::DropField { .. } => {
                            panic!("Should never heppen in new struct")
                        },
                    }
                },
                None => {
                    match inner_t.as_ref() {
                        ValidVersionedStructType::Struct(inner) => {
                            let aux_type_name = aux_index.get(path).unwrap();

                            *output += "input.";
                            *output += &path.join(".");
                            *output += ".map(|input| ";
                            *output += aux_type_name;
                            *output += " {\n";

                            for (key, fval) in &inner.fields {
                                let path = vec![key.clone()];
                                generate_inner_fields_from_migration(output, padding + 4, &path, &fval.field_type, migrations, aux_index)
                            }

                            for _ in 0..padding {
                                *output += " ";
                            }
                            *output += "}),\n";
                        },
                        _ => {
                            // just assign the same field
                            *output += "input.";
                            *output += &path.join(".");
                            *output += ",\n";
                        }
                    }
                },
            }
        }
        ValidVersionedStructType::Array(inner_t) => {
            match found_mig {
                Some(mig) => {
                    match mig {
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::AddField { .. } => {
                            *output += "Vec::new(),\n";
                        },
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::RenameField { from_path, .. } => {
                            *output += "input.";
                            *output += &from_path.join(".");
                            *output += ",\n";
                        },
                        crate::static_analysis::bw_compat_types::parser::MigrationMutatorGeneric::DropField { .. } => {
                            panic!("Should never happen in new struct")
                        },
                    }
                },
                None => {
                    // just assign the same field
                    if !inner_t.is_struct() {
                        *output += "input.";
                        *output += &path.join(".");
                    } else {
                        panic!("Implement array of struct reassign")
                    }
                    *output += ",\n";
                },
            }
        }
    }
}

fn generate_binary_serialization_function(
    db: &crate::database::Database,
    ptr: TableRowPointerVersionedType,
    ct: &[ComputedType],
    nominal_type: &str,
) -> RustGeneratedFunction {
    let mut ser_func = String::with_capacity(256);

    let last_version = ct.last().unwrap();
    let fname = format!(
        "bw_type_{}_serialize_bin",
        db.versioned_type().c_type_name(ptr).to_case(Case::Snake)
    );
    ser_func += "fn ";
    ser_func += &fname;
    ser_func += "(input: &";
    ser_func += nominal_type;
    ser_func += ") -> Vec<u8> {\n";
    ser_func += "    let mut output = Vec::with_capacity(32);\n";
    ser_func += "    ::bincode::serialize_into(&mut output, &(";
    ser_func += &format!("{:#018x}", last_version.version_hash());
    ser_func += " as u64)).expect(\"should never happen\");\n";
    ser_func +=
        "    ::bincode::serialize_into(&mut output, input).expect(\"should never happen\");\n";
    ser_func += "    output\n";
    ser_func += "}\n";

    RustGeneratedFunction {
        function_body: ser_func,
        function_name: fname,
    }
}

fn generate_json_serialization_function(
    db: &crate::database::Database,
    ptr: TableRowPointerVersionedType,
    nominal_type: &str,
) -> RustGeneratedFunction {
    let mut ser_func = String::with_capacity(256);

    let fname = format!(
        "bw_type_{}_serialize_json",
        db.versioned_type().c_type_name(ptr).to_case(Case::Snake)
    );
    ser_func += "fn ";
    ser_func += &fname;
    ser_func += "(input: &";
    ser_func += nominal_type;
    ser_func += ") -> String {\n";
    ser_func += "    ::serde_json::to_string(input).expect(\"should never happen\")\n";
    ser_func += "}\n";

    RustGeneratedFunction {
        function_body: ser_func,
        function_name: fname,
    }
}

fn generate_struct_for_versions(
    db: &crate::database::Database,
    ptr: TableRowPointerVersionedType,
    ct: &Vec<ComputedType>,
    index: &mut HashMap<u16, String>,
    aux_types: &mut HashMap<u16, HashMap<Vec<String>, String>>,
    nominal_type: &str,
) -> String {
    let mut final_defs = String::with_capacity(256);

    let mut last_type = String::new();
    for v in ct {
        let vtype_name = format!(
            "BwType{}V{}",
            db.versioned_type().c_type_name(ptr).to_case(Case::Pascal),
            v.type_version()
        );
        let ires = index.insert(v.type_version(), vtype_name.clone());
        assert!(ires.is_none());
        let vtype_aux_types = format!("{}Aux", vtype_name);

        let mut sd = String::with_capacity(64);

        sd += "#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]\npub struct ";
        sd += &vtype_name;
        sd += " {\n";
        last_type = vtype_name;

        let mut rtb = RustTypeBuilder {
            aux_type_prefix: vtype_aux_types,
            aux_types: vec![],
            aux_names_index: HashMap::new(),
        };
        // todo: finish generating all rust for bw types
        for (k, v) in &v.the_type().fields {
            sd += "    pub ";
            sd += k;
            sd += ": ";
            bw_type_to_rust_type(&[k.clone()], &v.field_type, &mut sd, &mut rtb);
            sd += ",\n";
        }

        sd += "}\n";

        for at in &rtb.aux_types {
            final_defs += at;
            final_defs += "\n";
        }

        let r = aux_types.insert(v.type_version(), rtb.aux_names_index);
        assert!(r.is_none());

        final_defs += &sd;
    }

    assert!(!last_type.is_empty());
    final_defs += &format!("pub type {} = {};\n", nominal_type, last_type);

    final_defs
}

struct RustTypeBuilder {
    aux_type_prefix: String,
    aux_types: Vec<String>,
    aux_names_index: HashMap<Vec<String>, String>,
}

fn bw_type_to_rust_type(
    path: &[String],
    input: &ValidVersionedStructType,
    output: &mut String,
    rtb: &mut RustTypeBuilder,
) {
    match input {
        ValidVersionedStructType::String => {
            output.push_str("String");
        }
        ValidVersionedStructType::I64 => {
            output.push_str("i64");
        }
        ValidVersionedStructType::F64 => {
            output.push_str("f64");
        }
        ValidVersionedStructType::Bool => {
            output.push_str("bool");
        }
        ValidVersionedStructType::Option(inner) => {
            output.push_str("Option<");
            bw_type_to_rust_type(path, inner, output, rtb);
            output.push('>');
        }
        ValidVersionedStructType::Array(inner) => {
            output.push_str("Vec<");
            bw_type_to_rust_type(path, inner, output, rtb);
            output.push('>');
        }
        ValidVersionedStructType::Struct(inner) => {
            // increase count, will set later
            rtb.aux_types.push("".to_string());
            let idx = rtb.aux_types.len() - 1;
            let aux_type_name = format!("{}{}", rtb.aux_type_prefix, rtb.aux_types.len());
            let r = rtb
                .aux_names_index
                .insert(path.to_vec(), aux_type_name.clone());
            assert!(r.is_none());
            let mut now_aux_type = format!(
                "#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]\npub struct {} {{\n",
                aux_type_name
            );
            for (k, f) in &inner.fields {
                let mut new_path = path.to_vec();
                new_path.push(k.clone());

                now_aux_type.push_str("    pub ");
                now_aux_type.push_str(k);
                now_aux_type.push_str(": ");
                bw_type_to_rust_type(&new_path, &f.field_type, &mut now_aux_type, rtb);
                now_aux_type.push_str(",\n");
            }
            now_aux_type.push('}');

            rtb.aux_types[idx] = now_aux_type;
            output.push_str(&aux_type_name);
        }
    }
}

pub fn rust_cargo_toml(
    edition: &str,
    db: &CheckedDB,
    env: TableRowPointerRustCompilationEnvironment,
) -> String {
    // TODO: figure how to cache build artefacts across build with flake
    // if we change app name or version then dependencies get rebuilt
    let mut res = format!(
        r#"[package]
name = "epl-app"
version = "0.1.0"
edition = "{edition}"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
"#
    );

    for cr in db
        .db
        .rust_compilation_environment()
        .c_children_rust_crate_version(env)
    {
        let cname = db.db.rust_crate_version().c_crate_name(*cr);
        let features = db.db.rust_crate_version().c_features(*cr);
        let version = db.db.rust_crate_version().c_version(*cr);
        let default_features = db.db.rust_crate_version().c_default_features(*cr);

        if features.is_empty() && default_features {
            res += &format!("{cname} = \"{version}\"\n");
        } else {
            let feat = features
                .split('+')
                .map(|i| format!("\"{i}\""))
                .collect::<Vec<_>>()
                .join(", ");
            let maybe_disable_defaults =
                if !default_features {
                    " default-features = false,"
                } else { "" };
            res += &format!("{cname} = {{ version = \"{version}\",{maybe_disable_defaults} features = [{feat}] }}\n");
        }
    }

    res
}

pub fn http_type_to_rust_type(input: ValidHttpPrimitiveType) -> &'static str {
    match input {
        ValidHttpPrimitiveType::Int => "i64",
        ValidHttpPrimitiveType::Float => "f64",
        ValidHttpPrimitiveType::Bool => "bool",
        ValidHttpPrimitiveType::Text => "String",
    }
}

pub fn pg_db_type_to_rust_type(vt: &postgres::ValidDbType) -> &'static str {
    match vt {
        postgres::ValidDbType::INT => "i32",
        postgres::ValidDbType::BIGINT => "i64",
        postgres::ValidDbType::FLOAT => "f32",
        postgres::ValidDbType::DOUBLE => "f64",
        postgres::ValidDbType::BOOL => "bool",
        postgres::ValidDbType::TEXT => "String",
    }
}

pub fn ch_db_type_to_rust_type(vt: &clickhouse::ValidDbType) -> &'static str {
    match vt {
        clickhouse::ValidDbType::Int32 => "i32",
        clickhouse::ValidDbType::Int64 => "i64",
        clickhouse::ValidDbType::Int128 => "i128",
        clickhouse::ValidDbType::Int256 => "::num256::Int256",
        clickhouse::ValidDbType::Float32 => "f32",
        clickhouse::ValidDbType::Float64 => "f64",
        clickhouse::ValidDbType::Bool => "bool",
        clickhouse::ValidDbType::String => "String",
        clickhouse::ValidDbType::DateTime => "::chrono::DateTime<::chrono::Utc>",
        clickhouse::ValidDbType::Date => "::chrono::NaiveDate",
    }
}

pub fn ch_str_type_to_rust_type(vt: &str) -> &'static str {
    match vt {
        "Int32" => "i32",
        "Int64" => "i64",
        "Int128" => "i128",
        "Int256" => "::num256::Int256",
        "Float32" => "f32",
        "Float64" => "f64",
        "Bool" => "bool",
        "String" => "String",
        "Date" => "::chrono::NaiveDate",
        "DateTime" => "::chrono::DateTime<::chrono::Utc>",
        other => panic!("Unknown ch type to rust type mapping: {other}")
    }
}

pub fn json_deserialization_error_types() -> &'static str {
    r#"
#[derive(Debug)]
pub enum JsonDeserializationError {
    UnknownType,
}

impl std::fmt::Display for JsonDeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "json deserialization error: {:?}", self)
    }
}

impl std::error::Error for JsonDeserializationError {}

"#
}
