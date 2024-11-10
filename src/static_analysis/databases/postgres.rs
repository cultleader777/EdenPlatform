
use std::collections::{HashMap, BTreeMap, HashSet};
use async_process::Stdio;
use nom::branch::alt;
use nom::character::complete::anychar;
use nom::multi::many0;
use nom::Parser;
use tokio::process::Command;
use tokio_postgres::{types::ToSql, Row, Transaction, Client};
use crate::database::{TableRowPointerPgSchema, TableRowPointerPgQuery, Database, TableRowPointerPgTestDataset, TableRowPointerPgMutator, TableRowPointerPgTransaction, TableRowPointerRegion};
use crate::static_analysis::AsyncCheckContext;
use crate::static_analysis::networking::check_servers_regional_distribution;
use super::super::{PlatformValidationError, projections::Projection};
use super::queries::TestTablesData;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum DbQueryOrMutator {
    Query(TableRowPointerPgQuery),
    Mutator(TableRowPointerPgMutator),
}

pub struct TransactionStep {
    pub query: DbQueryOrMutator,
    pub is_multi: bool,
}

impl<'a> DbQueryOrMutator {
    fn name(&self, db: &'a crate::database::Database) -> &'a str {
        match self {
            DbQueryOrMutator::Query(q) => db.pg_query().c_query_name(*q),
            DbQueryOrMutator::Mutator(m) => db.pg_mutator().c_mutator_name(*m),
        }
    }
}

pub async fn validations(ctx: AsyncCheckContext, db: &crate::database::Database) -> Result<HashMap<TableRowPointerPgSchema, EnrichedPgDbData>, PlatformValidationError> {
    let mut dbs = Vec::new();
    for db_ptr in db.pg_schema().rows_iter() {
        dbs.push(verify_single_db(ctx.clone(), db, db_ptr));
    }

    let enriched: Vec<EnrichedPgDbData> = super::super::join_validation_errors(dbs).await?;

    let mut res: HashMap<TableRowPointerPgSchema, EnrichedPgDbData> = HashMap::with_capacity(enriched.len());

    for e in enriched {
        let r = res.insert(e.db, e);
        assert!(r.is_none());
    }

    Ok(res)
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutputSignatureField {
    pub name: String,
    pub the_type: ValidDbType,
    pub optional: bool,
}
pub struct VerifiedQuery {
    pub query_ptr: TableRowPointerPgQuery,
    pub full_query: FullQuery,
    pub output_signature: Vec<OutputSignatureField>,
    pub is_mutating: bool,
}

pub struct VerifiedMutator {
    pub mutator_ptr: TableRowPointerPgMutator,
    pub full_query: FullQuery,
}

pub struct EnrichedPgDbData {
    pub db: TableRowPointerPgSchema,
    pub queries: HashMap<TableRowPointerPgQuery, VerifiedQuery>,
    pub mutators: HashMap<TableRowPointerPgMutator, VerifiedMutator>,
}

pub fn sync_checks(db: &crate::database::Database) -> Result<(), PlatformValidationError> {
    for tquery in db.pg_query().rows_iter() {
        if db.pg_query().c_children_pg_query_test(tquery).is_empty() {
            return Err(PlatformValidationError::PgQueryHasNoTests {
                pg_schema: db.pg_schema().c_schema_name(db.pg_query().c_parent(tquery)).clone(),
                query_name: db.pg_query().c_query_name(tquery).clone(),
                original_query: db.pg_query().c_query_expression(tquery).clone(),
            });
        }
    }

    for tmut in db.pg_mutator().rows_iter() {
        if db.pg_mutator().c_children_pg_mutator_test(tmut).is_empty() {
            return Err(PlatformValidationError::PgMutatorHasNoTests {
                pg_schema: db.pg_schema().c_schema_name(db.pg_mutator().c_parent(tmut)).clone(),
                mutator_name: db.pg_mutator().c_mutator_name(tmut).clone(),
                original_query: db.pg_mutator().c_mutator_expression(tmut).clone(),
            });
        }
    }

    for mview in db.pg_mat_view().rows_iter() {
        if db.pg_mat_view().c_children_pg_mat_view_test(mview).is_empty() {
            return Err(PlatformValidationError::PgMaterializedViewHasNoTests {
                pg_schema: db.pg_schema().c_schema_name(db.pg_mat_view().c_parent(mview)).clone(),
                materialized_view_name: db.pg_mat_view().c_mview_name(mview).clone(),
            });
        }
    }

    for depl in db.pg_deployment().rows_iter() {
        let region = db.pg_deployment().c_region(depl);
        let depl_name = db.pg_deployment().c_deployment_name(depl);
        let mut db_names: HashSet<String> = HashSet::new();
        for child in db.pg_deployment().c_children_pg_deployment_schemas(depl) {
            assert!(db_names.insert(db.pg_deployment_schemas().c_db_name(*child).clone()));
        }

        let synchronous_replication = db.pg_deployment().c_synchronous_replication(depl);
        let child_instances = db.pg_deployment().c_children_pg_deployment_instance(depl).len();

        let min_instances = 2;
        if child_instances < min_instances {
            return Err(PlatformValidationError::PgDeploymentMustHaveAtLeastTwoNodes {
                pg_deployment: depl_name.clone(),
                db_region: db.region().c_region_name(region).clone(),
                found_instances: child_instances,
                minimum_instances: min_instances,
            });
        }

        let min_sync_replication_instances = 3;
        if synchronous_replication && child_instances < min_sync_replication_instances {
            return Err(PlatformValidationError::PgDeploymentForSynchronousReplicationYouMustRunAtLeastThreeNodes {
                pg_deployment: depl_name.clone(),
                db_region: db.region().c_region_name(region).clone(),
                found_instances: child_instances,
                minimum_instances: min_sync_replication_instances,
                synchronous_replication_enabled: synchronous_replication,
            });
        }

        // I've never seen more than 5 in production, but
        // even then one was a snapshot replica which had to be stopped for
        // backup because people didn't that ZFS exists
        let max_child_instances = 5;
        if child_instances > max_child_instances {
            return Err(PlatformValidationError::PgDeploymentHasMoreThanMaximumChildInstancesAllowed {
                pg_deployment: depl_name.clone(),
                db_region: db.region().c_region_name(region).clone(),
                found_instances: child_instances,
                maximum_instances: max_child_instances,
            });
        }

        if db.pg_deployment().c_distribute_over_dcs(depl) {
            check_servers_regional_distribution(
                db,
                region,
                db.pg_deployment().c_children_pg_deployment_instance(depl).iter().map(|i| {
                    let srv_volume = db.pg_deployment_instance().c_pg_server(*i);
                    db.server_volume().c_parent(srv_volume)
                }),
                format!("pg_deployment=>{depl_name}")
            )?;
        }

        for child in db.pg_deployment().c_children_pg_deployment_unmanaged_db(depl) {
            let dupe = db.pg_deployment_unmanaged_db().c_db_name(*child);
            if !db_names.insert(dupe.clone()) {
                return Err(PlatformValidationError::PgDeploymentDuplicateDatabases {
                    pg_deployment: db.pg_deployment().c_deployment_name(depl).clone(),
                    db_name_a: dupe.clone(),
                    db_name_b: dupe.clone(),
                });
            }
        }
    }

    Ok(())
}

pub struct DbConns {
    pub ro_conn: tokio_postgres::Client,
    pub rw_conn: tokio_postgres::Client,
}

impl DbConns {
    async fn new(conn_string: &str, database: &str) -> Result<DbConns, PlatformValidationError> {
        let (ro_conn, conn) = tokio_postgres::connect(conn_string, tokio_postgres::NoTls).await.map_err(|e| {
            PlatformValidationError::PgRuntimeError {
                database: database.to_string(),
                error: e.to_string(),
            }
        })?;

        tokio::spawn(async move {
            let _ = conn.await;
        });

        let _ = ro_conn.execute("SET SESSION CHARACTERISTICS AS TRANSACTION READ ONLY;", &[]).await.map_err(|e| {
            PlatformValidationError::PgRuntimeError {
                database: database.to_string(),
                error: e.to_string(),
            }
        })?;

        let (rw_conn, conn) = tokio_postgres::connect(conn_string, tokio_postgres::NoTls).await.map_err(|e| {
            PlatformValidationError::PgRuntimeError {
                database: database.to_string(),
                error: e.to_string(),
            }
        })?;

        tokio::spawn(async move {
            let _ = conn.await;
        });

        Ok(DbConns {
            ro_conn,
            rw_conn,
        })
    }
}

pub async fn verify_single_db(ctx: AsyncCheckContext, db: &crate::database::Database, db_ptr: TableRowPointerPgSchema) -> Result<EnrichedPgDbData, PlatformValidationError> {
    match verify_single_db_iter(ctx.clone(), db, db_ptr).await {
        Ok(res) => { return Ok(res) },
        Err(e) => {
            // try one more time, random error
            if e.to_string().contains("db error: FATAL: the database system is shutting down") {
                return verify_single_db_iter(ctx, db, db_ptr).await;
            } else {
                return Err(e);
            }
        }
    }
}

async fn verify_single_db_iter(ctx: AsyncCheckContext, db: &crate::database::Database, db_ptr: TableRowPointerPgSchema) -> Result<EnrichedPgDbData, PlatformValidationError> {
    let max_migration_time = static_db_checks(db, db_ptr)?;

    let mut enr = EnrichedPgDbData {
        db: db_ptr,
        queries: HashMap::with_capacity(db.pg_schema().c_children_pg_query(db_ptr).len()),
        mutators: HashMap::with_capacity(db.pg_schema().c_children_pg_mutator(db_ptr).len()),
    };

    let datasets = parse_db_datasets(db, db_ptr)?;

    // hardcode, maybe have multiple version tests in the future?
    //let img = "postgres:15.4";
    let img = "postgres@sha256:992c5398ca50c716e8c2556a6f6733a0e471e178a6320478a86bb4abe9566299";
    let db_cont = TempDb::new(ctx, db.pg_schema().c_schema_name(db_ptr), img).await;

    let mut conns = DbConns::new(&db_cont.connection_string, db.pg_schema().c_schema_name(db_ptr)).await?;

    let mut schema_map = HashMap::new();
    verify_upgrades_with_queries(&mut conns, db, db_ptr, &mut schema_map, &mut enr, &datasets, max_migration_time).await?;
    verify_consistent_downgrades(&mut conns.rw_conn, db, db_ptr, &schema_map).await?;

    Ok(enr)
}

fn static_db_checks(db: &Database, db_ptr: TableRowPointerPgSchema) -> Result<i64, PlatformValidationError> {
    let mut max_migration_time = 0i64;
    for mig in db.pg_schema().c_children_pg_migration(db_ptr).iter() {
        max_migration_time = max_migration_time.max(db.pg_migration().c_time(*mig));
    }

    for mig in db.pg_schema().c_children_pg_migration(db_ptr).windows(2) {
        let prev = mig[0];
        let curr = mig[1];
        let prev_t = db.pg_migration().c_time(prev);
        let curr_t = db.pg_migration().c_time(curr);
        if prev_t >= curr_t {
            return Err(PlatformValidationError::PgMigrationsAreNotOrdered {
                previous_migration_time: prev_t,
                current_migration_time: curr_t,
                previous_migration: db.pg_migration().c_upgrade(prev).clone(),
                current_migration: db.pg_migration().c_upgrade(curr).clone(),
            })
        }
    }

    Ok(max_migration_time)
}

type DbDatasets = Vec<(TableRowPointerPgTestDataset, BTreeMap<String, Vec<BTreeMap<String, String>>>)>;

fn parse_db_datasets(db: &Database, db_ptr: TableRowPointerPgSchema) -> Result<DbDatasets, PlatformValidationError> {
    let mut datasets = Vec::with_capacity(db.pg_schema().c_children_pg_test_dataset(db_ptr).len());
    for td in db.pg_schema().c_children_pg_test_dataset(db_ptr) {
        let parsed = super::queries::deserialize_test_dataset(
            db.pg_test_dataset().c_dataset_contents(*td),
        ).map_err(|e| {
            PlatformValidationError::PgCantDeserializeTestDataset {
                pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                error: e.to_string(),
                input_dataset_name: db.pg_test_dataset().c_dataset_name(*td).clone(),
                input_data: db.pg_test_dataset().c_dataset_contents(*td).clone(),
            }
        })?;

        datasets.push((*td, parsed));
    }

    Ok(datasets)
}

async fn verify_upgrades_with_queries(
    client: &mut DbConns,
    db: &Database,
    db_ptr: TableRowPointerPgSchema,
    schema_map: &mut HashMap<i64, DbSchemaSnapshot>,
    enr: &mut EnrichedPgDbData,
    datasets: &DbDatasets,
    max_migration_time: i64,
) -> Result<(), PlatformValidationError> {
    let db_name = db.pg_schema().c_schema_name(db_ptr);
    let migration_count = db.pg_schema().c_children_pg_migration(db_ptr).len();
    for (idx, mig) in db.pg_schema().c_children_pg_migration(db_ptr).iter().enumerate() {
        // last three migrations are relevant
        let should_test_queries = idx >= migration_count.max(4) - 4;

        let upg_sql = db.pg_migration().c_upgrade(*mig);
        if let Err(e) = client.rw_conn.batch_execute(upg_sql).await {
            return Err(PlatformValidationError::PgMigrationUpgradeError {
                pg_schema: db_name.clone(),
                upgrade_sql: upg_sql.clone(),
                upgrade_time: db.pg_migration().c_time(*mig),
                error: e.to_string(),
            });
        }

        let schema = get_pg_schema(&client.ro_conn, upg_sql).await?;

        for tname in schema.field_type_index.keys() {
            if tname == "epl_schema_migrations" {
                return Err(PlatformValidationError::PgReservedTableName {
                    pg_schema: db_name.clone(),
                    table_name: tname.clone(),
                    upgrade_sql: upg_sql.clone(),
                    upgrade_time: db.pg_migration().c_time(*mig),
                });
            }
        }

        if should_test_queries {
            for (ds_ptr, ds) in datasets {
                let current_migration_time = db.pg_test_dataset().c_min_time(*ds_ptr);

                if current_migration_time > max_migration_time {
                    return Err(PlatformValidationError::PgDatasetIsNeverTested {
                        pg_schema: db_name.clone(),
                        input_dataset_name: db.pg_test_dataset().c_dataset_name(*ds_ptr).clone(),
                        minimum_dataset_time: db.pg_test_dataset().c_min_time(*ds_ptr),
                        maximum_migration_time: max_migration_time,
                    });
                }

                if current_migration_time <= db.pg_migration().c_time(*mig) {
                    with_test_dataset_inserted(&mut client.rw_conn, db, db_name, *ds_ptr, ds, &schema).await?;

                    let verified = verify_dataset_queries(&mut client.rw_conn, db, db_name, *ds_ptr, &schema).await?;
                    for v in verified {
                        let res = enr.queries.insert(v.query_ptr, v);
                        assert!(res.is_none());
                    }
                    let verified = verify_dataset_mutators(&mut client.rw_conn, db, db_name, *ds_ptr, &schema).await?;
                    for v in verified {
                        let res = enr.mutators.insert(v.mutator_ptr, v);
                        assert!(res.is_none());
                    }
                    verify_dataset_materialized_views(&mut client.rw_conn, db, db_name, *ds_ptr, current_migration_time).await?;

                    truncate_all_rows_in_db(&client.rw_conn, db_name).await?;
                }
            }
        }

        schema_map.insert(db.pg_migration().c_time(*mig), schema);
    }

    Ok(())
}

async fn truncate_all_rows_in_db(client: &Client, db_name: &str) -> Result<(), PlatformValidationError> {
    let res = client.query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'", &[]).await.map_err(|e| {
        PlatformValidationError::RuntimeError { error: e.to_string() }
    })?;

    let rows = rows_to_string_matrix(&res);
    let tables = &rows[1..];

    let mut truncate_buffer = String::with_capacity(512);

    for t in tables {
        let tname = &t[0];
        truncate_buffer += "TRUNCATE TABLE ";
        truncate_buffer += tname;
        truncate_buffer += " RESTART IDENTITY CASCADE;\n";
    }

    client.batch_execute(&truncate_buffer).await.map_err(|e| {
        PlatformValidationError::PgRuntimeError { database: db_name.to_string(), error: e.to_string() }
    })?;

    Ok(())
}

