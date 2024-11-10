use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::database::{
    Database, TableRowPointerBackendApplication, TableRowPointerBackendApplicationPgShard,
    TableRowPointerBackendApplicationDeployment,
    TableRowPointerBackendApplicationDeploymentIngress,
    TableRowPointerBackendApplicationNatsStream, TableRowPointerBackendHttpEndpoint,
    TableRowPointerPgDeploymentSchemas, TableRowPointerPgMutator, TableRowPointerPgQuery,
    TableRowPointerPgTransaction, TableRowPointerFrontendApplication,
    TableRowPointerFrontendApplicationDeployment,
    TableRowPointerFrontendApplicationDeploymentIngress,
    TableRowPointerFrontendApplicationExternalLink, TableRowPointerFrontendApplicationExternalPage,
    TableRowPointerFrontendApplicationUsedEndpoint, TableRowPointerNatsJetstreamStream, TableRowPointerRegion, TableRowPointerBackendApplicationS3Bucket, TableRowPointerMinioBucket, TableRowPointerBackendApplicationConfig, TableRowPointerBackendApplicationChShard, TableRowPointerChDeploymentSchemas, TableRowPointerChQuery, TableRowPointerChMutator,
};

use super::{projections::Projection, PlatformValidationError};

pub struct ApplicationPgQueries {
    pub queries: BTreeSet<TableRowPointerPgQuery>,
    pub mutators: BTreeSet<TableRowPointerPgMutator>,
    pub transactions: BTreeSet<TableRowPointerPgTransaction>,
}

pub fn check_application_build_environments(db: &Database) -> Result<(), PlatformValidationError> {
    for backend_app in db.backend_application().rows_iter() {
        let env = db.backend_application().c_build_environment(backend_app);
        let kind = db.rust_compilation_environment().c_environment_kind(env);
        let expected_kind = "backend_app";
        if kind != expected_kind {
            return Err(
                PlatformValidationError::ApplicationInvalidBuildEnvironmentKind {
                    application_name: db
                        .backend_application()
                        .c_application_name(backend_app)
                        .clone(),
                    expected_compilation_environment_kind: expected_kind.to_string(),
                    found_compilation_environment_kind: kind.clone(),
                },
            );
        }
    }

    Ok(())
}

pub fn check_application_pg_queries(
    db: &Database,
) -> Result<
    Projection<TableRowPointerBackendApplicationPgShard, ApplicationPgQueries>,
    PlatformValidationError,
> {
    let mut queries_index: HashMap<String, TableRowPointerPgQuery> = HashMap::new();
    let mut mutators_index: HashMap<String, TableRowPointerPgMutator> = HashMap::new();
    let mut transactions_index: HashMap<String, TableRowPointerPgTransaction> = HashMap::new();

    for dbq in db.pg_query().rows_iter() {
        let schema = db.pg_query().c_parent(dbq);
        let key = format!(
            "{}=>{}",
            db.pg_schema().c_schema_name(schema),
            db.pg_query().c_query_name(dbq)
        );
        assert!(
            queries_index.insert(key, dbq).is_none(),
            "Duplicate keys detected in pg queries, this should have been detected earlier"
        );
    }

    for dbm in db.pg_mutator().rows_iter() {
        let schema = db.pg_mutator().c_parent(dbm);
        let key = format!(
            "{}=>{}",
            db.pg_schema().c_schema_name(schema),
            db.pg_mutator().c_mutator_name(dbm)
        );
        assert!(
            mutators_index.insert(key, dbm).is_none(),
            "Duplicate keys detected in pg mutators, this should have been detected earlier"
        );
    }

    for dbt in db.pg_transaction().rows_iter() {
        let schema = db.pg_transaction().c_parent(dbt);
        let key = format!(
            "{}=>{}",
            db.pg_schema().c_schema_name(schema),
            db.pg_transaction().c_transaction_name(dbt)
        );
        assert!(
            transactions_index.insert(key, dbt).is_none(),
            "Duplicate keys detected in pg transactions, this should have been detected earlier"
        );
    }

    Projection::maybe_create(db.backend_application_pg_shard().rows_iter(), |shard| {
        let queries_src = db.backend_application_pg_shard().c_used_queries(shard);
        let mutators_src = db.backend_application_pg_shard().c_used_mutators(shard);
        let transactions_src = db.backend_application_pg_shard().c_used_transactions(shard);
        let pg_schema = db.backend_application_pg_shard().c_pg_schema(shard);
        let pg_schema_name = db.pg_schema().c_schema_name(pg_schema);

        let mut queries = ApplicationPgQueries {
            queries: BTreeSet::new(),
            mutators: BTreeSet::new(),
            transactions: BTreeSet::new(),
        };

        for line in queries_src
            .lines()
            .filter_map(|i| Some(i.trim()).filter(|i| !i.is_empty()))
        {
            let key = format!("{pg_schema_name}=>{line}");

            if let Some(v) = queries_index.get(&key) {
                if !queries.queries.insert(*v) {
                    return Err(
                        PlatformValidationError::ApplicationPgShardQueryDefinedTwice {
                            used_queries_src: queries_src.to_string(),
                            used_query_defined_twice: line.to_string(),
                            application_pg_schema: pg_schema_name.clone(),
                            application_pg_shard: db
                                .backend_application_pg_shard()
                                .c_shard_name(shard)
                                .to_string(),
                            application: db
                                .backend_application()
                                .c_application_name(db.backend_application_pg_shard().c_parent(shard))
                                .to_string(),
                        },
                    );
                }
            } else {
                return Err(
                    PlatformValidationError::ApplicationPgShardQueryNotFoundInPgSchema {
                        queries_src: queries_src.clone(),
                        query_not_found: line.to_string(),
                        application_pg_schema: pg_schema_name.clone(),
                        application_pg_shard: db
                            .backend_application_pg_shard()
                            .c_shard_name(shard)
                            .to_string(),
                        application: db
                            .backend_application()
                            .c_application_name(db.backend_application_pg_shard().c_parent(shard))
                            .to_string(),
                    },
                );
            }
        }

        for line in mutators_src
            .lines()
            .filter_map(|i| Some(i.trim()).filter(|i| !i.is_empty()))
        {
            let key = format!("{pg_schema_name}=>{line}");

            if let Some(v) = mutators_index.get(&key) {
                if !queries.mutators.insert(*v) {
                    return Err(
                        PlatformValidationError::ApplicationPgShardMutatorDefinedTwice {
                            used_mutators_src: mutators_src.to_string(),
                            used_mutator_defined_twice: line.to_string(),
                            application_pg_schema: pg_schema_name.clone(),
                            application_pg_shard: db
                                .backend_application_pg_shard()
                                .c_shard_name(shard)
                                .to_string(),
                            application: db
                                .backend_application()
                                .c_application_name(db.backend_application_pg_shard().c_parent(shard))
                                .to_string(),
                        },
                    );
                }
            } else {
                return Err(
                    PlatformValidationError::ApplicationPgShardMutatorNotFoundInPgSchema {
                        mutators_src: mutators_src.clone(),
                        mutator_not_found: line.to_string(),
                        application_pg_schema: pg_schema_name.clone(),
                        application_pg_shard: db
                            .backend_application_pg_shard()
                            .c_shard_name(shard)
                            .to_string(),
                        application: db
                            .backend_application()
                            .c_application_name(db.backend_application_pg_shard().c_parent(shard))
                            .to_string(),
                    },
                );
            }
        }

        for line in transactions_src
            .lines()
            .filter_map(|i| Some(i.trim()).filter(|i| !i.is_empty()))
        {
            let key = format!("{pg_schema_name}=>{line}");

            if let Some(v) = transactions_index.get(&key) {
                if !queries.transactions.insert(*v) {
                    return Err(
                        PlatformValidationError::ApplicationPgShardTransactionDefinedTwice {
                            used_transactions_src: transactions_src.to_string(),
                            used_transaction_defined_twice: line.to_string(),
                            application_pg_schema: pg_schema_name.clone(),
                            application_pg_shard: db
                                .backend_application_pg_shard()
                                .c_shard_name(shard)
                                .to_string(),
                            application: db
                                .backend_application()
                                .c_application_name(db.backend_application_pg_shard().c_parent(shard))
                                .to_string(),
                        },
                    );
                }
            } else {
                return Err(
                    PlatformValidationError::ApplicationPgShardTransactionNotFoundInPgSchema {
                        transactions_src: transactions_src.clone(),
                        transaction_not_found: line.to_string(),
                        application_pg_schema: pg_schema_name.clone(),
                        application_pg_shard: db
                            .backend_application_pg_shard()
                            .c_shard_name(shard)
                            .to_string(),
                        application: db
                            .backend_application()
                            .c_application_name(db.backend_application_pg_shard().c_parent(shard))
                            .to_string(),
                    },
                );
            }
        }

        Ok(queries)
    })
}

pub struct ApplicationChQueries {
    pub queries: BTreeSet<TableRowPointerChQuery>,
    pub mutators: BTreeSet<TableRowPointerChMutator>,
    pub inserters: BTreeSet<String>,
}

pub fn check_application_ch_queries(
    db: &Database,
) -> Result<
    Projection<TableRowPointerBackendApplicationChShard, ApplicationChQueries>,
    PlatformValidationError,
