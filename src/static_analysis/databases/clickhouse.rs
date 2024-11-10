use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use std::fmt::Write;
use serde::Deserialize;
use nom::branch::alt;
use nom::character::complete::anychar;
use nom::multi::many0;
use nom::Parser;

use crate::database::{TableRowPointerChQuery, TableRowPointerChSchema, Database, TableRowPointerChTestDataset, TableRowPointerRegion, TableRowPointerChMutator};
use crate::static_analysis::bw_compat_types::parser::ValidVersionedStructType;
use crate::static_analysis::{AsyncCheckContext, Projections};
use crate::static_analysis::projections::Projection;

use super::super::PlatformValidationError;
use super::queries::TestTablesData;

pub fn sync_checks(db: &crate::database::Database) -> Result<(), PlatformValidationError> {
    for tquery in db.ch_query().rows_iter() {
        if db.ch_query().c_children_ch_query_test(tquery).is_empty() {
            return Err(PlatformValidationError::ChQueryHasNoTests {
                ch_schema: db.ch_schema().c_schema_name(db.ch_query().c_parent(tquery)).clone(),
                query_name: db.ch_query().c_query_name(tquery).clone(),
                original_query: db.ch_query().c_query_expression(tquery).clone(),
            });
        }
    }

    for chm in db.ch_mutator().rows_iter() {
        if db.ch_mutator().c_children_ch_mutator_test(chm).is_empty() {
            return Err(PlatformValidationError::ChMutatorHasNoTests {
                ch_schema: db.ch_schema().c_schema_name(db.ch_mutator().c_parent(chm)).clone(),
                mutator_name: db.ch_mutator().c_mutator_name(chm).clone(),
                mutator_expression: db.ch_mutator().c_mutator_expression(chm).clone(),
            });
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutputSignatureField {
    pub name: String,
    pub the_type: ValidDbType,
    pub optional: bool,
}

pub struct VerifiedQuery {
    pub query_ptr: TableRowPointerChQuery,
    pub full_query: FullQuery,
    pub output_signature: Vec<OutputSignatureField>,
}

pub struct VerifiedMutator {
    pub mutator_ptr: TableRowPointerChMutator,
    pub full_query: FullQuery,
}

pub struct EnrichedChDbData {
    pub db: TableRowPointerChSchema,
    pub queries: HashMap<TableRowPointerChQuery, VerifiedQuery>,
    pub mutators: HashMap<TableRowPointerChMutator, VerifiedMutator>,
    pub schema_snapshots: BTreeMap<i64, DbSchemaSnapshot>,
}

pub async fn validations(ctx: AsyncCheckContext, db: &crate::database::Database, proj: &Projections) -> Result<HashMap<TableRowPointerChSchema, EnrichedChDbData>, PlatformValidationError> {
    let mut dbs = Vec::new();
    for db_ptr in db.ch_schema().rows_iter() {
        dbs.push(verify_single_db(ctx.clone(), db, db_ptr, proj));
    }

    let enriched: Vec<EnrichedChDbData> = super::super::join_validation_errors(dbs).await?;

    let mut res: HashMap<TableRowPointerChSchema, EnrichedChDbData> = HashMap::with_capacity(enriched.len());

    for e in enriched {
        let r = res.insert(e.db, e);
        assert!(r.is_none());
    }

    check_nats_streams_compatibility(db, &res, proj)?;

    Ok(res)
}

pub async fn verify_single_db(ctx: AsyncCheckContext, db: &crate::database::Database, db_ptr: TableRowPointerChSchema, proj: &Projections) -> Result<EnrichedChDbData, PlatformValidationError> {
    match verify_single_db_iter(ctx.clone(), db, db_ptr, proj).await {
        Ok(res) => { return Ok(res) },
        Err(e) => {
            return Err(e);
        }
    }
}

async fn verify_single_db_iter(ctx: AsyncCheckContext, db: &Database, db_ptr: TableRowPointerChSchema, proj: &Projections) -> Result<EnrichedChDbData, PlatformValidationError> {
    let max_migration_time = static_db_checks(db, db_ptr)?;

    let mut enr = EnrichedChDbData {
        db: db_ptr,
        queries: HashMap::with_capacity(db.ch_schema().c_children_ch_query(db_ptr).len()),
        mutators: HashMap::with_capacity(db.ch_schema().c_children_ch_mutator(db_ptr).len()),
        schema_snapshots: BTreeMap::new(),
    };

    let datasets = parse_db_datasets(db, db_ptr)?;

    // hardcode, maybe have multiple version tests in the future?
    // clickhouse/clickhouse-server:24.6.2.17
    let img = "clickhouse/clickhouse-server@sha256:52ff41bee2400c5ff145ae2f3362c042b1b52fb0e250484427c89e97d8265f05";
    let db_cont = get_temp_db_with_retries(ctx, db.ch_schema().c_schema_name(db_ptr), img, 3)
        .await.expect("Can't start clickhouse database with retries");
    let admin_client = clickhouse::Client::default()
        .with_url(&db_cont.connection_string);
    let user_client = clickhouse::Client::default()
        .with_url(&db_cont.connection_string)
        .with_database(&db_cont.database)
        .with_user(&db_cont.user)
        .with_password(&db_cont.pass);
    let writer_client = clickhouse::Client::default()
        .with_url(&db_cont.connection_string)
        .with_database(&db_cont.database)
        .with_user(&db_cont.user_writer)
        .with_password(&db_cont.pass);

    verify_upgrades_with_queries(
        &user_client, &writer_client, &admin_client,
        &db_cont.connection_string, db, db_ptr,
        &mut enr, &datasets,
        max_migration_time
    ).await?;

    check_backend_inserter_existence(&enr, db, db_ptr, proj)?;

    verify_consistent_downgrades(
        &user_client, &admin_client, db, db_ptr, &enr.schema_snapshots
    ).await?;

    Ok(enr)
}

fn check_nats_streams_compatibility(db: &Database, schemas: &HashMap<TableRowPointerChSchema, EnrichedChDbData>, proj: &Projections) -> Result<(), PlatformValidationError> {
    for si in db.ch_nats_stream_import().rows_iter() {
        let depl_schema = db.ch_nats_stream_import().c_parent(si);
        let into_table = db.ch_nats_stream_import().c_into_table(si);
        let ch_cluster = db.ch_deployment_schemas().c_parent(depl_schema);
        let db_name = db.ch_deployment_schemas().c_db_name(depl_schema);
        let consumer = db.ch_nats_stream_import().c_consumer_name(si);
        let ch_schema = db.ch_deployment_schemas().c_ch_schema(depl_schema);
        let ch_schema_name = db.ch_schema().c_schema_name(ch_schema);
        let stream = db.ch_nats_stream_import().c_stream(si);
        let vtype = db.nats_jetstream_stream().c_stream_type(stream);
        let vtype_data = proj.versioned_types.value(vtype);
        let vtype_name = db.versioned_type().c_type_name(vtype);
        let last_type = vtype_data.last().unwrap();
        let enable_subjects = db.nats_jetstream_stream().c_enable_subjects(stream);

        let schema = schemas.get(&ch_schema).unwrap();
        let last_snapshot = schema.schema_snapshots.values().rev().next();

        if last_snapshot.is_none() {
            return Err(PlatformValidationError::ChNatsStreamIntoTableDoesntExist {
                ch_nats_stream_import_consumer_name: consumer.clone(),
                ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                ch_database: db_name.clone(),
                ch_schema: ch_schema_name.clone(),
                into_table: into_table.clone(),
                existing_tables: Vec::new(),
            });
        }

        let last_snapshot = last_snapshot.unwrap();
        let the_table = last_snapshot.field_type_index.get(into_table);
        if the_table.is_none() {
            return Err(PlatformValidationError::ChNatsStreamIntoTableDoesntExist {
                ch_nats_stream_import_consumer_name: consumer.clone(),
                ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                ch_database: db_name.clone(),
                ch_schema: ch_schema_name.clone(),
                into_table: into_table.clone(),
                existing_tables: last_snapshot.field_type_index.keys().map(|i| i.clone()).collect(),
            });
        }

        let the_table = the_table.unwrap();
        let mut inserted_values: BTreeSet<&str> = BTreeSet::new();

        for f in &last_type.the_type().fields {
            if let Some(tval) = the_table.fields.get(&f.0) {
                let mut equivalent_ch_type: Option<&'static str> = None;
                let error_msg: Option<(String, &'static str)> =
                    match &f.1.field_type {
                        ValidVersionedStructType::Array(_) => {
                            Some(("Arrays not supported to be imported to Clickhouse tables".to_string(), "Array"))
                        }
                        ValidVersionedStructType::Struct(_) => {
                            Some(("Structs are not supported to be imported to Clickhouse tables".to_string(), "Struct"))
                        }
                        ValidVersionedStructType::Option(_) => {
                            Some(("Nullable types are not supported to be imported to Clickhouse tables".to_string(), "Option"))
                        }
                        ValidVersionedStructType::Bool => {
                            equivalent_ch_type = Some("Bool");
                            None
                        }
                        ValidVersionedStructType::F64 => {
                            equivalent_ch_type = Some("Float64");
                            None
                        }
                        ValidVersionedStructType::I64 => {
                            equivalent_ch_type = Some("Int64");
                            None
                        }
                        ValidVersionedStructType::String => {
                            equivalent_ch_type = Some("String");
                            None
                        }
                    };

                if let Some((msg, the_type)) = error_msg {
                    return Err(PlatformValidationError::ChNatsStreamUnsupportedFieldType {
                        ch_nats_stream_import_consumer_name: consumer.clone(),
                        ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                        ch_database: db_name.clone(),
                        into_table: into_table.clone(),
                        nats_jetstream_stream: db.nats_jetstream_stream().c_stream_name(stream).clone(),
                        bw_compat_unsupported_field_name: f.0.clone(),
                        bw_compat_unsupported_field_type: the_type.to_string(),
                        bw_compat_type: vtype_name.clone(),
                        ch_schema: ch_schema_name.clone(),
                        message: msg,
                    });
                }

                let eq_type = equivalent_ch_type.expect("We must have picked equivalent type now");
                if tval.col_type != eq_type {
                    return Err(PlatformValidationError::ChNatsStreamIntoTableColumnTypeMismatch {
                        ch_nats_stream_import_consumer_name: consumer.clone(),
                        ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                        ch_database: db_name.clone(),
                        bw_compat_type_field_name: f.0.clone(),
                        bw_compat_type_field_type: eq_type.to_string(),
                        table_column_type: tval.col_type.clone(),
                        into_table: into_table.clone(),
                        nats_jetstream_stream: db.nats_jetstream_stream().c_stream_name(stream).clone(),
                        bw_compat_type: vtype_name.clone(),
                        ch_schema: ch_schema_name.clone(),
                    });
                }

                if !tval.insertion_allowed {
                    return Err(PlatformValidationError::ChNatsStreamIntoTableColumnFieldIsNotAllowedToBeInserted {
                        ch_nats_stream_import_consumer_name: consumer.clone(),
                        ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                        ch_database: db_name.clone(),
                        bw_compat_type: vtype_name.clone(),
                        bw_compat_type_field_name: f.0.clone(),
                        table_column_type: tval.col_type.clone(),
                        into_table: into_table.clone(),
                        nats_jetstream_stream: db.nats_jetstream_stream().c_stream_name(stream).clone(),
                        ch_schema: ch_schema_name.clone(),
                        explanation: "Column field is either ALIAS or MATERIALIZED and cannot be inserted, only computed".to_string(),
                    });
                }

                if enable_subjects {
                    return Err(PlatformValidationError::ChNatsStreamHasEnableSubjectsWhichIsUnsupported {
                        ch_nats_stream_import_consumer_name: consumer.clone(),
                        ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                        ch_database: db_name.clone(),
                        into_table: into_table.clone(),
                        nats_jetstream_stream: db.nats_jetstream_stream().c_stream_name(stream).clone(),
                        ch_schema: ch_schema_name.clone(),
                        nats_jetstream_stream_enable_subjects: enable_subjects,
                    });
                }

                assert!(inserted_values.insert(f.0.as_str()));
            } else {
                return Err(PlatformValidationError::ChNatsStreamIntoTableColumnDoesntExist {
                    ch_nats_stream_import_consumer_name: consumer.clone(),
                    ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                    ch_database: db_name.clone(),
                    bw_compat_type: vtype_name.clone(),
                    bw_compat_non_existing_field_name: f.0.clone(),
                    ch_schema: ch_schema_name.clone(),
                    existing_columns_in_table: the_table.fields.keys().map(|i| i.clone()).collect(),
                    into_table: into_table.clone(),
                });
            }
        }

        for (cname, ctype) in &the_table.fields {
            if !ctype.has_default && ctype.insertion_allowed {
                if !inserted_values.contains(cname.as_str()) {
                    return Err(PlatformValidationError::ChNatsStreamIntoTableColumnFieldHasNoDefaultAndDoesntExistInBwType {
                        ch_nats_stream_import_consumer_name: consumer.clone(),
                        ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                        ch_database: db_name.clone(),
                        bw_compat_type: vtype_name.clone(),
                        ch_schema: ch_schema_name.clone(),
                        into_table: into_table.clone(),
                        missing_table_column_from_nats: cname.clone(),
                        nats_jetstream_stream: db.nats_jetstream_stream().c_stream_name(stream).clone(),
                        explanation: format!("Field {cname} exists in table, has no default value but doesn't exist in backwards compatible type {vtype_name}"),
                        table_column_type: ctype.col_type.clone(),
                    });
                }
            }
        }
    }

    Ok(())
}

pub static SUPPORTED_INSERTER_TYPES: &'static [&str] = &[
    "String",
    "Int32",
    "Int64",
    "Int128",
    "Int256",
    "DateTime",
    "Float32",
    "Float64",
    "Bool",
];

pub fn is_supported_inserter_type(the_type: &str) -> bool {
    SUPPORTED_INSERTER_TYPES.contains(&the_type)
}

pub fn check_nats_stream_import_regionality(db: &Database) -> Result<(), PlatformValidationError> {
    for si in db.ch_nats_stream_import().rows_iter() {
        let stream = db.ch_nats_stream_import().c_stream(si);
        let nats_cluster = db.nats_jetstream_stream().c_parent(stream);
        let nats_region = db.nats_cluster().c_region(nats_cluster);
        let depl_schema = db.ch_nats_stream_import().c_parent(si);
        let ch_cluster = db.ch_deployment_schemas().c_parent(depl_schema);
        let ch_region = db.ch_deployment().c_region(ch_cluster);

        if nats_region != ch_region {
            return Err(PlatformValidationError::ChNatsStreamAndChDeploymentAreInDifferentRegions {
                ch_deployment: db.ch_deployment().c_deployment_name(ch_cluster).clone(),
                ch_deployment_region: db.region().c_region_name(ch_region).clone(),
                nats_cluster: db.nats_cluster().c_cluster_name(nats_cluster).clone(),
                nats_cluster_region: db.region().c_region_name(nats_region).clone(),
                nats_jetstream_stream: db.nats_jetstream_stream().c_stream_name(stream).clone(),
            });
        }
    }

    Ok(())
}

fn check_backend_inserter_existence(enr: &EnrichedChDbData, db: &Database, db_ptr: TableRowPointerChSchema, proj: &Projections) -> Result<(), PlatformValidationError> {
    for shard in db.backend_application_ch_shard().rows_iter() {
        if db_ptr == db.backend_application_ch_shard().c_ch_schema(shard) {
            let queries = proj.application_ch_shard_queries.value(shard);
            if !queries.inserters.is_empty() {
                let (_, snapshot) = enr.schema_snapshots.iter().rev().next().unwrap();
                let app = db.backend_application_ch_shard().c_parent(shard);
                for ins in &queries.inserters {
                    // we do this check only here and not in static checks because we must
                    // know table schema to check if inserter table exists
                    match snapshot.field_type_index.get(ins) {
                        Some(columns) => {
                            if columns.is_view {
                                return Err(PlatformValidationError::ApplicationChShardInserterIntoViewIsNotAllowed {
                                    application: db.backend_application().c_application_name(app).clone(),
                                    inserter_table: ins.clone(),
                                    inserter_table_type: "view or materialized view".to_string(),
                                })
                            }
                            for (cname, ctype) in &columns.fields {
                                if ctype.insertion_allowed && !is_supported_inserter_type(&ctype.col_type.as_str()) {
                                    return Err(PlatformValidationError::ApplicationChShardInserterTableTypeNotSupported {
                                        application: db.backend_application().c_application_name(app).clone(),
                                        inserter_table: ins.clone(),
                                        inserter_table_column: cname.clone(),
                                        inserter_table_column_type: ctype.col_type.clone(),
                                        supported_inserter_types: SUPPORTED_INSERTER_TYPES.iter().map(|i| i.to_string()).collect(),
                                    })
                                }
                            }
                        }
                        None => {
                            return Err(PlatformValidationError::ApplicationChShardInserterTableDoesntExist {
                                application: db.backend_application().c_application_name(app).clone(),
                                existing_tables_in_schema: snapshot.field_type_index.keys().cloned().collect(),
                                non_existant_inserter_table: ins.clone(),
                            })
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn static_db_checks(db: &Database, db_ptr: TableRowPointerChSchema) -> Result<i64, PlatformValidationError> {
    let mut max_migration_time = 0i64;
    for mig in db.ch_schema().c_children_ch_migration(db_ptr).iter() {
        max_migration_time = max_migration_time.max(db.ch_migration().c_time(*mig));
    }

    for mig in db.ch_schema().c_children_ch_migration(db_ptr).windows(2) {
        let prev = mig[0];
        let curr = mig[1];
        let prev_t = db.ch_migration().c_time(prev);
        let curr_t = db.ch_migration().c_time(curr);
        if prev_t >= curr_t {
            return Err(PlatformValidationError::ChMigrationsAreNotOrdered {
                previous_migration_time: prev_t,
                current_migration_time: curr_t,
                previous_migration: db.ch_migration().c_upgrade(prev).clone(),
                current_migration: db.ch_migration().c_upgrade(curr).clone(),
            })
        }
    }

    for mig in db.ch_schema().c_children_ch_migration(db_ptr) {
        let upg = db.ch_migration().c_upgrade(*mig);
        let down = db.ch_migration().c_downgrade(*mig);
        let ch_schema = db.ch_schema().c_schema_name(db_ptr);

        // TODO: refactor to parse DDL segments just once
        verify_clickhouse_ddl_segments(&ch_schema, upg, &parse_clickhouse_ddl_segments(&upg))?;
        verify_clickhouse_ddl_segments(&ch_schema, down, &parse_clickhouse_ddl_segments(&down))?;
    }

    Ok(max_migration_time)
}

type DbDatasets = Vec<(TableRowPointerChTestDataset, TestTablesData)>;

fn parse_db_datasets(db: &Database, db_ptr: TableRowPointerChSchema) -> Result<DbDatasets, PlatformValidationError> {
    let mut datasets = Vec::with_capacity(db.ch_schema().c_children_ch_test_dataset(db_ptr).len());
    for td in db.ch_schema().c_children_ch_test_dataset(db_ptr) {
        let parsed = super::queries::deserialize_test_dataset(
            db.ch_test_dataset().c_dataset_contents(*td),
        ).map_err(|e| {
            PlatformValidationError::ChCantDeserializeTestDataset {
                ch_schema: db.ch_schema().c_schema_name(db_ptr).clone(),
                error: e.to_string(),
                input_dataset_name: db.ch_test_dataset().c_dataset_name(*td).clone(),
                input_data: db.ch_test_dataset().c_dataset_contents(*td).clone(),
            }
        })?;

        datasets.push((*td, parsed));
    }

    Ok(datasets)
}

fn strip_sql_comments(input: &str) -> (String, Vec<String>) {
    let mut res = String::with_capacity(input.len());
    let mut comments: Vec<String> = Vec::new();
    let comment_marker = "--";

    for line in input.lines() {
        match line.split_once(comment_marker) {
            Some((prefix, comment)) => {
                res += prefix;
                comments.push(format!("{comment_marker}{comment}"));
            },
            None => {
                res += line;
                comments.push(String::default());
            },
        }

        res += "\n";
    }

    (res, comments)
}

pub fn parse_clickhouse_ddl_segments(input: &str) -> Vec<String> {
    let (stripped, _) = strip_sql_comments(input);
    let mut segments = Vec::new();
    for seg in stripped.split(";") {
        let trimmed = seg.trim();
        if !trimmed.is_empty() {
            segments.push(trimmed.replace(";", "").to_string());
        }
    }
    segments
}

// we assume user creates merge trees locally in DB
// when testing but in production it will be replicated
// merge tree always
pub const SUPPORTED_TABLE_ENGINES: &[&'static str] = &[
    "MergeTree",
    "ReplacingMergeTree",
    "SummingMergeTree",
    "AggregatingMergeTree",
    "CollapsingMergeTree",
    "VersionedCollapsingMergeTree",
    // graphite is legacy software, we don't care about it
    // " GraphiteMergeTree",
];

fn verify_clickhouse_ddl_segments(database: &str, mig_sql: &str, input: &[String]) -> Result<(), PlatformValidationError> {
    lazy_static! {
        static ref TABLE_ENGINE_REGEX: regex::Regex = regex::Regex::new(r"(?i)\s+ENGINE\s+=\s+([A-Za-z0-9]+)").unwrap();
        static ref CREATE_IF_NOT_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?i)\s*CREATE\s+TABLE\s+(.*)(").unwrap();
        static ref CREATE_TABLE_REGEX: regex::Regex = regex::Regex::new(r"(?i)\s*CREATE\s+TABLE\s+(.*)\(").unwrap();
        static ref CREATE_VIEW_REGEX: regex::Regex = regex::Regex::new(r"(?i)\s*CREATE\s+(OR\s+REPLACE\s+)?VIEW\s+(.*)\s+AS\s+").unwrap();
        static ref CREATE_MATERIALIZED_VIEW_REGEX: regex::Regex = regex::Regex::new(r"(?i)\s*CREATE\s+MATERIALIZED\s+VIEW\s+(.*)\s+AS\s+").unwrap();
        static ref IF_NOT_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?i)IF\s+NOT\s+EXISTS\s+").unwrap();
        static ref DROP_TABLE_REGEX: regex::Regex = regex::Regex::new(r"(?i)\s*DROP\s+TABLE\s+(.*)$").unwrap();
        static ref DROP_VIEW_REGEX: regex::Regex = regex::Regex::new(r"(?i)\s*DROP\s+VIEW\s+(.*)$").unwrap();
        static ref IF_EXISTS_REGEX: regex::Regex = regex::Regex::new(r"(?i)^IF\s+EXISTS\s+").unwrap();
        static ref FORBIDDEN_STATEMENTS_REGEX: regex::Regex = regex::Regex::new(r"(?i)^(RENAME|EXCHANGE)\s+").unwrap();
    }

    for seg in input {
        // backticks are gay, we shouldn't need those
        if seg.contains("`") {
            return Err(PlatformValidationError::ChMigrationContainsBacktick {
                input_sql: seg.clone(),
                forbidden_character: "`".to_string(),
            });
        }

        for cap in TABLE_ENGINE_REGEX.captures_iter(&seg) {
            let engine = cap.get(1).unwrap();
            let engine = engine.as_str();
            if !SUPPORTED_TABLE_ENGINES.contains(&engine) {

                if engine.starts_with("Replicated") {
                    let stripped = engine.strip_prefix("Replicated").unwrap();
                    if SUPPORTED_TABLE_ENGINES.contains(&stripped) {
                        return Err(PlatformValidationError::ChMigrationUseUnreplicatedMergeTreesInEpl {
                            database: database.to_string(),
                            migration_sql: mig_sql.to_string(),
                            table_engine: engine.to_string(),
                            expected_table_engine: stripped.to_string(),
                            explanation: "In EPL Clickhouse schema migrations use unreplicated *MergeTree table engines. All these are converted to Replicated* engines automatically in production.".to_string(),
                        });
                    }
                }

                return Err(PlatformValidationError::ChMigrationUnsupportedTableEngine {
                    database: database.to_string(),
                    migration_sql: mig_sql.to_string(),
                    table_engine: engine.to_string(),
                    supported_table_engines: SUPPORTED_TABLE_ENGINES.iter().map(|i| i.to_string()).collect(),
                });
            }
        }

        for cap in CREATE_TABLE_REGEX.captures_iter(&seg) {
            let grp = cap.get(1).unwrap();
            if !IF_NOT_EXISTS_REGEX.is_match(grp.as_str()) {
                return Err(PlatformValidationError::ChMigrationCreateTableMustHaveIfNotExistsStatement {
                    database: database.to_string(),
                    migration_sql: mig_sql.to_string(),
                    expected_create_table_statement: "CREATE TABLE IF NOT EXISTS ...".to_string(),
                });
            }
        }

        for cap in CREATE_VIEW_REGEX.captures_iter(&seg) {
            let grp = cap.get(2).unwrap();
            if !IF_NOT_EXISTS_REGEX.is_match(grp.as_str()) {
                return Err(PlatformValidationError::ChMigrationCreateViewMustHaveIfNotExistsStatement {
                    database: database.to_string(),
                    migration_sql: mig_sql.to_string(),
                    expected_create_view_statement: "CREATE VIEW IF NOT EXISTS ...".to_string(),
                });
            }
        }

        for cap in CREATE_MATERIALIZED_VIEW_REGEX.captures_iter(&seg) {
            let grp = cap.get(1).unwrap();
            if !IF_NOT_EXISTS_REGEX.is_match(grp.as_str()) {
                return Err(PlatformValidationError::ChMigrationCreateMaterializedViewMustHaveIfNotExistsStatement {
                    database: database.to_string(),
                    migration_sql: mig_sql.to_string(),
                    expected_create_materialized_view_statement: "CREATE MATERIALIZED VIEW IF NOT EXISTS ...".to_string(),
                });
            }
        }

        for cap in DROP_TABLE_REGEX.captures_iter(&seg) {
            let grp = cap.get(1).unwrap();
            if !IF_EXISTS_REGEX.is_match(grp.as_str()) {
                return Err(PlatformValidationError::ChMigrationDropTableMustHaveIfExistsStatement {
                    database: database.to_string(),
                    migration_sql: mig_sql.to_string(),
                    expected_drop_table_statement: "DROP TABLE IF EXISTS ...".to_string(),
                });
            }
        }

        for cap in DROP_VIEW_REGEX.captures_iter(&seg) {
            let grp = cap.get(1).unwrap();
            if !IF_EXISTS_REGEX.is_match(grp.as_str()) {
                return Err(PlatformValidationError::ChMigrationDropViewMustHaveIfExistsStatement {
                    database: database.to_string(),
                    migration_sql: mig_sql.to_string(),
                    expected_drop_view_statement: "DROP VIEW IF EXISTS ...".to_string(),
                });
            }
        }

        for _cap in FORBIDDEN_STATEMENTS_REGEX.captures_iter(&seg) {
            return Err(PlatformValidationError::ChMigrationDoesntSupportRenamesOrExchanges {
                database: database.to_string(),
                migration_sql: mig_sql.to_string(),
                unsupported_statements: vec![
                    "RENAME".to_string(),
                    "EXCHANGE".to_string(),
                ]
            });
        }
    }

    Ok(())
}

#[test]
fn test_clickhouse_sql_segments_parser() {
    assert_eq!(
        parse_clickhouse_ddl_segments(r#"
          SELECT 1
        "#),
        vec![
            "SELECT 1".to_string(),
        ]
    );
    assert_eq!(
        parse_clickhouse_ddl_segments(r#"
          SELECT 1;
        "#),
        vec![
            "SELECT 1".to_string(),
        ]
    );
    assert_eq!(
        parse_clickhouse_ddl_segments(r#"
          SELECT 1;
          SELECT 2;
        "#),
        vec![
            "SELECT 1".to_string(),
            "SELECT 2".to_string(),
        ]
    );
    assert_eq!(
        parse_clickhouse_ddl_segments(r#"
          ;SELECT 1;
          SELECT 2;
        "#),
        vec![
            "SELECT 1".to_string(),
            "SELECT 2".to_string(),
        ]
    );
    assert_eq!(
        parse_clickhouse_ddl_segments(r#"
          ;SELECT 1;
          SELECT 2;;
          SELECT 3
        "#),
        vec![
            "SELECT 1".to_string(),
            "SELECT 2".to_string(),
            "SELECT 3".to_string(),
        ]
    );
}

async fn execute_clickhouse_batch_ddl(
    client: &clickhouse::Client,
    segments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {

    for segment in segments {
        client.query(segment.as_str()).execute().await?;
    }

    Ok(())
}

async fn verify_upgrades_with_queries(
    user_client: &clickhouse::Client,
    writer_client: &clickhouse::Client,
    admin_client: &clickhouse::Client,
    client_url: &str,
    db: &Database,
    db_ptr: TableRowPointerChSchema,
    enr: &mut EnrichedChDbData,
    datasets: &DbDatasets,
    max_migration_time: i64,
) -> Result<(), PlatformValidationError> {
    let db_name = db.ch_schema().c_schema_name(db_ptr);
    let migration_count = db.ch_schema().c_children_ch_migration(db_ptr).len();
    for (idx, mig) in db.ch_schema().c_children_ch_migration(db_ptr).iter().enumerate() {
        // last three migrations are relevant
        let should_test_queries = idx >= migration_count.max(4) - 4;

        let upg_sql = db.ch_migration().c_upgrade(*mig);
        let segments = parse_clickhouse_ddl_segments(&upg_sql);

        if let Err(e) = execute_clickhouse_batch_ddl(user_client, &segments).await {
            return Err(PlatformValidationError::ChMigrationUpgradeError {
                ch_schema: db_name.clone(),
                upgrade_sql: upg_sql.clone(),
                upgrade_time: db.ch_migration().c_time(*mig),
                error: e.to_string(),
            });
        }

        let schema = get_ch_schema(&upg_sql, admin_client).await?;

        for tname in schema.field_type_index.keys() {
            if tname == "epl_schema_migrations" {
                return Err(PlatformValidationError::ChReservedTableName {
                    ch_schema: db_name.clone(),
                    table_name: tname.clone(),
                    upgrade_sql: upg_sql.clone(),
                    upgrade_time: db.ch_migration().c_time(*mig),
                });
            }
        }

        if should_test_queries {
            for (ds_ptr, ds) in datasets {
                let current_migration_time = db.ch_test_dataset().c_min_time(*ds_ptr);

                if current_migration_time > max_migration_time {
                    return Err(PlatformValidationError::ChDatasetIsNeverTested {
                        ch_schema: db_name.clone(),
                        input_dataset_name: db.ch_test_dataset().c_dataset_name(*ds_ptr).clone(),
                        minimum_dataset_time: db.ch_test_dataset().c_min_time(*ds_ptr),
                        maximum_migration_time: max_migration_time,
                    });
                }

                if current_migration_time <= db.ch_migration().c_time(*mig) {
                    with_test_dataset_inserted(true, user_client, admin_client, db, db_name, *ds_ptr, ds, &schema).await?;

                    let verified = verify_dataset_queries(client_url, db, db_name, *ds_ptr).await?;
                    for v in verified {
                        let _ = enr.queries.insert(v.query_ptr, v);
                    }

                    let verified = verify_dataset_mutators(
                        writer_client, admin_client, client_url, db, db_name, *ds_ptr, ds, &schema
                    ).await?;
                    for v in verified {
                        let _ = enr.mutators.insert(v.mutator_ptr, v);
                    }
                }
            }
        }

        enr.schema_snapshots.insert(db.ch_migration().c_time(*mig), schema);
    }

    Ok(())
}

pub fn check_ch_schema_name_clash(db: &Database) -> Result<(), PlatformValidationError> {
    for schema in db.ch_schema().rows_iter() {
        let mut uniq_names: BTreeSet<&String> = BTreeSet::new();
        for query in db.ch_schema().c_children_ch_query(schema) {
            assert!(uniq_names.insert(db.ch_query().c_query_name(*query)));
        }

        for mutator in db.ch_schema().c_children_ch_mutator(schema) {
            let name = db.ch_mutator().c_mutator_name(*mutator);
            if !uniq_names.insert(name) {
                return Err(PlatformValidationError::ChSchemaHasDuplicateQueriesOrMutators {
                    ch_schema: db.ch_schema().c_schema_name(schema).clone(),
                    query_name: name.clone(),
                    mutator_with_same_name: name.clone(),
                });
            }
        }
    }

    Ok(())
}

async fn verify_consistent_downgrades(
    user_client: &clickhouse::Client,
    admin_client: &clickhouse::Client,
    db: &Database,
    db_ptr: TableRowPointerChSchema,
    schema_map: &BTreeMap<i64, DbSchemaSnapshot>,
) -> Result<(), PlatformValidationError> {
    let mut rev_mig = db.ch_schema().c_children_ch_migration(db_ptr).to_vec();
    rev_mig.reverse();
    for mig in rev_mig.windows(2) {
        let latest_mig = mig[0];
        let prev_mig = mig[1];
        let downg_sql = db.ch_migration().c_downgrade(latest_mig);
        let segments = parse_clickhouse_ddl_segments(&downg_sql);
        if let Err(e) = execute_clickhouse_batch_ddl(user_client, &segments).await {
            return Err(PlatformValidationError::ChMigrationDowngradeError {
                database: db.ch_schema().c_schema_name(db_ptr).to_string(),
                downgrade_sql: downg_sql.clone(),
                upgrade_time: db.ch_migration().c_time(latest_mig),
                error: e.to_string(),
            });
        }

        let schema_after_downgrade = get_ch_schema(&downg_sql, admin_client).await?;
        let schema_before_upgrade = schema_map.get(&db.ch_migration().c_time(prev_mig)).unwrap();
        if schema_before_upgrade.raw_fields != schema_after_downgrade.raw_fields {
            let tbl_dwngrade = super::queries::rows_to_table(&schema_after_downgrade.raw_fields);
            let tbl_before_upg = super::queries::rows_to_table(&schema_before_upgrade.raw_fields);
            let diff = prettydiff::diff_lines(&tbl_before_upg, &tbl_dwngrade)
                .set_trim_new_lines(true);
            let diff_str = diff.to_string();
            return Err(PlatformValidationError::ChMigrationInconsistentDowngrade {
                database: db.ch_schema().c_schema_name(db_ptr).to_string(),
                upgrade_sql: db.ch_migration().c_upgrade(latest_mig).clone(),
                downgrade_sql: db.ch_migration().c_downgrade(latest_mig).clone(),
                upgrade_time: db.ch_migration().c_time(latest_mig),
                schema_diff: diff_str,
            });
        }
    }

    Ok(())
}

async fn verify_dataset_queries(
    client_url: &str,
    db: &Database,
    db_name: &String,
    ds_ptr: TableRowPointerChTestDataset,
) -> Result<Vec<VerifiedQuery>, PlatformValidationError> {
    let mut output = Vec::new();

    for query_test in db.ch_test_dataset().c_referrers_ch_query_test__test_dataset(ds_ptr) {
        if db.ch_query_test().c_test_dataset(*query_test) == ds_ptr {
            let parsed_args = super::queries::deserialize_test_arguments(db.ch_query_test().c_arguments(*query_test))
                .map_err(|e| {
                    PlatformValidationError::ChCantParseTestArguments {
                        input_data: db.ch_query_test().c_arguments(*query_test).clone(),
                        error: e.to_string(),
                    }
                })?;
            let expected_outputs = super::queries::deserialize_test_output(db.ch_query_test().c_outputs(*query_test))
                .map_err(|e| {
                    PlatformValidationError::ChCantParseTestOutputs {
                        input_data: db.ch_query_test().c_outputs(*query_test).clone(),
                        error: e.to_string(),
                    }
                })?;
            let test_query = db.ch_query_test().c_parent(*query_test);
            let q = db.ch_query().c_query_expression(test_query);
            let parsed = parse_and_analyze_query(q.as_str())?;
            let opt_fields = parse_query_opt_fields(db, test_query)?;
            let url_params = prepare_arguments_params(
                &parsed.args, &parsed_args, db_name,
                db.ch_query().c_query_name(test_query),
                db.ch_query().c_query_expression(test_query),
                db.ch_query_test().c_arguments(*query_test)
            )?;
            let query_arguments = || db.ch_query_test().c_arguments(*query_test).clone();

            let interpolated_query = parsed.interpolated_expression.clone();
            // execute later, first let original query succeed
            let schema_expr = format!("DESCRIBE TABLE( {} )", interpolated_query);
            let query_res = perform_ch_query(client_url, &parsed.interpolated_expression, &url_params)
                .await.map_err(|e| {
                    PlatformValidationError::ChQueryError {
                        ch_schema: db_name.clone(),
                        query_name: db.ch_query().c_query_name(test_query).clone(),
                        error: e.to_string(),
                        original_query: parsed.original_expression.clone(),
                        interpolated_query: parsed.interpolated_expression.clone(),
                        query_arguments: query_arguments(),
                    }
                })?;

            if query_res.is_empty() {
                return Err(PlatformValidationError::ChQueryErrorEmptyRowSet {
                    ch_schema: db_name.clone(),
                    query_name: db.ch_query().c_query_name(test_query).clone(),
                    test_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                    original_query: parsed.original_expression.clone(),
                    interpolated_query: parsed.interpolated_expression.clone(),
                    query_arguments: query_arguments(),
                });
            }

            let schema_res = perform_ch_query(client_url, &schema_expr, &url_params)
                .await.map_err(|e| {
                    PlatformValidationError::ChQueryError {
                        ch_schema: db_name.clone(),
                        query_name: db.ch_query().c_query_name(test_query).clone(),
                        error: e.to_string(),
                        original_query: parsed.original_expression.clone(),
                        interpolated_query: parsed.interpolated_expression.clone(),
                        query_arguments: query_arguments(),
                    }
                })?;

            let mut cname_idx: HashSet<String> = HashSet::new();
            let mut column_schema: Vec<(String, String)> = Vec::new();
            for row in &schema_res {
                let _ = cname_idx.insert(row[0].clone());
                column_schema.push((row[0].clone(), row[1].clone()));
            }
            assert!(!column_schema.is_empty());

            for opt_field in &opt_fields {
                if !cname_idx.contains(opt_field) {
                    return Err(PlatformValidationError::ChQueryOptFieldDoesntExistInQueryResults {
                        ch_schema: db_name.clone(),
                        query_name: db.ch_query().c_query_name(test_query).clone(),
                        original_query: parsed.original_expression.clone(),
                        bad_optional_field: opt_field.clone(),
                        optional_fields: db.ch_query().c_opt_fields(test_query).clone(),
                    });
                }
            }

            let mut output_signature = Vec::with_capacity(column_schema.len());
            for (field_idx, cs) in column_schema.iter().enumerate() {
                if super::queries::valid_variable_name(&cs.0).is_err() {
                    return Err(PlatformValidationError::ChQueryInvalidOutputFieldNameFormat {
                        ch_schema: db_name.clone(),
                        query_name: db.ch_query().c_query_name(test_query).clone(),
                        original_query: parsed.original_expression.clone(),
                        interpolated_query: parsed.interpolated_expression.clone(),
                        query_arguments: query_arguments(),
                        output_field_index: field_idx + 1,
                        output_field_name: cs.0.to_string(),
                        output_field_type: cs.1.to_string(),
                        expectation: "Field should be snake case",
                    });
                }

                let vt = match map_returned_query_type(&cs.1) {
                    Some(t) => t,
                    None => {
                        return Err(PlatformValidationError::ChQueryUnsupportedTypeError {
                            ch_schema: db_name.clone(),
                            query_name: db.ch_query().c_query_name(test_query).clone(),
                            original_query: parsed.original_expression.clone(),
                            interpolated_query: parsed.interpolated_expression.clone(),
                            query_arguments: query_arguments(),
                            output_field_index: field_idx + 1,
                            output_field_name: cs.0.to_string(),
                            output_field_type: cs.1.to_string(),
                        });
                    }
                };

                output_signature.push(OutputSignatureField {
                    name: cs.0.to_string(),
                    the_type: vt,
                    optional: opt_fields.contains(&cs.0),
                });
            }

            for (idx_a, (cname_a, _)) in column_schema.iter().enumerate() {
                for (idx_b, (cname_b, _)) in column_schema.iter().enumerate() {
                    if idx_a != idx_b && cname_a == cname_b {
                        // his is unreachable because clickhouse already throws error about this
                        // but lets keep his just in case
                        return Err(PlatformValidationError::ChQueryDuplicateOutputFieldNames {
                            ch_schema: db_name.clone(),
                            query_name: db.ch_query().c_query_name(test_query).clone(),
                            original_query: parsed.original_expression.clone(),
                            interpolated_query: parsed.interpolated_expression.clone(),
                            query_arguments: query_arguments(),
                            output_field_name: cname_a.to_string(),
                        });
                    }
                }
            }

            let mut rows_vec: Vec<HashMap<String, String>> = Vec::new();
            for row in &query_res {
                rows_vec.push(coerce_ch_row_to_hashmap(row, &column_schema));
            }

            if rows_vec != expected_outputs {
                return Err(PlatformValidationError::ChQueryUnexpectedOutputs {
                    ch_schema: db_name.clone(),
                    query_name: db.ch_query().c_query_name(test_query).clone(),
                    original_query: parsed.original_expression.clone(),
                    interpolated_query: parsed.interpolated_expression.clone(),
                    query_arguments: query_arguments(),
                    expected_outputs: serde_yaml::to_string(&expected_outputs).unwrap(),
                    actual_outputs: serde_yaml::to_string(&rows_vec).unwrap(),
                });
            }

            output.push(VerifiedQuery {
                query_ptr: test_query,
                full_query: parsed,
                output_signature,
            });
        }
    }

    Ok(output)
}

async fn check_if_data_exists_in_database(
    db_name: &str,
    mutator_name: &str,
    error_if_exists: bool,
    client_url: &str,
    ds: &TestTablesData,
    schema: &DbSchemaSnapshot,
    resulting_data: &str,
) -> Result<(), PlatformValidationError> {

    let should_debug = resulting_data.starts_with("# DEBUG");
    let mut debugging_map: TestTablesData = BTreeMap::new();
    for (table, test_data) in ds {
        let table_fields =
            schema.field_type_index.get(table.as_str()).expect("we should have checked already that we have schema?");
        let fields_vec = table_fields.fields.keys().map(|i| i.as_str()).collect::<Vec<_>>();
        let fields = fields_vec.join(", ");
        let mut field_name_index: BTreeMap<&str, usize> = BTreeMap::new();
        for (idx, fv) in fields_vec.iter().enumerate() {
            field_name_index.insert(fv, idx);
        }
        let query = format!("SELECT {fields} FROM {table}");
        let matrix = perform_ch_query(client_url, &query, "").await.expect("Can't query all fields?");

        // for debugging
        if should_debug {
            let table_entry = debugging_map.entry(table.clone()).or_default();
            for m in &matrix {
                let mut this_map: BTreeMap<String, String> = BTreeMap::new();
                for (idx, v) in m.iter().enumerate() {
                    this_map.insert(fields_vec[idx].to_string(), v.clone());
                }
                table_entry.push(this_map);
            }
        }

        let mut test_rows_found: BTreeMap<usize, usize> = BTreeMap::new();

        for (res_row_idx, res_row) in matrix.iter().enumerate() {
            for (test_row_idx, test_row) in test_data.iter().enumerate() {
                // check if row matches
                let mut values_matched = 0;
                for (test_row_col_name, test_row_col_value) in test_row {
                    let row_idx = field_name_index.get(test_row_col_name.as_str())
                                                  .expect("only valid columns should exist here now?");
                    let res_row_value = &res_row[*row_idx];
                    let field_col_data = table_fields.fields.get(test_row_col_name)
                                                            .expect("We must get types now");
                    let comparison_res =
                        if field_col_data.col_type.contains("Float") {
                            let lhs = res_row_value.parse::<f64>().expect("Can't parse db float");
                            let rhs = test_row_col_value.parse::<f64>().expect("Can't parse validated result");
                            if lhs != 0.0 && rhs != 0.0 {
                                // 1/10000th diff is okay
                                ((lhs / rhs) - 1.0).abs() < 0.0001
                            } else {
                                (lhs - rhs).abs() < std::f64::EPSILON
                            }
                        } else {
                            res_row_value == test_row_col_value
                        };

                    if comparison_res {
                        values_matched += 1;
                    } else {
                        // if at least one didn't match its over
                        break;
                    }
                    // if any don't match, continue
                    // only if all match
                }

                // test data row has matched, make a decision
                if values_matched == test_row.len() {
                    if error_if_exists {
                        let test_row_yaml = serde_yaml::to_string(&test_row).unwrap();
                        return Err(PlatformValidationError::ChResultingDatasetRowIsFoundInTestDatasetBeforeMutatorIsExecuted {
                            ch_schema: db_name.to_string(),
                            mutator_name: mutator_name.to_string(),
                            resulting_data: resulting_data.to_string(),
                            resulting_data_table: table.clone(),
                            row_found_before_mutator_execution: test_row_yaml,
                        });
                    }

                    if let Some(existing_res_row_idx) = test_rows_found.insert(test_row_idx, res_row_idx) {
                        let mut ambig_rows: Vec<BTreeMap<String, String>> = Vec::with_capacity(2);

                        for dup_row in [existing_res_row_idx, res_row_idx] {
                            let mut map_val = BTreeMap::new();

                            for (field_name, field_idx) in &field_name_index {
                                assert!(map_val.insert(field_name.to_string(), matrix[dup_row][*field_idx].clone())
                                        .is_none());
                            }

                            ambig_rows.push(map_val);
                        }

                        let ambig_row_yaml = serde_yaml::to_string(&test_row).unwrap();
                        let result_rows_yaml = serde_yaml::to_string(&ambig_rows).unwrap();

                        // basically our expected result row is fond more than once in the table
                        // not good enough for us, likely user needs to make data more specific
                        return Err(PlatformValidationError::ChResultingDatasetRowFoundMoreThanOnceInTable {
                            ch_schema: db_name.to_string(),
                            mutator_name: mutator_name.to_string(),
                            resulting_data: resulting_data.to_string(),
                            resulting_data_table: table.clone(),
                            resulting_data_ambigous_row: ambig_row_yaml,
                            output_dataset_matching_rows: result_rows_yaml,
                        });
                    }
                }
            }
        }

        // list all rows that are not found?
        if test_rows_found.len() < test_data.len() && !error_if_exists {
            let mut not_found_rows: Vec<BTreeMap<String, String>> = Vec::with_capacity(test_data.len() - test_rows_found.len());

            for (test_row_idx, test_row) in test_data.iter().enumerate() {
                if !test_rows_found.contains_key(&test_row_idx) {
                    not_found_rows.push(test_row.clone());
                }
            }

            let not_found_rows_yaml = serde_yaml::to_string(&not_found_rows).unwrap();
            return Err(PlatformValidationError::ChResultingDatasetRowsAreNotFoundInTableAfterMutatorExecution {
                ch_schema: db_name.to_string(),
                mutator_name: mutator_name.to_string(),
                resulting_data: resulting_data.to_string(),
                resulting_data_table: table.clone(),
                rows_not_found_after_mutator_execution: not_found_rows_yaml,
            });
        }
    }

    if should_debug {
        if error_if_exists {
            println!("------- PRE MUTATOR DEBUG DATA  -------");
        } else {
            println!("------- POST MUTATOR DEBUG DATA -------");
        }
        println!("{}", serde_yaml::to_string(&debugging_map).unwrap());
    }

    Ok(())
}

async fn verify_dataset_mutators(
    user_client: &clickhouse::Client,
    admin_client: &clickhouse::Client,
    client_url: &str,
    db: &Database,
    db_name: &String,
    ds_ptr: TableRowPointerChTestDataset,
    ds: &TestTablesData,
    schema: &DbSchemaSnapshot,
) -> Result<Vec<VerifiedMutator>, PlatformValidationError> {
    let mut output = Vec::new();
    for mutator_test in db.ch_test_dataset().c_referrers_ch_mutator_test__test_dataset(ds_ptr) {
        if db.ch_mutator_test().c_test_dataset(*mutator_test) == ds_ptr {
            let test_mutator = db.ch_mutator_test().c_parent(*mutator_test);
            let resulting_data_str = db.ch_mutator_test().c_resulting_data(*mutator_test);
            let resulting_data =
                if resulting_data_str.is_empty() {
                    return Err(
                        PlatformValidationError::ChResultingDatasetForMutatorTestIsUndefined {
                            ch_schema: db_name.clone(),
                            mutator_name: db.ch_mutator().c_mutator_name(test_mutator).clone(),
                            resulting_data: resulting_data_str.clone(),
                            ch_mutator_test_arguments: db.ch_mutator_test().c_arguments(*mutator_test).clone(),
                        }
                    );
                } else {
                    let parsed: TestTablesData = serde_yaml::from_str(resulting_data_str.as_str())
                        .map_err(|e| {
                            PlatformValidationError::ChCantDeserializeMutatorResultingData {
                                ch_schema: db_name.clone(),
                                error: e.to_string(),
                                mutator_name: db.ch_mutator().c_mutator_name(test_mutator).clone(),
                                mutator_test_data: resulting_data_str.clone()
                            }
                        })?;

                    verify_test_dataset_against_schema(
                        db_name.as_str(),
                        db.ch_mutator().c_mutator_name(test_mutator).as_str(),
                        &parsed, schema,
                        resulting_data_str.as_str(),
                    )?;

                    parsed
                };

            let parsed_args = super::queries::deserialize_test_arguments(db.ch_mutator_test().c_arguments(*mutator_test))
                .map_err(|e| {
                    PlatformValidationError::ChCantParseTestArguments {
                        input_data: db.ch_mutator_test().c_arguments(*mutator_test).clone(),
                        error: e.to_string(),
                    }
                })?;
            let q = db.ch_mutator().c_mutator_expression(test_mutator);

            let parsed = parse_and_analyze_query(q.as_str())?;
            let url_params = prepare_arguments_params(
                &parsed.args, &parsed_args, db_name,
                db.ch_mutator().c_mutator_name(test_mutator),
                db.ch_mutator().c_mutator_expression(test_mutator),
                db.ch_mutator_test().c_arguments(*mutator_test)
            )?;

            let query_arguments = || db.ch_mutator_test().c_arguments(*mutator_test).clone();

            // if some data comes from migration just insert it from test data
            for (table, tdata) in &schema.field_type_index {
                if !tdata.is_view {
                    admin_client.query(&format!("TRUNCATE TABLE test.{}", table))
                                .execute().await.expect("Failure truncating table");
                }
            }

            // ensure we're using clean slate of data before using mutators
            // because we don't have transactions in clickhouse
            with_test_dataset_inserted(false, user_client, admin_client, db, db_name, ds_ptr, ds, &schema).await?;

            check_if_data_exists_in_database(
                db_name, db.ch_mutator().c_mutator_name(test_mutator).as_str(),
                true, client_url, &resulting_data, schema, resulting_data_str.as_str()
            ).await?;

            let query_res = perform_ch_mutation(client_url, &parsed.interpolated_expression, &url_params)
                .await.map_err(|e| {
                    PlatformValidationError::ChMutatorError {
                        ch_schema: db_name.clone(),
                        mutator_name: db.ch_mutator().c_mutator_name(test_mutator).clone(),
                        error: e.to_string(),
                        original_query: parsed.original_expression.clone(),
                        interpolated_query: parsed.interpolated_expression.clone(),
                        query_arguments: query_arguments(),
                    }
                })?;

            if !query_res.is_empty() {
                return Err(
                    PlatformValidationError::ChMutatorCannotReturnAnyRows {
                        ch_schema: db_name.clone(),
                        mutator_name: db.ch_mutator().c_mutator_name(test_mutator).clone(),
                        original_query: parsed.original_expression.clone(),
                        interpolated_query: parsed.interpolated_expression.clone(),
                        query_arguments: query_arguments(),
                        returned_rows_count: query_res.len(),
                        returned_rows: query_res,
                    }
                );
            }

            check_if_data_exists_in_database(
                db_name, db.ch_mutator().c_mutator_name(test_mutator).as_str(),
                false, client_url, &resulting_data, schema, resulting_data_str.as_str()
            ).await?;

            output.push(VerifiedMutator {
                mutator_ptr: test_mutator,
                full_query: parsed,
            });
        }
    }

    Ok(output)
}

#[derive(Debug)]
enum ChQueryError {
    Timeout(String),
    // debug should print this, wat? why this shows unused error?
    #[allow(dead_code)]
    QueryFailed {
        http_status_code: u16,
        body: String,
    }
}

impl std::error::Error for ChQueryError {
}

impl std::fmt::Display for ChQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

async fn perform_ch_mutation(ch_url: &str, query: &str, url_params: &str) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    perform_ch_query_inner(true, "test_writer", ch_url, query, url_params).await
}

async fn perform_ch_query(ch_url: &str, query: &str, url_params: &str) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    perform_ch_query_inner(false, "test", ch_url, query, url_params).await
}

async fn perform_ch_query_inner(is_mutation: bool, user: &str, ch_url: &str, query: &str, url_params: &str) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let mut matrix_res = Vec::new();

    let maybe_params = if url_params.is_empty() {
        "".to_string()
    } else { format!("&{url_params}") };
    let q_url = format!("{ch_url}/?database=test{maybe_params}");
    let req = reqwest::Client::new();

    let req = if is_mutation {
        req.post(q_url)
    } else {
        req.get(q_url)
    };

    let req = req
        .basic_auth(user, Some("123"))
        .body(query.to_string());

    let query_time_limit_ms = 500;
    tokio::select! {
        res = req.send() => {
            let res = res?;
            let code = res.status().as_u16();
            let text_res = res.text().await?;

            if code != 200 {
                return Err(Box::new(ChQueryError::QueryFailed {
                    body: text_res,
                    http_status_code: code,
                }));
            }

            for line in text_res.lines() {
                matrix_res.push(line.split("\t").map(|i| i.to_string()).collect());
            }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(query_time_limit_ms)) => {
            return Err(Box::new(ChQueryError::Timeout(format!("{query_time_limit_ms}ms query timeout exceeded"))));
        }
    }

    Ok(matrix_res)
}

#[derive(Debug, PartialEq, Eq)]
enum ParsedQuerySegment {
    Text(String),
    QueryArg {
        arg_name: String,
        the_type: Option<String>,
    },
}

fn parse_and_analyze_query(original_query: &str) -> Result<FullQuery, PlatformValidationError> {
    analyze_query_segments(original_query, parse_full_query(original_query)?)
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValidDbType {
    Int32,
    Int64,
    Int128,
    Int256,
    Float32,
    Float64,
    Bool,
    String,
    DateTime,
    Date,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueryArg {
    pub the_type: ValidDbType,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FullQuery {
    pub original_expression: String,
    pub interpolated_expression: String,
    pub args: Vec<QueryArg>,
}

fn analyze_query_segments(
    original_query: &str,
    segments: Vec<ParsedQuerySegment>,
) -> Result<FullQuery, PlatformValidationError> {
    struct InternalQueryArg {
        the_type: Option<ValidDbType>,
        name: String,
    }

    let mut args: Vec<InternalQueryArg> = Vec::new();
    for seg in &segments {
        match seg {
            ParsedQuerySegment::Text(_) => {}
            ParsedQuerySegment::QueryArg { arg_name, the_type } => {
                let the_type = match &the_type {
                    Some(t) => match t.as_str() {
                        "Int32" => Some(ValidDbType::Int32),
                        "Int64" => Some(ValidDbType::Int64),
                        "Int128" => Some(ValidDbType::Int128),
                        "Int256" => Some(ValidDbType::Int256),
                        "Float32" => Some(ValidDbType::Float32),
                        "Float64" => Some(ValidDbType::Float64),
                        "Bool" => Some(ValidDbType::Bool),
                        "String" => Some(ValidDbType::String),
                        "DateTime" => Some(ValidDbType::DateTime),
                        "Date" => Some(ValidDbType::Date),
                        _ => {
                            return Err(PlatformValidationError::ChUnsupportedArgumentType {
                                query_expression: original_query.to_string(),
                                unsupported_type: t.clone(),
                                allowed_types: vec!["Int32", "Int64", "Int128", "Int256", "Float32", "Float64", "Bool", "String", "DateTime", "Date"],
                            })
                        }
                    },
                    None => None,
                };
                let found = args
                    .iter_mut()
                    .find(|arg| &arg.name == arg_name);
                if let Some(found) = found {
                    match (&found.the_type, &the_type) {
                        (Some(prev), Some(curr)) => {
                            if prev != curr {
                                return Err(
                                    PlatformValidationError::ChDivergingTypesForSameArgument {
                                        query_expression: original_query.to_string(),
                                        argument_name: arg_name.clone(),
                                        type_a: format!("{:?}", prev),
                                        type_b: format!("{:?}", curr),
                                    },
                                );
                            }
                        }
                        (None, None) => {}
                        (None, Some(t)) => {
                            // none type exists yet, use this
                            found.the_type = Some(*t)
                        }
                        (Some(_), None) => {
                            // type already exists, further references inferred
                        }
                    }
                } else {
                    args.push(InternalQueryArg {
                        the_type,
                        name: arg_name.clone(),
                    });
                }
            }
        }
    }

    let mut final_args: Vec<QueryArg> = Vec::with_capacity(args.len());
    for arg in &args {
        match arg.the_type {
            None => {
                return Err(
                    PlatformValidationError::ChArgumentTypeUnspecifiedAtLeastOnce {
                        query_expression: original_query.to_string(),
                        argument_name: arg.name.clone(),
                    },
                );
            }
            Some(t) => final_args.push(QueryArg {
                the_type: t,
                name: arg.name.clone(),
            }),
        }
    }

    let mut substituted_query = String::with_capacity(original_query.len());
    for seg in &segments {
        match seg {
            ParsedQuerySegment::Text(se) => {
                substituted_query += &se;
            }
            ParsedQuerySegment::QueryArg { arg_name, .. } => {
                let found = args.iter().find(|arg| &arg.name == arg_name)
                                       .expect("All args must be found here");
                if let Some(the_type) = &found.the_type {
                    let ch_type = valid_db_type_to_ch_type(the_type);
                    write!(&mut substituted_query, "{{{arg_name}:{ch_type}}}").unwrap();
                } else {
                    panic!("All types must be found here")
                }
            }
        }
    }

    Ok(FullQuery {
        original_expression: original_query.to_string(),
        interpolated_expression: substituted_query,
        args: final_args,
    })
}

fn valid_db_type_to_ch_type(the_type: &ValidDbType) -> &'static str {
    match the_type {
        ValidDbType::Int32 => "Int32",
        ValidDbType::Int64 => "Int64",
        ValidDbType::Int128 => "Int128",
        ValidDbType::Int256 => "Int256",
        ValidDbType::Float32 => "Float32",
        ValidDbType::Float64 => "Float64",
        ValidDbType::Bool => "Bool",
        ValidDbType::String => "String",
        ValidDbType::DateTime => "DateTime",
        ValidDbType::Date => "Date",
    }
}

fn parse_full_query(input: &str) -> Result<Vec<ParsedQuerySegment>, PlatformValidationError> {
    let char_check = |c: char| {
        if input.contains(c) {
            return Err(
                PlatformValidationError::ChOriginalQueryParametersAreNotAllowed {
                    query_expression: input.to_string(),
                    found_forbidden_value: c.to_string(),
                },
            );
        }

        Ok(())
    };

    char_check('?')?;

    enum ParsedQuerySegmentInt {
        Char(char),
        QueryArg {
            arg_name: String,
            the_type: Option<String>,
            whitespace_count: usize,
        },
    }

    let (tail, res) = many0(alt((
        super::queries::parse_query_argument.map(|(arg_name, the_type, whitespace_count)| {
            ParsedQuerySegmentInt::QueryArg {
                arg_name: arg_name.to_string(),
                the_type: the_type.map(|i| i.to_string()),
                whitespace_count,
            }
        }),
        anychar.map(ParsedQuerySegmentInt::Char),
    )))
    .parse(input)
    .map_err(|e| PlatformValidationError::ChQueryParsingError {
        query_expression: input.to_owned(),
        syntax_error: e.to_string(),
    })?;

    let mut final_vec = Vec::with_capacity(8);
    let mut accum = String::with_capacity(32);
    for r in res {
        match r {
            ParsedQuerySegmentInt::Char(c) => {
                accum.push(c);
            }
            ParsedQuerySegmentInt::QueryArg { arg_name, the_type, whitespace_count } => {
                if whitespace_count > 0 {
                    return Err(PlatformValidationError::ChWhitespaceForbiddenInQueryArguments {
                        bad_argument: arg_name,
                        expected_whitespace: 0,
                        found_whitespace: whitespace_count,
                        query: input.to_string(),
                    })
                }

                if !accum.is_empty() {
                    let mut new_accum = String::with_capacity(32);
                    std::mem::swap(&mut accum, &mut new_accum);
                    final_vec.push(ParsedQuerySegment::Text(new_accum));
                }
                final_vec.push(ParsedQuerySegment::QueryArg { arg_name, the_type });
            }
        }
    }
    accum += tail;
    if !accum.is_empty() {
        final_vec.push(ParsedQuerySegment::Text(accum));
    }

    Ok(final_vec)
}

fn coerce_ch_row_to_hashmap(row: &[String], fields: &[(String, String)]) -> HashMap<String, String> {
    let mut res = HashMap::new();

    for (idx, col)  in row.iter().enumerate() {
        assert!(res.insert(fields[idx].0.clone(), col.clone()).is_none());
    }

    res
}

fn map_returned_query_type(input: &str) -> Option<ValidDbType> {
    match input {
        "Int32" => Some(ValidDbType::Int32),
        "Int64" => Some(ValidDbType::Int64),
        "Int128" => Some(ValidDbType::Int128),
        "Int256" => Some(ValidDbType::Int256),
        "Float32" => Some(ValidDbType::Float32),
        "Float64" => Some(ValidDbType::Float64),
        "Bool" => Some(ValidDbType::Bool),
        "String" => Some(ValidDbType::String),
        "Date" => Some(ValidDbType::Date),
        "DateTime" => Some(ValidDbType::DateTime),
        _ => None
    }
}

fn prepare_arguments_params(
    args: &[QueryArg],
    args_map: &HashMap<String, String>,
    database: &str,
    query_name: &str,
    query_expression: &str,
    original_arguments: &str,
) -> Result<String, PlatformValidationError> {
    let mut used_args: HashSet<String> = HashSet::new();
    let mut url_params: Vec<String> = Vec::new();
    for arg in args {
        match args_map.get(arg.name.as_str()) {
            Some(v) => {
                let _ = used_args.insert(arg.name.clone());
                url_params.push(format!("param_{}={}", arg.name, urlencoding::encode(v)));
                match &arg.the_type {
                    ValidDbType::Int32 => {
                        v.as_str().parse::<i32>().map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "i32".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::Int64 => {
                        v.as_str().parse::<i64>().map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "i64".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::Int128 => {
                        v.as_str().parse::<i128>().map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "i128".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::Int256 => {
                        v.as_str().parse::<num256::Int256>().map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "i256".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::DateTime => {
                        let _ = chrono::NaiveDateTime::parse_from_str(v.as_str(), "%Y-%m-%d %H:%M:%S").map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "DateTime".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?.and_utc();
                    },
                    ValidDbType::Date => {
                        let _ = chrono::NaiveDate::parse_from_str(v.as_str(), "%Y-%m-%d").map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "Date".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::Float32 => {
                        v.as_str().parse::<f32>().map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "f32".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::Float64 => {
                        v.as_str().parse::<f64>().map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "f64".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::Bool => {
                        v.as_str().parse::<bool>().map_err(|e| {
                            PlatformValidationError::ChQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "bool".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    },
                    ValidDbType::String => {},
                }
            }
            None => {
                return Err(PlatformValidationError::ChQueryArgumentNotFoundInTest {
                    database: database.to_string(),
                    query_name: query_name.to_string(),
                    query_expression: query_expression.to_string(),
                    arguments: original_arguments.to_string(),
                    argument_not_found: arg.name.clone(),
                });
            }
        }
    }

    for (key, _) in args_map.iter() {
        if !used_args.contains(key) {
            return Err(PlatformValidationError::ChQueryArgumentNotUsedInQuery {
                ch_schema: database.to_string(),
                query_name: query_name.to_string(),
                query_expression: query_expression.to_string(),
                arguments: original_arguments.to_string(),
                argument_not_used: key.clone(),
            });
        }
    }

    Ok(url_params.join("&"))
}

fn parse_query_opt_fields(db: &Database, query: TableRowPointerChQuery) -> Result<HashSet<String>, PlatformValidationError> {
    lazy_static! {
        static ref FIELD_NAME_CHECK_REGEX: regex::Regex = regex::Regex::new(r"^([a-zA-Z0-9_]+)$").unwrap();
    }

    let mut res = HashSet::new();

    let opt_fields = db.ch_query().c_opt_fields(query);
    for f in opt_fields.split_whitespace() {
        if !FIELD_NAME_CHECK_REGEX.is_match(f) {
            return Err(PlatformValidationError::ChQueryOptFieldMustBeSnakeCase {
                ch_schema: db.ch_schema().c_schema_name(db.ch_query().c_parent(query)).clone(),
                query_name: db.ch_query().c_query_name(query).clone(),
                bad_optional_field: f.to_string(),
                optional_fields: opt_fields.clone(),
            });
        }

        if !res.insert(f.to_string()) {
            return Err(PlatformValidationError::ChQueryOptFieldDuplicate {
                ch_schema: db.ch_schema().c_schema_name(db.ch_query().c_parent(query)).clone(),
                query_name: db.ch_query().c_query_name(query).clone(),
                duplicate_optional_field: f.to_string(),
                optional_fields: opt_fields.clone(),
            });
        }
    }

    Ok(res)
}

fn remove_ch_type_wrap<'a>(inp: &'a str) -> &str {
    let lhs = "LowCardinality(";
    let rhs = ")";
    if inp.starts_with(lhs) && inp.ends_with(rhs) {
        return &inp[lhs.len()..inp.len()-rhs.len()];
    }

    let lhs = "Nullable(";
    let rhs = ")";
    if inp.starts_with(lhs) && inp.ends_with(rhs) {
        return &inp[lhs.len()..inp.len()-rhs.len()];
    }

    inp
}

async fn with_test_dataset_inserted(
    truncate_tables: bool,
    client: &clickhouse::Client,
    admin_client: &clickhouse::Client,
    db: &Database,
    db_name: &String,
    ds_ptr: TableRowPointerChTestDataset,
    ds: &TestTablesData,
    schema: &DbSchemaSnapshot
) -> Result<(), PlatformValidationError> {

    // insert test data
    for (table, rows) in ds.iter() {
        let table_schema = schema.field_type_index.get(table).ok_or_else(|| {
            PlatformValidationError::ChDatasetTableNotFoundInSchema {
                ch_schema: db_name.clone(),
                table_tried_to_insert: table.to_string(),
                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).clone(),
            }
        })?;
        let table_fields = &table_schema.fields;

        if !table_schema.is_view && truncate_tables {
            admin_client.query(&format!("TRUNCATE TABLE test.{}", table))
                        .execute().await.expect("Failure truncating table");
        }

        for (row_idx, row) in rows.iter().enumerate() {
            let is_last_row = row_idx == rows.len() - 1;
            let wait_async_insert = if is_last_row { 1 } else { 0 };
            let mut inserter = format!("INSERT INTO {table}(");
            let keys: Vec<_> = row.iter().map(|(rk, _)| { rk.as_str() }).collect();
            let values_joined: String = row.iter().map(|(_, rv)| { rv.as_str() }).collect::<Vec<_>>().join(", ");
            let mut values_buf = String::new();
            values_buf += "(";
            for (col_idx, (tc, rv)) in row.iter().enumerate() {
                let is_last_column = col_idx == row.len() - 1;
                let tcolumn_type = table_fields.get(tc).ok_or_else(|| {
                    PlatformValidationError::ChDatasetTableColumnNotFoundInSchema {
                        ch_schema: db_name.clone(),
                        table: table.clone(),
                        table_column_tried_to_insert: tc.clone(),
                        input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                    }
                })?;

                if !tcolumn_type.insertion_allowed {
                    return Err(PlatformValidationError::ChDatasetTableColumnIsNotAllowedToBeInserted {
                        ch_schema: db_name.clone(),
                        table: table.clone(),
                        table_column_tried_to_insert: tc.clone(),
                        input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                        explanation: "Column is either MATERIALIZED or ALIAS and is computed and cannot be inserted by a dataset".to_string(),
                    });
                }

                let type_to_match = remove_ch_type_wrap(tcolumn_type.col_type.as_str());
                match type_to_match {
                    "Int32" => {
                        let no = rv.as_str().parse::<i32>().map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "{no}").unwrap();
                    }
                    "Int64" => {
                        let no = rv.as_str().parse::<i64>().map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "{no}").unwrap();
                    }
                    "Int128" => {
                        let no = rv.as_str().parse::<i128>().map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "{no}").unwrap();
                    }
                    "Int256" => {
                        let no = rv.as_str().parse::<num256::Int256>().map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "{no}").unwrap();
                    }
                    "UInt256" => {
                        let no = rv.as_str().parse::<num256::Uint256>().map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "{no}").unwrap();
                    }
                    "Float32" => {
                        let no = rv.as_str().parse::<f32>().map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "{no}").unwrap();
                    }
                    "Float64" => {
                        let no = rv.as_str().parse::<f64>().map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "{no}").unwrap();
                    }
                    "Bool" => {
                        let res = match rv.as_str() {
                            "true" | "yes" | "on" | "y" | "1" => true,
                            "false" | "no" | "off" | "n" | "0" => false,
                            _ => {
                                return Err(PlatformValidationError::ChDatasetColumnValueInvalidBoolean {
                                    ch_schema: db_name.clone(),
                                    table: table.clone(),
                                    column: tc.clone(),
                                    column_value: rv.clone(),
                                    accepted_true_values: "true, yes, on, y, 1",
                                    accepted_false_values: "false, no, off, n, 0",
                                });
                            }
                        };
                        write!(&mut values_buf, "{res}").unwrap();
                    }
                    "String" => {
                        let enc_b64 = base64::encode(rv.as_str());
                        write!(&mut values_buf, "base64Decode('{enc_b64}')").unwrap();
                    }
                    "DateTime" => {
                        let no: chrono::DateTime<chrono::Utc> = chrono::NaiveDateTime::parse_from_str(rv.as_str(), "%Y-%m-%d %H:%M:%S").map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?.and_utc();
                        write!(&mut values_buf, "'{}'", no.format("%Y-%m-%d %H:%M:%S")).unwrap();
                    }
                    "Date" => {
                        let no: chrono::NaiveDate = chrono::NaiveDate::parse_from_str(rv.as_str(), "%Y-%m-%d").map_err(|e| {
                            PlatformValidationError::ChDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        write!(&mut values_buf, "'{}'", no.format("%Y-%m-%d")).unwrap();
                    }
                    _ => {
                        return Err(PlatformValidationError::ChDatasetUnsupportedColumnType {
                            database: db_name.clone(),
                            table: table.clone(),
                            column: tc.clone(),
                            column_value: rv.clone(),
                            column_type: tcolumn_type.col_type.to_string(),
                            input_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                        });
                    }
                }
                if !is_last_column {
                    values_buf += ", ";
                }
            }

            values_buf += ")";
            if !is_last_row {
                values_buf += ",\n";
            } else {
                values_buf += ";\n";
            }

            inserter += &keys.join(",");
            write!(&mut inserter, ") SETTINGS async_insert=1, wait_for_async_insert={wait_async_insert} VALUES ").unwrap();
            inserter += &values_buf;

            client.query(&inserter).execute().await.map_err(|e| {
                PlatformValidationError::ChErrorInsertingTestDataset {
                    error: e.to_string(),
                    insert_sql: inserter.clone(),
                    insert_values: values_joined,
                    test_dataset_name: db.ch_test_dataset().c_dataset_name(ds_ptr).to_string(),
                }
            })?;
        }
    }

    Ok(())
}

fn verify_test_dataset_against_schema(
    db_name: &str,
    mutator_name: &str,
    ds: &TestTablesData,
    schema: &DbSchemaSnapshot,
    resulting_data: &str,
) -> Result<(), PlatformValidationError> {
    for (table, rows) in ds.iter() {
        let table_schema = schema.field_type_index.get(table.as_str()).ok_or_else(|| {
            PlatformValidationError::ChResultingDatasetTableDoesntExist {
                ch_schema: db_name.to_string(),
                mutator_name: mutator_name.to_string(),
                resulting_data_non_existing_table: table.to_string(),
                resulting_data: resulting_data.to_string(),
            }
        })?;
        let table_fields = &table_schema.fields;

        for row in rows.iter() {
            if row.is_empty() {
                return Err(PlatformValidationError::ChResultingDatasetTableRowIsEmpty {
                    ch_schema: db_name.to_string(),
                    mutator_name: mutator_name.to_string(),
                    resulting_data_table: table.to_string(),
                    resulting_data: resulting_data.to_string(),
                });
            }

            for (tc, rv) in row {
                let tcolumn_type = table_fields.get(tc.as_str()).ok_or_else(|| {
                    PlatformValidationError::ChResultingDatasetTableColumnDoesntExist {
                        ch_schema: db_name.to_string(),
                        mutator_name: mutator_name.to_string(),
                        resulting_data_table: table.to_string(),
                        resulting_data_non_existing_column: tc.clone(),
                        resulting_data: resulting_data.to_string(),
                    }
                })?;

                let type_to_match = remove_ch_type_wrap(tcolumn_type.col_type.as_str());
                match type_to_match {
                    "Int32" => {
                        let _ = rv.as_str().parse::<i32>().map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "Int64" => {
                        let _ = rv.as_str().parse::<i64>().map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "Int128" => {
                        let _ = rv.as_str().parse::<i128>().map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "Int256" => {
                        let _ = rv.as_str().parse::<num256::Int256>().map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "DateTime" => {
                        let _: chrono::DateTime<chrono::Utc> = chrono::NaiveDateTime::parse_from_str(rv.as_str(), "%Y-%m-%d %H:%M:%S").map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?.and_utc();
                    }
                    "Date" => {
                        let _: chrono::NaiveDate = chrono::NaiveDate::parse_from_str(rv.as_str(), "%Y-%m-%d").map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "Float32" => {
                        let _ = rv.as_str().parse::<f32>().map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "Float64" => {
                        let _ = rv.as_str().parse::<f64>().map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "Bool" => {
                        let _ = rv.as_str().parse::<bool>().map_err(|e| {
                            PlatformValidationError::ChResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                ch_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.col_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    "String" => {}
                    _ => {
                        return Err(PlatformValidationError::ChResultingDatasetUnsupportedColumnType {
                            ch_schema: db_name.to_string(),
                            mutator_name: mutator_name.to_string(),
                            resulting_data_table: table.to_string(),
                            resulting_data_column: tc.clone(),
                            resulting_data_column_value: rv.clone(),
                            resulting_data_column_type: tcolumn_type.col_type.clone(),
                            resulting_data: resulting_data.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(clickhouse::Row, Deserialize)]
struct ChSchemaRow {
    table: String,
    name: String,
    #[serde(rename = "type")]
    col_type: String,
    position: u64,
    default_kind: String,
    default_expression: String,
    comment: String,
    is_in_partition_key: u8,
    is_in_sorting_key: u8,
    is_in_primary_key: u8,
    is_in_sampling_key: u8,
    compression_codec: String,
    character_octet_length: Option<u64>,
    numeric_precision: Option<u64>,
    numeric_precision_radix: Option<u64>,
    numeric_scale: Option<u64>,
    datetime_precision: Option<u64>,
}

#[derive(clickhouse::Row, Deserialize)]
struct ChTableRow {
    table: String,
    has_own_data: u8,
}

pub async fn get_ch_schema(migration_sql: &str, client: &clickhouse::Client) -> Result<DbSchemaSnapshot, PlatformValidationError> {
    lazy_static! {
        static ref IDENTIFIER_NAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-z_][a-z0-9_]*$").unwrap();
    }

    // mvp, we really need pgdump
    let query = r#"
        SELECT
          table,
          name,
          type,
          position,
          default_kind,
          default_expression,
          comment,
          is_in_partition_key,
          is_in_sorting_key,
          is_in_primary_key,
          is_in_sampling_key,
          compression_codec,
          character_octet_length,
          numeric_precision,
          numeric_precision_radix,
          numeric_scale,
          datetime_precision
        FROM system.columns
        WHERE database = 'test'
        ORDER BY table, name
    "#;
    let out_vec = client.query(query).fetch_all::<ChSchemaRow>().await.map_err(|e| {
        PlatformValidationError::RuntimeError { error: e.to_string() }
    })?;

    let tables_query = r#"
        SELECT table, has_own_data
        FROM system.tables
        WHERE database = 'test'
    "#;
    let out_tables = client.query(tables_query).fetch_all::<ChTableRow>().await.map_err(|e| {
        PlatformValidationError::RuntimeError { error: e.to_string() }
    })?;

    let mut table_index: BTreeMap<String, ChTableRow> = BTreeMap::new();
    for ot in out_tables {
        assert!(table_index.insert(ot.table.clone(), ot).is_none());
    }

    let mut ftype_index: BTreeMap<String, ChTable> = BTreeMap::new();
    for values in &out_vec {
        let tname = &values.table;
        let cname = &values.name;
        let ctype = &values.col_type;
        let def_kind = &values.default_kind;
        let def_expr = &values.default_expression;

        if !IDENTIFIER_NAME_REGEX.is_match(&tname) {
            return Err(
                PlatformValidationError::ChTableNameIsNotSnakeCase {
                    bad_table_name: tname.clone(),
                    migration_sql: migration_sql.to_string(),
                }
            );
        }

        if !IDENTIFIER_NAME_REGEX.is_match(&cname) {
            return Err(
                PlatformValidationError::ChColumnNameIsNotSnakeCase {
                    bad_column_name: cname.clone(),
                    table_name: tname.clone(),
                    migration_sql: migration_sql.to_string(),
                }
            );
        }

        if tname.starts_with("nats_") {
            return Err(
                PlatformValidationError::ChTableForbiddenPrefix {
                    table_name: tname.clone(),
                    migration_sql: migration_sql.to_string(),
                    forbidden_prefix: "nats_".to_string(),
                }
            );
        }

        if ctype.starts_with("Nullable") {
            return Err(
                PlatformValidationError::ChColumnNullableValuesNotAllowed {
                    column_name: cname.clone(),
                    table_name: tname.clone(),
                    migration_sql: migration_sql.to_string(),
                    column_type: ctype.clone(),
                }
            );
        }

        let tval = table_index.get(tname).unwrap();
        let is_view = tval.has_own_data == 0;
        let e = ftype_index.entry(tname.clone()).or_insert_with(|| {
            ChTable { fields: BTreeMap::new(), is_view }
        });
        let insertion_allowed = !is_view && (def_kind == "" || def_kind == "DEFAULT" || def_kind == "EPHEMERAL");
        let is_def_expr_empty = def_expr.is_empty() || def_expr.starts_with("defaultValueOfTypeName(");
        let col = ClickhouseSchemaColumn {
            col_type: ctype.clone(),
            has_default: (def_kind == "DEFAULT" || def_kind == "EPHEMERAL") && !is_def_expr_empty,
            insertion_allowed,
        };
        // we don't care about alias/materialized stuff
        let res = e.fields.insert(cname.clone(), col);
        // println!("{}\t{}\t{}\t{}", tname, cname, ctype, nullable);
        assert!(res.is_none());
    }

    let res = DbSchemaSnapshot {
        raw_fields: out_vec.into_iter().map(|i| {
            vec![
                i.table,
                i.name,
                i.col_type,
                i.position.to_string(),
                i.default_kind,
                i.default_expression,
                i.comment,
                i.is_in_partition_key.to_string(),
                i.is_in_sorting_key.to_string(),
                i.is_in_primary_key.to_string(),
                i.is_in_sampling_key.to_string(),
                i.compression_codec,
                format!("{:?}", i.character_octet_length),
                format!("{:?}", i.numeric_precision),
                format!("{:?}", i.numeric_precision_radix),
                format!("{:?}", i.numeric_scale),
                format!("{:?}", i.datetime_precision),
            ]
        }).collect(),
        field_type_index: ftype_index,
    };

    Ok(res)
}

pub fn ch_schemas_in_region(db: &Database) -> Projection<TableRowPointerRegion, HashSet<TableRowPointerChSchema>> {
    Projection::create(db.region().rows_iter(), |region| {
        let mut res = HashSet::new();

        for depl in db.region().c_referrers_ch_deployment__region(region) {
            for depl_schema in db.ch_deployment().c_children_ch_deployment_schemas(*depl) {
                let _ = res.insert(db.ch_deployment_schemas().c_ch_schema(*depl_schema));
            }
        }

        res
    })
}

pub struct ClickhouseSchemaColumn {
    pub col_type: String,
    pub has_default: bool,
    pub insertion_allowed: bool,
}

pub struct ChTable {
    pub fields: BTreeMap<String, ClickhouseSchemaColumn>,
    pub is_view: bool,
}

pub struct DbSchemaSnapshot {
    pub raw_fields: Vec<Vec<String>>,
    pub field_type_index: BTreeMap<String, ChTable>,
}

struct TempDb {
    ctx: AsyncCheckContext,
    container_name: String,
    kill_channel: tokio::sync::mpsc::Sender<bool>,
    connection_string: String,
    user: String,
    user_writer: String,
    pass: String,
    database: String,
}

async fn get_temp_db_with_retries(ctx: AsyncCheckContext, db_name: &str, docker_image: &str, max_retries: i32) -> Result<TempDb, Box<dyn std::error::Error>> {
    use rand::Rng;

    for i in 0..max_retries-1 {
        match TempDb::new(ctx.clone(), db_name, docker_image).await {
            Ok(res) => return Ok(res),
            Err(err) => {
                eprintln!("Failed to start clickhouse temp db in [{i}] iteration, retrying in 7 seconds: {err}")
            }
        }


        tokio::time::sleep(tokio::time::Duration::from_millis(3000 + (rand::thread_rng().gen::<u64>() % 7000))).await;
    }
    TempDb::new(ctx, db_name, docker_image).await
}

impl TempDb {
    async fn new(ctx: AsyncCheckContext, db_name: &str, docker_image: &str) -> Result<TempDb, Box<dyn std::error::Error>> {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let inst_id: u64 = rng.gen();

        let container_name = format!("epl_test_clickhouse_{}_{}", inst_id, db_name);

        let image_status =
            tokio::process::Command::new("docker")
                .arg("images")
                .arg("-q")
                .arg(docker_image)
                .output()
                .await.expect("Can't query image status");

        // image doesn't exist, pull
        if image_status.stdout.is_empty() {
            let pull_res =
                tokio::process::Command::new("docker")
                    .stdout(async_process::Stdio::piped())
                    .stderr(async_process::Stdio::piped())
                    .arg("pull")
                    .arg("-q")
                    .arg(docker_image)
                    .status()
                    .await.expect("Can't pull docker image");

            assert_eq!(pull_res.code(), Some(0), "Failed to pull docker image");
        }

        let mut child =
            tokio::process::Command::new("docker")
                .stdout(async_process::Stdio::piped())
                .stderr(async_process::Stdio::piped())
                .arg("run")
                .arg("-i")
                .arg("--rm")
                .arg("--name")
                .arg(container_name.as_str())
                .arg("--mount")
                .arg("type=tmpfs,destination=/var/lib/clickhouse,tmpfs-mode=1777")
                .arg(docker_image)
                .spawn().expect("Failed to spawn");

        let container_ip;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let spawned = tokio::process::Command::new("docker")
                .stdout(async_process::Stdio::piped())
                .stderr(async_process::Stdio::piped())
                .arg("inspect")
                .arg("-f")
                .arg("{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}")
                .arg(container_name.as_str())
                .spawn().expect("Failed to spawn");

            match spawned.wait_with_output().await {
                Ok(res) => {
                    container_ip = String::from_utf8(res.stdout).expect("Can't parse ip out of output");
                    break;
                }
                Err(err) => {
                    return Err(Box::new(err));
                }
            }
        }

        let admin_connection_string = format!("http://{}:8123", container_ip.trim());

        let wait_seconds = 20;
        let wait_beginning = tokio::time::Instant::now();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let client = clickhouse::Client::default().with_url(&admin_connection_string);
            match client.query("SELECT 7").fetch_all::<u8>().await {
                Ok(res) => {
                    if res.len() == 1 && res[0] == 7 {
                        break;
                    }
                }
                Err(e) => {
                    if tokio::time::Instant::now() - wait_beginning > tokio::time::Duration::from_secs(wait_seconds) {
                        let _ = tokio::process::Command::new("docker")
                            .stdout(async_process::Stdio::piped())
                            .stderr(async_process::Stdio::piped())
                            .arg("rm")
                            .arg("-f")
                            .arg(&container_name)
                            .spawn();
                        return Err(Box::new(e));
                    }
                }
            }
        }

        let client = clickhouse::Client::default().with_url(&admin_connection_string);
        // user that performs all migrations
        client.query("CREATE DATABASE test").execute().await.expect("Can't create test database");
        client.query("CREATE USER test IDENTIFIED WITH PLAINTEXT_PASSWORD BY '123'").execute().await.expect("Can't create test user");
        client.query("GRANT TABLE ENGINE ON * TO test").execute().await.expect("Can't grant permissions to test user");
        client.query("GRANT ALTER TABLE, ALTER VIEW, CREATE TABLE, CREATE VIEW, DROP TABLE, DROP VIEW, TRUNCATE, SELECT, SHOW, INSERT ON test.* TO test").execute().await.expect("Can't grant permissions to test user");
        // user that is allowed to do mutations only but no alteration of schema
        client.query("CREATE USER test_writer IDENTIFIED WITH PLAINTEXT_PASSWORD BY '123'").execute().await.expect("Can't create test_writer user");
        client.query("GRANT TABLE ENGINE ON * TO test_writer").execute().await.expect("Can't grant permissions to test_writer user");
        client.query("GRANT TRUNCATE, SELECT, SHOW, INSERT, OPTIMIZE ON test.* TO test_writer").execute().await.expect("Can't grant permissions to test user");

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            tokio::select! {
                _ = rx.recv() => {
                    let _ = child.kill().await;
                }
                status = child.wait() => {
                    println!("child status was: {:?}", status);
                }
            }
        });

        // We don't always succeed at cleaning used containers
        // worst case will be done asynchronously after tests have ran
        let async_rm = format!("sleep {wait_seconds}; docker rm -f {}", container_name);
        let _ = tokio::process::Command::new("/bin/sh")
            .stdout(async_process::Stdio::piped())
            .stderr(async_process::Stdio::piped())
            .arg("-c")
            .arg(&async_rm)
            .spawn();

        Ok(TempDb {
            ctx, container_name, kill_channel: tx,
            connection_string: admin_connection_string,
            user: "test".to_string(),
            user_writer: "test_writer".to_string(),
            pass: "123".to_string(),
            database: "test".to_string(),
        })
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = self.kill_channel.try_send(true);
        let cont = self.container_name.clone();
        let fut = tokio::spawn(async move {
            let _ = tokio::process::Command::new("docker")
                .stdout(async_process::Stdio::null())
                .stderr(async_process::Stdio::null())
                .arg("rm")
                .arg("-f")
                .arg(&cont)
                .status().await;
        });
        let _ = self.ctx.wait_for.send(fut);
    }
}