async fn trx_read_only<'a>(trx: &mut Transaction<'a>, database: &str) -> Result<(), PlatformValidationError> {
    trx.execute("SET TRANSACTION READ ONLY", &[]).await.map_err(|e| {
        PlatformValidationError::PgRuntimeError { database: database.to_string(), error: e.to_string() }
    })?;

    Ok(())
}

async fn verify_consistent_downgrades(
    client: &mut tokio_postgres::Client,
    db: &Database,
    db_ptr: TableRowPointerPgSchema,
    schema_map: &HashMap<i64, DbSchemaSnapshot>,
) -> Result<(), PlatformValidationError> {
    let mut rev_mig = db.pg_schema().c_children_pg_migration(db_ptr).to_vec();
    rev_mig.reverse();
    for mig in rev_mig.windows(2) {
        let latest_mig = mig[0];
        let prev_mig = mig[1];
        let downg_sql = db.pg_migration().c_downgrade(latest_mig);
        if let Err(e) = client.batch_execute(downg_sql).await {
            return Err(PlatformValidationError::PgMigrationDowngradeError {
                database: db.pg_schema().c_schema_name(db_ptr).to_string(),
                downgrade_sql: downg_sql.clone(),
                upgrade_time: db.pg_migration().c_time(latest_mig),
                error: e.to_string(),
            });
        }

        let schema_after_downgrade = get_pg_schema(client, downg_sql).await?;
        let schema_before_upgrade = schema_map.get(&db.pg_migration().c_time(prev_mig)).unwrap();
        if schema_before_upgrade.raw_fields != schema_after_downgrade.raw_fields {
            let tbl_dwngrade = super::queries::rows_to_table(&schema_after_downgrade.raw_fields);
            let tbl_before_upg = super::queries::rows_to_table(&schema_before_upgrade.raw_fields);
            let diff = prettydiff::diff_lines(&tbl_before_upg, &tbl_dwngrade)
                .set_trim_new_lines(true);
            let diff_str = diff.to_string();
            return Err(PlatformValidationError::PgMigrationInconsistentDowngrade {
                database: db.pg_schema().c_schema_name(db_ptr).to_string(),
                upgrade_sql: db.pg_migration().c_upgrade(latest_mig).clone(),
                downgrade_sql: db.pg_migration().c_downgrade(latest_mig).clone(),
                upgrade_time: db.pg_migration().c_time(latest_mig),
                schema_diff: diff_str,
            });
        }
    }

    Ok(())
}


