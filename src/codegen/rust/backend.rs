use std::collections::HashSet;
use std::fmt::Write;

use convert_case::{Case, Casing};

use crate::{
    codegen::Directory,
    database::{
        TableRowPointerBackendApplication, TableRowPointerBackendHttpEndpoint,
        TableRowPointerPgMutator, TableRowPointerPgQuery, TableRowPointerVersionedType, TableRowPointerChQuery, TableRowPointerChMutator,
    },
    static_analysis::{
        databases::{postgres, clickhouse},
        http_endpoints::{CheckedHttpEndpoint, CorePathSegment, PathArgs, ValidHttpPrimitiveType},
        projections::Projection,
        CheckedDB,
    },
};

use super::{GeneratedRustSourceForHttpEndpoint, RustCodegenContext, RustVersionedTypeSnippets};

pub fn generate_rust_backend_app(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    dir: &mut Directory,
) {
    let db = &checked.db;
    let app_name = db
        .backend_application()
        .c_application_name(app)
        .to_case(Case::Kebab);
    let comp_env = db.backend_application().c_build_environment(app);
    let nixpkgs = db
        .rust_compilation_environment()
        .c_nixpkgs_environment(comp_env);
    let nixpkgs_hash = db
        .nixpkgs_version()
        .c_checksum(db.nixpkgs_environment().c_version(nixpkgs));
    let edition = db.rust_compilation_environment().c_rust_edition(comp_env);
    let src_dir = dir.create_directory("src");
    let mut cgen_context = RustCodegenContext::new();
    src_dir.create_file("main.rs", rust_backend_main_rs());
    src_dir.create_file(
        "generated.rs",
        rust_app_generated_part(checked, app, &mut cgen_context),
    );
    src_dir.create_file_if_not_exists_condition(
        "implementation.rs", implementation_backend_mock_rs(),
        // if directory exists user is creating dir structure of modules, don't clobber
        crate::codegen::SpecialFileCreationCondition::DontCreateIfDirectoryExists("implementation".to_string())
    );
    dir.create_file(
        "Cargo.toml",
        super::rust_cargo_toml(edition.as_str(), checked, comp_env),
    );
    dir.create_file(
        "flake.nix",
        generate_rust_backend_flake(&app_name, nixpkgs_hash),
    );
    dir.create_file(
        ".envrc",
        "use flake".to_string(),
    );
}

fn rust_backend_main_rs() -> String {
    r#"
mod generated;
mod implementation;

fn main() {
    crate::generated::generated_main();
}
"#
    .to_string()
}

pub enum ResourceKind {
    PgConnection,
    ClickhouseClient {
        shard_name: String,
    },
    NatsJsConnection,
    S3Bucket,
    EnvVar,
    TypedEnvVar(String),
}

struct EplAppEnvVar {
    env_var_name: String,
    env_var_var_name: String,
}

struct RustAppResources {
    env_vars: Vec<EplAppEnvVar>,
    struct_slot: String,
    kind: ResourceKind,
}