> {
    let mut queries_index: HashMap<String, TableRowPointerChQuery> = HashMap::new();
    let mut mutators_index: HashMap<String, TableRowPointerChMutator> = HashMap::new();

    for dbq in db.ch_query().rows_iter() {
        let schema = db.ch_query().c_parent(dbq);
        let key = format!(
            "{}=>{}",
            db.ch_schema().c_schema_name(schema),
            db.ch_query().c_query_name(dbq)
        );
        assert!(
            queries_index.insert(key, dbq).is_none(),
            "Duplicate keys detected in ch queries, this should have been detected earlier"
        );
    }

    for chm in db.ch_mutator().rows_iter() {
        let schema = db.ch_mutator().c_parent(chm);
        let key = format!(
            "{}=>{}",
            db.ch_schema().c_schema_name(schema),
            db.ch_mutator().c_mutator_name(chm)
        );
        assert!(
            mutators_index.insert(key, chm).is_none(),
            "Duplicate keys detected in ch mutators, this should have been detected earlier"
        );
    }

    Projection::maybe_create(db.backend_application_ch_shard().rows_iter(), |shard| {
        let queries_src = db.backend_application_ch_shard().c_used_queries(shard);
        let inserters_src = db.backend_application_ch_shard().c_used_inserters(shard);
        let mutators_src = db.backend_application_ch_shard().c_used_mutators(shard);
        let ch_schema = db.backend_application_ch_shard().c_ch_schema(shard);
        let ch_schema_name = db.ch_schema().c_schema_name(ch_schema);

        let mut queries = ApplicationChQueries {
            queries: BTreeSet::new(),
            inserters: BTreeSet::new(),
            mutators: BTreeSet::new(),
        };

        for line in queries_src
            .lines()
            .filter_map(|i| Some(i.trim()).filter(|i| !i.is_empty()))
        {
            let key = format!("{ch_schema_name}=>{line}");

            if let Some(v) = queries_index.get(&key) {
                if !queries.queries.insert(*v) {
                    return Err(
                        PlatformValidationError::ApplicationChShardQueryDefinedTwice {
                            used_query_defined_twice: line.to_string(),
                            used_queries_src: queries_src.clone(),
                            application_ch_schema: ch_schema_name.clone(),
                            application_ch_shard: db
                                .backend_application_ch_shard()
                                .c_shard_name(shard)
                                .to_string(),
                            application: db
                                .backend_application()
                                .c_application_name(db.backend_application_ch_shard().c_parent(shard))
                                .to_string(),
                        },
                    );
                }
            } else {
                return Err(
                    PlatformValidationError::ApplicationChShardQueryNotFoundInChSchema {
                        query_not_found: line.to_string(),
                        application_ch_schema: ch_schema_name.clone(),
                        application_ch_shard: db
                            .backend_application_ch_shard()
                            .c_shard_name(shard)
                            .to_string(),
                        application: db
                            .backend_application()
                            .c_application_name(db.backend_application_ch_shard().c_parent(shard))
                            .to_string(),
                    },
                );
            }
        }

        for line in mutators_src
            .lines()
            .filter_map(|i| Some(i.trim()).filter(|i| !i.is_empty()))
        {
            let key = format!("{ch_schema_name}=>{line}");

            if let Some(v) = mutators_index.get(&key) {
                if !queries.mutators.insert(*v) {
                    return Err(
                        PlatformValidationError::ApplicationChShardMutatorDefinedTwice {
                            used_mutator_defined_twice: line.to_string(),
                            used_mutators_src: mutators_src.clone(),
                            application_ch_schema: ch_schema_name.clone(),
                            application_ch_shard: db
                                .backend_application_ch_shard()
                                .c_shard_name(shard)
                                .to_string(),
                            application: db
                                .backend_application()
                                .c_application_name(db.backend_application_ch_shard().c_parent(shard))
                                .to_string(),
                        },
                    );
                }
            } else {
                return Err(
                    PlatformValidationError::ApplicationChShardMutatorNotFoundInChSchema {
                        mutator_not_found: line.to_string(),
                        application_ch_schema: ch_schema_name.clone(),
                        application_ch_shard: db
                            .backend_application_ch_shard()
                            .c_shard_name(shard)
                            .to_string(),
                        application: db
                            .backend_application()
                            .c_application_name(db.backend_application_ch_shard().c_parent(shard))
                            .to_string(),
                    },
                );
            }
        }

        for line in inserters_src
            .lines()
            .filter_map(|i| Some(i.trim()).filter(|i| !i.is_empty()))
        {
            if !queries.inserters.insert(line.to_string()) {
                return Err(
                    PlatformValidationError::ApplicationChShardInserterDefinedTwice {
                        inserter_defined_twice: line.to_string(),
                        inserters_src: inserters_src.clone(),
                        application_ch_schema: ch_schema_name.clone(),
                        application_ch_shard: db
                            .backend_application_ch_shard()
                            .c_shard_name(shard)
                            .to_string(),
                        application: db
                            .backend_application()
                            .c_application_name(db.backend_application_ch_shard().c_parent(shard))
                            .to_string(),
                    },
                );
            }
        }


        Ok(queries)
    })
}

lazy_static! {
    static ref APP_DB_WIRING_EXTRACT_REGEX: regex::Regex =
        regex::Regex::new(r"^([a-z0-9_]+)\s*:\s*([a-z0-9_-]+=>[a-z0-9_-]+)$")
            .expect("Invalid deployment wiring extraction regex");
    static ref NATS_STREAM_WIRING_EXTRACT_REGEX: regex::Regex =
        regex::Regex::new(r"^([a-z0-9_]+)\s*:\s*([a-z0-9_-]+=>[a-z0-9_-]+)$")
            .expect("Invalid NATS stream wiring extraction regex");
    static ref EXPLICIT_FRONTEND_WIRING_REGEX: regex::Regex =
        regex::Regex::new(r"^([a-z0-9_]+)\s*:\s*([a-z0-9_-]+)$")
            .expect("Invalid explicit frontend wiring regex");
    static ref S3_BUCKET_WIRING_EXTRACT_REGEX: regex::Regex =
        regex::Regex::new(r"^([a-z0-9_]+)\s*:\s*([a-z0-9_-]+=>[a-z0-9_-]+)$")
            .expect("Invalid S3 bucket wiring extraction regex");
    static ref CONFIG_EXTRACT_REGEX: regex::Regex =
        regex::Regex::new(r"^([a-z0-9_]+)\s*:\s*(.*?)\s*$")
            .expect("Invalid config extraction regex");
}

#[test]
fn test_wiring_extraction_regex() {
    let res = APP_DB_WIRING_EXTRACT_REGEX.captures("henlo : this0_=>is_7_gut");
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 3);
    assert_eq!(res.get(1).unwrap().as_str(), "henlo");
    assert_eq!(res.get(2).unwrap().as_str(), "this0_=>is_7_gut");
}

pub fn application_deployments_pg_shard_wiring(
    db: &Database,
) -> Result<
    HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationPgShard, TableRowPointerPgDeploymentSchemas>,
    >,
    PlatformValidationError,
> {
    let mut pg_deployment_index: HashMap<String, TableRowPointerPgDeploymentSchemas> =
        HashMap::new();
    for db_depl in db.pg_deployment().rows_iter() {
        for pg_schema in db.pg_deployment().c_children_pg_deployment_schemas(db_depl) {
            let key = format!(
                "{}=>{}",
                db.pg_deployment().c_deployment_name(db_depl),
                db.pg_deployment_schemas().c_db_name(*pg_schema)
            );
            assert!(pg_deployment_index.insert(key, *pg_schema).is_none());
        }
    }

    let mut app_shard_index: HashMap<
        TableRowPointerBackendApplication,
        HashMap<String, TableRowPointerBackendApplicationPgShard>,
    > = HashMap::new();
    for app_db_shard in db.backend_application_pg_shard().rows_iter() {
        let e = app_shard_index
            .entry(db.backend_application_pg_shard().c_parent(app_db_shard))
            .or_default();
        assert!(e
            .insert(
                db.backend_application_pg_shard()
                    .c_shard_name(app_db_shard)
                    .clone(),
                app_db_shard
            )
            .is_none());
    }

    let mut res: HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationPgShard, TableRowPointerPgDeploymentSchemas>,
    > = HashMap::new();

    for app_depl in db.backend_application_deployment().rows_iter() {
        let db_wiring = db
            .backend_application_deployment()
            .c_pg_shard_wiring(app_depl);
        let deployment_name = db
            .backend_application_deployment()
            .c_deployment_name(app_depl);
        let application = db
            .backend_application_deployment()
            .c_application_name(app_depl);
        let application_name = db.backend_application().c_application_name(application);

        let depl_shards = res.entry(app_depl).or_default();

        for line in db_wiring
            .lines()
            .map(|i| i.trim())
            .filter(|i| !i.is_empty())
        {
            if let Some(cap) = APP_DB_WIRING_EXTRACT_REGEX.captures(line) {
                let app_shard = cap.get(1).unwrap().as_str();
                let target_shard = cap.get(2).unwrap().as_str();

                if let Some(app) = app_shard_index.get(&application) {
                    if let Some(app_sh) = app.get(app_shard) {
                        if let Some(db_depl) = pg_deployment_index.get(target_shard) {
                            let app_shard_schema =
                                db.backend_application_pg_shard().c_pg_schema(*app_sh);
                            let depl_pg_schema = db.pg_deployment_schemas().c_pg_schema(*db_depl);
                            if app_shard_schema != depl_pg_schema {
                                return Err(PlatformValidationError::ApplicationPgWiringSchemaMismatch {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    application_expected_pg_schema: db.pg_schema().c_schema_name(app_shard_schema).clone(),
                                    target_deployment_pg_schema: db.pg_schema().c_schema_name(depl_pg_schema).clone(),
                                    explanation: "Application expected Postgres schema mismatches wired shard actual shard schema",
                                });
                            }

                            if depl_shards.contains_key(app_sh) {
                                // shard defined multiple times
                                return Err(PlatformValidationError::ApplicationPgWiringApplicationShardDefinedMultipleTimes {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    redefined_app_db_shard_name: app_shard.to_string(),
                                    explanation: "Specified application Postgres shard redefined multiple times",
                                });
                            }
                            assert!(depl_shards.insert(*app_sh, *db_depl).is_none());
                        } else {
                            return Err(PlatformValidationError::ApplicationPgWiringTargetDbDeploymentDoesntExist {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                missing_pg_deployment: target_shard.to_string(),
                                explanation: "Specified Postgres deployment with schema doesn't exist",
                            });
                        }
                    } else {
                        // app has no DB stream, error
                        let valid_app_db_shards = db
                            .backend_application()
                            .c_children_backend_application_pg_shard(application)
                            .iter()
                            .map(|i| db.backend_application_pg_shard().c_shard_name(*i).clone())
                            .collect::<Vec<_>>();
                        return Err(
                            PlatformValidationError::ApplicationPgWiringApplicationHasNoDbShard {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                missing_application_db_shard: app_shard.to_string(),
                                valid_app_db_shards,
                                explanation: "Specified application Postgres shard is missing",
                            },
                        );
                    }
                } else {
                    // app has no DB shards, error
                    return Err(
                        PlatformValidationError::ApplicationPgWiringApplicationHasNoDbShards {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            explanation: "This application has no Postgres shards that need to be wired",
                        },
                    );
                }
            } else {
                return Err(PlatformValidationError::ApplicationPgWiringInvalidFormat {
                    application_deployment: deployment_name.clone(),
                    application_name: application_name.clone(),
                    bad_line: line.to_string(),
                    explanation:
                        "Valid example of db wiring \"pg_shard_a: pg_deployment_b=>db_name_c\"",
                });
            }
        }

        for shard in db
            .backend_application()
            .c_children_backend_application_pg_shard(application)
        {
            if !depl_shards.contains_key(shard) {
                return Err(
                    PlatformValidationError::ApplicationPgWiringUndefinedAppDbShard {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        undefined_application_db_shard: db
                            .backend_application_pg_shard()
                            .c_shard_name(*shard)
                            .clone(),
                        explanation: "Specified application db shard was not defined in wiring",
                    },
                );
            }
        }

        let depl_shards: &HashMap<_, _> = depl_shards;

        for (k1, v1) in depl_shards {
            for (k2, v2) in depl_shards {
                if k1 != k2 && v1 == v2 {
                    // multiple different app shards use same physical database
                    // pointless, just use one db connection pool instead
                    let mut app_shard_a_name =
                        db.backend_application_pg_shard().c_shard_name(*k1).clone();
                    let mut app_shard_b_name =
                        db.backend_application_pg_shard().c_shard_name(*k2).clone();
                    let mut target_physical_db_a = format!(
                        "{}=>{}",
                        db.pg_deployment()
                            .c_deployment_name(db.pg_deployment_schemas().c_parent(*v1)),
                        db.pg_deployment_schemas().c_db_name(*v1)
                    );
                    let mut target_physical_db_b = format!(
                        "{}=>{}",
                        db.pg_deployment()
                            .c_deployment_name(db.pg_deployment_schemas().c_parent(*v2)),
                        db.pg_deployment_schemas().c_db_name(*v2)
                    );
                    // make shard order deterministic for testing
                    if app_shard_a_name > app_shard_b_name {
                        std::mem::swap(&mut app_shard_a_name, &mut app_shard_b_name);
                        std::mem::swap(&mut target_physical_db_a, &mut target_physical_db_b);
                    }
                    return Err(PlatformValidationError::ApplicationPgWiringDifferentAppShardsPointToSameDatabase {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        app_shard_a_name,
                        app_shard_b_name,
                        target_physical_db_a,
                        target_physical_db_b,
                        explanation: "Two different db connections in application point to the same physical database",
                    });
                }
            }
        }
    }

    Ok(res)
}