lazy_static! {
    static ref EXTRACT_SEQ_SCAN: regex::Regex = regex::Regex::new(r"Seq\s+Scan\s+on\s+([a-zA-Z0-9_]+)").unwrap();
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
    INT,
    BIGINT,
    FLOAT,
    DOUBLE,
    BOOL,
    TEXT,
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

    let mut substituted_query = String::with_capacity(original_query.len());
    for seg in segments {
        match seg {
            ParsedQuerySegment::Text(se) => {
                substituted_query += &se;
            }
            ParsedQuerySegment::QueryArg { arg_name, the_type } => {
                let the_type = match &the_type {
                    Some(t) => match t.as_str() {
                        "INT" => Some(ValidDbType::INT),
                        "BIGINT" => Some(ValidDbType::BIGINT),
                        "FLOAT" => Some(ValidDbType::FLOAT),
                        "BOOL" => Some(ValidDbType::BOOL),
                        "TEXT" => Some(ValidDbType::TEXT),
                        _ => {
                            return Err(PlatformValidationError::PgUnsupportedArgumentType {
                                query_expression: original_query.to_string(),
                                unsupported_type: t.clone(),
                                allowed_types: vec!["INT", "BIGINT", "FLOAT", "BOOL", "TEXT"],
                            })
                        }
                    },
                    None => None,
                };
                let found = args
                    .iter_mut()
                    .enumerate()
                    .find(|arg| arg.1.name == arg_name);
                if let Some((found_idx, found)) = found {
                    match (&found.the_type, &the_type) {
                        (Some(prev), Some(curr)) => {
                            if prev != curr {
                                return Err(
                                    PlatformValidationError::PgDivergingTypesForSameArgument {
                                        query_expression: original_query.to_string(),
                                        argument_name: arg_name,
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
                    substituted_query += &format!("${}", found_idx + 1);
                } else {
                    args.push(InternalQueryArg {
                        the_type,
                        name: arg_name,
                    });
                    substituted_query += &format!("${}", args.len());
                }
            }
        }
    }

    let mut final_args: Vec<QueryArg> = Vec::with_capacity(args.len());
    for arg in args {
        match arg.the_type {
            None => {
                return Err(
                    PlatformValidationError::PgArgumentTypeUnspecifiedAtLeastOnce {
                        query_expression: original_query.to_string(),
                        argument_name: arg.name,
                    },
                );
            }
            Some(t) => {
                final_args.push(QueryArg {
                    the_type: t,
                    name: arg.name,
                })
            },
        }
    }

    Ok(FullQuery {
        original_expression: original_query.to_string(),
        interpolated_expression: substituted_query,
        args: final_args,
    })
}

fn parse_full_query(input: &str) -> Result<Vec<ParsedQuerySegment>, PlatformValidationError> {
    if input.contains('$') {
        return Err(
            PlatformValidationError::PgOriginalQueryParametersAreNotAllowed {
                query_expression: input.to_string(),
                found_forbidden_value: "$".to_string(),
            },
        );
    }

    if input.contains('?') {
        return Err(
            PlatformValidationError::PgOriginalQueryParametersAreNotAllowed {
                query_expression: input.to_string(),
                found_forbidden_value: "?".to_string(),
            },
        );
    }

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
    .map_err(|e| PlatformValidationError::PgQueryParsingError {
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
                    return Err(PlatformValidationError::PgWhitespaceForbiddenInQueryArguments {
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

async fn verify_dataset_queries(
    client: &mut tokio_postgres::Client,
    db: &Database,
    db_name: &String,
    ds_ptr: TableRowPointerPgTestDataset,
    schema_snapshot: &DbSchemaSnapshot,
) -> Result<Vec<VerifiedQuery>, PlatformValidationError> {
    let mut output = Vec::new();

    for query_test in db.pg_test_dataset().c_referrers_pg_query_test__test_dataset(ds_ptr) {
        if db.pg_query_test().c_test_dataset(*query_test) == ds_ptr {
            let parsed_args = super::queries::deserialize_test_arguments(db.pg_query_test().c_arguments(*query_test))
                .map_err(|e| {
                    PlatformValidationError::PgCantParseTestArguments {
                        input_data: db.pg_query_test().c_arguments(*query_test).clone(),
                        error: e.to_string(),
                    }
                })?;
            let expected_outputs = super::queries::deserialize_test_output(db.pg_query_test().c_outputs(*query_test))
                .map_err(|e| {
                    PlatformValidationError::PgCantParseTestOutputs {
                        input_data: db.pg_query_test().c_outputs(*query_test).clone(),
                        error: e.to_string(),
                    }
                })?;
            let test_query = db.pg_query_test().c_parent(*query_test);
            let q = db.pg_query().c_query_expression(test_query);
            let parsed = parse_and_analyze_query(q.as_str())?;
            let opt_fields = parse_query_opt_fields(db, test_query)?;
            let db_vec = prepare_arguments_vector(
                &parsed.args, &parsed_args, db_name,
                db.pg_query().c_query_name(test_query),
                db.pg_query().c_query_expression(test_query),
                db.pg_query_test().c_arguments(*query_test)
            )?;
            let mapped = args_to_refs(&db_vec);
            let query_arguments = || db.pg_query_test().c_arguments(*query_test).clone();
            let is_mutating = db.pg_query().c_is_mutating(test_query);

            let mut this_trx = client.transaction().await.map_err(|e| {
                PlatformValidationError::PgRuntimeError { database: db_name.to_string(), error: e.to_string() }
            })?;
            if is_mutating {
                if !does_query_have_mutation(q.as_str()) {
                    return Err(PlatformValidationError::PgMutatingQueryDoesNotHaveMutationKeywords {
                        pg_schema: db_name.clone(),
                        query_name: db.pg_query().c_query_name(test_query).clone(),
                        original_query: parsed.original_expression.clone(),
                        expected_keywords: vec!["INSERT", "UPDATE", "DELETE"],
                    });
                }
            } else {
                trx_read_only(&mut this_trx, db_name).await?;
            }

            let wal_before = this_trx.query("SELECT pg_current_wal_insert_lsn()::TEXT", &[]).await.map_err(|e| {
                PlatformValidationError::PgRuntimeError { database: db_name.to_string(), error: e.to_string() }
            })?[0].get::<usize, String>(0);

            let query_limit_ms = 100;

            let original_query = parsed.original_expression.clone();
            let interpolated_query = parsed.interpolated_expression.clone();
            tokio::select! {
                res = this_trx.query(parsed.interpolated_expression.as_str(), &mapped) => {
                    let res = res.map_err(|e| {
                        if let Some(dbe) = e.as_db_error() {
                            if *dbe.code() == tokio_postgres::error::SqlState::READ_ONLY_SQL_TRANSACTION {
                                return PlatformValidationError::PgQueryCannotMutateDatabase {
                                    pg_schema: db_name.clone(),
                                    query_name: db.pg_query().c_query_name(test_query).clone(),
                                    original_query: parsed.original_expression.clone(),
                                };
                            }
                        }

                        PlatformValidationError::PgQueryError {
                            pg_schema: db_name.clone(),
                            query_name: db.pg_query().c_query_name(test_query).clone(),
                            error: e.to_string(),
                            original_query: parsed.original_expression.clone(),
                            interpolated_query: parsed.interpolated_expression.clone(),
                            query_arguments: query_arguments(),
                        }
                    })?;

                    if is_mutating {
                        let wal_after = this_trx.query("SELECT pg_current_wal_insert_lsn()::TEXT", &[]).await.map_err(|e| {
                            PlatformValidationError::PgRuntimeError { database: db_name.to_string(), error: e.to_string() }
                        })?[0].get::<usize, String>(0);
                        if wal_before == wal_after {
                            return Err(PlatformValidationError::PgMutatingQueryDidNotModifyDatabase {
                                pg_schema: db_name.clone(),
                                query_name: db.pg_query().c_query_name(test_query).clone(),
                                original_query: parsed.original_expression.clone(),
                                interpolated_query: parsed.interpolated_expression.clone(),
                                query_arguments: query_arguments(),
                                test_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            })
                        }
                    }

                    if res.is_empty() {
                        return Err(PlatformValidationError::PgQueryErrorEmptyRowSet {
                            pg_schema: db_name.clone(),
                            query_name: db.pg_query().c_query_name(test_query).clone(),
                            test_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            original_query: parsed.original_expression.clone(),
                            interpolated_query: parsed.interpolated_expression.clone(),
                            query_arguments: query_arguments(),
                        });
                    }

                    let mut cname_idx = HashSet::new();
                    let mut column_schema = Vec::new();
                    for row in &res {
                        if column_schema.is_empty() {
                            for column in row.columns() {
                                let _ = cname_idx.insert(column.name().to_string());
                                column_schema.push((column.name(), column.type_().name()));
                            }
                        } else {
                            // not sure if schema can ever change, but let's assert
                            for (idx, column) in row.columns().iter().enumerate() {
                                assert_eq!(column_schema[idx].0, column.name());
                                assert_eq!(column_schema[idx].1, column.type_().name());
                            }
                        }
                    }
                    assert!(!column_schema.is_empty());

                    for opt_field in &opt_fields {
                        if !cname_idx.contains(opt_field) {
                            return Err(PlatformValidationError::PgQueryOptFieldDoesntExistInQueryResults {
                                pg_schema: db_name.clone(),
                                query_name: db.pg_query().c_query_name(test_query).clone(),
                                original_query: parsed.original_expression.clone(),
                                bad_optional_field: opt_field.clone(),
                                optional_fields: db.pg_query().c_opt_fields(test_query).clone(),
                            });
                        }
                    }

                    let mut output_signature = Vec::with_capacity(column_schema.len());
                    for (field_idx, cs) in column_schema.iter().enumerate() {
                        if super::queries::valid_variable_name(cs.0).is_err() {
                            return Err(PlatformValidationError::PgQueryInvalidOutputFieldNameFormat {
                                pg_schema: db_name.clone(),
                                query_name: db.pg_query().c_query_name(test_query).clone(),
                                original_query: parsed.original_expression.clone(),
                                interpolated_query: parsed.interpolated_expression.clone(),
                                query_arguments: query_arguments(),
                                output_field_index: field_idx + 1,
                                output_field_name: cs.0.to_string(),
                                output_field_type: cs.1.to_string(),
                                expectation: "Field should be snake case",
                            });
                        }

                        let vt = match map_returned_query_type(cs.1) {
                            Some(t) => t,
                            None => {
                                return Err(PlatformValidationError::PgQueryUnsupportedTypeError {
                                    pg_schema: db_name.clone(),
                                    query_name: db.pg_query().c_query_name(test_query).clone(),
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
                            optional: opt_fields.contains(cs.0),
                        });
                    }

                    for (idx_a, (cname_a, _)) in column_schema.iter().enumerate() {
                        for (idx_b, (cname_b, _)) in column_schema.iter().enumerate() {
                            if idx_a != idx_b && cname_a == cname_b {
                                return Err(PlatformValidationError::PgQueryDuplicateOutputFieldNames {
                                    pg_schema: db_name.clone(),
                                    query_name: db.pg_query().c_query_name(test_query).clone(),
                                    original_query: parsed.original_expression.clone(),
                                    interpolated_query: parsed.interpolated_expression.clone(),
                                    query_arguments: query_arguments(),
                                    output_field_name: cname_a.to_string(),
                                });
                            }
                        }
                    }

                    let mut rows_vec: Vec<HashMap<String, String>> = Vec::new();
                    for row in &res {
                        rows_vec.push(coerce_pg_row_to_hashmap(row));
                    }

                    if rows_vec != expected_outputs {
                        return Err(PlatformValidationError::PgQueryUnexpectedOutputs {
                            pg_schema: db_name.clone(),
                            query_name: db.pg_query().c_query_name(test_query).clone(),
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
                        is_mutating,
                    });
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(query_limit_ms)) => {
                    return Err(PlatformValidationError::PgQueryTimeoutError {
                        pg_schema: db_name.clone(),
                        original_query: parsed.original_expression.clone(),
                        interpolated_query: parsed.interpolated_expression.clone(),
                        query_name: db.pg_query().c_query_name(test_query).clone(),
                        query_arguments: query_arguments(),
                        limit_ms: query_limit_ms,
                    });
                }
            }

            drop(this_trx);

            run_seq_scan_analysis(
                client,
                db_name,
                &mapped,
                schema_snapshot,
                db.pg_query().c_seqscan_ok(test_query),
                db.pg_query().c_query_name(test_query).clone(),
                original_query,
                interpolated_query,
                query_arguments()
            ).await?;
        }
    }

    Ok(output)
}

fn parse_query_opt_fields(db: &Database, query: TableRowPointerPgQuery) -> Result<HashSet<String>, PlatformValidationError> {
    lazy_static! {
        static ref FIELD_NAME_CHECK_REGEX: regex::Regex = regex::Regex::new(r"^([a-zA-Z0-9_]+)$").unwrap();
    }

    let mut res = HashSet::new();

    let opt_fields = db.pg_query().c_opt_fields(query);
    for f in opt_fields.split_whitespace() {
        if !FIELD_NAME_CHECK_REGEX.is_match(f) {
            return Err(PlatformValidationError::PgQueryOptFieldMustBeSnakeCase {
                pg_schema: db.pg_schema().c_schema_name(db.pg_query().c_parent(query)).clone(),
                query_name: db.pg_query().c_query_name(query).clone(),
                bad_optional_field: f.to_string(),
                optional_fields: opt_fields.clone(),
            });
        }

        if !res.insert(f.to_string()) {
            return Err(PlatformValidationError::PgQueryOptFieldDuplicate {
                pg_schema: db.pg_schema().c_schema_name(db.pg_query().c_parent(query)).clone(),
                query_name: db.pg_query().c_query_name(query).clone(),
                duplicate_optional_field: f.to_string(),
                optional_fields: opt_fields.clone(),
            });
        }
    }

    Ok(res)
}

#[allow(clippy::too_many_arguments)]
async fn run_seq_scan_analysis(
    client: &mut Client,
    db_name: &String,
    mapped: &[&(dyn ToSql + Sync)],
    schema_snapshot: &DbSchemaSnapshot,
    seqscan_ok: bool,
    query_name: String,
    original_query: String,
    interpolated_query: String,
    query_arguments: String
) -> Result<(), PlatformValidationError> {
    let analyze_query = format!("EXPLAIN ANALYZE {}", interpolated_query);

    let explain_trx = client.transaction().await.map_err(|e| {
        PlatformValidationError::PgRuntimeError { database: db_name.to_string(), error: e.to_string() }
    })?;
    let _ = explain_trx.execute("SET enable_seqscan = OFF;", &[]).await.expect("Failed to disable seq scans in transaction");
    let _ = explain_trx.execute("SET jit = OFF;", &[]).await.expect("Failed to disable JIT in transaction");
    let res = explain_trx.query(&analyze_query, mapped).await.expect("Cannot run EXPLAIN ANALYZE query");
    let matrix = rows_to_string_matrix(&res);
    let mut plan_string = String::new();
    for r in &matrix {
        assert_eq!(r.len(), 1, "EXPLAIN ANALYZE is only expected to return one column");
        plan_string += r[0].as_str();
        plan_string += "\n";
    }
    let _ = explain_trx.rollback().await;
    for m in EXTRACT_SEQ_SCAN.captures_iter(&plan_string) {
        let tbl_name = m.get(1).unwrap();
        if schema_snapshot.field_type_index.contains_key(tbl_name.as_str()) && !seqscan_ok {
            return Err(PlatformValidationError::PgQuerySequentialScansFound {
                pg_schema: db_name.clone(),
                original_query,
                interpolated_query,
                query_name,
                query_arguments,
                seq_scan_table: tbl_name.as_str().to_string(),
                query_plan: plan_string,
            });
        }
    }

    Ok(())
}

fn does_query_have_mutation(input_sql: &str) -> bool {
    lazy_static! {
        static ref RE_1: regex::Regex = regex::Regex::new(r"\s+(update|delete|insert)\s+").unwrap();
        static ref RE_2: regex::Regex = regex::Regex::new(r"^(update|delete|insert)\s+").unwrap();
    }

    let mut buffer = String::with_capacity(input_sql.len());

    for line in input_sql.lines() {
        match line.split_once("--") {
            Some((left, _)) => { buffer += left }
            None => { buffer += line }
        }
        buffer += "\n";
    }

    let buffer = buffer.to_lowercase();

    RE_1.is_match(buffer.as_str()) || RE_2.is_match(buffer.as_str())
}

#[test]
fn test_check_query_mutation() {
    assert!(does_query_have_mutation("update bois set id = 1"));
    assert!(does_query_have_mutation(" update bois set id = 1"));
    assert!(does_query_have_mutation("UPDATE bois set id = 1"));
    assert!(does_query_have_mutation(" UPDATE bois set id = 1"));
    assert!(does_query_have_mutation("delete from bois"));
    assert!(does_query_have_mutation(" delete from bois"));
    assert!(does_query_have_mutation("DELETE from bois"));
    assert!(does_query_have_mutation(" DELETE from bois"));
    assert!(does_query_have_mutation("insert into bois values(1)"));
    assert!(does_query_have_mutation(" insert into bois vlues(1)"));
    assert!(does_query_have_mutation("INSERT into bois values(1)"));
    assert!(does_query_have_mutation(" INSERT into bois vlues(1)"));
    assert!(!does_query_have_mutation("SINSERT into bois vlues(1)"));
    assert!(!does_query_have_mutation("select * from foo"));
}

fn coerce_pg_row_to_vec(row: &Row) -> Vec<String> {
    let mut res = Vec::new();

    for (idx, col)  in row.columns().iter().enumerate() {
        let str_val = match map_returned_query_type(col.type_().name()) {
            Some(t) => {
                match t {
                    ValidDbType::INT => row.try_get::<usize, i32>(idx).map(|i| i.to_string()),
                    ValidDbType::BIGINT => row.try_get::<usize, i64>(idx).map(|i| i.to_string()),
                    ValidDbType::FLOAT => row.try_get::<usize, f32>(idx).map(|i| i.to_string()),
                    ValidDbType::DOUBLE => row.try_get::<usize, f64>(idx).map(|i| i.to_string()),
                    ValidDbType::BOOL => row.try_get::<usize, bool>(idx).map(|i| i.to_string()),
                    ValidDbType::TEXT => row.try_get::<usize, &str>(idx).map(|i| i.to_string()),
                }
            }
            None => Ok("NULL".to_string())
        };
        res.push(str_val.unwrap_or_else(|_| "NULL".to_string()));
    }

    res
}

fn coerce_pg_row_to_hashmap(row: &Row) -> HashMap<String, String> {
    let mut res = HashMap::new();

    for (idx, col)  in row.columns().iter().enumerate() {
        let str_val = match map_returned_query_type(col.type_().name()).unwrap() {
            ValidDbType::INT => row.try_get::<usize, i32>(idx).map(|i| i.to_string()).unwrap_or_else(|_| "None".to_string()),
            ValidDbType::BIGINT => row.try_get::<usize, i64>(idx).map(|i| i.to_string()).unwrap_or_else(|_| "None".to_string()),
            ValidDbType::FLOAT => row.try_get::<usize, f32>(idx).map(|i| i.to_string()).unwrap_or_else(|_| "None".to_string()),
            ValidDbType::DOUBLE => row.try_get::<usize, f64>(idx).map(|i| i.to_string()).unwrap_or_else(|_| "None".to_string()),
            ValidDbType::BOOL => row.try_get::<usize, bool>(idx).map(|i| i.to_string()).unwrap_or_else(|_| "None".to_string()),
            ValidDbType::TEXT => row.try_get::<usize, &str>(idx).map(|i| i.to_string()).unwrap_or_else(|_| "None".to_string()),
        };
        res.insert(col.name().to_string(), str_val);
    }

    res
}

async fn check_if_data_exists_in_database<'a>(
    db_name: &str,
    mutator_name: &str,
    error_if_exists: bool,
    client: &mut tokio_postgres::Transaction<'a>,
    ds: &TestTablesData,
    schema: &DbSchemaSnapshot,
    resulting_data: &str,
) -> Result<(), PlatformValidationError> {

    let should_debug = resulting_data.starts_with("# DEBUG");
    let mut debugging_map: TestTablesData = BTreeMap::new();
    for (table, test_data) in ds {
        let table_fields =
            schema.field_type_index.get(table.as_str())
                                   .expect("we should have checked already that we have schema?");
        let fields_vec = table_fields.keys().map(|i| i.as_str()).collect::<Vec<_>>();
        let fields = fields_vec.join(", ");
        let mut field_name_index: BTreeMap<&str, usize> = BTreeMap::new();
        for (idx, fv) in fields_vec.iter().enumerate() {
            field_name_index.insert(fv, idx);
        }
        let query = format!("SELECT {fields} FROM {table}");
        let all_fields = client.query(&query, &[]).await.expect("We can't query all fields?");
        let matrix = rows_to_string_matrix(&all_fields);

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

                    // TODO: add float comparison support as in clickhouse
                    if res_row_value == test_row_col_value {
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
                        return Err(PlatformValidationError::PgResultingDatasetRowIsFoundInTestDatasetBeforeMutatorIsExecuted {
                            pg_schema: db_name.to_string(),
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
                        return Err(PlatformValidationError::PgResultingDatasetRowFoundMoreThanOnceInTable {
                            pg_schema: db_name.to_string(),
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
            return Err(PlatformValidationError::PgResultingDatasetRowsAreNotFoundInTableAfterMutatorExecution {
                pg_schema: db_name.to_string(),
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

fn verify_test_dataset_against_schema(
    db_name: &str,
    mutator_name: &str,
    ds: &TestTablesData,
    schema: &DbSchemaSnapshot,
    resulting_data: &str,
) -> Result<(), PlatformValidationError> {
    for (table, rows) in ds.iter() {
        let table_fields = schema.field_type_index.get(table.as_str()).ok_or_else(|| {
            PlatformValidationError::PgResultingDatasetTableDoesntExist {
                pg_schema: db_name.to_string(),
                mutator_name: mutator_name.to_string(),
                resulting_data_non_existing_table: table.to_string(),
                resulting_data: resulting_data.to_string(),
            }
        })?;

        for row in rows.iter() {
            if row.is_empty() {
                return Err(PlatformValidationError::PgResultingDatasetTableRowIsEmpty {
                    pg_schema: db_name.to_string(),
                    mutator_name: mutator_name.to_string(),
                    resulting_data_table: table.to_string(),
                    resulting_data: resulting_data.to_string(),
                });
            }

            for (tc, rv) in row {
                let tcolumn_type = table_fields.get(tc.as_str()).ok_or_else(|| {
                    PlatformValidationError::PgResultingDatasetTableColumnDoesntExist {
                        pg_schema: db_name.to_string(),
                        mutator_name: mutator_name.to_string(),
                        resulting_data_table: table.to_string(),
                        resulting_data_non_existing_column: tc.clone(),
                        resulting_data: resulting_data.to_string(),
                    }
                })?;

                let vt = match map_resulting_data_type(tcolumn_type.as_str()) {
                    Some(t) => t,
                    None => {
                        return Err(PlatformValidationError::PgResultingDatasetUnsupportedColumnType {
                            pg_schema: db_name.to_string(),
                            mutator_name: mutator_name.to_string(),
                            resulting_data_table: table.to_string(),
                            resulting_data_column: tc.clone(),
                            resulting_data_column_value: rv.clone(),
                            resulting_data_column_type: tcolumn_type.clone(),
                            resulting_data: resulting_data.to_string(),
                        });
                    }
                };

                match vt {
                    ValidDbType::INT => {
                        let _ = rv.as_str().parse::<i32>().map_err(|e| {
                            PlatformValidationError::PgResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    ValidDbType::BIGINT => {
                        let _ = rv.as_str().parse::<i64>().map_err(|e| {
                            PlatformValidationError::PgResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    ValidDbType::FLOAT => {
                        let _ = rv.as_str().parse::<f32>().map_err(|e| {
                            PlatformValidationError::PgResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    ValidDbType::DOUBLE => {
                        let _ = rv.as_str().parse::<f64>().map_err(|e| {
                            PlatformValidationError::PgResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    ValidDbType::BOOL => {
                        let _ = rv.as_str().parse::<bool>().map_err(|e| {
                            PlatformValidationError::PgResultingDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.to_string(),
                                mutator_name: mutator_name.to_string(),
                                resulting_data_table: table.to_string(),
                                resulting_data_column: tc.clone(),
                                resulting_data_column_value: rv.clone(),
                                resulting_data: resulting_data.to_string(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                            }
                        })?;
                    }
                    ValidDbType::TEXT => {}
                }
            }
        }
    }

    Ok(())
}

async fn verify_dataset_mutators(
    client: &mut tokio_postgres::Client,
    db: &Database,
    db_name: &String,
    ds_ptr: TableRowPointerPgTestDataset,
    current_schema: &DbSchemaSnapshot,
) -> Result<Vec<VerifiedMutator>, PlatformValidationError> {
    let mut output = Vec::new();
    for mutator_test in db.pg_test_dataset().c_referrers_pg_mutator_test__test_dataset(ds_ptr) {
        if db.pg_mutator_test().c_test_dataset(*mutator_test) == ds_ptr {
            let test_mutator = db.pg_mutator_test().c_parent(*mutator_test);
            let resulting_data_str = db.pg_mutator_test().c_resulting_data(*mutator_test);
            let resulting_data =
                if resulting_data_str.is_empty() {
                    return Err(
                        PlatformValidationError::PgResultingDatasetForMutatorTestIsUndefined {
                            pg_schema: db_name.clone(),
                            mutator_name: db.pg_mutator().c_mutator_name(test_mutator).clone(),
                            resulting_data: resulting_data_str.clone(),
                            pg_mutator_test_arguments: db.pg_mutator_test().c_arguments(*mutator_test).clone(),
                        }
                    );
                } else {
                    let parsed: TestTablesData = serde_yaml::from_str(resulting_data_str.as_str())
                        .map_err(|e| {
                            PlatformValidationError::PgCantDeserializeMutatorResultingData {
                                pg_schema: db_name.clone(),
                                error: e.to_string(),
                                mutator_name: db.pg_mutator().c_mutator_name(test_mutator).clone(),
                                mutator_test_data: resulting_data_str.clone()
                            }
                        })?;

                    verify_test_dataset_against_schema(
                        db_name.as_str(),
                        db.pg_mutator().c_mutator_name(test_mutator).as_str(),
                        &parsed, current_schema,
                        resulting_data_str.as_str(),
                    )?;

                    parsed
                };

            let parsed_args = super::queries::deserialize_test_arguments(db.pg_mutator_test().c_arguments(*mutator_test))
                .map_err(|e| {
                    PlatformValidationError::PgCantParseTestArguments {
                        input_data: db.pg_mutator_test().c_arguments(*mutator_test).clone(),
                        error: e.to_string(),
                    }
                })?;
            let q = db.pg_mutator().c_mutator_expression(test_mutator);
            let parsed = parse_and_analyze_query(q.as_str())?;
            let db_vec = prepare_arguments_vector(
                &parsed.args, &parsed_args, db_name,
                db.pg_mutator().c_mutator_name(test_mutator),
                db.pg_mutator().c_mutator_expression(test_mutator),
                db.pg_mutator_test().c_arguments(*mutator_test)
            )?;
            let mapped = args_to_refs(&db_vec);

            let query_arguments = || db.pg_mutator_test().c_arguments(*mutator_test).clone();

            let query_limit_ms = 100;
            let mut trx = client.transaction().await.map_err(|e| {
                PlatformValidationError::PgRuntimeError {
                    database: db_name.clone(),
                    error: e.to_string(),
                }
            })?;

            let original_query = parsed.original_expression.clone();
            let interpolated_query = parsed.interpolated_expression.clone();

            check_if_data_exists_in_database(
                db_name, db.pg_mutator().c_mutator_name(test_mutator).as_str(),
                true, &mut trx, &resulting_data, current_schema, resulting_data_str.as_str()
            ).await?;

            tokio::select! {
                res = trx.execute(parsed.interpolated_expression.as_str(), &mapped) => {
                    let res = res.map_err(|e| {
                        PlatformValidationError::PgMutatorError {
                            pg_schema: db_name.clone(),
                            mutator_name: db.pg_mutator().c_mutator_name(test_mutator).clone(),
                            error: e.to_string(),
                            original_query: parsed.original_expression.clone(),
                            interpolated_query: parsed.interpolated_expression.clone(),
                            query_arguments: query_arguments(),
                        }
                    })?;

                    let schema = get_pg_schema_trx(&trx).await?;
                    if schema.raw_fields != current_schema.raw_fields {
                        return Err(PlatformValidationError::PgMutatorCannotChangeDbSchema {
                            pg_schema: db_name.clone(),
                            mutator_name: db.pg_mutator().c_mutator_name(test_mutator).clone(),
                            original_query: parsed.original_expression.clone(),
                        });
                    }

                    if res == 0 {
                        return Err(PlatformValidationError::PgMutatorDidNotModifyDatabase {
                            test_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            pg_schema: db_name.clone(),
                            mutator_name: db.pg_mutator().c_mutator_name(test_mutator).clone(),
                            original_query: parsed.original_expression.clone(),
                            interpolated_query: parsed.interpolated_expression.clone(),
                            query_arguments: query_arguments(),
                        });
                    }

                    check_if_data_exists_in_database(
                        db_name, db.pg_mutator().c_mutator_name(test_mutator).as_str(),
                        false, &mut trx, &resulting_data, current_schema, resulting_data_str.as_str()
                    ).await?;

                    output.push(VerifiedMutator {
                        mutator_ptr: test_mutator,
                        full_query: parsed,
                    });
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(query_limit_ms)) => {
                    return Err(PlatformValidationError::PgMutatorTimeoutError {
                        pg_schema: db_name.clone(),
                        original_query: parsed.original_expression.clone(),
                        interpolated_query: parsed.interpolated_expression.clone(),
                        mutator_name: db.pg_mutator().c_mutator_name(test_mutator).clone(),
                        query_arguments: query_arguments(),
                        limit_ms: query_limit_ms,
                    });
                }
            }

            trx.rollback().await.map_err(|e| {
                PlatformValidationError::PgRuntimeError { database: db_name.clone(), error: e.to_string() }
            })?;

            run_seq_scan_analysis(
                client,
                db_name,
                &mapped,
                current_schema,
                db.pg_mutator().c_seqscan_ok(test_mutator),
                db.pg_mutator().c_mutator_name(test_mutator).clone(),
                original_query,
                interpolated_query,
                query_arguments()
            ).await?;
        }
    }

    Ok(output)
}

async fn verify_dataset_materialized_views(
    client: &mut tokio_postgres::Client,
    db: &Database,
    db_name: &str,
    ds_ptr: TableRowPointerPgTestDataset,
    current_migration_time: i64,
) -> Result<(), PlatformValidationError> {
    let mat_views = get_db_materialized_views(client).await?;
    let mviews_hashset: HashSet<String> = mat_views.iter().skip(1).map(|i| i[1].clone()).collect();

    for mview_test in db.pg_test_dataset().c_referrers_pg_mat_view_test__test_dataset(ds_ptr) {
        let test_data_raw = db.pg_mat_view_test().c_expected_data(*mview_test);
        let test_output = super::queries::deserialize_test_output(test_data_raw)
            .map_err(|e| {
                PlatformValidationError::PgCantParseTestOutputs {
                    input_data: test_data_raw.clone(),
                    error: e.to_string(),
                }
            })?;
        let mat_view = db.pg_mat_view_test().c_parent(*mview_test);
        let mview_name = db.pg_mat_view().c_mview_name(mat_view);

        if !mviews_hashset.contains(mview_name) {
            return Err(PlatformValidationError::PgMaterializedViewWasNotCreatedInMigrations {
                database: db_name.to_owned(),
                materialized_view: db.pg_mat_view().c_mview_name(mat_view).clone(),
                migration_time: current_migration_time,
            });
        }

        // refresh mview
        let mview_query = format!("REFRESH MATERIALIZED VIEW {}", mview_name);
        let _ = client.execute(&mview_query, &[]).await.map_err(|e| {
            PlatformValidationError::PgRuntimeError {
                database: db_name.to_owned(),
                error: e.to_string(),
            }
        })?;

        let select_query = format!("SELECT * FROM {}", mview_name);
        let all_fields = client.query(&select_query, &[]).await.map_err(|e| {
            PlatformValidationError::PgRuntimeError {
                database: db_name.to_owned(),
                error: e.to_string(),
            }
        })?;

        let matrix = rows_to_string_matrix(&all_fields);
        let headers = &matrix[0];
        let actual_headers_set = headers.iter().cloned().collect::<HashSet<String>>();
        let values = &matrix[1..];

        if values.len() != test_output.len() {
            return Err(PlatformValidationError::PgMaterializedViewTestOutputRowCountMistmatch {
                pg_schema: db_name.to_owned(),
                materialized_view: mview_name.clone(),
                expected_materialized_view_rows_count: test_output.len(),
                actual_materialized_view_rows_count: values.len(),
                expected_results: test_data_raw.clone(),
            });
        }

        let mut rows_from_test = Vec::with_capacity(test_output.len());

        for single_output in &test_output {
            for k in single_output.keys() {
                if !actual_headers_set.contains(k) {
                    return Err(PlatformValidationError::PgMaterializedViewExpectedOutputColumnNotFound {
                        pg_schema: db_name.to_owned(),
                        materialized_view: mview_name.clone(),
                        actual_materialized_view_columns: headers.clone(),
                        expected_dataset_column: k.clone(),
                        expected_results: test_data_raw.clone(),
                    });
                }
            }

            let mut rows_by_header = Vec::with_capacity(headers.len());
            for header in headers {
                match single_output.get(header) {
                    Some(v) => {
                        rows_by_header.push(v.clone());
                    }
                    None => {
                        return Err(PlatformValidationError::PgMaterializedViewExpectedOutputIsMissingActualColumn {
                            pg_schema: db_name.to_owned(),
                            materialized_view: mview_name.clone(),
                            actual_materialized_view_columns: headers.clone(),
                            missing_column_in_expected_row: header.clone(),
                            expected_results: test_data_raw.clone(),
                        });
                    }
                }
            }

            rows_from_test.push(rows_by_header);
        }

        let mut actual_values = values.to_vec();
        actual_values.sort();
        rows_from_test.sort();

        let av_as_str = format!("{:#?}", actual_values);
        let t_as_str = format!("{:#?}", rows_from_test);
        let diff = prettydiff::diff_lines(&t_as_str, &av_as_str);
        if !diff.diff().is_empty() {
            return Err(PlatformValidationError::PgMaterializedViewSortedOutputRowsMismatch {
                pg_schema: db_name.to_owned(),
                materialized_view: mview_name.clone(),
                diff: format!("{}", diff),
            });
        }
    }

    Ok(())
}

fn prepare_arguments_vector(
    args: &[QueryArg],
    args_map: &HashMap<String, String>,
    database: &str,
    query_name: &str,
    query_expression: &str,
    original_arguments: &str,
) -> Result<Vec<Box<dyn ToSql + Sync>>, PlatformValidationError> {
    let mut used_args: HashSet<String> = HashSet::new();
    let mut db_vec: Vec<Box<dyn ToSql + Sync>> = Vec::with_capacity(args.len());
    for arg in args {
        match args_map.get(arg.name.as_str()) {
            Some(v) => {
                let _ = used_args.insert(arg.name.clone());
                match &arg.the_type {
                    ValidDbType::INT => {
                        db_vec.push(Box::new(v.as_str().parse::<i32>().map_err(|e| {
                            PlatformValidationError::PgQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "i32".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?));
                    },
                    ValidDbType::BIGINT => {
                        db_vec.push(Box::new(v.as_str().parse::<i64>().map_err(|e| {
                            PlatformValidationError::PgQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "i64".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?));
                    },
                    ValidDbType::FLOAT => {
                        db_vec.push(Box::new(v.as_str().parse::<f32>().map_err(|e| {
                            PlatformValidationError::PgQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "f32".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?));
                    },
                    ValidDbType::DOUBLE => {
                        db_vec.push(Box::new(v.as_str().parse::<f64>().map_err(|e| {
                            PlatformValidationError::PgQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "f64".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?));
                    },
                    ValidDbType::BOOL => {
                        db_vec.push(Box::new(v.as_str().parse::<bool>().map_err(|e| {
                            PlatformValidationError::PgQueryCannotParseArgumentToType {
                                database: database.to_string(),
                                query_name: query_name.to_string(),
                                query_expression: query_expression.to_string(),
                                arguments: original_arguments.to_string(),
                                argument_name: arg.name.clone(),
                                argument_value: v.clone(),
                                argument_expected_type: "bool".to_string(),
                                parsing_error: e.to_string(),
                            }
                        })?));
                    },
                    ValidDbType::TEXT => {
                        db_vec.push(Box::new(v.clone()));
                    },
                }
            }
            None => {
                return Err(PlatformValidationError::PgQueryArgumentNotFoundInTest {
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
            return Err(PlatformValidationError::PgQueryArgumentNotUsedInQuery {
                pg_schema: database.to_string(),
                query_name: query_name.to_string(),
                query_expression: query_expression.to_string(),
                arguments: original_arguments.to_string(),
                argument_not_used: key.clone(),
            });
        }
    }

    Ok(db_vec)
}

fn args_to_refs(args: &[Box<dyn ToSql + Sync>]) -> Vec<&(dyn ToSql + Sync)> {
    args.iter().map(|i| i.as_ref()).collect::<Vec<_>>()
}

fn map_returned_query_type(input: &str) -> Option<ValidDbType> {
    match input {
        "int4" => Some(ValidDbType::INT),
        "int8" => Some(ValidDbType::BIGINT),
        "float4" => Some(ValidDbType::FLOAT),
        "float8" => Some(ValidDbType::DOUBLE),
        "bool" => Some(ValidDbType::BOOL),
        "text" | "name" | "varchar" => Some(ValidDbType::TEXT),
        _ => None
    }
}

fn map_resulting_data_type(input: &str) -> Option<ValidDbType> {
    match input {
        "integer" => Some(ValidDbType::INT),
        "bigint" => Some(ValidDbType::BIGINT),
        "real" => Some(ValidDbType::FLOAT),
        "double precision" => Some(ValidDbType::DOUBLE),
        "boolean" => Some(ValidDbType::BOOL),
        "text" => Some(ValidDbType::TEXT),
        _ => None
    }
}

async fn with_test_dataset_inserted(
    client: &mut tokio_postgres::Client,
    db: &Database,
    db_name: &String,
    ds_ptr: TableRowPointerPgTestDataset,
    ds: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
    schema: &DbSchemaSnapshot
) -> Result<(), PlatformValidationError> {
    let trx =
        client.build_transaction()
            .isolation_level(tokio_postgres::IsolationLevel::Serializable)
            .start()
            .await
            .map_err(|e| {
                PlatformValidationError::PgRuntimeError {
                    database: db_name.clone(),
                    error: e.to_string(),
                }
            })?;

    // insert test data
    for (table, rows) in ds.iter() {
        let table_fields = schema.field_type_index.get(table).ok_or_else(|| {
            PlatformValidationError::PgDatasetTableNotFoundInSchema {
                pg_schema: db_name.clone(),
                table_tried_to_insert: table.to_string(),
                input_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).clone(),
            }
        })?;

        for row in rows {
            let mut inserter = format!("INSERT INTO {}(", table);
            let keys: Vec<_> = row.iter().map(|(rk, _)| { rk.as_str() }).collect();
            let values_joined: String = row.iter().map(|(_, rv)| { rv.as_str() }).collect::<Vec<_>>().join(", ");
            let mut values_buf: Vec<Box<dyn ToSql + Sync>> = Vec::with_capacity(row.len());
            for (tc, rv) in row.iter() {
                let tcolumn_type = table_fields.get(tc).ok_or_else(|| {
                    PlatformValidationError::PgDatasetTableColumnNotFoundInSchema {
                        pg_schema: db_name.clone(),
                        table: table.clone(),
                        table_column_tried_to_insert: tc.clone(),
                        input_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                    }
                })?;
                match tcolumn_type.as_str() {
                    "integer" => {
                        let no = rv.as_str().parse::<i32>().map_err(|e| {
                            PlatformValidationError::PgDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        values_buf.push(Box::new(no));
                    }
                    "bigint" => {
                        let no = rv.as_str().parse::<i64>().map_err(|e| {
                            PlatformValidationError::PgDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        values_buf.push(Box::new(no));
                    }
                    "real" => {
                        let no = rv.as_str().parse::<f32>().map_err(|e| {
                            PlatformValidationError::PgDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        values_buf.push(Box::new(no));
                    }
                    "double precision" => {
                        let no = rv.as_str().parse::<f64>().map_err(|e| {
                            PlatformValidationError::PgDatasetColumnValueCannotBeParsedToExpectedType {
                                pg_schema: db_name.clone(),
                                table: table.clone(),
                                column: tc.clone(),
                                column_value: rv.clone(),
                                type_tried_to_parse_to: tcolumn_type.clone(),
                                parsing_error: e.to_string(),
                                input_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                            }
                        })?;
                        values_buf.push(Box::new(no));
                    }
                    "boolean" => {
                        let res = match rv.as_str() {
                            "true" | "yes" | "on" | "y" | "1" => true,
                            "false" | "no" | "off" | "n" | "0" => false,
                            _ => {
                                return Err(PlatformValidationError::PgDatasetColumnValueInvalidBoolean {
                                    pg_schema: db_name.clone(),
                                    table: table.clone(),
                                    column: tc.clone(),
                                    column_value: rv.clone(),
                                    accepted_true_values: "true, yes, on, y, 1",
                                    accepted_false_values: "false, no, off, n, 0",
                                });
                            }
                        };
                        values_buf.push(Box::new(res));
                    }
                    "text" => {
                        values_buf.push(Box::new(rv.clone()));
                    }
                    _ => {
                        return Err(PlatformValidationError::PgDatasetUnsupportedColumnType {
                            database: db_name.clone(),
                            table: table.clone(),
                            column: tc.clone(),
                            column_value: rv.clone(),
                            column_type: tcolumn_type.to_string(),
                            input_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                        });
                    }
                }
            }
            let values_ptr = values_buf.iter().map(|i| i.as_ref()).collect::<Vec<_>>();
            let values_idx: Vec<_> = row.iter().enumerate().map(|(idx, _)| { format!("${}", idx + 1) }).collect();
            inserter += &keys.join(",");
            inserter += ") VALUES (";
            inserter += &values_idx.join(",");
            inserter += ");";

            let res = trx.execute(&inserter, values_ptr.as_slice()).await.map_err(|e| {
                PlatformValidationError::PgErrorInsertingTestDataset {
                    error: e.to_string(),
                    insert_sql: inserter.clone(),
                    insert_values: values_joined,
                    test_dataset_name: db.pg_test_dataset().c_dataset_name(ds_ptr).to_string(),
                }
            })?;

            assert!(res > 0, "We must have inserted something into DB");
        }
    }

    trx.commit().await.map_err(|e| {
        PlatformValidationError::PgRuntimeError { database: db_name.to_string(), error: e.to_string() }
    })?;

    Ok(())
}

pub struct DbSchemaSnapshot {
    raw_fields: Vec<Vec<String>>,
    field_type_index: HashMap<String, HashMap<String, String>>,
}

pub async fn get_pg_schema_trx<'a>(client: &tokio_postgres::Transaction<'a>) -> Result<DbSchemaSnapshot, PlatformValidationError> {
    let query = r#"
        SELECT table_name, column_name, column_default, data_type, is_nullable, is_generated, generation_expression, is_updatable
        FROM information_schema.columns
        WHERE table_schema='public'
        ORDER BY table_name, column_name;
    "#;
    let res = client.query(query, &[]).await.map_err(|e| {
        PlatformValidationError::RuntimeError { error: e.to_string() }
    })?;

    let mut ftype_index = HashMap::new();
    let out_vec = rows_to_string_matrix(&res);
    for values in &out_vec {
        let tname = &values[0];
        let cname = &values[1];
        let ctype = &values[3];
        let e = ftype_index.entry(tname.clone()).or_insert_with(HashMap::new);
        let res = e.insert(cname.clone(), ctype.clone());
        assert!(res.is_none());
    }

    let res = DbSchemaSnapshot {
        raw_fields: out_vec,
        field_type_index: ftype_index,
    };

    Ok(res)
}

pub async fn get_db_materialized_views(client: &tokio_postgres::Client) -> Result<Vec<Vec<String>>, PlatformValidationError> {
    let query = r#"
        SELECT schemaname, matviewname, hasindexes, ispopulated, definition
        FROM pg_matviews
        WHERE schemaname = 'public';
    "#;
    let mat_views = client.query(query, &[]).await.map_err(|e| {
        PlatformValidationError::RuntimeError { error: e.to_string() }
    })?;

    Ok(rows_to_string_matrix(&mat_views))
}

pub fn rows_to_string_matrix(rows: &[Row]) -> Vec<Vec<String>> {
    let mut out_vec = Vec::new();
    for r in rows {
        if out_vec.is_empty() {
            // print header
            let mut header = Vec::new();
            for c in r.columns() {
                header.push(c.name().to_owned());
            }
            out_vec.push(header);
        }

        out_vec.push(coerce_pg_row_to_vec(r));
    }

    out_vec
}

pub async fn get_pg_schema(client: &tokio_postgres::Client, migration_sql: &str) -> Result<DbSchemaSnapshot, PlatformValidationError> {
    lazy_static! {
        static ref IDENTIFIER_NAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-z_][a-z0-9_]*$").unwrap();
    }

    // mvp, we really need pgdump
    let query = r#"
        SELECT table_name, column_name, column_default, data_type, is_nullable, is_generated, generation_expression, is_updatable
        FROM information_schema.columns
        WHERE table_schema='public'
        ORDER BY table_name, column_name;
    "#;
    let res = client.query(query, &[]).await.map_err(|e| {
        PlatformValidationError::RuntimeError { error: e.to_string() }
    })?;

    let mut ftype_index = HashMap::new();
    let out_vec = rows_to_string_matrix(&res);
    for values in &out_vec {
        let tname = &values[0];
        let cname = &values[1];
        if !IDENTIFIER_NAME_REGEX.is_match(&tname) {
            return Err(
                PlatformValidationError::PgTableNameIsNotSnakeCase {
                    bad_table_name: tname.clone(),
                    migration_sql: migration_sql.to_string(),
                }
            );
        }

        if !IDENTIFIER_NAME_REGEX.is_match(&cname) {
            return Err(
                PlatformValidationError::PgColumnNameIsNotSnakeCase {
                    bad_column_name: cname.clone(),
                    table_name: tname.clone(),
                    migration_sql: migration_sql.to_string(),
                }
            );
        }
        let ctype = &values[3];
        let nullable = &values[4];
        if nullable == "YES" {
            return Err(PlatformValidationError::PgNullableColumnsAreNotAllowed {
                table_name: tname.clone(),
                table_column_name: cname.clone(),
                table_column_type: ctype.clone(),
                migration_sql: migration_sql.to_string(),
            });
        }

        let e = ftype_index.entry(tname.clone()).or_insert_with(HashMap::new);
        let res = e.insert(cname.clone(), ctype.clone());
        // println!("{}\t{}\t{}\t{}", tname, cname, ctype, nullable);
        assert!(res.is_none());
    }

    let res = DbSchemaSnapshot {
        raw_fields: out_vec,
        field_type_index: ftype_index,
    };

    Ok(res)
}

struct TempDb {
    ctx: AsyncCheckContext,
    container_name: String,
    kill_channel: tokio::sync::mpsc::Sender<bool>,
    connection_string: String,
}

impl TempDb {
    async fn new(ctx: AsyncCheckContext, db_name: &str, docker_image: &str) -> TempDb {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let inst_id: u64 = rng.gen();

        let container_name = format!("epl_test_postgres_{}_{}", inst_id, db_name);
        let socket_path = format!("/tmp/{}.pgsocket", container_name);
        let bind = format!("{}:/var/run/postgresql", socket_path);
        let pg_user = format!("POSTGRES_USER={}", db_name);
        let pg_db = format!("POSTGRES_DB={}", db_name);
        let url_encoded_socket_path = urlencoding::encode(&socket_path);
        let connection_string = format!("postgresql://{}:strongpassword@{}/{}", db_name, url_encoded_socket_path, db_name);

        let image_status =
            Command::new("docker")
                .arg("images")
                .arg("-q")
                .arg(docker_image)
                .output()
                .await.expect("Can't query image status");

        // image doesn't exist, pull
        if image_status.stdout.is_empty() {
            let pull_res =
                Command::new("docker")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .arg("pull")
                    .arg("-q")
                    .arg(docker_image)
                    .status()
                    .await.expect("Can't pull docker image");

            assert_eq!(pull_res.code(), Some(0), "Failed to pull docker image");
        }

        let mut child =
            Command::new("docker")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .arg("run")
                .arg("-i")
                .arg("--rm")
                .arg("--name")
                .arg(container_name.as_str())
                .arg("-e")
                .arg(&pg_db)
                .arg("-e")
                .arg(&pg_user)
                .arg("-e")
                .arg("POSTGRES_PASSWORD=strongpassword")
                .arg("-e")
                .arg("PGDATA=/var/lib/postgresql/data/pgdata")
                .arg("--mount")
                .arg("type=tmpfs,destination=/var/lib/postgresql/data/pgdata,tmpfs-mode=1777")
                .arg("-v")
                .arg(&bind)
                .arg(docker_image)
                .spawn().expect("Failed to spawn");

        let wait_beginning = tokio::time::Instant::now();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            match tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await {
                Ok(_) => { break; }
                Err(_) => {
                    if tokio::time::Instant::now() - wait_beginning > tokio::time::Duration::from_secs(10) {
                        panic!("Failed to start temp database in 10 seconds");
                    }
                }
            }
        }

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
        let async_rm = format!("sleep 20; docker rm -f {}", container_name);
        let _ = Command::new("/bin/sh")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-c")
            .arg(&async_rm)
            .spawn();

        TempDb { ctx, container_name, kill_channel: tx, connection_string }
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = self.kill_channel.try_send(true);
        let cont = self.container_name.clone();
        let fut = tokio::spawn(async move {
            let _ = Command::new("docker")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .arg("rm")
                .arg("-f")
                .arg(&cont)
                .status().await;
        });
        let _ = self.ctx.wait_for.send(fut);
    }
}

pub(crate) fn check_transaction_steps(db: &Database) -> Result<Projection<TableRowPointerPgTransaction, Vec<TransactionStep>>, PlatformValidationError> {
    let mut f_map: HashMap<TableRowPointerPgTransaction, Vec<TransactionStep>> = HashMap::new();

    for db_ptr in db.pg_schema().rows_iter() {
        let mut m: HashMap<String, DbQueryOrMutator> = HashMap::new();

        for query in db.pg_schema().c_children_pg_query(db_ptr) {
            let output = m.insert(db.pg_query().c_query_name(*query).clone(), DbQueryOrMutator::Query(*query));

            if output.is_some() {
                return Err(PlatformValidationError::PgQueryAndMutatorShareSameName {
                    pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                    query_or_mutator_name: db.pg_query().c_query_name(*query).clone(),
                });
            }
        }

        for mutator in db.pg_schema().c_children_pg_mutator(db_ptr) {
            let output = m.insert(db.pg_mutator().c_mutator_name(*mutator).clone(), DbQueryOrMutator::Mutator(*mutator));

            if output.is_some() {
                return Err(PlatformValidationError::PgQueryAndMutatorShareSameName {
                    pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                    query_or_mutator_name: db.pg_mutator().c_mutator_name(*mutator).clone(),
                });
            }
        }

        for transaction in db.pg_schema().c_children_pg_transaction(db_ptr) {
            let split = db.pg_transaction().c_steps(*transaction);
            let mut res = Vec::new();
            let mut mutators_found = false;
            for line in split.lines() {
                let line = line.trim();
                let (line, is_multi) = if line.ends_with("[]") {
                    (line.trim_end_matches("[]"), true)
                } else { (line, false) };

                if !line.is_empty() {
                    match m.get(line) {
                        Some(step) => {
                            let query = step.clone();
                            let step = TransactionStep {
                                query, is_multi,
                            };
                            match &step.query {
                                DbQueryOrMutator::Query(q) => {
                                    if db.pg_query().c_is_mutating(*q) {
                                        mutators_found = true;
                                    }
                                },
                                DbQueryOrMutator::Mutator(_) => {
                                    mutators_found = true;
                                },
                            }
                            res.push(step);
                        },
                        None => {
                            return Err(PlatformValidationError::PgTransactionStepNotFound {
                                pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                                transaction_name: db.pg_transaction().c_transaction_name(*transaction).clone(),
                                step_not_found: line.to_string(),
                            });
                        },
                    }
                }
            }

            if res.len() < 2 {
                return Err(PlatformValidationError::PgTransactionMustHaveAtLeastTwoSteps {
                    pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                    transaction_name: db.pg_transaction().c_transaction_name(*transaction).clone(),
                    step_count: res.len(),
                });
            }

            if !mutators_found && !db.pg_transaction().c_is_read_only(*transaction) {
                return Err(PlatformValidationError::PgTransactionWithoutMutatorsMustBeMarkedAsReadOnly {
                    pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                    transaction_name: db.pg_transaction().c_transaction_name(*transaction).clone(),
                });
            }

            if mutators_found && db.pg_transaction().c_is_read_only(*transaction) {
                return Err(PlatformValidationError::PgTransactionReadOnlyTransactionHasMutators {
                    pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                    transaction_name: db.pg_transaction().c_transaction_name(*transaction).clone(),
                });
            }

            let mut step_set: HashSet<DbQueryOrMutator> = HashSet::new();

            for step in &res {
                if !step_set.insert(step.query.clone()) {
                    return Err(PlatformValidationError::PgTransactionDuplicateStepsDetected {
                        pg_schema: db.pg_schema().c_schema_name(db_ptr).clone(),
                        transaction_name: db.pg_transaction().c_transaction_name(*transaction).clone(),
                        duplicate_step_name: step.query.name(db).to_string(),
                    });
                }
            }

            let prev = f_map.insert(*transaction, res);
            assert!(prev.is_none());
        }
    }

    Ok(Projection::create(db.pg_transaction().rows_iter(), |trx| {
        f_map.remove(&trx).unwrap()
    }))
}

pub fn pg_schemas_in_region(db: &Database) -> Projection<TableRowPointerRegion, HashSet<TableRowPointerPgSchema>> {
    Projection::create(db.region().rows_iter(), |region| {
        let mut res = HashSet::new();

        for depl in db.region().c_referrers_pg_deployment__region(region) {
            for depl_schema in db.pg_deployment().c_children_pg_deployment_schemas(*depl) {
                let _ = res.insert(db.pg_deployment_schemas().c_pg_schema(*depl_schema));
            }
        }

        res
    })
}

#[test]
fn test_basic_parse() {
    let res = parse_full_query(
        "SELECT doofus FROM roofus WHERE id = {some_customer_id:INT} AND {some_status} = status",
    );

    assert_eq!(
        res,
        Ok(vec![
            ParsedQuerySegment::Text("SELECT doofus FROM roofus WHERE id = ".to_string()),
            ParsedQuerySegment::QueryArg {
                arg_name: "some_customer_id".to_string(),
                the_type: Some("INT".to_string())
            },
            ParsedQuerySegment::Text(" AND ".to_string()),
            ParsedQuerySegment::QueryArg {
                arg_name: "some_status".to_string(),
                the_type: None
            },
            ParsedQuerySegment::Text(" = status".to_string()),
        ])
    )
}

#[test]
fn test_question_mark_disallowed() {
    let expr = "SELECT something FROM moo WHERE id = ?";
    let res = parse_full_query(expr);

    assert_eq!(
        res,
        Err(
            PlatformValidationError::PgOriginalQueryParametersAreNotAllowed {
                query_expression: expr.to_string(),
                found_forbidden_value: "?".to_string(),
            }
        )
    )
}

#[test]
fn test_dollar_sign_disallowed() {
    let expr = "SELECT something FROM moo WHERE id = $1";
    let res = parse_full_query(expr);

    assert_eq!(
        res,
        Err(
            PlatformValidationError::PgOriginalQueryParametersAreNotAllowed {
                query_expression: expr.to_string(),
                found_forbidden_value: "$".to_string(),
            }
        )
    )
}

#[test]
fn test_unsupported_argument_type() {
    let expr = "SELECT something FROM moo WHERE id = {id:UNSUPPORTED}";
    let res = parse_and_analyze_query(expr);

    assert_eq!(
        res,
        Err(PlatformValidationError::PgUnsupportedArgumentType {
            query_expression: expr.to_string(),
            unsupported_type: "UNSUPPORTED".to_string(),
            allowed_types: vec!["INT", "BIGINT", "FLOAT", "BOOL", "TEXT"],
        })
    )
}

#[test]
fn test_unsupported_argument_type_lowercase() {
    let expr = "SELECT something FROM moo WHERE id = {id:int}";
    let res = parse_and_analyze_query(expr);

    assert_eq!(
        res,
        Err(PlatformValidationError::PgUnsupportedArgumentType {
            query_expression: expr.to_string(),
            unsupported_type: "int".to_string(),
            allowed_types: vec!["INT", "BIGINT", "FLOAT", "BOOL", "TEXT"],
        })
    )
}

#[test]
fn test_diverging_db_types() {
    let expr = "SELECT something FROM moo WHERE id = {id:INT} AND other_id = {id:TEXT}";
    let res = parse_and_analyze_query(expr);

    assert_eq!(
        res,
        Err(PlatformValidationError::PgDivergingTypesForSameArgument {
            query_expression: expr.to_string(),
            argument_name: "id".to_string(),
            type_a: "INT".to_string(),
            type_b: "TEXT".to_string(),
        })
    )
}

#[test]
fn test_unspecified_arg_types() {
    let expr = "SELECT something FROM moo WHERE id = {id}";
    let res = parse_and_analyze_query(expr);

    assert_eq!(
        res,
        Err(
            PlatformValidationError::PgArgumentTypeUnspecifiedAtLeastOnce {
                query_expression: expr.to_string(),
                argument_name: "id".to_string(),
            }
        )
    )
}

#[test]
fn test_full_parse() {
    let expr = "
    SELECT something FROM moo WHERE id = {id:INT} AND other_id = {id} AND some_text = {some_text:TEXT}
    ";
    let res = parse_and_analyze_query(expr);

    assert_eq!(
        res,
        Ok(FullQuery {
            original_expression: expr.to_string(),
            interpolated_expression: "
    SELECT something FROM moo WHERE id = $1 AND other_id = $1 AND some_text = $2
    "
            .to_string(),
            args: vec![
                QueryArg {
                    the_type: ValidDbType::INT,
                    name: "id".to_string()
                },
                QueryArg {
                    the_type: ValidDbType::TEXT,
                    name: "some_text".to_string()
                },
            ]
        })
    )
}