fn rust_app_generated_part(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    cgen_context: &mut RustCodegenContext,
) -> String {
    let mut res = String::with_capacity(256);
    let mut resources = Vec::new();
    let app_name = checked.db.backend_application().c_application_name(app);

    resources.push(RustAppResources {
        env_vars: vec![
            EplAppEnvVar {
                env_var_name: "EPL_DEPLOYMENT_NAME".to_string(),
                env_var_var_name: "epl_deployment_name".to_string(),
            }
        ],
        struct_slot: "deployment_name".to_string(),
        kind: ResourceKind::EnvVar,
    });

    let pg_shards = checked
        .db
        .backend_application()
        .c_children_backend_application_pg_shard(app);
    for app_db in pg_shards {
        cgen_context.used_libraries.postgres = true;
        let db_shard_name = checked
            .db
            .backend_application_pg_shard()
            .c_shard_name(*app_db);

        resources.push(RustAppResources {
            env_vars: vec![
                EplAppEnvVar {
                    env_var_name: format!("EPL_PG_CONN_{}", db_shard_name).to_uppercase(),
                    env_var_var_name: format!("epl_pg_conn_{}", db_shard_name).to_string(),
                }
            ],
            struct_slot: format!("pg_conn_{}", db_shard_name),
            kind: ResourceKind::PgConnection,
        });
    }

    let ch_shards = checked
        .db
        .backend_application()
        .c_children_backend_application_ch_shard(app);
    for app_db in ch_shards {
        let db_shard_name = checked
            .db
            .backend_application_ch_shard()
            .c_shard_name(*app_db);

        resources.push(RustAppResources {
            env_vars: vec![
                EplAppEnvVar {
                    env_var_name: format!("EPL_CH_{}_USER", db_shard_name).to_uppercase(),
                    env_var_var_name: format!("epl_ch_{}_user", db_shard_name).to_string(),
                },
                EplAppEnvVar {
                    env_var_name: format!("EPL_CH_{}_PASSWORD", db_shard_name).to_uppercase(),
                    env_var_var_name: format!("epl_ch_{}_password", db_shard_name).to_string(),
                },
            ],
            struct_slot: format!("ch_{}_client", db_shard_name),
            kind: ResourceKind::ClickhouseClient { shard_name: db_shard_name.clone() },
        });

        resources.push(RustAppResources {
            env_vars: vec![
                EplAppEnvVar {
                    env_var_name: format!("EPL_CH_{}_DATABASE", db_shard_name).to_uppercase(),
                    env_var_var_name: format!("ch_{}_database", db_shard_name).to_string(),
                },
            ],
            struct_slot: format!("ch_{}_database", db_shard_name),
            kind: ResourceKind::EnvVar,
        });
        resources.push(RustAppResources {
            env_vars: vec![
                EplAppEnvVar {
                    env_var_name: format!("EPL_CH_{}_URL", db_shard_name).to_uppercase(),
                    env_var_var_name: format!("ch_{}_url", db_shard_name).to_string(),
                },
            ],
            struct_slot: format!("ch_{}_url", db_shard_name),
            kind: ResourceKind::EnvVar,
        });
    }

    let mut streams_used = false;
    for app_stream in checked
        .db
        .backend_application()
        .c_children_backend_application_nats_stream(app)
        .iter()
    {
        streams_used = true;
        let stream_name = checked
            .db
            .backend_application_nats_stream()
            .c_stream_name(*app_stream);
        resources.push(RustAppResources {
            env_vars: vec![
                EplAppEnvVar {
                    env_var_name: format!("EPL_NATS_CONN_{}", stream_name.to_case(Case::Snake))
                        .to_uppercase(),
                    env_var_var_name: format!("epl_nats_conn_{}", stream_name.to_case(Case::Snake))
                        .to_string(),
                }
            ],
            struct_slot: format!("nats_conn_id_{}", stream_name.to_case(Case::Snake)),
            kind: ResourceKind::NatsJsConnection,
        });
        if checked
            .db
            .backend_application_nats_stream()
            .c_enable_producer(*app_stream)
            || checked
                .db
                .backend_application_nats_stream()
                .c_enable_consumer(*app_stream)
        {
            resources.push(RustAppResources {
                env_vars: vec![
                    EplAppEnvVar {
                        env_var_name: format!("EPL_NATS_STREAM_{}", stream_name.to_case(Case::Snake))
                            .to_uppercase(),
                        env_var_var_name: format!("epl_nats_stream_{}", stream_name.to_case(Case::Snake))
                            .to_string(),
                    }
                ],
                struct_slot: format!("nats_stream_{}", stream_name.to_case(Case::Snake)),
                kind: ResourceKind::EnvVar,
            });
        }
    }

    for s3_bucket in checked
        .db
        .backend_application()
        .c_children_backend_application_s3_bucket(app)
    {
        let bucket_name = checked.db.backend_application_s3_bucket().c_bucket_name(*s3_bucket);
        resources.push(RustAppResources {
            env_vars: vec![
                EplAppEnvVar {
                    env_var_name: format!("EPL_S3_{}_BUCKET", bucket_name.to_case(Case::Snake).to_uppercase()),
                    env_var_var_name: format!("epl_s3_{}_bucket", bucket_name.to_case(Case::Snake)),
                },
                EplAppEnvVar {
                    env_var_name: format!("EPL_S3_{}_URI", bucket_name.to_case(Case::Snake).to_uppercase()),
                    env_var_var_name: format!("epl_s3_{}_url", bucket_name.to_case(Case::Snake)),
                },
                EplAppEnvVar {
                    env_var_name: format!("EPL_S3_{}_USER", bucket_name.to_case(Case::Snake).to_uppercase()),
                    env_var_var_name: format!("epl_s3_{}_user", bucket_name.to_case(Case::Snake)),
                },
                EplAppEnvVar {
                    env_var_name: format!("EPL_S3_{}_PASSWORD", bucket_name.to_case(Case::Snake).to_uppercase()),
                    env_var_var_name: format!("epl_s3_{}_password", bucket_name.to_case(Case::Snake)),
                },
            ],
            struct_slot: format!("s3_{}", bucket_name.to_case(Case::Snake)),
            kind: ResourceKind::S3Bucket,
        })
    }

    for app_cfg in checked
        .db
        .backend_application()
        .c_children_backend_application_config(app)
    {
        let cfg_name = checked.db.backend_application_config().c_config_name(*app_cfg);
        let cfg_type = checked.db.backend_application_config().c_config_type(*app_cfg).as_str();
        let rust_type = match cfg_type {
            "int" => "i64",
            "float" => "f64",
            "string" => "String",
            "bool" => "bool",
            other => panic!("Unexpected type {other}")
        };
        resources.push(RustAppResources {
            env_vars: vec![
                EplAppEnvVar {
                    env_var_name: format!("EPL_CFG_{}", cfg_name.to_case(Case::Snake).to_uppercase()),
                    env_var_var_name: format!("epl_cfg_{}", cfg_name.to_case(Case::Snake)),
                }
            ],
            struct_slot: format!("cfg_{}", cfg_name.to_case(Case::Snake)),
            kind: ResourceKind::TypedEnvVar(rust_type.to_string()),
        })
    }

    res += "#[allow(unused_imports)]\n";
    res += "use serde::{Serialize, Deserialize};\n";
    res += "#[allow(unused_imports)]\n";
    res += "use opentelemetry::trace::{TraceContextExt, Tracer, Span};\n";
    res += "#[allow(unused_imports)]\n";
    res += "use base64::Engine;\n";
    res += "\n";
    res += &generated_main_function(checked, app);
    res += "\n";
    res += &generate_tracing_hemmarhoids(&app_name.as_str());
    res += "\n";
    res += eden_platform_http_lib();
    res += "\n";
    res += "pub struct AppResources {\n";
    res += "    tracer: ::opentelemetry::global::BoxedTracer,\n";

    if streams_used {
        res += "    nats_conns: Vec<::async_nats::jetstream::Context>,\n";
    }

    for r in &resources {
        res += "    ";
        res += &r.struct_slot;
        match &r.kind {
            ResourceKind::PgConnection => {
                res += ": ::bb8::Pool<::bb8_postgres::PostgresConnectionManager<::tokio_postgres::NoTls>>,\n";
            }
            ResourceKind::NatsJsConnection => {
                res += ": usize,\n";
            }
            ResourceKind::EnvVar => {
                res += ": String,\n";
            }
            ResourceKind::TypedEnvVar(rust_type) => {
                res += ": ";
                res += rust_type;
                res += ",\n";
            }
            ResourceKind::S3Bucket => {
                res += ": ::s3::Bucket,\n"
            }
            ResourceKind::ClickhouseClient { .. } => {
                res += ": ::reqwest::Client,\n"
            }
        }
    }

    res += "}\n";
    res += "\n";

    res += "impl AppResources {\n";
    res += "    pub async fn new() -> Result<AppResources, Box<dyn std::error::Error + Send + Sync>> {\n";

    if cgen_context.used_libraries.postgres {
        res += "        use ::std::str::FromStr;\n";
    }

    for r in &resources {
        for ev in &r.env_vars {
            match &r.kind {
                ResourceKind::PgConnection | ResourceKind::NatsJsConnection | ResourceKind::S3Bucket | ResourceKind::ClickhouseClient { .. } => {
                    write!(&mut res, "        let {} = ::std::env::var(\"{}\").expect(\"Mandatory environment variable {} not configured\");\n", ev.env_var_var_name, ev.env_var_name, ev.env_var_name).unwrap();
                }
                ResourceKind::EnvVar => {
                    assert_eq!(r.env_vars.len(), 1);
                    write!(&mut res, "        let {} = ::std::env::var(\"{}\").expect(\"Mandatory environment variable {} not configured\").to_string();\n", r.struct_slot, ev.env_var_name, ev.env_var_name).unwrap();
                }
                ResourceKind::TypedEnvVar(rust_type) => {
                    assert_eq!(r.env_vars.len(), 1);
                    if rust_type == "String" {
                        write!(&mut res, "        let {} = ::std::env::var(\"{}\").expect(\"Mandatory environment variable {} not configured\").to_string();\n", r.struct_slot, ev.env_var_name, ev.env_var_name).unwrap();
                    } else {
                        write!(&mut res, "        let {} = ::std::env::var(\"{}\").expect(\"Mandatory environment variable {} not configured\").parse::<{}>().expect(\"Can't parse config {}\");\n", r.struct_slot, ev.env_var_name, ev.env_var_name, rust_type, ev.env_var_name).unwrap();
                    }
                }
            }
        }
    }

    if streams_used {
        res += "        let mut nats_conns_urls = Vec::with_capacity(1);\n"
    }

    let mut is_first_stream = true;
    for r in &resources {
        if let ResourceKind::NatsJsConnection = r.kind {
            if is_first_stream {
                is_first_stream = false;
                res += &format!("        nats_conns_urls.push({});\n", r.env_vars[0].env_var_var_name);
                res += &format!(
                    "        let {} = nats_conns_urls.len() - 1;\n",
                    r.struct_slot
                );
            } else {
                res += &format!(
                    r#"        let {} =
            if let Some(cid) = nats_conns_urls.iter().enumerate().find(|(_, cs)| cs.as_str() == {}) {{
                cid.0
            }} else {{
                nats_conns_urls.push({});
                nats_conns_urls.len() - 1
            }};
"#,
                    r.struct_slot, r.env_vars[0].env_var_var_name, r.env_vars[0].env_var_var_name
                );
            }
        }
    }

    res += "\n";

    for r in &resources {
        match &r.kind {
            ResourceKind::PgConnection => {
                res += "        let manager = ::bb8_postgres::PostgresConnectionManager::new(\n";
                res += &format!("            ::tokio_postgres::Config::from_str(&{})?, ::tokio_postgres::NoTls\n", r.env_vars[0].env_var_var_name);
                res += "        );\n";
                res += &format!("        let {} = ", r.struct_slot);
                res += "::bb8::Pool::builder()\n";
                res += "            .build(manager)\n";
                res += "            .await?;\n";
                res += "\n";
            }
            _ => {}
        }
    }

    for r in &resources {
        match &r.kind {
            ResourceKind::ClickhouseClient { shard_name } => {
                let slot = &r.struct_slot;
                write!(&mut res, r#"
        let mut hm = ::reqwest::header::HeaderMap::new();
        hm.append("X-Clickhouse-User", epl_ch_{shard_name}_user.parse()?);
        hm.append("X-Clickhouse-Key", epl_ch_{shard_name}_password.parse()?);
        let {slot} = ::reqwest::ClientBuilder::new()
            // without disabled pooling we encounter hyper::Error(IncompleteMessage)
            // errors https://github.com/hyperium/hyper/issues/2136
            .pool_max_idle_per_host(0)
            .default_headers(hm)
            .timeout(::std::time::Duration::from_secs(6000))
            .build()?;
"#).unwrap();
            }
            _ => {}
        }
    }

    if streams_used {
        res += r#"
        let mut nats_conns_ctx = Vec::with_capacity(nats_conns_urls.len());
        for url in &nats_conns_urls {
            nats_conns_ctx.push(::async_nats::connect(url));
        }

        let joined = ::futures::future::join_all(nats_conns_ctx).await;
        let mut nats_conns = Vec::with_capacity(nats_conns_urls.len());
        for ctx in joined {
            nats_conns.push(::async_nats::jetstream::new(ctx?));
        }
"#;
    }

    for r in &resources {
        match &r.kind {
            ResourceKind::S3Bucket => {
                let struct_slot = &r.struct_slot;
                write!(&mut res, r#"
        let {struct_slot} = ::s3::Bucket::new(
            &epl_{struct_slot}_bucket,
            ::s3::Region::Custom {{
                region: "any".to_string(),
                endpoint: epl_{struct_slot}_url,
            }},
            ::s3::creds::Credentials {{
                access_key: Some(epl_{struct_slot}_user),
                secret_key: Some(epl_{struct_slot}_password),
                security_token: None,
                session_token: None,
                expiration: None,
            }}
        )?.with_path_style();
"#).unwrap()
            }
            _ => {}
        }
    }

    res += "\n";
    writeln!(&mut res, "        let tracer = ::opentelemetry::global::tracer(\"{app_name}\");").unwrap();

    res += "        Ok(AppResources {\n";
    res += "            tracer,\n";
    if streams_used {
        res += "            nats_conns,\n";
    }
    for r in &resources {
        res += "            ";
        res += &r.struct_slot;
        res += ",\n";
    }
    res += "        })\n";

    res += "    }\n";
    res += "}\n";
    res += "\n";

    res += r#"
struct AppServicerApi {
    r: ::std::sync::Arc<AppResources>,
    app_implementation: ::std::sync::Arc<dyn AppRequirements + Send + Sync>,
}

fn generate_trace_id() -> ::opentelemetry::trace::TraceId {
    use rand::Rng;
    ::opentelemetry::trace::TraceId::from_bytes(rand::thread_rng().gen::<[u8; 16]>())
}

fn http_fetch_trace_id(req: &::actix_web::HttpRequest) -> ::opentelemetry::trace::TraceId {
    if let Some(v) = req.headers().get("trace-id") {
        if let Ok(h_str) = v.to_str() {
            if let Ok(tid) = ::opentelemetry::trace::TraceId::from_hex(h_str) {
                return tid;
            }
        }
    }

    generate_trace_id()
}

fn nats_fetch_trace_id(msg: &::async_nats::Message) -> ::opentelemetry::trace::TraceId {
    if let Some(h) = &msg.headers {
        if let Some(h_val) = h.get("trace-id") {
            if let Ok(tid) = ::opentelemetry::trace::TraceId::from_hex(h_val.as_str()) {
                return tid;
            }
        }
    }

    generate_trace_id()
}

fn build_app_api(app_svc: &AppServicerApi, span_name: &str, trace_id: ::opentelemetry::trace::TraceId) -> AppApi {
    let span = app_svc.r.tracer
        .span_builder(span_name.to_string())
        .with_trace_id(trace_id.clone())
        .start(&app_svc.r.tracer);
    let cx = ::opentelemetry::Context::current_with_span(span);
    let app_api = AppApi::new(app_svc.r.clone(), cx);
    app_api
}

// used in free background job without context
#[allow(unused)]
fn build_app_api_no_context(app_svc: &AppServicerApi) -> AppApi {
    AppApi::new_without_context(app_svc.r.clone())
}

// We distinguish two situations,
// where user is already inside a span once get got AppApi
// like http request, and another situation,
// where user is in his custom controlled background job
// but he wants to create new distinct spans in AppApi struct,
// so we're not in a span there
#[derive(Clone)]
enum EplTracingContext {
    Context(::opentelemetry::Context),
    NoContext,
}

/// This struct carries context for one app request or message processing
#[derive(Clone)]
pub struct AppApi {
    r: ::std::sync::Arc<AppResources>,
    context: EplTracingContext,
}

impl AppApi {
    fn new(r: ::std::sync::Arc<AppResources>, context: ::opentelemetry::Context) -> AppApi {
        AppApi { r, context: EplTracingContext::Context(context) }
    }

    fn new_without_context(r: ::std::sync::Arc<AppResources>) -> AppApi {
        AppApi { r, context: EplTracingContext::NoContext }
    }

    pub fn span(&self, name: &str) -> ::opentelemetry::global::BoxedSpan {
        match &self.context {
            EplTracingContext::Context(ctx) => {
                self.r.tracer.start_with_context(name.to_string(), ctx)
            }
            EplTracingContext::NoContext => {
                self.r.tracer.span_builder(name.to_string())
                    .with_trace_id(generate_trace_id())
                    .start(&self.r.tracer)
            }
        }
    }
"#;

    let mut pg_types_needed = false;
    let mut ch_types_needed = false;

    generate_pg_queries(checked, app, &mut pg_types_needed, &mut res, cgen_context);
    generate_pg_mutators(checked, app, &mut pg_types_needed, &mut res, cgen_context);
    generate_pg_transaction_scripts(checked, app, &mut pg_types_needed, &mut res, cgen_context);
    generate_ch_queries(checked, app, &mut ch_types_needed, &mut res, cgen_context);
    generate_ch_mutators(checked, app, &mut ch_types_needed, &mut res, cgen_context);
    generate_ch_inserters(checked, app, &mut ch_types_needed, &mut res, cgen_context);
    generate_nats_jetstream_publishers(checked, app, &mut res, cgen_context);
    generate_s3_buckets(checked, app, &mut res);
    generate_app_configs(checked, app, &mut res);

    res += "}\n";
    res += "\n";

    if !cgen_context.transaction_script_types.is_empty() {
        res += r#"
struct TransactionState {
    // extend lifetimes to static because we have trouble with travelling builder pattern
    trx: ::tokio_postgres::Transaction<'static>,
    _conn: Box<::bb8::PooledConnection<'static, ::bb8_postgres::PostgresConnectionManager<::tokio_postgres::NoTls>>>,
    _r: ::std::sync::Arc<AppResources>,
    start_time: ::std::time::Instant,
    context: ::opentelemetry::Context,
}
"#;
    }

    for trx_scr_type in &cgen_context.transaction_script_types {
        res += trx_scr_type;
        res += "\n";
    }

    for aux_type in cgen_context.db_aux_types.values() {
        res += aux_type;
        res += "\n";
    }

    res += "#[::async_trait::async_trait]\n";
    res += "pub trait AppRequirements {\n";

    for http_api in checked
        .db
        .backend_application()
        .c_children_backend_http_endpoint(app)
    {
        cgen_context.used_libraries.http = true;

        let gen_src = checked
            .projections
            .rust_sources_for_http_endpoints
            .value(*http_api);
        res += "    async fn http_endpoint_";
        res += checked
            .db
            .backend_http_endpoint()
            .c_http_endpoint_name(*http_api);
        res += "(&self, api: &AppApi, payload: ";
        res += &gen_src.rust_args_struct_name;
        res += ") -> Result<";
        res += &gen_src.rust_output_struct_name;
        res += ", Box<dyn ::std::error::Error + Send + Sync>>;\n";
    }

    for stream_ptr in checked
        .db
        .backend_application()
        .c_children_backend_application_nats_stream(app)
    {
        // consumer should be deployment name + application name?
        if checked
            .db
            .backend_application_nats_stream()
            .c_enable_consumer(*stream_ptr)
        {
            cgen_context.used_libraries.nats = true;

            let stream_name = checked
                .db
                .backend_application_nats_stream()
                .c_stream_name(*stream_ptr);
            let stream_type = checked
                .db
                .backend_application_nats_stream()
                .c_stream_type(*stream_ptr);
            let snippet = checked
                .projections
                .rust_versioned_type_snippets
                .value(stream_type);
            let enable_subjects = checked.db.backend_application_nats_stream().c_enable_subjects(*stream_ptr);
            let input_struct_name = &snippet.nominal_type_name;

            res += "    async fn jetstream_consume_";
            res += stream_name;
            res += "(&self, api: &AppApi, payload: ";
            res += input_struct_name;
            if enable_subjects {
                res += ", subject: &str";
            }
            res += ") -> Result<(), Box<dyn ::std::error::Error + Send + Sync>>;\n";
        }
    }

    let bg_jobs = checked.db.backend_application().c_children_backend_application_background_job(app);
    for bg_job in bg_jobs {
        let bg_job_name = checked.db.backend_application_background_job().c_job_name(*bg_job);
        writeln!(&mut res, "    async fn bg_job_{bg_job_name}(&self, api: AppApi) -> Result<(), Box<dyn ::std::error::Error + Send + Sync>>;").unwrap();
    }

    res += "}\n";
    res += "\n";

    let used_types = checked.projections.application_used_bw_types.value(app);
    let mut binary_deser_types_needed = false;
    for (used_type, flags) in used_types {
        let snippet = checked
            .projections
            .rust_versioned_type_snippets
            .value(*used_type);
        res += &snippet.struct_definitions;
        res += "\n";

        if flags.binary_serialization {
            cgen_context.used_libraries.binary_ser = true;
            res += &snippet.binary_serialization_function.function_body;
            res += "\n";
        }

        if flags.binary_deserialization {
            cgen_context.used_libraries.binary_ser = true;
            binary_deser_types_needed = true;
            res += &snippet.binary_deserialization_function.function_body;
            res += "\n";
        }

        if flags.json_serialization {
            cgen_context.used_libraries.json_ser = true;
            res += &snippet.json_serialization_function.function_body;
            res += "\n";
        }

        if flags.json_deserialization {
            cgen_context.used_libraries.json_ser = true;
            res += &snippet.json_deserialization_function.function_body;
            res += "\n";
        }

        if flags.json_deserialization || flags.binary_deserialization {
            res += &snippet.migration_functions;
            res += "\n";
        }
    }

    if binary_deser_types_needed {
        res += "\n";
        res += binary_deserialization_error_types();
    }

    res += "\n";
    res += super::json_deserialization_error_types();

    if pg_types_needed {
        res += "\n";
        res += pg_interaction_error_types();
    }

    if ch_types_needed {
        res += "\n";
        res += ch_interaction_error_types();
    }

    res += "\n";
    res += "\n";
    res += prometheus_metrics_endpoint();

    for endpoint in checked
        .db
        .backend_application()
        .c_children_backend_http_endpoint(app)
        .iter()
    {
        let http_src = checked
            .projections
            .rust_sources_for_http_endpoints
            .value(*endpoint);

        for (pv_name, pv_src) in &http_src.prometheus_variables {
            cgen_context
                .register_prom_variable(pv_name.clone(), pv_src.initialization_body.clone());
        }

        res += "\n";
        res += "\n";
        res += &http_src.rust_args_struct_definition;
        res += "\n";
        res += "\n";
        res += &http_src.rust_endpoint_declaration;
    }

    res += &generate_jetstream_setup_function(cgen_context, checked, app);

    if !cgen_context.prom_variables.is_empty() {
        res += "\n";
        res += "\n";
        res += "::lazy_static::lazy_static! {\n";
        res += "    static ref EPL_DEPLOYMENT_NAME: String = ::std::env::var(\"EPL_DEPLOYMENT_NAME\").expect(\"Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured\");\n";
        for stream in checked
            .db
            .backend_application()
            .c_children_backend_application_nats_stream(app)
        {
            let stream_name = checked
                .db
                .backend_application_nats_stream()
                .c_stream_name(*stream)
                .to_uppercase();
            res += &format!("    static ref EPL_NATS_STREAM_{stream_name}: String = ::std::env::var(\"EPL_NATS_STREAM_{stream_name}\").expect(\"Mandatory environment variable EPL_NATS_STREAM_{stream_name} is not configured\");\n");
        }
        for prom_var in cgen_context.prom_variables.values() {
            res += &prom_var.initialization_body;
        }
        res += "}\n";
    }

    res
}

fn generate_nats_jetstream_publishers(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
) {
    for stream_ptr in checked
        .db
        .backend_application()
        .c_children_backend_application_nats_stream(app)
    {
        if checked
            .db
            .backend_application_nats_stream()
            .c_enable_producer(*stream_ptr)
        {
            let stream_name = checked
                .db
                .backend_application_nats_stream()
                .c_stream_name(*stream_ptr);
            let stream_type = checked
                .db
                .backend_application_nats_stream()
                .c_stream_type(*stream_ptr);
            let snippet = checked
                .projections
                .rust_versioned_type_snippets
                .value(stream_type);
            let enable_subjects = checked.db.backend_application_nats_stream().c_enable_subjects(*stream_ptr);
            let input_struct_name = &snippet.nominal_type_name;

            let mut qargs = vec!["&self".to_string()];
            qargs.push(format!("input: &{}", input_struct_name));
            if enable_subjects {
                qargs.push("subject: &str".to_string());
            }

            *res += "\n";
            *res += "    pub async fn jetstream_publish_";
            *res += stream_name;
            *res += "(";

            *res += &qargs.join(", ");

            *res += ") -> Result<::async_nats::jetstream::context::PublishAckFuture, ::async_nats::error::Error<::async_nats::jetstream::context::PublishErrorKind>> {\n";
            let prom_var_nats_latency =
                format!("METRIC_NATS_PUBLISH_LATENCY_{}", stream_name.to_uppercase());
            cgen_context.register_prom_variable(
                prom_var_nats_latency.clone(),
                prometheus_nats_publish_latency_metric(
                    &prom_var_nats_latency,
                    checked.db.backend_application().c_application_name(app),
                    stream_name,
                ),
            );

            let prom_var_nats_bytes =
                format!("METRIC_NATS_PUBLISH_BYTES_{}", stream_name.to_uppercase());
            cgen_context.register_prom_variable(
                prom_var_nats_bytes.clone(),
                prometheus_nats_bytes_published_metric(
                    &prom_var_nats_bytes,
                    checked.db.backend_application().c_application_name(app),
                    stream_name,
                ),
            );

            *res += "        let pre = ::std::time::Instant::now();\n";
            writeln!(res, "        let mut span = self.span(\"js_publish_{}\");", stream_name).unwrap();
            writeln!(res, "        let trace_id = span.span_context().trace_id().to_string();").unwrap();
            *res += "        let payload: ::std::string::String = ";
            *res += &snippet.json_serialization_function.function_name;
            *res += "(input);\n";
            *res += "        let payload_size = payload.len();\n";
            *res += "        let mut headers = ::async_nats::HeaderMap::new();\n";
            *res += "        headers.insert(\"trace-id\", trace_id.as_str());\n";
            if enable_subjects {
                writeln!(res, "        let subject = format!(\"{{}}.{{}}\", self.r.nats_stream_{}, subject);", stream_name).unwrap();
            } else {
                writeln!(res, "        let subject = self.r.nats_stream_{}.clone();", stream_name).unwrap();
            }
            *res += "        let res = self.r.nats_conns[self.r.nats_conn_id_";
            *res += stream_name;
            *res += "].publish_with_headers(subject, headers, payload.into()).await;\n";
            *res += "        match &res {\n";
            *res += "            Ok(_res) => {\n";
            *res += "                ";
            *res += &prom_var_nats_latency;
            *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
            *res += "                ";
            *res += &prom_var_nats_bytes;
            *res += ".inc_by(payload_size.try_into().unwrap_or_default());\n";
            *res += "            }\n";
            *res += "            Err(e) => {\n";
            *res += "                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));\n";
            *res += "            }\n";
            *res += "        };\n";
            *res += "        res\n";
            *res += "    }\n";
        }
    }
}

fn generate_app_configs(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    res: &mut String,
) {
    for cfg in checked
        .db
        .backend_application()
        .c_children_backend_application_config(app)
    {
        let config_type = checked
            .db
            .backend_application_config()
            .c_config_type(*cfg);
        let config_name = checked
            .db
            .backend_application_config()
            .c_config_name(*cfg);

        let output_type = match config_type.as_str() {
            "string" => "&str",
            "int" => "i64",
            "float" => "f64",
            "bool" => "bool",
            other => panic!("Unexpected rust output type {other}")
        };

        let maybe_postfix = if config_type == "string" {
            ".as_str()"
        } else { "" };

        write!(res, r#"
    pub fn cfg_{config_name}(&self) -> {output_type} {{
        self.r.cfg_{config_name}{maybe_postfix}
    }}
"#).unwrap();
    }
}

fn generate_s3_buckets(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    res: &mut String,
) {
    for bucket in checked
        .db
        .backend_application()
        .c_children_backend_application_s3_bucket(app)
    {
        let bucket_name = checked
            .db
            .backend_application_s3_bucket()
            .c_bucket_name(*bucket);

        write!(res, r#"
    pub fn s3_{bucket_name}(&self) -> &::s3::Bucket {{
        &self.r.s3_{bucket_name}
    }}
"#).unwrap();
    }
}

fn generate_ch_queries(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    ch_types_needed: &mut bool,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
) {
    for shard in checked
        .db
        .backend_application()
        .c_children_backend_application_ch_shard(app)
    {
        let shard_name = checked
            .db
            .backend_application_ch_shard()
            .c_shard_name(*shard);
        let queries = checked.projections.application_ch_shard_queries.value(*shard);

        for query in &queries.queries {
            *ch_types_needed = true;
            let db = checked.db.ch_query().c_parent(*query);
            let checked_db = checked.async_res.checked_ch_dbs.get(&db).unwrap();
            let checked_query = checked_db.queries.get(query).unwrap();
            let qname = checked.db.ch_query().c_query_name(*query);
            let method_prefix = "chq_";

            let aux_struct_name = cgen_context.ch_query_output_type_name(checked, *query);

            *res += "\n";
            *res += "    pub async fn ";
            *res += method_prefix;
            *res += qname;
            *res += "(";

            let mut qargs = vec!["&self".to_string()];

            for fq in &checked_query.full_query.args {
                let arg_t = match fq.the_type {
                    clickhouse::ValidDbType::Int32 => "i32",
                    clickhouse::ValidDbType::Int64 => "i64",
                    clickhouse::ValidDbType::Int128 => "i128",
                    clickhouse::ValidDbType::Int256 => "::num256::Int256",
                    clickhouse::ValidDbType::Float32 => "f32",
                    clickhouse::ValidDbType::Float64 => "f64",
                    clickhouse::ValidDbType::Bool => "bool",
                    clickhouse::ValidDbType::String => "&str",
                    clickhouse::ValidDbType::DateTime => "::chrono::DateTime<::chrono::Utc>",
                    clickhouse::ValidDbType::Date => "::chrono::NaiveDate",
                };
                qargs.push(format!("{}: {}", fq.name, arg_t));
            }

            *res += &qargs.join(", ");

            *res += ") -> Result<Vec<";
            *res += &aux_struct_name;
            *res += ">, ChInteractionError> {\n";

            generate_chq_block(
                checked,
                res,
                cgen_context,
                app,
                *query,
                &shard_name,
            );

            *res += "        Ok(res)\n";
            *res += "    }\n";
        }
    }
}

fn generate_ch_mutators(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    ch_types_needed: &mut bool,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
) {
    for shard in checked
        .db
        .backend_application()
        .c_children_backend_application_ch_shard(app)
    {
        let shard_name = checked
            .db
            .backend_application_ch_shard()
            .c_shard_name(*shard);
        let queries = checked.projections.application_ch_shard_queries.value(*shard);

        for mutator in &queries.mutators {
            *ch_types_needed = true;
            let db = checked.db.ch_mutator().c_parent(*mutator);
            let checked_db = checked.async_res.checked_ch_dbs.get(&db).unwrap();
            let checked_query = checked_db.mutators.get(mutator).unwrap();
            let qname = checked.db.ch_mutator().c_mutator_name(*mutator);
            let method_prefix = "chm_";

            *res += "\n";
            *res += "    pub async fn ";
            *res += method_prefix;
            *res += qname;
            *res += "(";

            let mut qargs = vec!["&self".to_string()];

            for fq in &checked_query.full_query.args {
                let arg_t = match fq.the_type {
                    clickhouse::ValidDbType::Int32 => "i32",
                    clickhouse::ValidDbType::Int64 => "i64",
                    clickhouse::ValidDbType::Int128 => "i128",
                    clickhouse::ValidDbType::Int256 => "::num256::Int256",
                    clickhouse::ValidDbType::DateTime => "::chrono::DateTime<::chrono::Utc>",
                    clickhouse::ValidDbType::Date => "::chrono::NaiveDate",
                    clickhouse::ValidDbType::Float32 => "f32",
                    clickhouse::ValidDbType::Float64 => "f64",
                    clickhouse::ValidDbType::Bool => "bool",
                    clickhouse::ValidDbType::String => "&str",
                };
                qargs.push(format!("{}: {}", fq.name, arg_t));
            }

            *res += &qargs.join(", ");

            *res += ") -> Result<(), ChInteractionError> {\n";

            generate_chm_block(
                checked,
                res,
                cgen_context,
                app,
                *mutator,
                &shard_name,
            );

            *res += "        Ok(())\n";
            *res += "    }\n";
        }
    }
}

fn generate_ch_inserters(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    ch_types_needed: &mut bool,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
) {
    for shard in checked
        .db
        .backend_application()
        .c_children_backend_application_ch_shard(app)
    {
        let shard_name = checked
            .db
            .backend_application_ch_shard()
            .c_shard_name(*shard);
        let shard_schema = checked
            .db
            .backend_application_ch_shard()
            .c_ch_schema(*shard);
        let db_res = checked.async_res.checked_ch_dbs.get(&shard_schema).unwrap();
        let latest_schema = db_res.schema_snapshots.values().rev().next();
        let queries = checked.projections.application_ch_shard_queries.value(*shard);
        let schema_name = checked.db.ch_schema().c_schema_name(shard_schema);

        for inserter in &queries.inserters {
            *ch_types_needed = true;
            let latest_schema = *latest_schema.as_ref().unwrap();
            let table_fields = latest_schema.field_type_index.get(inserter).unwrap();
            assert!(!table_fields.is_view, "TODO: nice error for preventing inserters for views");

            let method_prefix = "ch_insert_into_";

            let aux_struct_name = cgen_context.ch_inserter_input_type_name(
                checked, shard_schema, &inserter
            );

            *res += "\n";
            *res += "    pub async fn ";
            *res += method_prefix;
            *res += shard_name;
            *res += "_";
            *res += inserter;
            *res += "(";

            let qargs = [
                "&self".to_string(),
                format!("rows: &[{aux_struct_name}]"),
            ];

            *res += &qargs.join(", ");

            *res += ") -> Result<(), ChInteractionError> {\n";
            *res += "        use ::std::fmt::Write;\n";
            *res += "        let pre = ::std::time::Instant::now();\n";

            let relevant_fields =
                table_fields.fields.iter().filter_map(|i| {
                    if i.1.insertion_allowed {
                        Some(i)
                    } else { None }
                }).collect::<Vec<_>>();

            let columns = relevant_fields.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(", ");

            let prom_variable = format!(
                "METRIC_CHI_{}_{}",
                shard_name.to_uppercase(),
                inserter.to_uppercase()
            );
            cgen_context.register_prom_variable(
                prom_variable.clone(),
                prometheus_ch_query_metric(
                    &prom_variable,
                    checked.db.backend_application().c_application_name(app),
                    shard_name,
                    schema_name,
                    inserter,
                    true,
                    false,
                ),
            );
            write!(res, r#"
        let span = self.span("{method_prefix}{shard_name}_{inserter}_serialize");

        let mut payload = "INSERT INTO {inserter}({columns}) VALUES ".to_string();
        let mut row_idx = 0usize;
        for row in rows {{
            payload += "(";
"#).unwrap();

            let mut col_idx = 0usize;
            for (fname, ftype) in &relevant_fields {
                assert!(!ftype.col_type.starts_with("Nullable"), "No nullable columns");
                if ftype.has_default {
                    match ftype.col_type.as_str() {
                        "String" => {
                            write!(res, r#"
            if let Some(row_val) = &row.{fname} {{
                payload += "base64Decode('";
                base64::prelude::BASE64_STANDARD.encode_string(row_val, &mut payload);
                payload += "')";
            }} else {{ payload += "DEFAULT" }}
"#).unwrap();
                        }
                        "Date" => {
                            write!(res, r#"
            if let Some(row_val) = &row.{fname} {{
                write!(&mut payload, "'{{}}'", row_val.format("%Y-%m-%d")).unwrap();
            }} else {{ payload += "DEFAULT" }}
"#).unwrap();
                        }
                        "DateTime" => {
                            write!(res, r#"
            if let Some(row_val) = &row.{fname} {{
                write!(&mut payload, "'{{}}'", row_val.format("%Y-%m-%d %H:%M:%S")).unwrap();
            }} else {{ payload += "DEFAULT" }}
"#).unwrap();
                        }
                        "Int32" | "Int64" | "Int128" | "Int256" | "Float32" | "Float64" | "Bool" => {
                            write!(res, r#"
            if let Some(row_val) = &row.{fname} {{
                write!(&mut payload, "{{}}", row_val).unwrap();
            }} else {{ payload += "DEFAULT" }}
"#).unwrap();
                        }
                        other => {
                            panic!("Unepected type for inserter: {other}")
                        }
                    }
                } else {
                    match ftype.col_type.as_str() {
                        "String" => {
                            // not most efficient way but don't want to spend too much time on this yet
                            write!(res, r#"
            payload += "base64Decode('";
            base64::prelude::BASE64_STANDARD.encode_string(&row.{fname}, &mut payload);
            payload += "')";
"#).unwrap();
                        }
                        "Date" => {
                            write!(res, r#"
            write!(&mut payload, "'{{}}'", row.{fname}.format("%Y-%m-%d")).unwrap();
"#).unwrap();
                        }
                        "DateTime" => {
                            write!(res, r#"
            write!(&mut payload, "'{{}}'", row.{fname}.format("%Y-%m-%d %H:%M:%S")).unwrap();
"#).unwrap();
                        }
                        "Int32" | "Int64" | "Float32" | "Float64" | "Bool" => {
                            write!(res, r#"
            write!(&mut payload, "{{}}", row.{fname}).unwrap();
"#).unwrap();
                        }
                        other => {
                            panic!("Unepected type for inserter: {other}")
                        }
                    }
                }
                if col_idx < relevant_fields.len() - 1 {
                        write!(res, r#"
            payload += ",";
"#).unwrap();
                }
                col_idx += 1;
            }

            write!(res, r#"
            payload += ")";
            if row_idx < rows.len() - 1 {{
                payload += ",";
            }}
            row_idx += 1;
        }}
        drop(span);
"#).unwrap();

            write!(res, r#"
        let mut span = self.span("{method_prefix}{shard_name}_{inserter}_send");
        let resp = self.r.ch_{shard_name}_client
            .request(::reqwest::Method::POST, &self.r.ch_{shard_name}_url)
            .query(&[
                ("database", &self.r.ch_{shard_name}_database),
            ])
            .body(payload)
            .send()
            .await
            .map_err(|e| {{
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            }})?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {{
            let e = ChInteractionError::HttpResponseError {{
                expected_status_code: 200,
                actual_status_code: status_code,
                body: resp.text().await.unwrap_or_default(),
            }};
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            return Err(e);
        }}
"#).unwrap();

            *res += "        ";
            *res += &prom_variable;
            *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";

            *res += "        Ok(())\n";
            *res += "    }\n";
        }
    }
}

fn generate_chm_block(
    checked: &CheckedDB,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
    app: TableRowPointerBackendApplication,
    mutator: TableRowPointerChMutator,
    shard_name: &str,
) {
    let db = checked.db.ch_mutator().c_parent(mutator);
    let checked_query = checked
        .async_res
        .checked_ch_dbs
        .get(&db)
        .unwrap()
        .mutators
        .get(&mutator)
        .unwrap();
    let qname = checked.db.ch_mutator().c_mutator_name(mutator);
    let span_prefix = "chm_";
    let schema_name = checked.db.ch_schema().c_schema_name(db);

    *res += "        let the_query = r#\"";
    assert!(
        !checked_query
            .full_query
            .interpolated_expression
            .contains("r#\"")
        && !checked_query
            .full_query
            .interpolated_expression
            .contains("\"#")
    );
    *res += &checked_query.full_query.interpolated_expression;
    *res += "\"#;\n";
    *res += "        let pre = ::std::time::Instant::now();\n";
    writeln!(res, "        let mut span = self.span(\"{span_prefix}{qname}\");").unwrap();
    let prom_variable = format!(
        "METRIC_CHM_{}_{}",
        shard_name.to_uppercase(),
        qname.to_uppercase()
    );
    cgen_context.register_prom_variable(
        prom_variable.clone(),
        prometheus_ch_query_metric(
            &prom_variable,
            checked.db.backend_application().c_application_name(app),
            shard_name,
            schema_name,
            qname,
            false,
            true,
        ),
    );

    write!(res, r#"
        let resp = self.r.ch_{shard_name}_client
            .request(::reqwest::Method::POST, &self.r.ch_{shard_name}_url)
            .query(&[
                ("database", &self.r.ch_{shard_name}_database),
"#).unwrap();

    for qa in &checked_query.full_query.args {
        let arg_name = &qa.name;
        if qa.the_type == clickhouse::ValidDbType::String {
            write!(res, r#"                ("param_{arg_name}", {arg_name}),
"#).unwrap();
        } else if qa.the_type == clickhouse::ValidDbType::DateTime {
            write!(res, r#"                ("param_{arg_name}", &{arg_name}.format("%Y-%m-%d %H:%M:%S")),
"#).unwrap();
        } else {
            write!(res, r#"                ("param_{arg_name}", &{arg_name}.to_string()),
"#).unwrap();
        }
    }

    write!(res, r#"            ])
            .body(the_query)
            .send()
            .await
            .map_err(|e| {{
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            }})?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {{
            let err_text = resp.text().await.unwrap_or_default();
            span.set_status(::opentelemetry::trace::Status::error(err_text.clone()));
            return Err(ChInteractionError::HttpResponseError {{
                expected_status_code: 200,
                actual_status_code: status_code,
                body: err_text,
            }});
        }}
"#).unwrap();

    *res += "        ";
    *res += &prom_variable;
    *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
}

fn generate_chq_block(
    checked: &CheckedDB,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
    app: TableRowPointerBackendApplication,
    query: TableRowPointerChQuery,
    shard_name: &str,
) {
    let db = checked.db.ch_query().c_parent(query);
    let checked_query = checked
        .async_res
        .checked_ch_dbs
        .get(&db)
        .unwrap()
        .queries
        .get(&query)
        .unwrap();
    let qname = checked.db.ch_query().c_query_name(query);
    let aux_struct_name = cgen_context.ch_query_output_type_name(checked, query);
    let span_prefix = "chq_";
    let schema_name = checked.db.ch_schema().c_schema_name(db);

    *res += "        let the_query = r#\"";
    assert!(
        !checked_query
            .full_query
            .interpolated_expression
            .contains("r#\"")
        && !checked_query
            .full_query
            .interpolated_expression
            .contains("\"#")
    );
    *res += &checked_query.full_query.interpolated_expression;
    *res += "\"#;\n";
    *res += "        let pre = ::std::time::Instant::now();\n";
    writeln!(res, "        let mut span = self.span(\"{span_prefix}{qname}_query\");").unwrap();
    let prom_variable = format!(
        "METRIC_CHQ_{}_{}",
        shard_name.to_uppercase(),
        qname.to_uppercase()
    );
    cgen_context.register_prom_variable(
        prom_variable.clone(),
        prometheus_ch_query_metric(
            &prom_variable,
            checked.db.backend_application().c_application_name(app),
            shard_name,
            schema_name,
            qname,
            false,
            true,
        ),
    );

    write!(res, r#"
        let resp = self.r.ch_{shard_name}_client
            .request(::reqwest::Method::POST, &self.r.ch_{shard_name}_url)
            .query(&[
                ("database", &self.r.ch_{shard_name}_database),
"#).unwrap();

    let expected_tab_fields = checked_query.output_signature.len();
    let output_struct_field_names = checked_query.output_signature.iter().map(|sfield| {
        sfield.name.as_str()
    }).collect::<Vec<_>>().join(", ");

    for qa in &checked_query.full_query.args {
        let arg_name = &qa.name;
        if qa.the_type == clickhouse::ValidDbType::String {
            // TODO: string formatting is nasty, maybe we can get rid of &{}.to_string()
            write!(res, r#"                ("param_{arg_name}", &{arg_name}.to_string()),
"#).unwrap();
        } else if qa.the_type == clickhouse::ValidDbType::DateTime {
            write!(res, r#"                ("param_{arg_name}", &{arg_name}.format("%Y-%m-%d %H:%M:%S")),
"#).unwrap();
        } else {
            write!(res, r#"                ("param_{arg_name}", &{arg_name}.to_string()),
"#).unwrap();
        }
    }

    write!(res, r#"            ])
            .body(the_query)
            .send()
            .await
            .map_err(|e| {{
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            }})?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {{
            let err_text = resp.text().await.unwrap_or_default();
            span.set_status(::opentelemetry::trace::Status::error(err_text.clone()));
            return Err(ChInteractionError::HttpResponseError {{
                expected_status_code: 200,
                actual_status_code: status_code,
                body: err_text,
            }});
        }}

        let text = resp.text().await.map_err(|e| {{
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            ChInteractionError::StreamReadError(e)
        }})?;

        drop(span);
        let mut span = self.span("{span_prefix}{qname}_deser");

        let mut res: Vec<{aux_struct_name}> = Vec::new();

        let expected_tab_fields = {expected_tab_fields};
        let mut line_no = 0usize;
        for line in text.lines() {{
            let line_spl = line.split("\t").collect::<Vec<_>>();
            if line_spl.len() == expected_tab_fields {{
"#).unwrap();

    for (idx, field) in checked_query.output_signature.iter().enumerate() {
        let field_name = &field.name;
        // 1. check if optional?
        // 2. check if optional?
        let parse_type = |type_str: &str| -> String {
            if field.optional {
                format!(r#"
                let {field_name} = if line_spl[{idx}] == "\\N" {{ None }} else {{
                    Some(line_spl[{idx}].parse::<{type_str}>().map_err(|e| {{
                        span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                        ChInteractionError::ColumnParseErrorError {{
                            error: e.to_string(),
                            column_number: {idx},
                            expected_type: "{type_str}",
                            row_content: line.to_string(),
                            row_number: line_no,
                        }}
                    }})?)
                }};
"#)
            } else {
                format!(r#"
                let {field_name} = line_spl[{idx}].parse::<{type_str}>().map_err(|e| {{
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {{
                        error: e.to_string(),
                        column_number: {idx},
                        expected_type: "{type_str}",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }}
                }})?;
"#)
            }
        };
        let block = match &field.the_type {
            clickhouse::ValidDbType::Int32 => {
                parse_type("i32")
            }
            clickhouse::ValidDbType::Int64 => {
                parse_type("i64")
            }
            clickhouse::ValidDbType::Int128 => {
                parse_type("i128")
            }
            clickhouse::ValidDbType::Int256 => {
                parse_type("num256::Int256")
            }
            clickhouse::ValidDbType::Float32 => {
                parse_type("f32")
            }
            clickhouse::ValidDbType::Float64 => {
                parse_type("f64")
            }
            clickhouse::ValidDbType::Bool => {
                parse_type("bool")
            }
            clickhouse::ValidDbType::DateTime => {
                if field.optional {
                    format!(r#"
                let {field_name} = if line_spl[{idx}] == "\\N" {{ None }} else {{
                    Some(::chrono::NaiveDateTime::parse_from_str(line_spl[{idx}], "%Y-%m-%d %H:%M:%S").map_err(|e| {{
                        span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                        ChInteractionError::ColumnParseErrorError {{
                            error: e.to_string(),
                            column_number: {idx},
                            expected_type: "DateTime",
                            row_content: line.to_string(),
                            row_number: line_no,
                        }}
                    }})?.and_utc())
                }};
"#)
                } else {
                    format!(r#"
                let {field_name} = ::chrono::NaiveDateTime::parse_from_str(line_spl[{idx}], "%Y-%m-%d %H:%M:%S").map_err(|e| {{
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {{
                        error: e.to_string(),
                        column_number: {idx},
                        expected_type: "DateTime",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }}
                }})?.and_utc();
"#)
                }
            }
            clickhouse::ValidDbType::Date => {
                if field.optional {
                    format!(r#"
                let {field_name} = if line_spl[{idx}] == "\\N" {{ None }} else {{
                    Some(::chrono::NaiveDate::parse_from_str(line_spl[{idx}], "%Y-%m-%d").map_err(|e| {{
                        span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                        ChInteractionError::ColumnParseErrorError {{
                            error: e.to_string(),
                            column_number: {idx},
                            expected_type: "Date",
                            row_content: line.to_string(),
                            row_number: line_no,
                        }}
                    }})?)
                }};
"#)
                } else {
                    format!(r#"
                let {field_name} = ::chrono::NaiveDate::parse_from_str(line_spl[{idx}], "%Y-%m-%d").map_err(|e| {{
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {{
                        error: e.to_string(),
                        column_number: {idx},
                        expected_type: "Date",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }}
                }})?;
"#)
                }
            }
            clickhouse::ValidDbType::String => {
                if field.optional {
                    format!(r#"
                let {field_name} = if line_spl[{idx}] == "\\N" {{ None }} else {{
                    Some(::unescaper::unescape(line_spl[{idx}]).map_err(|e| {{
                        span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                        ChInteractionError::ColumnParseErrorError {{
                            error: e.to_string(),
                            column_number: {idx},
                            expected_type: "String",
                            row_content: line.to_string(),
                            row_number: line_no,
                        }}
                    }})?)
                }};
"#)
                } else {
                    format!(r#"
                let {field_name} = ::unescaper::unescape(line_spl[{idx}]).map_err(|e| {{
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {{
                        error: e.to_string(),
                        column_number: {idx},
                        expected_type: "String",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }}
                }})?;
"#)
                }
            }
        };
        *res += &block;
    }

    write!(res, r#"
                res.push({aux_struct_name} {{
                    {output_struct_field_names}
                }});
"#).unwrap();

    write!(res, r#"
            }} else {{
                let e = ChInteractionError::IncorrectFormatError {{
                    expected_tab_fields,
                    actual_tab_fields: res.len(),
                    row_content: line.to_string(),
                    row_number: line_no,
                }};
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                return Err(e);
            }}
            line_no += 1;
        }}

"#).unwrap();

    *res += "        ";
    *res += &prom_variable;
    *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
}

fn generate_pg_queries(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    pg_types_needed: &mut bool,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
) {
    for shard in checked
        .db
        .backend_application()
        .c_children_backend_application_pg_shard(app)
    {
        let shard_name = checked
            .db
            .backend_application_pg_shard()
            .c_shard_name(*shard);
        let queries = checked.projections.application_pg_shard_queries.value(*shard);

        for query in &queries.queries {
            *pg_types_needed = true;
            let db = checked.db.pg_query().c_parent(*query);
            let checked_db = checked.async_res.checked_pg_dbs.get(&db).unwrap();
            let checked_query = checked_db.queries.get(query).unwrap();
            let qname = checked.db.pg_query().c_query_name(*query);
            let method_prefix =
                if checked.db.pg_query().c_is_mutating(*query) {
                    "pgmq_"
                } else {
                    "pgq_"
                };

            let aux_struct_name = cgen_context.pg_query_output_type_name(checked, *query);

            *res += "\n";
            *res += "    pub async fn ";
            *res += method_prefix;
            *res += qname;
            *res += "(";

            let mut qargs = vec!["&self".to_string()];

            for fq in &checked_query.full_query.args {
                let arg_t = match fq.the_type {
                    postgres::ValidDbType::INT => "i32",
                    postgres::ValidDbType::BIGINT => "i64",
                    postgres::ValidDbType::FLOAT => "f32",
                    postgres::ValidDbType::DOUBLE => "f64",
                    postgres::ValidDbType::BOOL => "bool",
                    postgres::ValidDbType::TEXT => "&str",
                };
                qargs.push(format!("{}: {}", fq.name, arg_t));
            }

            *res += &qargs.join(", ");

            *res += ") -> Result<Vec<";
            *res += &aux_struct_name;
            *res += ">, PgInteractionError> {\n";
            *res += "        let pre = ::std::time::Instant::now();\n";
            writeln!(res, "        let mut span = self.span(\"pg_conn_get\");").unwrap();
            *res += &format!(
                r#"        let conn = self.r.pg_conn_{shard_name}.get().await.map_err(|e| {{
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                PgInteractionError::ConnectionPoolError(e)
            }})?;"#
            );
            *res += "\n";
            writeln!(res, "        drop(span);").unwrap();

            let prom_variable = format!("METRIC_PG_CONN_{}", shard_name.to_uppercase());
            cgen_context.register_prom_variable(
                prom_variable.clone(),
                prometheus_conn_pool_metric(
                    &prom_variable,
                    checked.db.backend_application().c_application_name(app),
                    shard_name.as_str(),
                ),
            );
            *res += "        ";
            *res += &prom_variable;
            *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";

            generate_pgq_block(
                checked,
                res,
                cgen_context,
                app,
                *query,
                QueryGenContext::Standalone,
                None,
            );

            *res += "        Ok(res)\n";
            *res += "    }\n";
        }
    }
}

fn generate_pg_mutators(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    pg_types_needed: &mut bool,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
) {
    for shard in checked
        .db
        .backend_application()
        .c_children_backend_application_pg_shard(app)
    {
        let shard_name = checked
            .db
            .backend_application_pg_shard()
            .c_shard_name(*shard);
        let queries = checked.projections.application_pg_shard_queries.value(*shard);

        for mutator in &queries.mutators {
            *pg_types_needed = true;

            let db = checked.db.pg_mutator().c_parent(*mutator);
            let checked_db = checked.async_res.checked_pg_dbs.get(&db).unwrap();
            let checked_mutator = checked_db.mutators.get(mutator).unwrap();
            let mutator_name = checked.db.pg_mutator().c_mutator_name(*mutator);

            *res += "\n";
            *res += "    pub async fn pgm_";
            *res += mutator_name;
            *res += "(";

            let mut qargs = vec!["&self".to_string()];

            for fq in &checked_mutator.full_query.args {
                let arg_t = match fq.the_type {
                    postgres::ValidDbType::INT => "i32",
                    postgres::ValidDbType::BIGINT => "i64",
                    postgres::ValidDbType::FLOAT => "f32",
                    postgres::ValidDbType::DOUBLE => "f64",
                    postgres::ValidDbType::BOOL => "bool",
                    postgres::ValidDbType::TEXT => "&str",
                };
                qargs.push(format!("{}: {}", fq.name, arg_t));
            }

            *res += &qargs.join(", ");

            *res += ") -> Result<u64, PgInteractionError> {\n";
            *res += "        let pre = ::std::time::Instant::now();\n";
            writeln!(res, "        let mut span = self.span(\"pg_conn_get\");").unwrap();
            *res += &format!(
                r#"        let conn = self.r.pg_conn_{shard_name}.get().await.map_err(|e| {{
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                PgInteractionError::ConnectionPoolError(e)
            }})?;"#
            );
            *res += "\n";
            writeln!(res, "        drop(span);").unwrap();

            let prom_variable = format!("METRIC_PG_CONN_{}", shard_name.to_uppercase());
            cgen_context.register_prom_variable(
                prom_variable.clone(),
                prometheus_conn_pool_metric(
                    &prom_variable,
                    checked.db.backend_application().c_application_name(app),
                    shard_name,
                ),
            );
            *res += "        ";
            *res += &prom_variable;
            *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";

            generate_pgm_block(
                checked,
                res,
                cgen_context,
                app,
                *mutator,
                QueryGenContext::Standalone,
                None,
            );

            *res += "        Ok(res)\n";
            *res += "    }\n";
        }
    }
}

fn generate_pg_transaction_scripts(
    checked: &CheckedDB,
    app: TableRowPointerBackendApplication,
    pg_types_needed: &mut bool,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
) {
    for shard in checked
        .db
        .backend_application()
        .c_children_backend_application_pg_shard(app)
    {
        let shard_name = checked
            .db
            .backend_application_pg_shard()
            .c_shard_name(*shard);
        let queries = checked.projections.application_pg_shard_queries.value(*shard);

        for trx_pointer in &queries.transactions {
            *pg_types_needed = true;

            let db = checked.db.pg_transaction().c_parent(*trx_pointer);
            let trx_name = checked.db.pg_transaction().c_transaction_name(*trx_pointer);
            let trx_struct_name = format!(
                "Trx{}{}S1",
                shard_name.to_case(Case::Pascal),
                trx_name.to_case(Case::Pascal)
            );

            *res += "\n";
            *res += "    pub async fn pgtrx_begin_";
            *res += trx_name;

            *res += "(&self) -> Result<";
            *res += &trx_struct_name;
            *res += ", PgInteractionError> {";

            // just begin the transaction and return the body
            write!(res,
                r#"
        let pre = ::std::time::Instant::now();
        let context = ::opentelemetry::Context::current_with_span(self.span("pgtrx_{trx_name}"));
        let mut span = self.r.tracer.start_with_context("pg_conn_get", &context);
        let conn = self.r.pg_conn_{shard_name}.get().await.map_err(|e| {{
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            PgInteractionError::ConnectionPoolError(e)
        }})?;
        drop(span);
"#
            ).unwrap();

            let prom_metric_name = format!("METRIC_PG_CONN_{}", shard_name.to_uppercase());
            cgen_context.register_prom_variable(
                prom_metric_name.clone(),
                prometheus_conn_pool_metric(
                    &prom_metric_name,
                    checked.db.backend_application().c_application_name(app),
                    shard_name.as_str(),
                ),
            );

            *res += "        ";
            *res += &prom_metric_name;
            *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";

            *res += r#"
        // make lifetime longer to survive builder pattern
        let conn: ::bb8::PooledConnection<'static, ::bb8_postgres::PostgresConnectionManager<::tokio_postgres::NoTls>> = unsafe {
            ::std::mem::transmute(conn)
        };
        // pin conn in memory so that transaction could rely on its location
        let mut conn = Box::new(conn);
        let trx = conn.transaction().await.map_err(|e| {
            PgInteractionError::PostgresError(e)
        })?;

        // make lifetime longer to survive builder pattern
        let trx: ::tokio_postgres::Transaction<'static> = unsafe { ::std::mem::transmute(trx) };

        let state = TransactionState {
            _r: self.r.clone(),
            _conn: conn,
            trx,
            start_time: ::std::time::Instant::now(),
            context,
        };

"#;

            *res += &format!("        Ok({} {{ state }})\n", trx_struct_name);

            *res += "    }\n";

            // generate transaction bodies
            let steps = checked.projections.transaction_steps.value(*trx_pointer);
            struct AccumulatedField {
                field_name: String,
                field_type: String,
                return_type: String,
                return_ref: bool,
            }
            let mut accumulated_state: Vec<AccumulatedField> = Vec::new(); // field name and type

            accumulated_state.push(AccumulatedField {
                field_name: "state".to_string(),
                field_type: "TransactionState".to_string(),
                return_type: "nay".to_string(),
                return_ref: false,
            });

            for (idx, step) in steps.iter().enumerate() {
                let idx = idx + 1;

                let trx_struct_name = format!(
                    "Trx{}{}S{}",
                    shard_name.to_case(Case::Pascal),
                    trx_name.to_case(Case::Pascal),
                    idx
                );
                let trx_next_struct_name = format!(
                    "Trx{}{}S{}",
                    shard_name.to_case(Case::Pascal),
                    trx_name.to_case(Case::Pascal),
                    idx + 1
                );

                let mut src = String::new();
                src += "pub struct ";
                src += &trx_struct_name;
                src += " {\n";

                for af in &accumulated_state {
                    src += "    ";
                    src += &af.field_name;
                    src += ": ";
                    src += &af.field_type;
                    src += ",\n";
                }

                if !step.is_multi {
                    match &step.query {
                        crate::static_analysis::databases::postgres::DbQueryOrMutator::Query(q) => {
                            let q_output_field_name =
                                format!("r_{}", checked.db.pg_query().c_query_name(*q));
                            let q_output_type = format!(
                                "Vec<{}>",
                                cgen_context.pg_query_output_type_name(checked, *q)
                            );
                            let q_return_type =
                                format!("&[{}]", cgen_context.pg_query_output_type_name(checked, *q));
                            accumulated_state.push(AccumulatedField {
                                field_name: q_output_field_name,
                                field_type: q_output_type,
                                return_type: q_return_type,
                                return_ref: true,
                            });
                        }
                        crate::static_analysis::databases::postgres::DbQueryOrMutator::Mutator(m) => {
                            let q_output_field_name =
                                format!("r_{}", checked.db.pg_mutator().c_mutator_name(*m));
                            accumulated_state.push(AccumulatedField {
                                field_name: q_output_field_name,
                                field_type: "u64".to_string(),
                                return_type: "u64".to_string(),
                                return_ref: false,
                            });
                        }
                    }
                }

                src += "}\n";
                src += "\n";

                src += "impl ";
                src += &trx_struct_name;
                src += " {\n";

                let new_fname = match &step.query {
                    crate::static_analysis::databases::postgres::DbQueryOrMutator::Query(q) => {
                        if checked.db.pg_query().c_is_mutating(*q) {
                            src += "    pub async fn pgmq_";
                        } else {
                            src += "    pub async fn pgq_";
                        }
                        src += checked.db.pg_query().c_query_name(*q);
                        src += "(";

                        let mut qargs = Vec::new();
                        if !step.is_multi {
                            qargs.push("self".to_string());
                        } else {
                            qargs.push("&mut self".to_string());
                        }

                        let checked_query = checked
                            .async_res
                            .checked_pg_dbs
                            .get(&db)
                            .unwrap()
                            .queries
                            .get(q)
                            .unwrap();

                        for fq in &checked_query.full_query.args {
                            let arg_t = match fq.the_type {
                                postgres::ValidDbType::INT => "i32",
                                postgres::ValidDbType::BIGINT => "i64",
                                postgres::ValidDbType::FLOAT => "f32",
                                postgres::ValidDbType::DOUBLE => "f64",
                                postgres::ValidDbType::BOOL => "bool",
                                postgres::ValidDbType::TEXT => "&str",
                            };
                            qargs.push(format!("{}: {}", fq.name, arg_t));
                        }

                        src += &qargs.join(", ");

                        src += ") -> Result<";
                        if !step.is_multi {
                            src += &trx_next_struct_name;
                        } else {
                            let aux_struct_name = cgen_context.pg_query_output_type_name(checked, *q);
                            src += "Vec<";
                            src += &aux_struct_name;
                            src += ">";
                        };
                        src += ", PgInteractionError> {\n";

                        generate_pgq_block(
                            checked,
                            &mut src,
                            cgen_context,
                            app,
                            *q,
                            QueryGenContext::InTransaction,
                            Some(trx_name.as_str()),
                        );

                        src += "\n";

                        let new_fname = format!("r_{}", checked.db.pg_query().c_query_name(*q));
                        if !step.is_multi {
                            src += "        Ok(";
                            src += &trx_next_struct_name;
                            src += " {\n";

                            src += "            ";
                            src += &new_fname;
                            src += ": res,\n";

                            for af in &accumulated_state {
                                if af.field_name == new_fname {
                                    continue;
                                }

                                src += "            ";
                                src += &af.field_name;
                                src += ": self.";
                                src += &af.field_name;
                                src += ",\n";
                            }

                            src += "        })\n";
                        } else {
                            src += "        Ok(res)\n";
                        }

                        src += "    }\n";

                        if step.is_multi {
                            src += "\n";
                            src += "    pub fn advance(self) -> ";
                            src += &trx_next_struct_name;
                            src += " {\n";
                            src += "        ";
                            src += &trx_next_struct_name;
                            src += " {\n";
                            for af in &accumulated_state {
                                src += "            ";
                                src += &af.field_name;
                                src += ": self.";
                                src += &af.field_name;
                                src += ",\n";
                            }
                            src += "        }\n";
                            src += "    }\n";
                        }

                        src += "\n";

                        new_fname
                    }
                    crate::static_analysis::databases::postgres::DbQueryOrMutator::Mutator(m) => {
                        src += "    pub async fn pgm_";
                        src += checked.db.pg_mutator().c_mutator_name(*m);
                        src += "(";

                        let mut qargs = Vec::new();
                        if !step.is_multi {
                            qargs.push("self".to_string());
                        } else {
                            qargs.push("&mut self".to_string());
                        }

                        let checked_mutator = checked
                            .async_res
                            .checked_pg_dbs
                            .get(&db)
                            .unwrap()
                            .mutators
                            .get(m)
                            .unwrap();

                        for fq in &checked_mutator.full_query.args {
                            let arg_t = match fq.the_type {
                                postgres::ValidDbType::INT => "i32",
                                postgres::ValidDbType::BIGINT => "i64",
                                postgres::ValidDbType::FLOAT => "f32",
                                postgres::ValidDbType::DOUBLE => "f64",
                                postgres::ValidDbType::BOOL => "bool",
                                postgres::ValidDbType::TEXT => "&str",
                            };
                            qargs.push(format!("{}: {}", fq.name, arg_t));
                        }

                        src += &qargs.join(", ");

                        src += ") -> Result<";
                        if !step.is_multi {
                            src += &trx_next_struct_name;
                        } else {
                            src += "u64";
                        }
                        src += ", PgInteractionError> {\n";

                        generate_pgm_block(
                            checked,
                            &mut src,
                            cgen_context,
                            app,
                            *m,
                            QueryGenContext::InTransaction,
                            Some(trx_name.as_str()),
                        );

                        src += "\n";

                        let new_fname = format!("r_{}", checked.db.pg_mutator().c_mutator_name(*m));
                        if !step.is_multi {
                            src += "        Ok(";
                            src += &trx_next_struct_name;
                            src += " {\n";

                            src += "            ";
                            src += &new_fname;
                            src += ": res,\n";

                            for af in &accumulated_state {
                                if af.field_name == new_fname {
                                    continue;
                                }

                                src += "            ";
                                src += &af.field_name;
                                src += ": self.";
                                src += &af.field_name;
                                src += ",\n";
                            }

                            src += "        })\n";
                        } else {
                            src += "        Ok(res)\n";
                        }

                        src += "    }\n";

                        if step.is_multi {
                            src += "\n";
                            src += "    pub fn advance(self) -> ";
                            src += &trx_next_struct_name;
                            src += " {\n";
                            src += "        ";
                            src += &trx_next_struct_name;
                            src += " {\n";
                            for af in &accumulated_state {
                                src += "            ";
                                src += &af.field_name;
                                src += ": self.";
                                src += &af.field_name;
                                src += ",\n";
                            }
                            src += "        }\n";
                            src += "    }\n";
                        }

                        src += "\n";

                        new_fname
                    }
                };

                for as_field in &accumulated_state {
                    if as_field.field_name == new_fname || as_field.field_name == "state" {
                        continue;
                    }

                    src += "    #[allow(dead_code)]\n";
                    src += "    pub fn ";
                    src += &as_field.field_name;
                    src += "(&self) -> ";
                    src += &as_field.return_type;
                    src += " {\n";
                    src += "        ";
                    if as_field.return_ref {
                        src += "&";
                    }
                    src += "self.";
                    src += &as_field.field_name;
                    src += "\n";
                    src += "    }\n";
                }

                // 1. get all the variables of this state
                // 2. consuming of advancing to next step or commit if last

                src += "}\n";

                cgen_context.transaction_script_types.push(src);
            }

            // generate last step
            {
                let mut src = String::new();

                let commit_struct_name = format!(
                    "Trx{}{}S{}",
                    shard_name.to_case(Case::Pascal),
                    trx_name.to_case(Case::Pascal),
                    steps.len() + 1
                );
                let output_struct_name = format!(
                    "Trx{}{}Output",
                    shard_name.to_case(Case::Pascal),
                    trx_name.to_case(Case::Pascal)
                );
                src += "pub struct ";
                src += &commit_struct_name;
                src += " {\n";

                for af in &accumulated_state {
                    src += "    ";
                    src += &af.field_name;
                    src += ": ";
                    src += &af.field_type;
                    src += ",\n";
                }

                src += "}\n";
                src += "\n";

                src += "pub struct ";
                src += &output_struct_name;
                src += " {\n";

                for af in &accumulated_state {
                    if af.field_name == "state" {
                        continue;
                    }

                    src += "    pub ";
                    src += &af.field_name;
                    src += ": ";
                    src += &af.field_type;
                    src += ",\n";
                }

                src += "}\n";
                src += "\n";

                src += "impl ";
                src += &commit_struct_name;
                src += " {\n";

                let prom_metric_name = format!(
                    "METRIC_PG_TRX_{}_{}",
                    shard_name.to_uppercase(),
                    trx_name.to_uppercase()
                );
                cgen_context.register_prom_variable(
                    prom_metric_name.clone(),
                    prometheus_transaction_metric(
                        &prom_metric_name,
                        checked.db.backend_application().c_application_name(app),
                        shard_name,
                        trx_name,
                        checked.db.pg_transaction().c_is_read_only(*trx_pointer),
                    ),
                );

                src += &format!(
                    r#"
    pub async fn commit(self) -> Result<{}, PgInteractionError> {{
        let _ = self.state.trx.commit().await.map_err(|e| {{
            PgInteractionError::PostgresError(e)
        }})?;

        {}.observe((::std::time::Instant::now() - self.state.start_time).as_secs_f64());

        Ok({} {{
"#,
                    output_struct_name, prom_metric_name, output_struct_name
                );

                for as_field in &accumulated_state {
                    if as_field.field_name == "state" {
                        continue;
                    }

                    src += "            ";
                    src += &as_field.field_name;
                    src += ": self.";
                    src += &as_field.field_name;
                    src += ",\n";
                }

                src += "        })
    }
";

                for as_field in &accumulated_state {
                    if as_field.field_name == "state" {
                        continue;
                    }

                    src += "    #[allow(dead_code)]\n";
                    src += "    pub fn ";
                    src += &as_field.field_name;
                    src += "(&self) -> ";
                    src += &as_field.return_type;
                    src += " {\n";
                    src += "        ";
                    if as_field.return_ref {
                        src += "&";
                    }
                    src += "self.";
                    src += &as_field.field_name;
                    src += "\n";
                    src += "    }\n";
                }

                src += "}\n";

                cgen_context.transaction_script_types.push(src);
            }
        }
    }
}

enum QueryGenContext {
    Standalone,
    InTransaction,
}

impl QueryGenContext {
    fn conn_object(&self) -> &'static str {
        match self {
            QueryGenContext::Standalone => "conn",
            QueryGenContext::InTransaction => "self.state.trx",
        }
    }

    fn prom_var_mutator_prefix(&self) -> &'static str {
        match self {
            QueryGenContext::Standalone => "PGM",
            QueryGenContext::InTransaction => "TRX_PGM",
        }
    }

    fn prom_var_query_prefix(&self) -> &'static str {
        match self {
            QueryGenContext::Standalone => "PGQ",
            QueryGenContext::InTransaction => "TRX_PGQ",
        }
    }
}

fn generate_pgq_block(
    checked: &CheckedDB,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
    app: TableRowPointerBackendApplication,
    query: TableRowPointerPgQuery,
    context: QueryGenContext,
    transaction: Option<&str>,
) {
    let db = checked.db.pg_query().c_parent(query);
    let checked_query = checked
        .async_res
        .checked_pg_dbs
        .get(&db)
        .unwrap()
        .queries
        .get(&query)
        .unwrap();
    let qname = checked.db.pg_query().c_query_name(query);
    let aux_struct_name = cgen_context.pg_query_output_type_name(checked, query);
    let span_prefix =
        if checked.db.pg_query().c_is_mutating(query) {
            "pgmq_"
        } else { "pgq_" };

    *res += "        let the_query = r#\"";
    assert!(
        !checked_query
            .full_query
            .interpolated_expression
            .contains("r#\"")
            && !checked_query
                .full_query
                .interpolated_expression
                .contains("\"#")
    );
    *res += &checked_query.full_query.interpolated_expression;
    *res += "\"#;\n";
    *res += "        let pre = ::std::time::Instant::now();\n";
    match context {
        QueryGenContext::Standalone => {
            writeln!(res, "        let mut span = self.span(\"{span_prefix}{qname}\");").unwrap();
        }
        QueryGenContext::InTransaction => {
            writeln!(res, "        let mut span = self.state._r.tracer.start_with_context(\"{span_prefix}{qname}\", &self.state.context);").unwrap();
        }
    }
    *res += "        let rows = ";
    *res += context.conn_object();
    *res += ".query(the_query, &[";
    let mut qargs2 = Vec::new();
    for fq in &checked_query.full_query.args {
        let arg_n = match fq.the_type {
            postgres::ValidDbType::INT => format!("&{}", fq.name),
            postgres::ValidDbType::BIGINT => format!("&{}", fq.name),
            postgres::ValidDbType::FLOAT => format!("&{}", fq.name),
            postgres::ValidDbType::DOUBLE => format!("&{}", fq.name),
            postgres::ValidDbType::BOOL => format!("&{}", fq.name),
            postgres::ValidDbType::TEXT => fq.name.clone(),
        };
        qargs2.push(arg_n);
    }
    *res += &qargs2.join(", ");
    *res += "]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;\n";
    *res += "        let mut res = Vec::with_capacity(rows.len());\n";
    let prom_variable = format!(
        "METRIC_{}_{}_{}",
        context.prom_var_query_prefix(),
        checked.db.pg_schema().c_schema_name(db).to_uppercase(),
        qname.to_uppercase()
    );
    cgen_context.register_prom_variable(
        prom_variable.clone(),
        prometheus_pg_query_metric(
            &prom_variable,
            checked.db.backend_application().c_application_name(app),
            checked.db.pg_schema().c_schema_name(db),
            qname,
            true,
            checked.db.pg_query().c_is_mutating(query),
            transaction,
        ),
    );
    *res += "        ";
    *res += &prom_variable;
    *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
    *res += "        for r in rows {\n";
    *res += "            res.push(";
    *res += &aux_struct_name;
    *res += " {\n";
    for (idx, oc) in checked_query.output_signature.iter().enumerate() {
        *res += "                ";
        *res += &oc.name;
        *res += ": r.try_get::<usize, ";
        if oc.optional {
            *res += "Option<";
        }
        *res += super::pg_db_type_to_rust_type(&oc.the_type);
        if oc.optional {
            *res += ">";
        }
        *res += ">(";
        *res += &idx.to_string();
        *res += ").map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::DeserializationError(e.to_string()) })?,\n";
    }
    *res += "            });\n";
    *res += "        }\n";
}

fn generate_pgm_block(
    checked: &CheckedDB,
    res: &mut String,
    cgen_context: &mut RustCodegenContext,
    app: TableRowPointerBackendApplication,
    mutator: TableRowPointerPgMutator,
    context: QueryGenContext,
    transaction: Option<&str>,
) {
    let db = checked.db.pg_mutator().c_parent(mutator);
    let checked_db = checked.async_res.checked_pg_dbs.get(&db).unwrap();
    let checked_mutator = checked_db.mutators.get(&mutator).unwrap();
    let mutator_name = checked.db.pg_mutator().c_mutator_name(mutator);

    *res += "        let the_query = r#\"";
    assert!(
        !checked_mutator
            .full_query
            .interpolated_expression
            .contains("r#\"")
            && !checked_mutator
                .full_query
                .interpolated_expression
                .contains("\"#")
    );
    *res += &checked_mutator.full_query.interpolated_expression;
    *res += "\"#;\n";
    *res += "        let pre = ::std::time::Instant::now();\n";

    match context {
        QueryGenContext::Standalone => {
            writeln!(res, "        let mut span = self.span(\"pgm_{mutator_name}\");").unwrap();
        }
        QueryGenContext::InTransaction => {
            writeln!(res, "        let mut span = self.state._r.tracer.start_with_context(\"pgm_{mutator_name}\", &self.state.context);").unwrap();
        }
    }

    *res += "        let res = ";
    *res += context.conn_object();
    *res += ".execute(the_query, &[";
    let mut qargs2 = Vec::new();
    for fq in &checked_mutator.full_query.args {
        let arg_n = match fq.the_type {
            postgres::ValidDbType::INT => format!("&{}", fq.name),
            postgres::ValidDbType::BIGINT => format!("&{}", fq.name),
            postgres::ValidDbType::FLOAT => format!("&{}", fq.name),
            postgres::ValidDbType::DOUBLE => format!("&{}", fq.name),
            postgres::ValidDbType::BOOL => format!("&{}", fq.name),
            postgres::ValidDbType::TEXT => fq.name.clone(),
        };
        qargs2.push(arg_n);
    }
    *res += &qargs2.join(", ");
    *res += "]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;\n";
    let prom_variable = format!(
        "METRIC_{}_{}_{}",
        context.prom_var_mutator_prefix(),
        checked.db.pg_schema().c_schema_name(db).to_uppercase(),
        mutator_name.to_uppercase()
    );
    cgen_context.register_prom_variable(
        prom_variable.clone(),
        prometheus_pg_query_metric(
            &prom_variable,
            checked.db.backend_application().c_application_name(app),
            checked.db.pg_schema().c_schema_name(db),
            mutator_name,
            false,
            true,
            transaction,
        ),
    );
    *res += "        ";
    *res += &prom_variable;
    *res += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
}

fn prometheus_conn_pool_metric(var_name: &str, app_name: &str, database_name: &str) -> String {
    format!(
        r#"
    static ref {var_name}: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_connpool_time",
        "Time for acquiring database connection from connection pool",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {{
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "{app_name}".to_string(),
            "database".to_string() => "{database_name}".to_string(),
        }},
    )).unwrap();
"#
    )
}

fn prometheus_ch_query_metric(
    var_name: &str,
    app_name: &str,
    shard_name: &str,
    schema_name: &str,
    qname: &str,
    is_query: bool,
    is_inserter: bool,
) -> String {
    format!(
        r#"
    static ref {var_name}: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_ch_query_time",
        "Clickhouse query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {{
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "{app_name}".to_string(),
            "application_ch_shard".to_string() => "{shard_name}".to_string(),
            "application_ch_schema".to_string() => "{schema_name}".to_string(),
            "query".to_string() => "{qname}".to_string(),
            "is_query".to_string() => "{is_query}".to_string(),
            "is_inserter".to_string() => "{is_inserter}".to_string(),
        }},
    )).unwrap();
"#
    )
}

fn prometheus_pg_query_metric(
    var_name: &str,
    app_name: &str,
    database_name: &str,
    qname: &str,
    is_query: bool,
    is_mutating: bool,
    transaction: Option<&str>,
) -> String {
    let trx = transaction.unwrap_or("none");
    format!(
        r#"
    static ref {var_name}: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_pg_query_time",
        "Postgres query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {{
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "{app_name}".to_string(),
            "pg_database".to_string() => "{database_name}".to_string(),
            "query".to_string() => "{qname}".to_string(),
            "is_query".to_string() => "{is_query}".to_string(),
            "is_mutating".to_string() => "{is_mutating}".to_string(),
            "transaction".to_string() => "{trx}".to_string(),
        }},
    )).unwrap();
"#
    )
}

fn prometheus_transaction_metric(
    var_name: &str,
    app_name: &str,
    database_name: &str,
    transaction: &str,
    is_read_only: bool,
) -> String {
    format!(
        r#"
    static ref {var_name}: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_transaction_time",
        "Database transaction time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {{
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "{app_name}".to_string(),
            "database".to_string() => "{database_name}".to_string(),
            "transaction".to_string() => "{transaction}".to_string(),
            "is_read_only".to_string() => "{is_read_only}".to_string(),
        }},
    )).unwrap();
"#
    )
}

fn prometheus_nats_processor_latency_metric(
    var_name: &str,
    app_name: &str,
    app_stream: &str,
) -> String {
    let nats_stream = app_stream.to_uppercase();
    format!(
        r#"
    static ref {var_name}: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_nats_js_message_process_time",
        "Time in which nats message was processed for stream",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {{
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "{app_name}".to_string(),
            "nats_stream".to_string() => EPL_NATS_STREAM_{nats_stream}.clone(),
            "app_stream".to_string() => "{app_stream}".to_string(),
        }},
    )).unwrap();
"#
    )
}

fn prometheus_nats_publish_latency_metric(
    var_name: &str,
    app_name: &str,
    app_stream: &str,
) -> String {
    let nats_stream = app_stream.to_uppercase();
    format!(
        r#"
    static ref {var_name}: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_nats_js_publish_time",
        "Time in which nats message was published to stream",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {{
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "{app_name}".to_string(),
            "nats_stream".to_string() => EPL_NATS_STREAM_{nats_stream}.clone(),
            "app_stream".to_string() => "{app_stream}".to_string(),
        }},
    )).unwrap();
"#
    )
}

fn prometheus_nats_bytes_published_metric(
    var_name: &str,
    app_name: &str,
    app_stream: &str,
) -> String {
    let nats_stream = app_stream.to_uppercase();
    format!(
        r#"
    static ref {var_name}: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_nats_js_published_bytes",
        "Bytes sent to nats stream",
        ::prometheus::labels! {{
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "{app_name}",
            "nats_stream" => EPL_NATS_STREAM_{nats_stream}.as_str(),
            "app_stream" => "{app_stream}",
        }},
    )).unwrap();
"#
    )
}

fn prometheus_nats_bytes_processed_metric(
    var_name: &str,
    app_name: &str,
    app_stream: &str,
) -> String {
    let nats_stream = app_stream.to_uppercase();
    format!(
        r#"
    static ref {var_name}: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_nats_js_processed_bytes",
        "Bytes processed successfully from nats stream",
        ::prometheus::labels! {{
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "{app_name}",
            "nats_stream" => EPL_NATS_STREAM_{nats_stream}.as_str(),
            "app_stream" => "{app_stream}",
        }},
    )).unwrap();
"#
    )
}

fn prometheus_http_latency_endpoint_metric(
    var_name: &str,
    app_name: &str,
    endpoint_name: &str,
    http_method: &str,
    http_path: &str,
) -> String {
    format!(
        r#"
    static ref {var_name}: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {{
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "{app_name}".to_string(),
            "endpoint_name".to_string() => "{endpoint_name}".to_string(),
            "http_method".to_string() => "{http_method}".to_string(),
            "http_path".to_string() => "{http_path}".to_string(),
        }},
    )).unwrap();
"#
    )
}

fn prometheus_http_errors_metric(
    var_name: &str,
    app_name: &str,
    endpoint_name: &str,
    http_method: &str,
    http_path: &str,
) -> String {
    format!(
        r#"
    static ref {var_name}: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {{
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "{app_name}",
            "endpoint_name" => "{endpoint_name}",
            "http_method" => "{http_method}",
            "http_path" => "{http_path}",
        }},
    )).unwrap();
"#
    )
}

fn prometheus_http_bytes_served_metric(
    var_name: &str,
    app_name: &str,
    endpoint_name: &str,
    http_method: &str,
    http_path: &str,
) -> String {
    format!(
        r#"
    static ref {var_name}: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {{
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "{app_name}",
            "endpoint_name" => "{endpoint_name}",
            "http_method" => "{http_method}",
            "http_path" => "{http_path}",
        }},
    )).unwrap();
"#
    )
}

fn prometheus_metrics_endpoint() -> &'static str {
    r#"
#[::actix_web::get("/metrics")]
async fn prometheus_metrics_endpoint() -> ::actix_web::HttpResponse {
    use ::prometheus::Encoder;
    let encoder = ::prometheus::TextEncoder::new();
    let metric_families = ::prometheus::gather();
    let mut buffer = Vec::with_capacity(128);
    encoder.encode(&metric_families, &mut buffer).unwrap();
    ::actix_web::HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain; version=0.0.4"))
        .body(buffer)
}
"#
}

fn pg_interaction_error_types() -> &'static str {
    r#"
#[derive(Debug)]
pub enum PgInteractionError {
    PostgresError(::tokio_postgres::Error),
    ConnectionPoolError(::bb8::RunError<::tokio_postgres::Error>),
    DeserializationError(String),
}

impl std::fmt::Display for PgInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Postgres interaction error: {:?}", self)
    }
}

impl std::error::Error for PgInteractionError {}

"#
}

fn ch_interaction_error_types() -> &'static str {
    r#"
#[derive(Debug)]
pub enum ChInteractionError {
    HttpError(::reqwest::Error),
    HttpResponseError {
        expected_status_code: u16,
        actual_status_code: u16,
        body: String,
    },
    StreamReadError(::reqwest::Error),
    IncorrectFormatError {
        expected_tab_fields: usize,
        actual_tab_fields: usize,
        row_number: usize,
        row_content: String,
    },
    ColumnParseErrorError {
        error: String,
        expected_type: &'static str,
        column_number: u32,
        row_number: usize,
        row_content: String,
    },
}

impl std::fmt::Display for ChInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Clickhouse interaction error: {:?}", self)
    }
}

impl std::error::Error for ChInteractionError {}

"#
}