pub fn application_deployments_ch_shard_wiring(
    db: &Database,
) -> Result<
    HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationChShard, TableRowPointerChDeploymentSchemas>,
    >,
    PlatformValidationError,
> {
    let mut ch_deployment_index: HashMap<String, TableRowPointerChDeploymentSchemas> =
        HashMap::new();
    for db_depl in db.ch_deployment().rows_iter() {
        for ch_schema in db.ch_deployment().c_children_ch_deployment_schemas(db_depl) {
            let key = format!(
                "{}=>{}",
                db.ch_deployment().c_deployment_name(db_depl),
                db.ch_deployment_schemas().c_db_name(*ch_schema)
            );
            assert!(ch_deployment_index.insert(key, *ch_schema).is_none());
        }
    }

    let mut app_shard_index: HashMap<
        TableRowPointerBackendApplication,
        HashMap<String, TableRowPointerBackendApplicationChShard>,
    > = HashMap::new();
    for app_db_shard in db.backend_application_ch_shard().rows_iter() {
        let e = app_shard_index
            .entry(db.backend_application_ch_shard().c_parent(app_db_shard))
            .or_default();
        assert!(e
            .insert(
                db.backend_application_ch_shard()
                    .c_shard_name(app_db_shard)
                    .clone(),
                app_db_shard
            )
            .is_none());
    }

    let mut res: HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationChShard, TableRowPointerChDeploymentSchemas>,
    > = HashMap::new();

    for app_depl in db.backend_application_deployment().rows_iter() {
        let db_wiring = db
            .backend_application_deployment()
            .c_ch_shard_wiring(app_depl);
        let deployment_name = db
            .backend_application_deployment()
            .c_deployment_name(app_depl);
        let application = db
            .backend_application_deployment()
            .c_application_name(app_depl);
        let application_name = db.backend_application().c_application_name(application);

        let depl_shards = res.entry(app_depl).or_default();

        for line in db_wiring
            .lines()
            .map(|i| i.trim())
            .filter(|i| !i.is_empty())
        {
            if let Some(cap) = APP_DB_WIRING_EXTRACT_REGEX.captures(line) {
                let app_shard = cap.get(1).unwrap().as_str();
                let target_shard = cap.get(2).unwrap().as_str();

                if let Some(app) = app_shard_index.get(&application) {
                    if let Some(app_sh) = app.get(app_shard) {
                        if let Some(db_depl) = ch_deployment_index.get(target_shard) {
                            let app_shard_schema =
                                db.backend_application_ch_shard().c_ch_schema(*app_sh);
                            let depl_ch_schema = db.ch_deployment_schemas().c_ch_schema(*db_depl);
                            if app_shard_schema != depl_ch_schema {
                                return Err(PlatformValidationError::ApplicationChWiringSchemaMismatch {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    application_expected_ch_schema: db.ch_schema().c_schema_name(app_shard_schema).clone(),
                                    target_deployment_ch_schema: db.ch_schema().c_schema_name(depl_ch_schema).clone(),
                                    explanation: "Application expected Clickhouse schema mismatches wired shard schema",
                                });
                            }

                            if depl_shards.contains_key(app_sh) {
                                // shard defined multiple times
                                return Err(PlatformValidationError::ApplicationChWiringApplicationShardDefinedMultipleTimes {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    redefined_app_db_shard_name: app_shard.to_string(),
                                    explanation: "Specified application Clickhouse shard redefined multiple times",
                                });
                            }
                            assert!(depl_shards.insert(*app_sh, *db_depl).is_none());
                        } else {
                            return Err(PlatformValidationError::ApplicationChWiringTargetChDeploymentDoesntExist {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                missing_ch_deployment: target_shard.to_string(),
                                explanation: "Specified Clickhouse deployment with schema doesn't exist",
                            });
                        }
                    } else {
                        // app has no DB stream, error
                        let valid_app_db_shards = db
                            .backend_application()
                            .c_children_backend_application_ch_shard(application)
                            .iter()
                            .map(|i| db.backend_application_ch_shard().c_shard_name(*i).clone())
                            .collect::<Vec<_>>();
                        return Err(
                            PlatformValidationError::ApplicationChWiringApplicationHasNoDbShard {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                missing_application_db_shard: app_shard.to_string(),
                                valid_app_db_shards,
                                explanation: "Specified application Clickhouse shard is missing",
                            },
                        );
                    }
                } else {
                    // app has no DB shards, error
                    return Err(
                        PlatformValidationError::ApplicationChWiringApplicationHasNoDbShards {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            explanation: "This application has no Clickhouse shards that need to be wired",
                        },
                    );
                }
            } else {
                return Err(PlatformValidationError::ApplicationChWiringInvalidFormat {
                    application_deployment: deployment_name.clone(),
                    application_name: application_name.clone(),
                    bad_line: line.to_string(),
                    explanation: "Valid example of db wiring \"ch_shard_a: ch_deployment_b=>db_name_c\"",
                });
            }
        }

        for shard in db
            .backend_application()
            .c_children_backend_application_ch_shard(application)
        {
            if !depl_shards.contains_key(shard) {
                return Err(
                    PlatformValidationError::ApplicationChWiringUndefinedAppDbShard {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        undefined_application_db_shard: db
                            .backend_application_ch_shard()
                            .c_shard_name(*shard)
                            .clone(),
                        explanation: "Specified application db shard was not defined in wiring",
                    },
                );
            }
        }

        let depl_shards: &HashMap<_, _> = depl_shards;

        for (k1, v1) in depl_shards {
            for (k2, v2) in depl_shards {
                if k1 != k2 && v1 == v2 {
                    // multiple different app shards use same physical database
                    // pointless, just use one db connection pool instead
                    let mut app_shard_a_name =
                        db.backend_application_ch_shard().c_shard_name(*k1).clone();
                    let mut app_shard_b_name =
                        db.backend_application_ch_shard().c_shard_name(*k2).clone();
                    let mut target_physical_db_a = format!(
                        "{}=>{}",
                        db.ch_deployment()
                            .c_deployment_name(db.ch_deployment_schemas().c_parent(*v1)),
                        db.ch_deployment_schemas().c_db_name(*v1)
                    );
                    let mut target_physical_db_b = format!(
                        "{}=>{}",
                        db.ch_deployment()
                            .c_deployment_name(db.ch_deployment_schemas().c_parent(*v2)),
                        db.ch_deployment_schemas().c_db_name(*v2)
                    );
                    // make shard order deterministic for testing
                    if app_shard_a_name > app_shard_b_name {
                        std::mem::swap(&mut app_shard_a_name, &mut app_shard_b_name);
                        std::mem::swap(&mut target_physical_db_a, &mut target_physical_db_b);
                    }
                    return Err(PlatformValidationError::ApplicationChWiringDifferentAppShardsPointToSameDatabase {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        app_shard_a_name,
                        app_shard_b_name,
                        target_physical_db_a,
                        target_physical_db_b,
                        explanation: "Two different db connections in application point to the same physical database",
                    });
                }
            }
        }
    }

    Ok(res)
}

pub fn application_deployments_nats_stream_wiring(
    db: &Database,
) -> Result<
    HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationNatsStream, TableRowPointerNatsJetstreamStream>,
    >,
    PlatformValidationError,