fn binary_deserialization_error_types() -> &'static str {
    r#"
#[derive(Debug)]
pub enum BinaryDeserializationError {
    MessageTooShort, // message too short, can't get version header
    UnsupportedVersionYet(u16), // higher than supported version, we can change
    UnknownVersion(u16), // higher than supported version, we can change
    VersionHashMismatch { expected: u64, actual: u64 }, // version there but doesn't match what is expected
    CorruptedData, // cannot deserialize
    ExtraBytesLeft,
}

impl std::fmt::Display for BinaryDeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "binary deserialization error: {:?}", self)
    }
}

impl std::error::Error for BinaryDeserializationError {}

"#
}

fn eden_platform_http_lib() -> &'static str {
    r#"
#[derive(Debug, ::derive_more::Display)]
enum EdenPlatformHttpError {
    #[display(fmt = "Cannot parse query argument.")]
    CannotParseQueryArgument,
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError(String),
    #[display(fmt = "Too big input payload size.")]
    InputBodySizeTooBig,
    #[display(fmt = "Error reading input payload")]
    InputPayloadReadError(String),
    #[display(fmt = "Cannot deserialize json message")]
    InputPayloadJsonDeserializationError(JsonDeserializationError),
    #[display(fmt = "Error pushing stream to the handler")]
    StreamsError(String),
}