> {
    let mut stream_deployment_index: HashMap<String, TableRowPointerNatsJetstreamStream> =
        HashMap::new();
    for nats_stream in db.nats_jetstream_stream().rows_iter() {
        let nats_cluster = db
            .nats_cluster()
            .c_cluster_name(db.nats_jetstream_stream().c_parent(nats_stream));
        let key = format!(
            "{}=>{}",
            nats_cluster,
            db.nats_jetstream_stream().c_stream_name(nats_stream)
        );
        assert!(stream_deployment_index.insert(key, nats_stream).is_none());
    }

    let mut app_streams_index: HashMap<
        TableRowPointerBackendApplication,
        HashMap<String, TableRowPointerBackendApplicationNatsStream>,
    > = HashMap::new();
    for app_nats_stream in db.backend_application_nats_stream().rows_iter() {
        let e = app_streams_index
            .entry(
                db.backend_application_nats_stream()
                    .c_parent(app_nats_stream),
            )
            .or_default();
        assert!(e
            .insert(
                db.backend_application_nats_stream()
                    .c_stream_name(app_nats_stream)
                    .clone(),
                app_nats_stream
            )
            .is_none());
    }

    let mut res: HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationNatsStream, TableRowPointerNatsJetstreamStream>,
    > = HashMap::new();

    for app_depl in db.backend_application_deployment().rows_iter() {
        let streams_wiring = db
            .backend_application_deployment()
            .c_nats_stream_wiring(app_depl);
        let deployment_name = db
            .backend_application_deployment()
            .c_deployment_name(app_depl);
        let application = db
            .backend_application_deployment()
            .c_application_name(app_depl);
        let application_name = db.backend_application().c_application_name(application);

        let depl_streams = res.entry(app_depl).or_default();

        for line in streams_wiring
            .lines()
            .map(|i| i.trim())
            .filter(|i| !i.is_empty())
        {
            if let Some(cap) = NATS_STREAM_WIRING_EXTRACT_REGEX.captures(line) {
                let app_stream = cap.get(1).unwrap().as_str();
                let target_stream = cap.get(2).unwrap().as_str();

                if let Some(app) = app_streams_index.get(&application) {
                    if let Some(app_sh) = app.get(app_stream) {
                        if let Some(nats_stream) = stream_deployment_index.get(target_stream) {
                            let app_stream_type =
                                db.backend_application_nats_stream().c_stream_type(*app_sh);
                            let depl_stream_type =
                                db.nats_jetstream_stream().c_stream_type(*nats_stream);

                            if app_stream_type != depl_stream_type {
                                return Err(PlatformValidationError::ApplicationStreamWiringTypeMismatch {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    application_expected_stream_type: db.versioned_type().c_type_name(app_stream_type).clone(),
                                    target_deployment_stream_type: db.versioned_type().c_type_name(depl_stream_type).clone(),
                                    explanation: "Application expected NATS stream type mismatches wired NATS cluster stream type",
                                });
                            }

                            let app_stream_subjects_enabled = db.backend_application_nats_stream().c_enable_subjects(*app_sh);
                            let final_stream_subjects_enabled = db.nats_jetstream_stream().c_enable_subjects(*nats_stream);

                            if app_stream_subjects_enabled != final_stream_subjects_enabled {
                                return Err(PlatformValidationError::ApplicationStreamWiringSubjectsEnabledMismatch {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    application_expected_enable_subjects: app_stream_subjects_enabled,
                                    target_deployment_stream_enable_subjects: final_stream_subjects_enabled,
                                    explanation: "Application expected NATS stream enable_subjects value mismatches wired NATS cluster stream enable_subjects value",
                                });
                            }

                            if depl_streams.contains_key(app_sh) {
                                // shard defined multiple times
                                return Err(PlatformValidationError::ApplicationStreamWiringApplicationStreamDefinedMultipleTimes {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    redefined_app_stream_name: app_stream.to_string(),
                                    explanation: "Specified application NATS stream redefined multiple times",
                                });
                            }

                            assert!(depl_streams.insert(*app_sh, *nats_stream).is_none());
                        } else {
                            return Err(PlatformValidationError::ApplicationStreamWiringTargetStreamDoesntExist {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                missing_nats_stream: target_stream.to_string(),
                                explanation: "Specified nats cluster with stream doesn't exist",
                            });
                        }
                    } else {
                        // app has no streams, error
                        let valid_app_streams = db
                            .backend_application()
                            .c_children_backend_application_nats_stream(application)
                            .iter()
                            .map(|i| {
                                db.backend_application_nats_stream()
                                    .c_stream_name(*i)
                                    .clone()
                            })
                            .collect::<Vec<_>>();
                        return Err(PlatformValidationError::ApplicationStreamsWiringApplicationHasNoStreamSpecified {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            missing_application_stream: app_stream.to_string(),
                            valid_app_streams,
                            explanation: "Specified application stream is missing",
                        });
                    }
                } else {
                    // app has no streams, error
                    return Err(
                        PlatformValidationError::ApplicationStreamsWiringApplicationHasNoStreams {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            explanation:
                                "This application has no NATS streams that need to be wired",
                        },
                    );
                }
            } else {
                return Err(PlatformValidationError::ApplicationStreamsWiringInvalidFormat {
                    application_deployment: deployment_name.clone(),
                    application_name: application_name.clone(),
                    bad_line: line.to_string(),
                    explanation: "Valid example of stream wiring \"stream_a: nats_cluster_b=>stream_name_c\"",
                });
            }
        }

        for stream in db
            .backend_application()
            .c_children_backend_application_nats_stream(application)
        {
            if !depl_streams.contains_key(stream) {
                return Err(
                    PlatformValidationError::ApplicationStreamsWiringUndefinedAppNatsStream {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        undefined_application_stream: db
                            .backend_application_nats_stream()
                            .c_stream_name(*stream)
                            .clone(),
                        explanation: "Specified application stream was not defined in wiring",
                    },
                );
            }
        }

        let depl_streams: &HashMap<_, _> = depl_streams;

        for (k1, v1) in depl_streams {
            for (k2, v2) in depl_streams {
                if k1 != k2 && v1 == v2 {
                    // multiple different app streams point to same physical stream
                    // probably a mistake, avoid such situations
                    let mut app_stream_a_name = db
                        .backend_application_nats_stream()
                        .c_stream_name(*k1)
                        .clone();
                    let mut app_stream_b_name = db
                        .backend_application_nats_stream()
                        .c_stream_name(*k2)
                        .clone();
                    let mut target_physical_stream_a = format!(
                        "{}=>{}",
                        db.nats_cluster()
                            .c_cluster_name(db.nats_jetstream_stream().c_parent(*v1)),
                        db.nats_jetstream_stream().c_stream_name(*v1)
                    );
                    let mut target_physical_stream_b = format!(
                        "{}=>{}",
                        db.nats_cluster()
                            .c_cluster_name(db.nats_jetstream_stream().c_parent(*v2)),
                        db.nats_jetstream_stream().c_stream_name(*v2)
                    );
                    // make stream order deterministic for testing
                    if app_stream_a_name > app_stream_b_name {
                        std::mem::swap(&mut app_stream_a_name, &mut app_stream_b_name);
                        std::mem::swap(
                            &mut target_physical_stream_a,
                            &mut target_physical_stream_b,
                        );
                    }
                    return Err(PlatformValidationError::ApplicationStreamWiringDifferentAppStreamsPointToSameNatsStream {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        app_stream_a_name,
                        app_stream_b_name,
                        target_physical_stream_a,
                        target_physical_stream_b,
                        explanation: "Two different NATS streams in application point to the same physical NATS stream",
                    });
                }
            }
        }
    }

    Ok(res)
}

pub fn application_deployments_s3_buckets_wiring(
    db: &Database,
) -> Result<
    HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationS3Bucket, TableRowPointerMinioBucket>,
    >,
    PlatformValidationError,
> {
    let mut s3_deployment_index: HashMap<String, TableRowPointerMinioBucket> =
        HashMap::new();
    for minio_bucket in db.minio_bucket().rows_iter() {
        let minio_cluster = db
            .minio_cluster()
            .c_cluster_name(db.minio_bucket().c_parent(minio_bucket));
        let key = format!(
            "{}=>{}",
            minio_cluster,
            db.minio_bucket().c_bucket_name(minio_bucket)
        );
        assert!(s3_deployment_index.insert(key, minio_bucket).is_none());
    }

    let mut app_s3_index: HashMap<
        TableRowPointerBackendApplication,
        HashMap<String, TableRowPointerBackendApplicationS3Bucket>,
    > = HashMap::new();
    for app_s3_bucket in db.backend_application_s3_bucket().rows_iter() {
        let e = app_s3_index
            .entry(db.backend_application_s3_bucket().c_parent(app_s3_bucket))
            .or_default();
        assert!(e
            .insert(
                db.backend_application_s3_bucket()
                    .c_bucket_name(app_s3_bucket)
                    .clone(),
                app_s3_bucket
            )
            .is_none());
    }

    let mut res: HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationS3Bucket, TableRowPointerMinioBucket>,
    > = HashMap::new();

    for app_depl in db.backend_application_deployment().rows_iter() {
        let bucket_wiring = db
            .backend_application_deployment()
            .c_s3_bucket_wiring(app_depl);
        let deployment_name = db
            .backend_application_deployment()
            .c_deployment_name(app_depl);
        let application = db
            .backend_application_deployment()
            .c_application_name(app_depl);
        let application_name = db.backend_application().c_application_name(application);

        let depl_buckets = res.entry(app_depl).or_default();

        for line in bucket_wiring
            .lines()
            .map(|i| i.trim())
            .filter(|i| !i.is_empty())
        {
            if let Some(cap) = S3_BUCKET_WIRING_EXTRACT_REGEX.captures(line) {
                let app_bucket = cap.get(1).unwrap().as_str();
                let target_bucket = cap.get(2).unwrap().as_str();

                if let Some(app) = app_s3_index.get(&application) {
                    if let Some(app_sh) = app.get(app_bucket) {
                        if let Some(s3_bucket) = s3_deployment_index.get(target_bucket) {
                            let app_s3_name =
                                db.backend_application_s3_bucket().c_bucket_name(*app_sh);

                            if depl_buckets.contains_key(app_sh) {
                                // shard defined multiple times
                                return Err(PlatformValidationError::ApplicationBucketWiringApplicationBucketDefinedMultipleTimes {
                                    application_deployment: deployment_name.clone(),
                                    application_name: application_name.clone(),
                                    bad_line: line.to_string(),
                                    redefined_app_s3_bucket_name: app_s3_name.to_string(),
                                    explanation: "Specified application S3 bucket redefined multiple times",
                                });
                            }

                            assert!(depl_buckets.insert(*app_sh, *s3_bucket).is_none());
                        } else {
                            return Err(PlatformValidationError::ApplicationBucketWiringTargetBucketDoesntExist {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                missing_minio_bucket: target_bucket.to_string(),
                                explanation: "Specified MinIO cluster with bucket doesn't exist",
                            });
                        }
                    } else {
                        // app has no bucket, error
                        let valid_app_buckets = db
                            .backend_application()
                            .c_children_backend_application_s3_bucket(application)
                            .iter()
                            .map(|i| {
                                db.backend_application_s3_bucket()
                                    .c_bucket_name(*i)
                                    .clone()
                            })
                            .collect::<Vec<_>>();
                        return Err(PlatformValidationError::ApplicationBucketsWiringApplicationHasNoBucketSpecified {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            missing_application_bucket: app_bucket.to_string(),
                            valid_app_buckets,
                            explanation: "Specified application bucket is missing",
                        });
                    }
                } else {
                    // app has no bucket, error
                    return Err(
                        PlatformValidationError::ApplicationBucketWiringApplicationHasNoBuckets {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            explanation: "This application has no S3 buckets that need to be wired",
                        },
                    );
                }
            } else {
                return Err(PlatformValidationError::ApplicationBucketWiringInvalidFormat {
                    application_deployment: deployment_name.clone(),
                    application_name: application_name.clone(),
                    bad_line: line.to_string(),
                    explanation: "Valid example of bucket wiring \"bucket_a: minio_cluster_b=>bucket_c\"",
                });
            }
        }

        for bucket in db
            .backend_application()
            .c_children_backend_application_s3_bucket(application)
        {
            if !depl_buckets.contains_key(bucket) {
                return Err(
                    PlatformValidationError::ApplicationBucketWiringUndefinedAppBucket {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        undefined_application_bucket: db
                            .backend_application_s3_bucket()
                            .c_bucket_name(*bucket)
                            .clone(),
                        explanation: "Specified application bucket was not defined in wiring",
                    },
                );
            }
        }

        let depl_buckets: &HashMap<_, _> = depl_buckets;

        for (k1, v1) in depl_buckets {
            for (k2, v2) in depl_buckets {
                if k1 != k2 && v1 == v2 {
                    // multiple different app buckets point to same physical bucket
                    // probably a mistake, avoid such situations
                    let mut app_bucket_a_name = db
                        .backend_application_s3_bucket()
                        .c_bucket_name(*k1)
                        .clone();
                    let mut app_bucket_b_name = db
                        .backend_application_s3_bucket()
                        .c_bucket_name(*k2)
                        .clone();
                    let mut target_physical_bucket_a = format!(
                        "{}=>{}",
                        db.minio_cluster()
                            .c_cluster_name(db.minio_bucket().c_parent(*v1)),
                        db.minio_bucket().c_bucket_name(*v1)
                    );
                    let mut target_physical_bucket_b = format!(
                        "{}=>{}",
                        db.minio_cluster()
                            .c_cluster_name(db.minio_bucket().c_parent(*v2)),
                        db.minio_bucket().c_bucket_name(*v2)
                    );
                    // make bucket order deterministic for testing
                    if app_bucket_a_name > app_bucket_b_name {
                        std::mem::swap(&mut app_bucket_a_name, &mut app_bucket_b_name);
                        std::mem::swap(
                            &mut target_physical_bucket_a,
                            &mut target_physical_bucket_b,
                        );
                    }
                    return Err(PlatformValidationError::ApplicationBucketWiringDifferentAppBucketsPointToSameMinioBucket {
                        application_deployment: deployment_name.clone(),
                        application_name: application_name.clone(),
                        app_bucket_a_name,
                        app_bucket_b_name,
                        target_physical_bucket_a,
                        target_physical_bucket_b,
                        explanation: "Two different MinIO buckets in application point to the same physical MinIO bucket",
                    });
                }
            }
        }
    }

    Ok(res)
}

enum AppConfigTypes {
    Int,
    Float,
    String,
    Bool,
}

impl AppConfigTypes {
    fn can_have_regex_check(&self) -> bool {
        match &self {
            AppConfigTypes::String | AppConfigTypes::Float | AppConfigTypes::Int => true,
            AppConfigTypes::Bool => false,
        }
    }

    fn can_have_min_max_check(&self) -> bool {
        match &self {
            AppConfigTypes::Float | AppConfigTypes::Int | AppConfigTypes::String => true,
            AppConfigTypes::Bool => false,
        }
    }
}

struct ParsedAppConfig {
    cfg: TableRowPointerBackendApplicationConfig,
    cfg_type: AppConfigTypes,
    min_int: Option<i64>,
    min_float: Option<f64>,
    min_string: Option<String>,
    max_int: Option<i64>,
    max_float: Option<f64>,
    max_string: Option<String>,
    regex_check: Option<::regex::Regex>,
    default_value: Option<String>,
}

fn validate_valid_config_value(cfg_type: &AppConfigTypes, input: &str) -> Result<(), String> {
    match cfg_type {
        AppConfigTypes::Bool => {
            match input {
                "true" | "false" => Ok(()),
                other => {
                    Err(format!("For bool type expected 'true' or 'false', got '{other}'"))
                }
            }
        }
        AppConfigTypes::Int => {
            let _output = input.parse::<i64>().map_err(|e| {
                format!("Failed to parse int type: {}", e.to_string())
            })?;
            Ok(())
        }
        AppConfigTypes::Float => {
            let output = input.parse::<f64>().map_err(|e| {
                format!("Failed to parse float type: {}", e.to_string())
            })?;

            let conv = output.to_string();
            if conv != input {
                return Err(format!("Float value loses precision when parsed, initial: '{input}', after parsing: {conv}"));
            }

            Ok(())
        }
        AppConfigTypes::String => Ok(())
    }
}

fn application_validate_config(
    db: &Database,
    config: TableRowPointerBackendApplicationConfig,
) -> Result<ParsedAppConfig, PlatformValidationError> {
    let cfg_type = db.backend_application_config().c_config_type(config);
    let default_value = db.backend_application_config().c_default_value(config);
    let min_value = db.backend_application_config().c_min_value(config);
    let max_value = db.backend_application_config().c_max_value(config);
    let regex_check_str = db.backend_application_config().c_regex_check(config);
    let app = db.backend_application_config().c_parent(config);
    let app_name = db.backend_application().c_application_name(app);
    let cfg_name = db.backend_application_config().c_config_name(config);
    let parsed_type = match cfg_type.as_str() {
        "string" => AppConfigTypes::String,
        "int" => AppConfigTypes::Int,
        "float" => AppConfigTypes::Float,
        "bool" => AppConfigTypes::Bool,
        _ => {
            panic!("Unexpected config type {cfg_type}")
        }
    };

    let mut regex_check = None;
    if !regex_check_str.is_empty() {
        if !parsed_type.can_have_regex_check() {
            return Err(PlatformValidationError::ApplicationConfigTypeCannotHaveRegexCheck {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_regex_check: regex_check_str.clone(),
                application_config_regex_check_only_allowed_value: "".to_string(),
            });
        }

        let regex = ::regex::Regex::new(&regex_check_str).map_err(|e| {
            return PlatformValidationError::ApplicationConfigInvalidRegexCheck {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_regex_check: regex_check_str.clone(),
                regex_compilation_error: e.to_string(),
            };
        })?;

        regex_check = Some(regex);
    }

    let mut min_float: Option<f64> = None;
    let mut max_float: Option<f64> = None;
    let mut min_int: Option<i64> = None;
    let mut max_int: Option<i64> = None;
    let mut min_string: Option<String> = None;
    let mut max_string: Option<String> = None;

    if !min_value.is_empty() {
        if !parsed_type.can_have_min_max_check() {
            return Err(PlatformValidationError::ApplicationConfigTypeCannotHaveMinCheck {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_min_value: min_value.clone(),
            });
        }

        if let Err(parsing_err) = validate_valid_config_value(&parsed_type, min_value.as_str()) {
            return Err(PlatformValidationError::ApplicationConfigTypeCannotParseMinValue {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_min_value: min_value.clone(),
                application_config_min_value_parsing_error: parsing_err,
            });
        }

        match &parsed_type {
            AppConfigTypes::Int => {
                min_int = Some(min_value.parse::<i64>().unwrap());
            }
            AppConfigTypes::Float => {
                min_float = Some(min_value.parse::<f64>().unwrap());
            }
            AppConfigTypes::String => {
                min_string = Some(min_value.clone());
            }
            _ => panic!("Should never be reached"),
        }
    }

    if !max_value.is_empty() {
        if !parsed_type.can_have_min_max_check() {
            return Err(PlatformValidationError::ApplicationConfigTypeCannotHaveMaxCheck {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_max_value: max_value.clone(),
            });
        }

        if let Err(parsing_err) = validate_valid_config_value(&parsed_type, max_value.as_str()) {
            return Err(PlatformValidationError::ApplicationConfigTypeCannotParseMaxValue {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_max_value: max_value.clone(),
                application_config_max_value_parsing_error: parsing_err,
            });
        }

        match &parsed_type {
            AppConfigTypes::Int => {
                max_int = Some(max_value.parse::<i64>().unwrap());
            }
            AppConfigTypes::Float => {
                max_float = Some(max_value.parse::<f64>().unwrap());
            }
            AppConfigTypes::String => {
                max_string = Some(max_value.clone());
            }
            _ => panic!("Should never be reached"),
        }
    }

    if let (Some(min_int), Some(max_int)) = (&min_int, &max_int) {
        if min_int >= max_int {
            return Err(PlatformValidationError::ApplicationConfigTypeMinValueMustBeLessThanMaxValue {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_min_value: min_value.clone(),
                application_config_max_value: max_value.clone(),
            });
        }
    }

    if let (Some(min_float), Some(max_float)) = (&min_float, &max_float) {
        if min_float >= max_float {
            return Err(PlatformValidationError::ApplicationConfigTypeMinValueMustBeLessThanMaxValue {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_min_value: min_value.clone(),
                application_config_max_value: max_value.clone(),
            });
        }
    }

    if let (Some(min_string), Some(max_string)) = (&min_string, &max_string) {
        if min_string >= max_string {
            return Err(PlatformValidationError::ApplicationConfigTypeMinValueMustBeLessThanMaxValue {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_min_value: min_value.clone(),
                application_config_max_value: max_value.clone(),
            });
        }
    }

    let mut res =
        ParsedAppConfig {
            cfg: config,
            cfg_type: parsed_type,
            regex_check,
            min_int,
            max_int,
            min_float,
            max_float,
            min_string,
            max_string,
            default_value: None,
        };

    if !default_value.is_empty() {
        if let Err(e) = application_validate_deployment_config(&res, default_value.as_str()) {
            return Err(PlatformValidationError::ApplicationConfigInvalidDefaultValue {
                application_name: app_name.clone(),
                application_config: cfg_name.clone(),
                application_config_type: cfg_type.clone(),
                application_config_default_value: default_value.clone(),
                application_config_default_value_error: e,
            });
        }

        res.default_value = Some(default_value.clone());
    }

    Ok(res)
}

fn application_validate_deployment_config(
    parsed: &ParsedAppConfig,
    value: &str
) -> Result<(), String> {
    validate_valid_config_value(&parsed.cfg_type, value)?;

    if let Some(regex_check) = &parsed.regex_check {
        if !regex_check.is_match(value) {
            return Err(format!("Value '{value}' doesn't match regex check of '{regex_check}'"));
        }
    }

    if let Some(min_int) = &parsed.min_int {
        let parsed = value.parse::<i64>().unwrap();
        if parsed < *min_int {
            return Err(format!("Value '{value}' is less than minimum config value of '{min_int}'"));
        }
    }

    if let Some(max_int) = &parsed.max_int {
        let parsed = value.parse::<i64>().unwrap();
        if parsed > *max_int {
            return Err(format!("Value '{value}' is more than maximum config value of '{max_int}'"));
        }
    }

    if let Some(min_float) = &parsed.min_float {
        let parsed = value.parse::<f64>().unwrap();
        if parsed < *min_float {
            return Err(format!("Value '{value}' is less than minimum config value of '{min_float}'"));
        }
    }

    if let Some(max_float) = &parsed.max_float {
        let parsed = value.parse::<f64>().unwrap();
        if parsed > *max_float {
            return Err(format!("Value '{value}' is more than maximum config value of '{max_float}'"));
        }
    }

    if let Some(min_string) = &parsed.min_string {
        if value < min_string.as_str() {
            return Err(format!("Value '{value}' is lexicographically less than minimum config value of '{min_string}'"));
        }
    }

    if let Some(max_string) = &parsed.max_string {
        if value > max_string.as_str() {
            return Err(format!("Value '{value}' is lexicographically more than maximum config value of '{max_string}'"));
        }
    }

    Ok(())
}

pub fn application_deployments_config(
    db: &Database,
) -> Result<
    HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationConfig, String>,
    >,
    PlatformValidationError,