impl std::error::Error for EdenPlatformHttpError {
}

impl actix_web::error::ResponseError for EdenPlatformHttpError {
    fn error_response(&self) -> ::actix_web::HttpResponse {
        ::actix_web::HttpResponse::build(self.status_code())
            .insert_header(::actix_web::http::header::ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> ::actix_web::http::StatusCode {
        match *self {
            EdenPlatformHttpError::CannotParseQueryArgument => ::actix_web::http::StatusCode::BAD_REQUEST,
            EdenPlatformHttpError::InputPayloadJsonDeserializationError(_) => ::actix_web::http::StatusCode::BAD_REQUEST,
            EdenPlatformHttpError::InputBodySizeTooBig => ::actix_web::http::StatusCode::PAYLOAD_TOO_LARGE,
            EdenPlatformHttpError::InternalError(_) => ::actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            EdenPlatformHttpError::InputPayloadReadError(_) => ::actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            EdenPlatformHttpError::StreamsError(_) => ::actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn fetch_body_with_limit(mut payload: ::actix_web::web::Payload, limit: usize) -> Result<::actix_web::web::BytesMut, EdenPlatformHttpError> {
    use ::futures_util::StreamExt;

    let mut body = ::actix_web::web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|e| {
            EdenPlatformHttpError::InputPayloadReadError(e.to_string())
        })?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > limit {
            return Err(EdenPlatformHttpError::InputBodySizeTooBig);
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

async fn dump_body_to_channel(
    payload: &mut ::actix_web::web::Payload,
    tx: ::tokio::sync::mpsc::Sender<Result<Vec<u8>, ::actix_web::error::PayloadError>>
) -> Result<(), EdenPlatformHttpError>
{
    use ::futures_util::StreamExt;
    while let Some(chunk) = payload.next().await {
        tx.send(chunk.map(|i| i.to_vec())).await.map_err(|e| {
            EdenPlatformHttpError::StreamsError(e.to_string())
        })?;
    }
    Ok(())
}
"#
}

fn generate_rust_backend_flake(app_name: &str, nixpkgs_rev: &str) -> String {
    let mut res = format!(
        r#"{{
  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs?rev={nixpkgs_rev}";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils?rev=5aed5285a952e0b949eb3ba02c12fa4fcfef535f";
  }};

  outputs = {{ self, nixpkgs, crane, flake-utils, ... }}:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {{ inherit system; }};
        craneLib = crane.lib.${{system}};
"#
    );

    res += "        appName = \"";
    res += app_name;
    res += "\";\n";

    res += r#"
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          # Additional arguments specific to this derivation can be added here.
          # Be warned that using `//` will not do a deep copy of nested
          # structures
          pname = "epl-rust";
          version = "0.1.0";
        });

        myCrate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        imageHash = pkgs.lib.head (pkgs.lib.strings.splitString "-" (baseNameOf myCrate.outPath));

        dockerImage = pkgs.dockerTools.buildImage {
          name = appName;
          tag = "v${cargoArtifacts.version}-${imageHash}";
          config = { Entrypoint = [ "${myCrate}/bin/epl-app" ]; };
        };
      in
    {
      packages.default = dockerImage;

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          clippy
          trunk
          rust-analyzer
        ];
      };
    });
}
"#;

    res
}