> {
    let mut res: HashMap<
        TableRowPointerBackendApplicationDeployment,
        HashMap<TableRowPointerBackendApplicationConfig, String>,
    > = HashMap::new();

    let mut app_config_index: HashMap<
        TableRowPointerBackendApplication,
        HashMap<String, ParsedAppConfig>,
    > = HashMap::new();

    for config in db.backend_application_config().rows_iter() {
        let validated_config = application_validate_config(db, config)?;
        let e = app_config_index
            .entry(db.backend_application_config().c_parent(config))
            .or_default();
        assert!(e
            .insert(
                db.backend_application_config()
                    .c_config_name(config)
                    .clone(),
                validated_config
            )
            .is_none());
    }

    for app_depl in db.backend_application_deployment().rows_iter() {
        let raw_configs = db
            .backend_application_deployment()
            .c_config(app_depl);
        let deployment_name = db
            .backend_application_deployment()
            .c_deployment_name(app_depl);
        let application = db
            .backend_application_deployment()
            .c_application_name(app_depl);
        let application_name = db.backend_application().c_application_name(application);

        let depl_configs = res.entry(app_depl).or_default();

        for line in raw_configs
            .lines()
            .map(|i| i.trim())
            .filter(|i| !i.is_empty())
        {
            if let Some(cap) = CONFIG_EXTRACT_REGEX.captures(line) {
                let config_key = cap.get(1).unwrap().as_str();
                let config_value = cap.get(2).unwrap().as_str();

                if let Some(app) = app_config_index.get(&application) {
                    if let Some(app_sh) = app.get(config_key) {
                        let config_ptr = app_sh.cfg;
                        if depl_configs.contains_key(&config_ptr) {
                            // shard defined multiple times
                            return Err(PlatformValidationError::ApplicationConfigDefinedMultipleTimesForDeployment {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                redefined_app_config: config_key.to_string(),
                                explanation: "Specified application config redefined multiple times",
                            });
                        }

                        if let Err(e) = application_validate_deployment_config(app_sh, config_value) {
                            return Err(PlatformValidationError::ApplicationConfigInvalidValue {
                                application_deployment: deployment_name.clone(),
                                application_name: application_name.clone(),
                                bad_line: line.to_string(),
                                bad_app_config: config_key.to_string(),
                                bad_app_config_type: db.backend_application_config().c_config_type(config_ptr).clone(),
                                error: e,
                            });
                        }

                        assert!(depl_configs.insert(config_ptr, config_value.to_string()).is_none());
                    } else {
                        // app has no config, error
                        let valid_app_configs = db
                            .backend_application()
                            .c_children_backend_application_config(application)
                            .iter()
                            .map(|i| {
                                db.backend_application_config()
                                    .c_config_name(*i)
                                    .clone()
                            })
                            .collect::<Vec<_>>();
                        return Err(PlatformValidationError::ApplicationConfigDoesntExist {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            missing_application_config: config_key.to_string(),
                            valid_app_configs,
                            explanation: "Specified application config is missing",
                        });
                    }
                } else {
                    // app has no config, error
                    return Err(
                        PlatformValidationError::ApplicationDoesntHaveAnyConfigs {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            bad_line: line.to_string(),
                            explanation: "This application has no configurations that need to be specified",
                        },
                    );
                }
            } else {
                return Err(PlatformValidationError::ApplicationConfigInvalidFormat {
                    application_deployment: deployment_name.clone(),
                    application_name: application_name.clone(),
                    bad_line: line.to_string(),
                    explanation: "Valid example of application config \"conf_a: abc\"",
                });
            }
        }

        for config in db
            .backend_application()
            .c_children_backend_application_config(application)
        {
            if !depl_configs.contains_key(config) {
                let index = app_config_index.entry(application).or_default();
                let parsed_cfg = index.get(db.backend_application_config().c_config_name(*config)).unwrap();

                if let Some(def_val) = &parsed_cfg.default_value {
                    assert!(depl_configs.insert(*config, def_val.clone()).is_none());
                } else {
                    return Err(
                        PlatformValidationError::ApplicationConfigUndefinedAppConfig {
                            application_deployment: deployment_name.clone(),
                            application_name: application_name.clone(),
                            undefined_application_config: db
                                .backend_application_config()
                                .c_config_name(*config)
                                .clone(),
                            explanation: "Specified application config was not defined and doesn't have default value",
                        },
                    );
                }
            }
        }
    }

    Ok(res)
}

pub fn check_frontend_application_endpoint_types(
    db: &Database,
) -> Result<(), PlatformValidationError> {
    for fe in db.frontend_application_used_endpoint().rows_iter() {
        let frontend_app = db.frontend_application_used_endpoint().c_parent(fe);
        let backend_endpoint = db
            .frontend_application_used_endpoint()
            .c_backend_endpoint(fe);
        let be_type = db.backend_http_endpoint().c_data_type(backend_endpoint);
        const ALLOWED_TYPES: &[&str] = &["json"];
        let this_type = db
            .http_endpoint_data_type()
            .c_http_endpoint_data_type(be_type)
            .as_str();
        if !ALLOWED_TYPES.contains(&this_type) {
            return Err(
                PlatformValidationError::FrontendApplicationEndpointDisallowedDataType {
                    frontend_application_name: db
                        .frontend_application()
                        .c_application_name(frontend_app)
                        .clone(),
                    backend_application_name: db
                        .backend_application()
                        .c_application_name(db.backend_http_endpoint().c_parent(backend_endpoint))
                        .clone(),
                    backend_endpoint_name: db
                        .backend_http_endpoint()
                        .c_http_endpoint_name(backend_endpoint)
                        .clone(),
                    backend_endpoint_data_type: this_type.to_string(),
                    backend_endpoint_allowed_types: ALLOWED_TYPES
                        .iter()
                        .map(|i| i.to_string())
                        .collect(),
                    backend_endpoint_path: db
                        .backend_http_endpoint()
                        .c_path(backend_endpoint)
                        .clone(),
                },
            );
        }
    }

    Ok(())
}

pub fn frontend_deployments_endpoint_wirings(
    db: &Database,
    backend_ingress_endpoints: &Projection<
        TableRowPointerBackendApplicationDeploymentIngress,
        BTreeSet<TableRowPointerBackendHttpEndpoint>,
    >,
) -> Result<
    HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationUsedEndpoint,
            TableRowPointerBackendApplicationDeploymentIngress,
        >,
    >,
    PlatformValidationError,
> {
    let mut res: HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationUsedEndpoint,
            TableRowPointerBackendApplicationDeploymentIngress,
        >,
    > = HashMap::new();

    let mut depl_index: HashMap<String, TableRowPointerBackendApplicationDeployment> =
        HashMap::new();
    for depl in db.backend_application_deployment().rows_iter() {
        assert!(depl_index
            .insert(
                db.backend_application_deployment()
                    .c_deployment_name(depl)
                    .clone(),
                depl
            )
            .is_none());
    }

    for depl in db.frontend_application_deployment().rows_iter() {
        let deployment_name = db.frontend_application_deployment().c_deployment_name(depl);
        let frontend_app = db
            .frontend_application_deployment()
            .c_application_name(depl);
        let application_name = db.frontend_application().c_application_name(frontend_app);
        let deployment_ingress_exists = !db
            .frontend_application_deployment()
            .c_referrers_frontend_application_deployment_ingress__deployment(depl)
            .is_empty();
        let frontend_ingress = db
            .frontend_application_deployment()
            .c_referrers_frontend_application_deployment_ingress__deployment(depl);
        let ingress_count = frontend_ingress.len();
        let wiring_src = db
            .frontend_application_deployment()
            .c_explicit_endpoint_wiring(depl);
        // if frontend app has any used endpoints this frontend app must be served at an ingress
        // the reason this is not allowed because this frontend app itself must
        // be served at an ingress point, say / and due to strict CORS rules (we prefer security)
        // it is only allowed to access APIs in its CORS domain. Hence, if frontend deployment has no ingress,
        // its application uses some APIs, it will never be able to access any endpoint.
        if !deployment_ingress_exists {
            return Err(PlatformValidationError::FrontendApplicationDeploymentHasNoIngress {
                deployment_name: deployment_name.clone(),
                application_name: application_name.clone(),
                ingress_count,
            });
        }

        assert_eq!(ingress_count, 1);
        let frontend_ingress = frontend_ingress[0];
        let mut frontend_api_wiring = parse_frontend_endpoint_wiring(
            db,
            wiring_src.as_str(),
            frontend_ingress,
            frontend_app,
            application_name,
            deployment_name,
            &depl_index,
        )?;
        let frontend_tld = db.tld().c_domain(
            db.frontend_application_deployment_ingress()
                .c_tld(frontend_ingress),
        );
        let frontend_subdomain = db
            .frontend_application_deployment_ingress()
            .c_subdomain(frontend_ingress);

        for used_endpoint in db
            .frontend_application()
            .c_children_frontend_application_used_endpoint(frontend_app)
        {
            if frontend_api_wiring.contains_key(used_endpoint) {
                // already specified in explicit wiring
                continue;
            }

            // wiring not specified, infer
            let used_backend_endpoint = db
                .frontend_application_used_endpoint()
                .c_backend_endpoint(*used_endpoint);
            let used_backend_app = db.backend_http_endpoint().c_parent(used_backend_endpoint);
            let mut found_ingresses = Vec::new();
            for backend_ingress in db.backend_application_deployment_ingress().rows_iter() {
                let ingress_backend_deployment = db
                    .backend_application_deployment_ingress()
                    .c_deployment(backend_ingress);
                let ingress_backend_app = db
                    .backend_application_deployment()
                    .c_application_name(ingress_backend_deployment);
                if used_backend_app == ingress_backend_app {
                    let same_tld = db
                        .frontend_application_deployment_ingress()
                        .c_tld(frontend_ingress)
                        == db
                            .backend_application_deployment_ingress()
                            .c_tld(backend_ingress);
                    let same_subdomain = db
                        .frontend_application_deployment_ingress()
                        .c_subdomain(frontend_ingress)
                        == db
                            .backend_application_deployment_ingress()
                            .c_subdomain(backend_ingress);
                    if same_tld && same_subdomain {
                        // We have potential to find route here
                        let endpoints = backend_ingress_endpoints.value(backend_ingress);
                        for endpoint in endpoints {
                            if *endpoint == used_backend_endpoint {
                                found_ingresses.push(backend_ingress);
                            }
                        }
                    }
                }
            }

            if found_ingresses.is_empty() {
                // error about no ingress found for endpoint
                return Err(
                    PlatformValidationError::FrontendApplicationUsedEndpointNotDeployed {
                        application_name: application_name.to_string(),
                        deployment_name: deployment_name.to_string(),
                        missing_endpoint_name: db
                            .frontend_application_used_endpoint()
                            .c_endpoint_name(*used_endpoint)
                            .clone(),
                        missing_endpoint_backend_name: db
                            .backend_application()
                            .c_application_name(used_backend_app)
                            .clone(),
                        missing_endpoint_backend_signature: db
                            .backend_http_endpoint()
                            .c_path(used_backend_endpoint)
                            .clone(),
                    },
                );
            } else if found_ingresses.len() != 1 {
                // error about ambigous ingresses
                // TODO: check in wiring if exists
                let matching_ingress_deployment_names = found_ingresses
                    .iter()
                    .map(|i| {
                        db.backend_application_deployment()
                            .c_deployment_name(
                                db.backend_application_deployment_ingress().c_deployment(*i),
                            )
                            .clone()
                    })
                    .collect::<Vec<_>>();
                let matching_ingress_mountpoints = found_ingresses
                    .iter()
                    .map(|i| {
                        db.backend_application_deployment_ingress()
                            .c_mountpoint(*i)
                            .clone()
                    })
                    .collect::<Vec<_>>();
                return Err(
                    PlatformValidationError::FrontendApplicationUsedEndpointAmbigiousIngress {
                        application_name: application_name.to_string(),
                        deployment_name: deployment_name.to_string(),
                        ambigous_endpoint_name: db
                            .frontend_application_used_endpoint()
                            .c_endpoint_name(*used_endpoint)
                            .clone(),
                        ambigous_endpoint_backend_name: db
                            .backend_application()
                            .c_application_name(used_backend_app)
                            .clone(),
                        ambigous_endpoint_backend_signature: db
                            .backend_http_endpoint()
                            .c_path(used_backend_endpoint)
                            .clone(),
                        matching_ingress_deployment_names,
                        matching_ingress_mountpoints,
                        tld: frontend_tld.clone(),
                        subdomain: frontend_subdomain.clone(),
                    },
                );
            } else {
                assert_eq!(found_ingresses.len(), 1);
                let target = found_ingresses[0];
                assert!(frontend_api_wiring.insert(*used_endpoint, target).is_none());
            }
        }

        let depl_urls = res.entry(depl).or_default();
        for (k, v) in &frontend_api_wiring {
            assert!(depl_urls.insert(*k, *v).is_none());
        }
    }

    Ok(res)
}

fn parse_frontend_endpoint_wiring(
    db: &Database,
    src: &str,
    frontend_ingress: TableRowPointerFrontendApplicationDeploymentIngress,
    frontend_app: TableRowPointerFrontendApplication,
    application_name: &str,
    deployment_name: &str,
    depl_index: &HashMap<String, TableRowPointerBackendApplicationDeployment>,
) -> Result<
    HashMap<
        TableRowPointerFrontendApplicationUsedEndpoint,
        TableRowPointerBackendApplicationDeploymentIngress,
    >,
    PlatformValidationError,
> {
    let mut res = HashMap::new();

    let mut used_endpoints = BTreeMap::new();
    for ue in db
        .frontend_application()
        .c_children_frontend_application_used_endpoint(frontend_app)
    {
        assert!(used_endpoints
            .insert(
                db.frontend_application_used_endpoint()
                    .c_endpoint_name(*ue)
                    .clone(),
                *ue
            )
            .is_none());
    }

    let mut endpoints_defined = BTreeSet::new();

    for line in src.lines().map(|i| i.trim()).filter(|i| !i.is_empty()) {
        if let Some(cap) = EXPLICIT_FRONTEND_WIRING_REGEX.captures(line) {
            let app_endpoint = cap.get(1).unwrap().as_str();
            let target_deployment_str = cap.get(2).unwrap().as_str();

            let used_endpoint = match used_endpoints.get(app_endpoint) {
                None => {
                    return Err(PlatformValidationError::FrontendApplicationEndpointWiringNonExistingUsedEndpoint {
                        application_name: application_name.to_string(),
                        deployment_name: deployment_name.to_string(),
                        endpoint_wiring: src.to_string(),
                        non_existing_endpoint: app_endpoint.to_string(),
                        valid_candidate_endpoints: used_endpoints.into_keys().collect(),
                    });
                }
                Some(e) => e,
            };

            if !endpoints_defined.insert(*used_endpoint) {
                return Err(PlatformValidationError::FrontendApplicationEndpointWiringEndpointDefinedTwice {
                    application_name: application_name.to_string(),
                    deployment_name: deployment_name.to_string(),
                    endpoint_wiring: src.to_string(),
                    endpoint_defined_twice: app_endpoint.to_string(),
                });
            }

            let target_deployment = match depl_index.get(target_deployment_str) {
                None => {
                    return Err(PlatformValidationError::FrontendApplicationEndpointWiringNonExistingBackendDeployment {
                        application_name: application_name.to_string(),
                        deployment_name: deployment_name.to_string(),
                        endpoint_wiring: src.to_string(),
                        non_existing_deployment: target_deployment_str.to_string(),
                    });
                }
                Some(e) => e,
            };

            let target_backend_app = db
                .backend_application_deployment()
                .c_application_name(*target_deployment);

            let expected_backend_endpoint = db
                .frontend_application_used_endpoint()
                .c_backend_endpoint(*used_endpoint);
            let expected_backend_app = db
                .backend_http_endpoint()
                .c_parent(expected_backend_endpoint);
            if target_backend_app != expected_backend_app {
                return Err(PlatformValidationError::FrontendApplicationEndpointWiringIncorrectTargetApplication {
                    application_name: application_name.to_string(),
                    deployment_name: deployment_name.to_string(),
                    expected_backend_application: db.backend_application().c_application_name(expected_backend_app).clone(),
                    actual_backend_application: db.backend_application().c_application_name(target_backend_app).clone(),
                    endpoint_wiring_line: line.to_string(),
                    endpoint_wiring: src.to_string(),
                });
            }

            let ingresses = db
                .backend_application_deployment()
                .c_referrers_backend_application_deployment_ingress__deployment(*target_deployment);
            if ingresses.is_empty() {
                return Err(PlatformValidationError::FrontendApplicationEndpointWiringTargetDeploymentHasNoIngress {
                    application_name: application_name.to_string(),
                    deployment_name: deployment_name.to_string(),
                    backend_application_deployment_without_ingress: target_deployment_str.to_string(),
                    endpoint_wiring_line: line.to_string(),
                    endpoint_wiring: src.to_string(),
                });
            }

            let mut found_backend_ingress = false;
            for backend_ingress in ingresses {
                // must be same TLD
                // must be same subdomain
                let same_tld = db
                    .frontend_application_deployment_ingress()
                    .c_tld(frontend_ingress)
                    == db
                        .backend_application_deployment_ingress()
                        .c_tld(*backend_ingress);
                let same_subdomain = db
                    .frontend_application_deployment_ingress()
                    .c_subdomain(frontend_ingress)
                    == db
                        .backend_application_deployment_ingress()
                        .c_subdomain(*backend_ingress);

                if same_tld && same_subdomain {
                    found_backend_ingress = true;
                    assert!(res.insert(*used_endpoint, *backend_ingress).is_none());
                    break;
                }
            }

            if !found_backend_ingress {
                return Err(PlatformValidationError::FrontendApplicationEndpointWiringCantFindCompatibleIngress {
                    application_name: application_name.to_string(),
                    deployment_name: deployment_name.to_string(),
                    explanation: "Can't find reachable backend endpoint from frontend application. Make sure frontend app is on the same subdomain and TLD not to violate CORS constraints.".to_string(),
                    endpoint_wiring_line: line.to_string(),
                    endpoint_wiring: src.to_string(),
                });
            }
            // find matching ingress for deployment?
        } else {
            return Err(PlatformValidationError::FrontendApplicationEndpointWiringInvalidLine {
                application_name: application_name.to_string(),
                deployment_name: deployment_name.to_string(),
                invalid_line: line.to_string(),
                endpoint_wiring: src.to_string(),
                explanation: "Example wiring 'frontend_endpoint: backend_deployment_ingress=>backend_endpoint_name'".to_string(),
            });
        }
    }

    Ok(res)
}

pub fn check_frontend_application_page_wirings(
    db: &Database,
) -> Result<
    HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationExternalPage,
            TableRowPointerFrontendApplicationDeploymentIngress,
        >,
    >,
    PlatformValidationError,
> {
    let mut res: HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationExternalPage,
            TableRowPointerFrontendApplicationDeploymentIngress,
        >,
    > = HashMap::new();

    let mut endpoint_map: HashMap<
        TableRowPointerFrontendApplication,
        HashMap<String, TableRowPointerFrontendApplicationExternalPage>,
    > = HashMap::new();

    for app in db.frontend_application().rows_iter() {
        let e = endpoint_map.entry(app).or_default();
        for ep in db
            .frontend_application()
            .c_children_frontend_application_external_page(app)
        {
            assert!(e
                .insert(
                    db.frontend_application_external_page()
                        .c_link_name(*ep)
                        .clone(),
                    *ep
                )
                .is_none());
        }
    }

    let mut deployment_map: HashMap<String, TableRowPointerFrontendApplicationDeployment> =
        HashMap::new();

    for depl in db.frontend_application_deployment().rows_iter() {
        assert!(deployment_map
            .insert(
                db.frontend_application_deployment()
                    .c_deployment_name(depl)
                    .clone(),
                depl
            )
            .is_none());
    }

    for depl in db.frontend_application_deployment().rows_iter() {
        let e_map: &mut _ = res.entry(depl).or_default();
        let frontend_app = db
            .frontend_application_deployment()
            .c_application_name(depl);
        let src = db.frontend_application_deployment().c_page_wiring(depl);

        let valid_pages = || {
            db.frontend_application()
                .c_children_frontend_application_external_page(frontend_app)
                .iter()
                .map(|i| {
                    db.frontend_application_external_page()
                        .c_link_name(*i)
                        .clone()
                })
                .collect::<Vec<_>>()
        };

        // point to the frontend deployment.
        // 1. it must have ingress
        // 2. deployment must have expected frontend app type
        // 3. it must always be defined in wirings because our infer space is too big
        // 4. wiring can't be defined for non existing endpoint
        // 5. wiring cannot be defined twice
        // 6. Cannot point to the same deployment!! Otherwise, we'd just use Rest functions

        for line in src.lines().map(|i| i.trim()).filter(|i| !i.is_empty()) {
            if let Some(cap) = EXPLICIT_FRONTEND_WIRING_REGEX.captures(line) {
                let app_endpoint = cap.get(1).unwrap().as_str();
                let target_deployment_str = cap.get(2).unwrap().as_str();

                let endpoints = endpoint_map.get(&frontend_app).unwrap();
                let endpoint = match endpoints.get(app_endpoint) {
                    Some(endpoint) => endpoint,
                    None => {
                        return Err(
                            PlatformValidationError::FrontendApplicationPageWiringUnknownPage {
                                application_name: db
                                    .frontend_application()
                                    .c_application_name(frontend_app)
                                    .clone(),
                                deployment_name: db
                                    .frontend_application_deployment()
                                    .c_deployment_name(depl)
                                    .clone(),
                                invalid_line: line.to_string(),
                                page_wiring: src.clone(),
                                non_existing_page: app_endpoint.to_string(),
                                valid_link_pages: valid_pages(),
                            },
                        );
                    }
                };
                let target_frontend_page = db
                    .frontend_application_external_page()
                    .c_frontend_page(*endpoint);
                let target_frontend_app = db.frontend_page().c_parent(target_frontend_page);

                let target_deployment = match deployment_map.get(target_deployment_str) {
                    Some(depl) => depl,
                    None => {
                        return Err(PlatformValidationError::FrontendApplicationPageWiringUnknownFrontendDeployment {
                            application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                            deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                            invalid_line: line.to_string(),
                            page_wiring: src.clone(),
                            non_existing_frontend_deployment: target_deployment_str.to_string(),
                        });
                    }
                };

                if depl == *target_deployment {
                    return Err(PlatformValidationError::FrontendApplicationPageWiringPointsToTheSameDeployment {
                        application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                        deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                        invalid_line: line.to_string(),
                        page_wiring: src.clone(),
                        same_frontend_deployment: target_deployment_str.to_string(),
                    });
                }

                let deployment_frontend_app = db
                    .frontend_application_deployment()
                    .c_application_name(*target_deployment);

                if deployment_frontend_app != target_frontend_app {
                    return Err(PlatformValidationError::FrontendApplicationPageWiringPointsToUnexpectedFrontendApp {
                        application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                        deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                        actual_frontend_app: db.frontend_application().c_application_name(deployment_frontend_app).clone(),
                        expected_frontend_app: db.frontend_application().c_application_name(target_frontend_app).clone(),
                        invalid_line: line.to_string(),
                        page_wiring: src.clone(),
                    });
                }

                let ingresses = db
                    .frontend_application_deployment()
                    .c_referrers_frontend_application_deployment_ingress__deployment(
                        *target_deployment,
                    );
                if ingresses.is_empty() {
                    return Err(PlatformValidationError::FrontendApplicationPageWiringTargetDeploymentHasNoIngress {
                        application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                        deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                        frontend_application_deployment_without_ingress: db.frontend_application_deployment().c_deployment_name(*target_deployment).clone(),
                        invalid_line: line.to_string(),
                        page_wiring: src.clone(),
                    });
                }

                if e_map.contains_key(endpoint) {
                    return Err(PlatformValidationError::FrontendApplicationPageWiringLinkDefinedMultipleTimes {
                        application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                        deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                        page_defined_multiple_times: app_endpoint.to_string(),
                        invalid_line: line.to_string(),
                        page_wiring: src.clone(),
                    });
                }

                assert!(!ingresses.is_empty());
                assert!(e_map.insert(*endpoint, ingresses[0]).is_none());
            } else {
                return Err(
                    PlatformValidationError::FrontendApplicationPageWiringInvalidLine {
                        application_name: db
                            .frontend_application()
                            .c_application_name(frontend_app)
                            .clone(),
                        deployment_name: db
                            .frontend_application_deployment()
                            .c_deployment_name(depl)
                            .clone(),
                        invalid_line: line.to_string(),
                        page_wiring: src.clone(),
                        explanation: "Valid wiring example: `page_name: frontend_deployment`"
                            .to_string(),
                    },
                );
            }
        }

        // go through to see if some is left undefined
        for ep in db
            .frontend_application()
            .c_children_frontend_application_external_page(frontend_app)
        {
            if !e_map.contains_key(ep) {
                return Err(
                    PlatformValidationError::FrontendApplicationPageWiringExternalPageUndefined {
                        application_name: db
                            .frontend_application()
                            .c_application_name(frontend_app)
                            .clone(),
                        deployment_name: db
                            .frontend_application_deployment()
                            .c_deployment_name(depl)
                            .clone(),
                        page_wiring: src.clone(),
                        undefined_page: db
                            .frontend_application_external_page()
                            .c_link_name(*ep)
                            .clone(),
                    },
                );
            }
        }
    }

    Ok(res)
}