fn generated_main_function(db: &CheckedDB, app: TableRowPointerBackendApplication) -> String {
    let mut res = String::with_capacity(256);
    res += r#"
pub fn generated_main() {
    ::json_env_logger2::init();
    let threads = ::std::env::var("EPL_THREADS").map(|t| t.parse::<usize>().unwrap()).unwrap_or(4);
    let rt = ::tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .expect("Cannot create tokio runtime");

    rt.block_on(async {
        let _ = init_tracer();
        let http_socket = ::std::env::var("EPL_HTTP_SOCKET").unwrap_or_else(|_| "127.0.0.1:7777".to_string());

        let app_implementation = ::std::sync::Arc::new(crate::implementation::AppImplementation::new());
        let resources = match crate::generated::AppResources::new().await {
            Ok(r) => ::std::sync::Arc::new(r),
            Err(e) => {
                ::log::error!("Failed to initialize all resources, exiting app: {:?}", e);
                ::std::process::exit(7);
            }
        };
"#;

    if !db
        .db
        .backend_application()
        .c_children_backend_application_nats_stream(app)
        .is_empty()
    {
        res += "
        let servicer_api = crate::generated::AppServicerApi {
            r: resources.clone(),
            app_implementation: app_implementation.clone(),
        };
        if let Err(e) = setup_jetstream_consumers_and_publishers(servicer_api).await {
            ::log::error!(\"Failed to schedule jetstream consumers, exiting: {}\", e.to_string());
            ::std::process::exit(7);
        }";
    }

    let bg_jobs = db.db.backend_application().c_children_backend_application_background_job(app);
    if !bg_jobs.is_empty() {
        res += "\n";
        res += "        // background jobs";
        for bg_job in bg_jobs {
            let bg_job_name = db.db.backend_application_background_job().c_job_name(*bg_job);
            write!(&mut res, r#"
        let bg_job_impl = app_implementation.clone();
        let bg_job_servicer_api = crate::generated::AppServicerApi {{
            r: resources.clone(),
            app_implementation: bg_job_impl.clone(),
        }};
        ::tokio::spawn(async move {{
            let bg_job_api = crate::generated::build_app_api_no_context(&bg_job_servicer_api);
            if let Err(e) = bg_job_impl.bg_job_{bg_job_name}(bg_job_api).await {{
                ::kv_log_macro::error!("Failure when running background job", {{ error: format!("\"{{}}\"", e).as_str() }})
            }}
        }});
"#).unwrap();
        }
    }

    res += r#"
        ::actix_web::HttpServer::new(move || {
            let servicer_api = crate::generated::AppServicerApi {
                r: resources.clone(),
                app_implementation: app_implementation.clone(),
            };

            ::actix_web::App::new()
                .app_data(::actix_web::web::PayloadConfig::new(1024*1024*1024*1024*1024))
                .app_data(::actix_web::web::Data::new(servicer_api))
                .service(crate::generated::prometheus_metrics_endpoint)
"#;

    for endpoint in db
        .db
        .backend_application()
        .c_children_backend_http_endpoint(app)
    {
        res += "                .service(crate::generated::http_endpoint_";
        res += db
            .db
            .backend_http_endpoint()
            .c_http_endpoint_name(*endpoint);
        res += ")\n";
    }

    res += r#"        })
        .workers(threads)
        .bind(http_socket)?
        .run()
        .await
    }).expect("Failure running tokio runtime");
}
"#;

    res
}

// It was so painful setting this up. How can something
// that just send traces periodically be so horribly complex?
fn generate_tracing_hemmarhoids(epl_app_name: &str) -> String {
    assert!(!epl_app_name.contains('"'));
    format!(r#"
fn init_tracer() -> ::opentelemetry_sdk::trace::Tracer {{
    let config =
        ::opentelemetry_sdk::trace::Config::default()
        .with_id_generator(::opentelemetry_sdk::trace::RandomIdGenerator::default())
        .with_resource(otel_resource());

    ::opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(config)
        .with_batch_config(::opentelemetry_sdk::trace::BatchConfig::default())
        .with_exporter(::opentelemetry_otlp::new_exporter().tonic())
        .install_batch(::opentelemetry_sdk::runtime::Tokio)
        .unwrap()
}}

fn otel_resource() -> ::opentelemetry_sdk::Resource {{
    use opentelemetry_semantic_conventions::{{
        resource::{{SERVICE_NAME, SERVICE_INSTANCE_ID}},
        SCHEMA_URL,
    }};
    ::opentelemetry_sdk::Resource::from_schema_url(
        [
            ::opentelemetry::KeyValue::new(SERVICE_NAME, ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable")),
            ::opentelemetry::KeyValue::new(SERVICE_INSTANCE_ID, ::std::env::var("NOMAD_ALLOC_ID").expect("Mandatory environment variable")),
            ::opentelemetry::KeyValue::new("epl.appname", "{epl_app_name}"),
        ],
        SCHEMA_URL,
    )
}}

"#)
}

fn generate_jetstream_setup_function(
    cgen_context: &mut RustCodegenContext,
    db: &CheckedDB,
    app: TableRowPointerBackendApplication,
) -> String {
    let mut res = String::new();

    if !db
        .db
        .backend_application()
        .c_children_backend_application_nats_stream(app)
        .is_empty()
    {
        let mut initialized_streams: HashSet<String> = HashSet::new();
        let app_name = db.db.backend_application().c_application_name(app);

        res += r#"
async fn setup_jetstream_consumers_and_publishers(servicer_data: AppServicerApi) -> Result<(), ::async_nats::Error> {
    use ::futures_util::StreamExt;
"#;

        for producer in db
            .db
            .backend_application()
            .c_children_backend_application_nats_stream(app)
        {
            if !db
                .db
                .backend_application_nats_stream()
                .c_enable_producer(*producer)
            {
                continue;
            }

            let stream_name = db
                .db
                .backend_application_nats_stream()
                .c_stream_name(*producer);
            if !initialized_streams.contains(stream_name) {
                res += &format!(
                    r#"
    // initialize not yet initialized producer stream
    let _ = servicer_data.r.nats_conns[servicer_data.r.nats_conn_id_{stream_name}].get_or_create_stream(::async_nats::jetstream::stream::Config {{
        name: servicer_data.r.nats_stream_{stream_name}.clone(),
        ..Default::default()
    }}).await?;"#
                );
            }
        }

        for stream in db
            .db
            .backend_application()
            .c_children_backend_application_nats_stream(app)
        {
            if !db
                .db
                .backend_application_nats_stream()
                .c_enable_consumer(*stream)
            {
                continue;
            }

            let stream_name = db
                .db
                .backend_application_nats_stream()
                .c_stream_name(*stream);

            let stream_type = db
                .db
                .backend_application_nats_stream()
                .c_stream_type(*stream);

            let stream_type_name = db
                .db
                .versioned_type().c_type_name(stream_type)
                .to_case(convert_case::Case::Snake);

            let enable_subjects = db.db.backend_application_nats_stream().c_enable_subjects(*stream);

            initialized_streams.insert(stream_name.clone());

            res += r#"
    {
"#;
            res += "        ";
            res += "let stream_name = &servicer_data.r.nats_stream_";
            res += stream_name;
            res += ";\n";

            res += "        ";
            res += "let consumer_name = &servicer_data.r.deployment_name;\n";

            write!(res,
                r#"
        let stream = servicer_data.r.nats_conns[servicer_data.r.nats_conn_id_{}].get_or_create_stream(::async_nats::jetstream::stream::Config {{
            name: stream_name.clone(),
            ..Default::default()
        }}).await?;"#,
                stream_name
            ).unwrap();

            write!(res, r#"
        let consumer = stream.get_or_create_consumer(consumer_name, ::async_nats::jetstream::consumer::pull::Config {{
            durable_name: Some(consumer_name.clone()),
            ..Default::default()
        }}).await?;
"#).unwrap();

            if enable_subjects {
                write!(res, r#"
        let stream_name = stream_name.clone();
"#).unwrap();
            }

            let maybe_subject_parse =
                if enable_subjects {
                    r#"
                                            let stripped_subject = msg.subject.as_str().strip_prefix(stream_name.as_str()).expect("Can't strip subject name");
                                            assert!(stripped_subject.len() > 0);
                                            let subject = &stripped_subject[1..];"#
                } else { "" };

            write!(res, r#"

        tokio::spawn(async move {{
            loop {{
                match consumer.messages().await {{
                    Ok(mut messages) => {{
                        loop {{
                            match messages.next().await {{
                                Some(msg) => {{
                                    match msg {{
                                        Ok(msg) => {{
                                            let app_api = build_app_api(&servicer_data, "gen_js_input_{stream_name}", nats_fetch_trace_id(&msg));
                                            let bytes = msg.message.payload.as_ref();
                                            let payload_size = bytes.len();
                                            let pre = ::std::time::Instant::now();
                                            let deserialized_message = bw_type_{stream_type_name}_deserialize_json(bytes);{maybe_subject_parse}
                                            match deserialized_message {{
                                                Ok(input) => {{"#).unwrap();
            let maybe_subject_input = if enable_subjects {
                ", &subject"
            } else { "" };

            let prom_latency_name = format!(
                "METRIC_NATS_CONSUMER_LATENCY_{}",
                stream_name.to_uppercase()
            );
            cgen_context.register_prom_variable(
                prom_latency_name.clone(),
                prometheus_nats_processor_latency_metric(&prom_latency_name, app_name, stream_name),
            );

            let prom_bytes_name =
                format!("METRIC_NATS_CONSUMER_BYTES_{}", stream_name.to_uppercase());
            cgen_context.register_prom_variable(
                prom_bytes_name.clone(),
                prometheus_nats_bytes_processed_metric(&prom_bytes_name, app_name, stream_name),
            );

            res += &format!(
                r#"
                                                    let mut span = app_api.span("impl_js_input_{stream_name}");
                                                    let res = servicer_data.app_implementation.jetstream_consume_{}(&app_api, input{maybe_subject_input}).await;
                                                    {prom_latency_name}.observe((::std::time::Instant::now() - pre).as_secs_f64());"#,
                stream_name
            );
            res += r#"
                                                    match res {
                                                        Ok(()) => {
                                                            if let Err(e) = msg.ack().await {
                                                                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                                                                ::log::error!("Error while acking successfully processed message: {:?}", e);
                                                                break;
                                                            };"#;
            res += &format!(
                r#"
                                                            {prom_bytes_name}.inc_by(payload_size.try_into().unwrap_or_default());"#
            );
            res += r#"
                                                        },
                                                        Err(e) => {
                                                            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                                                            ::log::error!("Got error while processing message: {:?}", e);
                                                            break;
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    ::log::error!("Error during deserialization of json data: {:?}", e);
                                                    break;
                                                },
                                            }
                                        },
                                        Err(e) => {
                                            ::log::error!("Got error from nats stream: {:?}", e);
                                            break;
                                        },
                                    }
                                },
                                None => {},
                            }
                        }
                    },
                    Err(e) => {
                        ::log::error!("Error while getting messages from nats stream: {:?}", e);
                    },
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(7)).await;
            }
        });
    }
"#;
        }

        res += r#"

    Ok(())
}
"#;
    }

    res
}

fn implementation_backend_mock_rs() -> String {
    r#"
use std::error::Error;

use async_trait::async_trait;
use crate::generated::{AppRequirements, AppApi};

pub struct AppImplementation;

impl AppImplementation {
    pub fn new() -> AppImplementation { AppImplementation }
}

#[async_trait]
impl AppRequirements for AppImplementation {
    // implement all methods here
}
"#
    .to_string()
}

pub fn compute_http_endpoints(
    db: &crate::database::Database,
    c_http: &Projection<TableRowPointerBackendHttpEndpoint, CheckedHttpEndpoint>,
    vt: &Projection<TableRowPointerVersionedType, RustVersionedTypeSnippets>,
) -> Projection<TableRowPointerBackendHttpEndpoint, GeneratedRustSourceForHttpEndpoint> {
    c_http.derive_another(|ptr: TableRowPointerBackendHttpEndpoint, ct: &CheckedHttpEndpoint| -> GeneratedRustSourceForHttpEndpoint {
        let mut src = String::new();
        let mut prometheus_variables = Vec::new();
        let app = db.backend_http_endpoint().c_parent(ptr);

        let endpoint_data_type = db.http_endpoint_data_type().c_http_endpoint_data_type(db.backend_http_endpoint().c_data_type(ptr));
        let endpoint_name = db.backend_http_endpoint().c_http_endpoint_name(ptr);
        let rust_args_struct_name = format!("HttpEndpointPayload{}", db.backend_http_endpoint().c_http_endpoint_name(ptr).to_case(Case::Pascal));
        let rust_output_struct_name = match endpoint_data_type.as_str() {
            "html" => "::maud::Markup".to_string(),
            "json" => {
                let obtype = ct.output_body_type.unwrap();
                let v_snippets = vt.value(obtype);

                v_snippets.nominal_type_name.clone()
            }
            "raw" => {
                "::actix_web::HttpResponse".to_string()
            }
            dt => { panic!("Unexpected endpoint data type: {}", dt)}
        };
        let mut rust_args_struct_definition = format!("pub struct {} {{\n", rust_args_struct_name);
        let endpoint_src_declaration = db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(ptr)).to_lowercase();
        let max_input_body_size = db.backend_http_endpoint().c_max_input_body_size_bytes(ptr);

        src += "#[::actix_web::";
        src += &endpoint_src_declaration;
        src += "(\"";

        let (path, path_arguments_count) = gen_rust_http_path_backend(&ct.path_args);
        src += &path;

        src += "\")]\n";

        src += "async fn http_endpoint_";
        src += endpoint_name;
        src += "(";

        let mut args = Vec::new();

        args.push("servicer_data: ::actix_web::web::Data<AppServicerApi>".to_string());
        args.push("#[allow(unused_variables)] req: ::actix_web::HttpRequest".to_string());
        if ct.input_body_type.is_some() || ct.is_raw_input_body {
            args.push(format!("payload: ::actix_web::web::Payload"));
        }

        let mandatory_path_args = ct.path_args.required_args.iter().filter_map(|input| -> Option<(String, ValidHttpPrimitiveType)> {
            match input {
                CorePathSegment::Argument(k, t) => {
                    Some((k.clone(), *t))
                },
                CorePathSegment::Text(_) => None,
                CorePathSegment::LastSlash => None,
            }
        }).collect::<Vec<_>>();

        if db.backend_http_endpoint().c_needs_headers(ptr) {
            rust_args_struct_definition += "    pub headers: ::actix_web::http::header::HeaderMap,\n";
        }

        if path_arguments_count > 0 {
            let mut src = String::new();
            src += "path: ::actix_web::web::Path<";

            if path_arguments_count > 1 {
                src += "(";
            }

            src += &mandatory_path_args.iter().map(|(k, at)| {
                rust_args_struct_definition += "    pub ";
                rust_args_struct_definition += k;
                rust_args_struct_definition += ": ";
                let t = super::http_type_to_rust_type(*at);
                rust_args_struct_definition += t;
                rust_args_struct_definition += ",\n";
                t
            }).collect::<Vec<_>>().join(", ");

            if path_arguments_count > 1 {
                src += ")";
            }

            src += ">";
            args.push(src);
        }

        src += &args.join(", ");
        src += ") -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {\n";
        src += "    let pre = ::std::time::Instant::now();\n";
        writeln!(&mut src, "    let app_api = build_app_api(&servicer_data, \"gen_http_{endpoint_name}\", http_fetch_trace_id(&req));").unwrap();

        if let Some(ctype) = ct.expected_input_body_content_type {
            // ensure content type
            src += &format!(r#"    match req.headers().get("content-type") {{
        Some(h) => {{
            if h.as_bytes() != "{ctype}".as_bytes() {{
                return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: {ctype}"));
            }}
        }}
        None => {{
            return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: {ctype}"));
        }}
    }}"#);
            src += "\n";
            src += "\n";
        }

        if path_arguments_count > 0 {
            src += "    let ";
            if path_arguments_count > 1 {
                src += "(";
            }

            src += &mandatory_path_args.iter().map(|(k, _)| { k.as_str() }).collect::<Vec<_>>().join(", ");

            if path_arguments_count > 1 {
                src += ")";
            }

            src += " = path.into_inner();\n";
        }

        if db.backend_http_endpoint().c_needs_headers(ptr) {
            src += "    let headers = req.headers().clone();\n";
        }

        if !ct.path_args.opt_query_args.is_empty() {
            src += "    let qstring = ::qstring::QString::from(req.query_string());\n";

            for (k, t) in &ct.path_args.opt_query_args {
                let ftype = if t.is_multiple { "Vec" } else { "Option" };
                let field_def = format!("qa_{}: {}<{}>", k, ftype, super::http_type_to_rust_type(t.the_type));

                rust_args_struct_definition += "    pub ";
                rust_args_struct_definition += &field_def;
                rust_args_struct_definition += ",\n";

                src += "    let mut ";
                src += &field_def;
                if t.is_multiple {
                    src += " = Vec::new();\n";
                } else {
                    src += " = None;\n";
                }
            }

            src += "    for (k, v) in qstring.to_pairs() {\n";
            src += "        match k {\n";
            for (k, t) in &ct.path_args.opt_query_args {
                src += "            \"";
                src += k;
                src += "\" => { ";
                let parse_exp =
                    match t.the_type {
                        ValidHttpPrimitiveType::Int => {
                            "v.parse::<i64>().map_err(|_| EdenPlatformHttpError::CannotParseQueryArgument)?"
                        },
                        ValidHttpPrimitiveType::Float => {
                            "v.parse::<f64>().map_err(|_| EdenPlatformHttpError::CannotParseQueryArgument)?"
                        },
                        ValidHttpPrimitiveType::Bool => {
                            "v.parse::<bool>().map_err(|_| EdenPlatformHttpError::CannotParseQueryArgument)?"
                        },
                        ValidHttpPrimitiveType::Text => {
                            "v.to_string()"
                        },
                    };
                if t.is_multiple {
                    src += "qa_";
                    src += k;
                    src += ".push(";
                    src += parse_exp;
                    src += ") }\n";
                } else {
                    src += "if qa_";
                    src += k;
                    src += ".is_none() { qa_";
                    src += k;
                    src += " = Some(";
                    src += parse_exp;
                    src += ") } }\n";
                }
            }
            src += "            _ => {},\n";
            src += "        }\n";
            src += "    }\n";
        }

        if let Some(input_body) = &ct.input_body_type {
            src += &format!("    let input_body = fetch_body_with_limit(payload, {}).await?;\n", db.backend_http_endpoint().c_max_input_body_size_bytes(ptr));
            match endpoint_data_type.as_str() {
                "json" => {
                    let v_snippets = vt.value(*input_body);

                    src += "    let input_body = ";
                    src += &v_snippets.json_deserialization_function.function_name;
                    src += "(input_body.as_ref()).map_err(|e| EdenPlatformHttpError::InputPayloadJsonDeserializationError(e))?;\n";

                    rust_args_struct_definition += "    pub input_body: ";
                    rust_args_struct_definition += &v_snippets.nominal_type_name;
                    rust_args_struct_definition += ",\n";
                }
                e => panic!("Dunno what to do with this type yet: {}", e)
            }
        }

        if ct.is_raw_input_body {
            assert_eq!(endpoint_data_type.as_str(), "raw", "What else could this be?");
            if !ct.receive_body_as_stream {
                rust_args_struct_definition += "    pub input_body: Vec<u8>,\n";
                writeln!(&mut src, "    let input_body = fetch_body_with_limit(payload, {max_input_body_size}).await?.to_vec();").unwrap();
            } else {
                rust_args_struct_definition += "    pub input_body: ::tokio::sync::mpsc::Receiver<Result<Vec<u8>, ::actix_web::error::PayloadError>>,\n";
                writeln!(&mut src, "    let mut payload_to_process = payload;").unwrap();
                writeln!(&mut src, "    let (tx, input_body) = ::tokio::sync::mpsc::channel(64);").unwrap();
            }
        }

        src += "    let payload = ";
        src += &rust_args_struct_name;
        src += " {\n";

        if ct.input_body_type.is_some() || ct.is_raw_input_body {
            src += "        input_body,\n";
        }

        if db.backend_http_endpoint().c_needs_headers(ptr) {
            src += "        headers,\n";
        }

        for (k, _) in &mandatory_path_args {
            src += "        ";
            src += k;
            src += ",\n";
        }

        for (k, _) in &ct.path_args.opt_query_args {
            src += "        qa_";
            src += k;
            src += ",\n";
        }

        src += "    };\n";

        // register in prometheus how long it took to process
        let prom_errors_var_name = format!("METRIC_HTTP_ERRORS_{}", db.backend_http_endpoint().c_http_endpoint_name(ptr).to_uppercase());
        prometheus_variables.push((
            prom_errors_var_name.clone(),
            super::GenratedPromVariable {
                initialization_body: prometheus_http_errors_metric(
                    &prom_errors_var_name,
                    db.backend_application().c_application_name(app),
                    db.backend_http_endpoint().c_http_endpoint_name(ptr),
                    db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(ptr)),
                    db.backend_http_endpoint().c_path(ptr),
                )
            },
        ));

        writeln!(&mut src, "    let span = app_api.span(\"impl_http_{endpoint_name}\");").unwrap();
        if !ct.receive_body_as_stream {
            src += "    let output = servicer_data.app_implementation.http_endpoint_";
            src += db.backend_http_endpoint().c_http_endpoint_name(ptr);
            src += &format!("(&app_api, payload).await.map_err(|e| {{\n");
            src += &format!("        {prom_errors_var_name}.inc();\n");
            src += "        let new_e = EdenPlatformHttpError::InternalError(e.to_string());\n";
            src += &format!("          ::kv_log_macro::error!(\"Error serving http request\", {{ error: format!(\"\\\"{{}}\\\"\", e).as_str(), endpoint_name: \"\\\"{}\\\"\", route: \"\\\"{}\\\"\" }});\n",
                            db.backend_http_endpoint().c_http_endpoint_name(ptr),
                            db.backend_http_endpoint().c_path(ptr),
            );
            src += "        new_e\n";
            src += "    })?;\n";
            src += "    drop(span);\n";
        } else {
            write!(&mut src, r#"    let (output, dump) = ::futures_util::join!(
        servicer_data.app_implementation.http_endpoint_{}(&app_api, payload),
        dump_body_to_channel(&mut payload_to_process, tx),
    );
    dump?;
"#, db.backend_http_endpoint().c_http_endpoint_name(ptr)).unwrap();
            src += &format!("    let output = output.map_err(|e| {{\n");
            src += &format!("        {prom_errors_var_name}.inc();\n");
            src += "        let new_e = EdenPlatformHttpError::InternalError(e.to_string());\n";
            src += &format!("          ::kv_log_macro::error!(\"Error serving http request\", {{ error: format!(\"\\\"{{}}\\\"\", e).as_str(), endpoint_name: \"\\\"{}\\\"\", route: \"\\\"{}\\\"\" }});\n",
                            db.backend_http_endpoint().c_http_endpoint_name(ptr),
                            db.backend_http_endpoint().c_path(ptr),
            );
            src += "        new_e\n";
            src += "    })?;\n";
            src += "    drop(span);\n";
        }

        // register in prometheus how long it took to process
        let prom_latency_var_name = format!("METRIC_HTTP_LATENCY_{}", db.backend_http_endpoint().c_http_endpoint_name(ptr).to_uppercase());
        prometheus_variables.push((
            prom_latency_var_name.clone(),
            super::GenratedPromVariable {
                initialization_body: prometheus_http_latency_endpoint_metric(
                    &prom_latency_var_name,
                    db.backend_application().c_application_name(app),
                    db.backend_http_endpoint().c_http_endpoint_name(ptr),
                    db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(ptr)),
                    db.backend_http_endpoint().c_path(ptr),
                )
            },
        ));

        // register bytes served
        let prom_bytes_var_name = format!("METRIC_HTTP_BYTES_{}", db.backend_http_endpoint().c_http_endpoint_name(ptr).to_uppercase());
        if endpoint_data_type != "raw" {
            // TODO: figure out how to count output bytes and incr on finish sending request
            prometheus_variables.push((
                prom_bytes_var_name.clone(),
                super::GenratedPromVariable {
                    initialization_body: prometheus_http_bytes_served_metric(
                        &prom_bytes_var_name,
                        db.backend_application().c_application_name(app),
                        db.backend_http_endpoint().c_http_endpoint_name(ptr),
                        db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(ptr)),
                        db.backend_http_endpoint().c_path(ptr),
                    )
                },
            ));
        }

        match endpoint_data_type.as_str() {
            "html" => {
                src += "    let output = output.into_string();\n";
                src += "    ";
                src += &prom_latency_var_name;
                src += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
                src += "    ";
                src += &prom_bytes_var_name;
                src += ".inc_by(output.len().try_into().unwrap_or_default());\n";
                src += "    Ok(\n";
                src += "        ::actix_web::HttpResponse::Ok()\n";
                src += "            .append_header((\"Content-Type\", \"text/html\"))\n";
                src += "            .body(output)\n";
                src += "    )\n";
            }
            "json" => {
                let obtype = ct.output_body_type.unwrap();
                let v_snippets = vt.value(obtype);
                src += "    let output = ";
                src += &v_snippets.json_serialization_function.function_name;
                src += "(&output);\n";
                src += "    ";
                src += &prom_latency_var_name;
                src += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
                src += "    ";
                src += &prom_bytes_var_name;
                src += ".inc_by(output.len().try_into().unwrap_or_default());\n";
                src += "    Ok(\n";
                src += "        ::actix_web::HttpResponse::Ok()\n";
                src += "            .append_header((\"Content-Type\", \"application/json\"))\n";
                src += "            .body(output)\n";
                src += "    )\n";
            }
            "raw" => {
                src += "    ";
                src += &prom_latency_var_name;
                src += ".observe((::std::time::Instant::now() - pre).as_secs_f64());\n";
                src += "    Ok(output)\n";
            }
            e => {
                panic!("Unsupported endpoint type {e}")
            }
        }

        src += "}\n";

        rust_args_struct_definition += "}";


        GeneratedRustSourceForHttpEndpoint {
            rust_endpoint_declaration: src,
            rust_args_struct_name,
            rust_args_struct_definition,
            rust_output_struct_name,
            prometheus_variables,
        }
    })
}

fn gen_rust_http_path_backend(path_args: &PathArgs) -> (String, u32) {
    let mut res = String::new();
    let mut path_arguments_count = 0;

    for segment in &path_args.required_args {
        res += "/";
        match segment {
            crate::static_analysis::http_endpoints::CorePathSegment::Text(t) => {
                res += t;
            }
            crate::static_analysis::http_endpoints::CorePathSegment::Argument(n, _) => {
                path_arguments_count += 1;
                res += "{";
                res += n;
                res += "}";
            }
            crate::static_analysis::http_endpoints::CorePathSegment::LastSlash => {
                res += "/";
            }
        }
    }

    (res, path_arguments_count)
}