pub fn check_frontend_application_link_wirings(
    db: &Database,
) -> Result<
    HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationExternalLink,
            TableRowPointerBackendApplicationDeploymentIngress,
        >,
    >,
    PlatformValidationError,
> {
    let mut res: HashMap<
        TableRowPointerFrontendApplicationDeployment,
        HashMap<
            TableRowPointerFrontendApplicationExternalLink,
            TableRowPointerBackendApplicationDeploymentIngress,
        >,
    > = HashMap::new();

    let mut endpoint_map: HashMap<
        TableRowPointerFrontendApplication,
        HashMap<String, TableRowPointerFrontendApplicationExternalLink>,
    > = HashMap::new();

    for app in db.frontend_application().rows_iter() {
        let e = endpoint_map.entry(app).or_default();
        for ep in db
            .frontend_application()
            .c_children_frontend_application_external_link(app)
        {
            let backend_endpoint = db
                .frontend_application_external_link()
                .c_backend_endpoint(*ep);
            let endpoint_type = db.backend_http_endpoint().c_data_type(backend_endpoint);
            let data_type_name = db
                .http_endpoint_data_type()
                .c_http_endpoint_data_type(endpoint_type);
            // if it is REST endpoint just do REST request,
            // only reasonable target is external html page
            // possibly in different CORS domain
            let allowed_endpoint_types = ["html"];
            if !allowed_endpoint_types.contains(&data_type_name.as_str()) {
                return Err(
                    PlatformValidationError::FrontendApplicationLinkWiringBadEndpointDataType {
                        application_name: db.frontend_application().c_application_name(app).clone(),
                        bad_endpoint: db
                            .frontend_application_external_link()
                            .c_link_name(*ep)
                            .clone(),
                        bad_endpoint_data_type: data_type_name.clone(),
                        allowed_endpoint_types: allowed_endpoint_types
                            .iter()
                            .map(|i| i.to_string())
                            .collect(),
                    },
                );
            }
            assert!(
                db.backend_http_endpoint()
                    .c_input_body_type(backend_endpoint)
                    .is_empty(),
                "Post body can't be expressed in URL and this case should never be allowed"
            );
            assert!(e
                .insert(
                    db.frontend_application_external_link()
                        .c_link_name(*ep)
                        .clone(),
                    *ep
                )
                .is_none());
        }
    }

    let mut deployment_map: HashMap<String, TableRowPointerBackendApplicationDeployment> =
        HashMap::new();

    for depl in db.backend_application_deployment().rows_iter() {
        assert!(deployment_map
            .insert(
                db.backend_application_deployment()
                    .c_deployment_name(depl)
                    .clone(),
                depl
            )
            .is_none());
    }

    for depl in db.frontend_application_deployment().rows_iter() {
        let e_map: &mut _ = res.entry(depl).or_default();
        let frontend_app = db
            .frontend_application_deployment()
            .c_application_name(depl);
        let src = db.frontend_application_deployment().c_link_wiring(depl);

        let valid_endpoints = || {
            db.frontend_application()
                .c_children_frontend_application_external_link(frontend_app)
                .iter()
                .map(|i| {
                    db.frontend_application_external_link()
                        .c_link_name(*i)
                        .clone()
                })
                .collect::<Vec<_>>()
        };

        // point to the backend deployment.
        // 1. it must have ingress
        // 2. deployment must have expected backend app type
        // 3. it must always be defined in wirings because our infer space is too big
        // 4. wiring can't be defined for non existing endpoint
        // 5. wiring cannot be defined twice

        for line in src.lines().map(|i| i.trim()).filter(|i| !i.is_empty()) {
            if let Some(cap) = EXPLICIT_FRONTEND_WIRING_REGEX.captures(line) {
                let app_endpoint = cap.get(1).unwrap().as_str();
                let target_deployment_str = cap.get(2).unwrap().as_str();

                let endpoints = endpoint_map.get(&frontend_app).unwrap();
                let endpoint = match endpoints.get(app_endpoint) {
                    Some(endpoint) => endpoint,
                    None => {
                        return Err(
                            PlatformValidationError::FrontendApplicationLinkWiringUnknownEndpoint {
                                application_name: db
                                    .frontend_application()
                                    .c_application_name(frontend_app)
                                    .clone(),
                                deployment_name: db
                                    .frontend_application_deployment()
                                    .c_deployment_name(depl)
                                    .clone(),
                                invalid_line: line.to_string(),
                                link_wiring: src.clone(),
                                non_existing_endpoint: app_endpoint.to_string(),
                                valid_endpoints: valid_endpoints(),
                            },
                        );
                    }
                };
                let target_http_endpoint = db
                    .frontend_application_external_link()
                    .c_backend_endpoint(*endpoint);
                let target_backend_app = db.backend_http_endpoint().c_parent(target_http_endpoint);

                let target_deployment = match deployment_map.get(target_deployment_str) {
                    Some(depl) => depl,
                    None => {
                        return Err(PlatformValidationError::FrontendApplicationLinkWiringUnknownBackendDeployment {
                            application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                            deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                            invalid_line: line.to_string(),
                            link_wiring: src.clone(),
                            non_existing_backend_deployment: target_deployment_str.to_string(),
                        });
                    }
                };

                let deployment_backend_app = db
                    .backend_application_deployment()
                    .c_application_name(*target_deployment);

                if deployment_backend_app != target_backend_app {
                    return Err(PlatformValidationError::FrontendApplicationLinkWiringPointsToUnexpectedBackendApp {
                        application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                        deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                        actual_backend_app: db.backend_application().c_application_name(deployment_backend_app).clone(),
                        expected_backend_app: db.backend_application().c_application_name(target_backend_app).clone(),
                        invalid_line: line.to_string(),
                        link_wiring: src.clone(),
                    });
                }

                let ingresses = db
                    .backend_application_deployment()
                    .c_referrers_backend_application_deployment_ingress__deployment(
                        *target_deployment,
                    );
                if ingresses.is_empty() {
                    return Err(PlatformValidationError::FrontendApplicationLinkWiringBackendDeploymentHasNoIngress {
                        application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                        deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                        backend_application_deployment_without_ingress: db.backend_application_deployment().c_deployment_name(*target_deployment).clone(),
                        invalid_line: line.to_string(),
                        link_wiring: src.clone(),
                    });
                }

                if e_map.contains_key(endpoint) {
                    return Err(PlatformValidationError::FrontendApplicationLinkWiringLinkDefinedMultipleTimes {
                        application_name: db.frontend_application().c_application_name(frontend_app).clone(),
                        deployment_name: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                        link_defined_multiple_times: app_endpoint.to_string(),
                        invalid_line: line.to_string(),
                        link_wiring: src.clone(),
                    });
                }

                assert!(!ingresses.is_empty());
                assert!(e_map.insert(*endpoint, ingresses[0]).is_none());
            } else {
                return Err(
                    PlatformValidationError::FrontendApplicationLinkWiringInvalidLine {
                        application_name: db
                            .frontend_application()
                            .c_application_name(frontend_app)
                            .clone(),
                        deployment_name: db
                            .frontend_application_deployment()
                            .c_deployment_name(depl)
                            .clone(),
                        invalid_line: line.to_string(),
                        link_wiring: src.clone(),
                        explanation: "Valid wiring example: `link_name: backend_deployment`"
                            .to_string(),
                    },
                );
            }
        }

        // go through to see if some is left undefined
        for ep in db
            .frontend_application()
            .c_children_frontend_application_external_link(frontend_app)
        {
            if !e_map.contains_key(ep) {
                return Err(
                    PlatformValidationError::FrontendApplicationLinkWiringExternalLinkUndefined {
                        application_name: db
                            .frontend_application()
                            .c_application_name(frontend_app)
                            .clone(),
                        deployment_name: db
                            .frontend_application_deployment()
                            .c_deployment_name(depl)
                            .clone(),
                        link_wiring: src.clone(),
                        undefined_link: db
                            .frontend_application_external_link()
                            .c_link_name(*ep)
                            .clone(),
                    },
                );
            }
        }
    }

    Ok(res)
}

pub fn backend_apps_in_region(
    db: &Database,
) -> Projection<TableRowPointerRegion, HashSet<TableRowPointerBackendApplication>>
{
    Projection::create(db.region().rows_iter(), |r| {
        let mut backend_apps_in_region: HashSet<TableRowPointerBackendApplication> = HashSet::new();
        for depl in db.region().c_referrers_backend_application_deployment__region(r) {
            let _ = backend_apps_in_region.insert(db.backend_application_deployment().c_application_name(*depl));
        }

        backend_apps_in_region
    })
}

pub fn frontend_apps_in_region(
    db: &Database,
) -> Projection<TableRowPointerRegion, HashSet<TableRowPointerFrontendApplication>>
{
    Projection::create(db.region().rows_iter(), |r| {
        let mut frontend_apps_in_region: HashSet<TableRowPointerFrontendApplication> = HashSet::new();
        for depl in db.region().c_referrers_frontend_application_deployment__region(r) {
            let _ = frontend_apps_in_region.insert(db.frontend_application_deployment().c_application_name(*depl));
        }

        frontend_apps_in_region
    })
}
