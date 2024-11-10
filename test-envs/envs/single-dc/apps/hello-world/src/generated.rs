#[allow(unused_imports)]
use serde::{Serialize, Deserialize};
#[allow(unused_imports)]
use opentelemetry::trace::{TraceContextExt, Tracer, Span};
#[allow(unused_imports)]
use base64::Engine;


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

        let servicer_api = crate::generated::AppServicerApi {
            r: resources.clone(),
            app_implementation: app_implementation.clone(),
        };
        if let Err(e) = setup_jetstream_consumers_and_publishers(servicer_api).await {
            ::log::error!("Failed to schedule jetstream consumers, exiting: {}", e.to_string());
            ::std::process::exit(7);
        }
        // background jobs
        let bg_job_impl = app_implementation.clone();
        let bg_job_servicer_api = crate::generated::AppServicerApi {
            r: resources.clone(),
            app_implementation: bg_job_impl.clone(),
        };
        ::tokio::spawn(async move {
            let bg_job_api = crate::generated::build_app_api_no_context(&bg_job_servicer_api);
            if let Err(e) = bg_job_impl.bg_job_incrementer(bg_job_api).await {
                ::kv_log_macro::error!("Failure when running background job", { error: format!("\"{}\"", e).as_str() })
            }
        });

        ::actix_web::HttpServer::new(move || {
            let servicer_api = crate::generated::AppServicerApi {
                r: resources.clone(),
                app_implementation: app_implementation.clone(),
            };

            ::actix_web::App::new()
                .app_data(::actix_web::web::PayloadConfig::new(1024*1024*1024*1024*1024))
                .app_data(::actix_web::web::Data::new(servicer_api))
                .service(crate::generated::prometheus_metrics_endpoint)
                .service(crate::generated::http_endpoint_hello_world)
                .service(crate::generated::http_endpoint_example)
                .service(crate::generated::http_endpoint_mutate_test_1)
                .service(crate::generated::http_endpoint_read_test_1)
                .service(crate::generated::http_endpoint_dummy)
                .service(crate::generated::http_endpoint_download_file)
                .service(crate::generated::http_endpoint_upload_file)
                .service(crate::generated::http_endpoint_upload_file_multipart)
                .service(crate::generated::http_endpoint_configs_test)
                .service(crate::generated::http_endpoint_bg_job_counter)
                .service(crate::generated::http_endpoint_ch_insert_select)
                .service(crate::generated::http_endpoint_nats_ch_sink)
                .service(crate::generated::http_endpoint_test_ch_mutator)
        })
        .workers(threads)
        .bind(http_socket)?
        .run()
        .await
    }).expect("Failure running tokio runtime");
}


fn init_tracer() -> ::opentelemetry_sdk::trace::Tracer {
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
}

fn otel_resource() -> ::opentelemetry_sdk::Resource {
    use opentelemetry_semantic_conventions::{
        resource::{SERVICE_NAME, SERVICE_INSTANCE_ID},
        SCHEMA_URL,
    };
    ::opentelemetry_sdk::Resource::from_schema_url(
        [
            ::opentelemetry::KeyValue::new(SERVICE_NAME, ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable")),
            ::opentelemetry::KeyValue::new(SERVICE_INSTANCE_ID, ::std::env::var("NOMAD_ALLOC_ID").expect("Mandatory environment variable")),
            ::opentelemetry::KeyValue::new("epl.appname", "hello-world"),
        ],
        SCHEMA_URL,
    )
}



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

pub struct AppResources {
    tracer: ::opentelemetry::global::BoxedTracer,
    nats_conns: Vec<::async_nats::jetstream::Context>,
    deployment_name: String,
    pg_conn_default: ::bb8::Pool<::bb8_postgres::PostgresConnectionManager<::tokio_postgres::NoTls>>,
    ch_chshard_client: ::reqwest::Client,
    ch_chshard_database: String,
    ch_chshard_url: String,
    nats_conn_id_some_test_stream_producer: usize,
    nats_stream_some_test_stream_producer: String,
    nats_conn_id_some_test_stream_consumer: usize,
    nats_stream_some_test_stream_consumer: String,
    nats_conn_id_simple_msg_stream: usize,
    nats_stream_simple_msg_stream: String,
    s3_storage: ::s3::Bucket,
    cfg_some_string: String,
    cfg_some_int: i64,
    cfg_some_float: f64,
    cfg_some_bool: bool,
}

impl AppResources {
    pub async fn new() -> Result<AppResources, Box<dyn std::error::Error + Send + Sync>> {
        use ::std::str::FromStr;
        let deployment_name = ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME not configured").to_string();
        let epl_pg_conn_default = ::std::env::var("EPL_PG_CONN_DEFAULT").expect("Mandatory environment variable EPL_PG_CONN_DEFAULT not configured");
        let epl_ch_chshard_user = ::std::env::var("EPL_CH_CHSHARD_USER").expect("Mandatory environment variable EPL_CH_CHSHARD_USER not configured");
        let epl_ch_chshard_password = ::std::env::var("EPL_CH_CHSHARD_PASSWORD").expect("Mandatory environment variable EPL_CH_CHSHARD_PASSWORD not configured");
        let ch_chshard_database = ::std::env::var("EPL_CH_CHSHARD_DATABASE").expect("Mandatory environment variable EPL_CH_CHSHARD_DATABASE not configured").to_string();
        let ch_chshard_url = ::std::env::var("EPL_CH_CHSHARD_URL").expect("Mandatory environment variable EPL_CH_CHSHARD_URL not configured").to_string();
        let epl_nats_conn_some_test_stream_producer = ::std::env::var("EPL_NATS_CONN_SOME_TEST_STREAM_PRODUCER").expect("Mandatory environment variable EPL_NATS_CONN_SOME_TEST_STREAM_PRODUCER not configured");
        let nats_stream_some_test_stream_producer = ::std::env::var("EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER").expect("Mandatory environment variable EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER not configured").to_string();
        let epl_nats_conn_some_test_stream_consumer = ::std::env::var("EPL_NATS_CONN_SOME_TEST_STREAM_CONSUMER").expect("Mandatory environment variable EPL_NATS_CONN_SOME_TEST_STREAM_CONSUMER not configured");
        let nats_stream_some_test_stream_consumer = ::std::env::var("EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER").expect("Mandatory environment variable EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER not configured").to_string();
        let epl_nats_conn_simple_msg_stream = ::std::env::var("EPL_NATS_CONN_SIMPLE_MSG_STREAM").expect("Mandatory environment variable EPL_NATS_CONN_SIMPLE_MSG_STREAM not configured");
        let nats_stream_simple_msg_stream = ::std::env::var("EPL_NATS_STREAM_SIMPLE_MSG_STREAM").expect("Mandatory environment variable EPL_NATS_STREAM_SIMPLE_MSG_STREAM not configured").to_string();
        let epl_s3_storage_bucket = ::std::env::var("EPL_S3_STORAGE_BUCKET").expect("Mandatory environment variable EPL_S3_STORAGE_BUCKET not configured");
        let epl_s3_storage_url = ::std::env::var("EPL_S3_STORAGE_URI").expect("Mandatory environment variable EPL_S3_STORAGE_URI not configured");
        let epl_s3_storage_user = ::std::env::var("EPL_S3_STORAGE_USER").expect("Mandatory environment variable EPL_S3_STORAGE_USER not configured");
        let epl_s3_storage_password = ::std::env::var("EPL_S3_STORAGE_PASSWORD").expect("Mandatory environment variable EPL_S3_STORAGE_PASSWORD not configured");
        let cfg_some_string = ::std::env::var("EPL_CFG_SOME_STRING").expect("Mandatory environment variable EPL_CFG_SOME_STRING not configured").to_string();
        let cfg_some_int = ::std::env::var("EPL_CFG_SOME_INT").expect("Mandatory environment variable EPL_CFG_SOME_INT not configured").parse::<i64>().expect("Can't parse config EPL_CFG_SOME_INT");
        let cfg_some_float = ::std::env::var("EPL_CFG_SOME_FLOAT").expect("Mandatory environment variable EPL_CFG_SOME_FLOAT not configured").parse::<f64>().expect("Can't parse config EPL_CFG_SOME_FLOAT");
        let cfg_some_bool = ::std::env::var("EPL_CFG_SOME_BOOL").expect("Mandatory environment variable EPL_CFG_SOME_BOOL not configured").parse::<bool>().expect("Can't parse config EPL_CFG_SOME_BOOL");
        let mut nats_conns_urls = Vec::with_capacity(1);
        nats_conns_urls.push(epl_nats_conn_some_test_stream_producer);
        let nats_conn_id_some_test_stream_producer = nats_conns_urls.len() - 1;
        let nats_conn_id_some_test_stream_consumer =
            if let Some(cid) = nats_conns_urls.iter().enumerate().find(|(_, cs)| cs.as_str() == epl_nats_conn_some_test_stream_consumer) {
                cid.0
            } else {
                nats_conns_urls.push(epl_nats_conn_some_test_stream_consumer);
                nats_conns_urls.len() - 1
            };
        let nats_conn_id_simple_msg_stream =
            if let Some(cid) = nats_conns_urls.iter().enumerate().find(|(_, cs)| cs.as_str() == epl_nats_conn_simple_msg_stream) {
                cid.0
            } else {
                nats_conns_urls.push(epl_nats_conn_simple_msg_stream);
                nats_conns_urls.len() - 1
            };

        let manager = ::bb8_postgres::PostgresConnectionManager::new(
            ::tokio_postgres::Config::from_str(&epl_pg_conn_default)?, ::tokio_postgres::NoTls
        );
        let pg_conn_default = ::bb8::Pool::builder()
            .build(manager)
            .await?;


        let mut hm = ::reqwest::header::HeaderMap::new();
        hm.append("X-Clickhouse-User", epl_ch_chshard_user.parse()?);
        hm.append("X-Clickhouse-Key", epl_ch_chshard_password.parse()?);
        let ch_chshard_client = ::reqwest::ClientBuilder::new()
            .default_headers(hm)
            .build()?;

        let mut nats_conns_ctx = Vec::with_capacity(nats_conns_urls.len());
        for url in &nats_conns_urls {
            nats_conns_ctx.push(::async_nats::connect(url));
        }

        let joined = ::futures::future::join_all(nats_conns_ctx).await;
        let mut nats_conns = Vec::with_capacity(nats_conns_urls.len());
        for ctx in joined {
            nats_conns.push(::async_nats::jetstream::new(ctx?));
        }

        let s3_storage = ::s3::Bucket::new(
            &epl_s3_storage_bucket,
            ::s3::Region::Custom {
                region: "any".to_string(),
                endpoint: epl_s3_storage_url,
            },
            ::s3::creds::Credentials {
                access_key: Some(epl_s3_storage_user),
                secret_key: Some(epl_s3_storage_password),
                security_token: None,
                session_token: None,
                expiration: None,
            }
        )?.with_path_style();

        let tracer = ::opentelemetry::global::tracer("hello-world");
        Ok(AppResources {
            tracer,
            nats_conns,
            deployment_name,
            pg_conn_default,
            ch_chshard_client,
            ch_chshard_database,
            ch_chshard_url,
            nats_conn_id_some_test_stream_producer,
            nats_stream_some_test_stream_producer,
            nats_conn_id_some_test_stream_consumer,
            nats_stream_some_test_stream_consumer,
            nats_conn_id_simple_msg_stream,
            nats_stream_simple_msg_stream,
            s3_storage,
            cfg_some_string,
            cfg_some_int,
            cfg_some_float,
            cfg_some_bool,
        })
    }
}


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

    pub async fn pgq_max_id_from_foo(&self, test_arg: i32) -> Result<Vec<PgqTestdbRowMaxIdFromFoo>, PgInteractionError> {
        let pre = ::std::time::Instant::now();
        let mut span = self.span("pg_conn_get");
        let conn = self.r.pg_conn_default.get().await.map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                PgInteractionError::ConnectionPoolError(e)
            })?;
        drop(span);
        METRIC_PG_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());
        let the_query = r#"SELECT max(id) AS max_id FROM foo WHERE $1 > 0"#;
        let pre = ::std::time::Instant::now();
        let mut span = self.span("pgq_max_id_from_foo");
        let rows = conn.query(the_query, &[&test_arg]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_PGQ_TESTDB_MAX_ID_FROM_FOO.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(PgqTestdbRowMaxIdFromFoo {
                max_id: r.try_get::<usize, Option<i32>>(0).map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::DeserializationError(e.to_string()) })?,
            });
        }
        Ok(res)
    }

    pub async fn pgmq_insert_id_returning(&self, test_arg: i32) -> Result<Vec<PgqTestdbRowInsertIdReturning>, PgInteractionError> {
        let pre = ::std::time::Instant::now();
        let mut span = self.span("pg_conn_get");
        let conn = self.r.pg_conn_default.get().await.map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                PgInteractionError::ConnectionPoolError(e)
            })?;
        drop(span);
        METRIC_PG_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());
        let the_query = r#"INSERT INTO foo(id) VALUES($1) RETURNING id"#;
        let pre = ::std::time::Instant::now();
        let mut span = self.span("pgmq_insert_id_returning");
        let rows = conn.query(the_query, &[&test_arg]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_PGQ_TESTDB_INSERT_ID_RETURNING.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(PgqTestdbRowInsertIdReturning {
                id: r.try_get::<usize, i32>(0).map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::DeserializationError(e.to_string()) })?,
            });
        }
        Ok(res)
    }

    pub async fn pgm_insert_id(&self, test_arg: i32) -> Result<u64, PgInteractionError> {
        let pre = ::std::time::Instant::now();
        let mut span = self.span("pg_conn_get");
        let conn = self.r.pg_conn_default.get().await.map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                PgInteractionError::ConnectionPoolError(e)
            })?;
        drop(span);
        METRIC_PG_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());
        let the_query = r#"INSERT INTO foo(id) VALUES($1)"#;
        let pre = ::std::time::Instant::now();
        let mut span = self.span("pgm_insert_id");
        let res = conn.execute(the_query, &[&test_arg]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;
        METRIC_PGM_TESTDB_INSERT_ID.observe((::std::time::Instant::now() - pre).as_secs_f64());
        Ok(res)
    }

    pub async fn pgtrx_begin_all_at_once(&self) -> Result<TrxDefaultAllAtOnceS1, PgInteractionError> {
        let pre = ::std::time::Instant::now();
        let context = ::opentelemetry::Context::current_with_span(self.span("pgtrx_all_at_once"));
        let mut span = self.r.tracer.start_with_context("pg_conn_get", &context);
        let conn = self.r.pg_conn_default.get().await.map_err(|e| {
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            PgInteractionError::ConnectionPoolError(e)
        })?;
        drop(span);
        METRIC_PG_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());

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

        Ok(TrxDefaultAllAtOnceS1 { state })
    }

    pub async fn chq_max_id_from_foo(&self, test_arg: i64) -> Result<Vec<ChqTestchRowMaxIdFromFoo>, ChInteractionError> {
        let the_query = r#"
          SELECT
            max(id) AS max_id,
            'a !@#$%^&*()_+' AS v_string,
            cast(123 as Int32) AS v_i32,
            cast(123 as Int64) AS v_i64,
            cast(12.3 as Float32) AS v_f32,
            cast(12.3 as Float64) AS v_f64,
            true AS v_bool_t,
            false AS v_bool_f
          FROM foo WHERE id > {test_arg:Int64}
        "#;
        let pre = ::std::time::Instant::now();
        let mut span = self.span("chq_max_id_from_foo_query");

        let resp = self.r.ch_chshard_client
            .request(::reqwest::Method::POST, &self.r.ch_chshard_url)
            .query(&[
                ("database", &self.r.ch_chshard_database),
                ("param_test_arg", &test_arg.to_string()),
            ])
            .body(the_query)
            .send()
            .await
            .map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            })?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {
            let err_text = resp.text().await.unwrap_or_default();
            span.set_status(::opentelemetry::trace::Status::error(err_text.clone()));
            return Err(ChInteractionError::HttpResponseError {
                expected_status_code: 200,
                actual_status_code: status_code,
                body: err_text,
            });
        }

        let text = resp.text().await.map_err(|e| {
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            ChInteractionError::StreamReadError(e)
        })?;

        drop(span);
        let mut span = self.span("chq_max_id_from_foo_deser");

        let mut res: Vec<ChqTestchRowMaxIdFromFoo> = Vec::new();

        let expected_tab_fields = 8;
        let mut line_no = 0usize;
        for line in text.lines() {
            let line_spl = line.split("\t").collect::<Vec<_>>();
            if line_spl.len() == expected_tab_fields {

                let max_id = line_spl[0].parse::<i64>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 0,
                        expected_type: "i64",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                let v_string = ::unescaper::unescape(line_spl[1]).map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 1,
                        expected_type: "String",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                let v_i32 = line_spl[2].parse::<i32>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 2,
                        expected_type: "i32",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                let v_i64 = line_spl[3].parse::<i64>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 3,
                        expected_type: "i64",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                let v_f32 = line_spl[4].parse::<f32>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 4,
                        expected_type: "f32",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                let v_f64 = line_spl[5].parse::<f64>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 5,
                        expected_type: "f64",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                let v_bool_t = line_spl[6].parse::<bool>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 6,
                        expected_type: "bool",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                let v_bool_f = line_spl[7].parse::<bool>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 7,
                        expected_type: "bool",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                res.push(ChqTestchRowMaxIdFromFoo {
                    max_id, v_string, v_i32, v_i64, v_f32, v_f64, v_bool_t, v_bool_f
                });

            } else {
                let e = ChInteractionError::IncorrectFormatError {
                    expected_tab_fields,
                    actual_tab_fields: res.len(),
                    row_content: line.to_string(),
                    row_number: line_no,
                };
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                return Err(e);
            }
            line_no += 1;
        }

        METRIC_CHQ_CHSHARD_MAX_ID_FROM_FOO.observe((::std::time::Instant::now() - pre).as_secs_f64());
        Ok(res)
    }

    pub async fn chq_max_id_from_foo_ids(&self) -> Result<Vec<ChqTestchRowMaxIdFromFooIds>, ChInteractionError> {
        let the_query = r#"
          SELECT max(id) AS max_id
          FROM foo_ids
        "#;
        let pre = ::std::time::Instant::now();
        let mut span = self.span("chq_max_id_from_foo_ids_query");

        let resp = self.r.ch_chshard_client
            .request(::reqwest::Method::POST, &self.r.ch_chshard_url)
            .query(&[
                ("database", &self.r.ch_chshard_database),
            ])
            .body(the_query)
            .send()
            .await
            .map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            })?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {
            let err_text = resp.text().await.unwrap_or_default();
            span.set_status(::opentelemetry::trace::Status::error(err_text.clone()));
            return Err(ChInteractionError::HttpResponseError {
                expected_status_code: 200,
                actual_status_code: status_code,
                body: err_text,
            });
        }

        let text = resp.text().await.map_err(|e| {
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            ChInteractionError::StreamReadError(e)
        })?;

        drop(span);
        let mut span = self.span("chq_max_id_from_foo_ids_deser");

        let mut res: Vec<ChqTestchRowMaxIdFromFooIds> = Vec::new();

        let expected_tab_fields = 1;
        let mut line_no = 0usize;
        for line in text.lines() {
            let line_spl = line.split("\t").collect::<Vec<_>>();
            if line_spl.len() == expected_tab_fields {

                let max_id = line_spl[0].parse::<i64>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 0,
                        expected_type: "i64",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                res.push(ChqTestchRowMaxIdFromFooIds {
                    max_id
                });

            } else {
                let e = ChInteractionError::IncorrectFormatError {
                    expected_tab_fields,
                    actual_tab_fields: res.len(),
                    row_content: line.to_string(),
                    row_number: line_no,
                };
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                return Err(e);
            }
            line_no += 1;
        }

        METRIC_CHQ_CHSHARD_MAX_ID_FROM_FOO_IDS.observe((::std::time::Instant::now() - pre).as_secs_f64());
        Ok(res)
    }

    pub async fn chq_max_id_from_imp(&self) -> Result<Vec<ChqTestchRowMaxIdFromImp>, ChInteractionError> {
        let the_query = r#"
          SELECT max(some_field) AS max_id
          FROM imp_table
        "#;
        let pre = ::std::time::Instant::now();
        let mut span = self.span("chq_max_id_from_imp_query");

        let resp = self.r.ch_chshard_client
            .request(::reqwest::Method::POST, &self.r.ch_chshard_url)
            .query(&[
                ("database", &self.r.ch_chshard_database),
            ])
            .body(the_query)
            .send()
            .await
            .map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            })?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {
            let err_text = resp.text().await.unwrap_or_default();
            span.set_status(::opentelemetry::trace::Status::error(err_text.clone()));
            return Err(ChInteractionError::HttpResponseError {
                expected_status_code: 200,
                actual_status_code: status_code,
                body: err_text,
            });
        }

        let text = resp.text().await.map_err(|e| {
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            ChInteractionError::StreamReadError(e)
        })?;

        drop(span);
        let mut span = self.span("chq_max_id_from_imp_deser");

        let mut res: Vec<ChqTestchRowMaxIdFromImp> = Vec::new();

        let expected_tab_fields = 1;
        let mut line_no = 0usize;
        for line in text.lines() {
            let line_spl = line.split("\t").collect::<Vec<_>>();
            if line_spl.len() == expected_tab_fields {

                let max_id = line_spl[0].parse::<i64>().map_err(|e| {
                    span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                    ChInteractionError::ColumnParseErrorError {
                        error: e.to_string(),
                        column_number: 0,
                        expected_type: "i64",
                        row_content: line.to_string(),
                        row_number: line_no,
                    }
                })?;

                res.push(ChqTestchRowMaxIdFromImp {
                    max_id
                });

            } else {
                let e = ChInteractionError::IncorrectFormatError {
                    expected_tab_fields,
                    actual_tab_fields: res.len(),
                    row_content: line.to_string(),
                    row_number: line_no,
                };
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                return Err(e);
            }
            line_no += 1;
        }

        METRIC_CHQ_CHSHARD_MAX_ID_FROM_IMP.observe((::std::time::Instant::now() - pre).as_secs_f64());
        Ok(res)
    }

    pub async fn chm_copy_ids_from_foo(&self, test_arg: i64) -> Result<(), ChInteractionError> {
        let the_query = r#"
	  INSERT INTO foo_ids(id)
          SELECT id
          FROM foo
	  WHERE id > {test_arg:Int64}
        "#;
        let pre = ::std::time::Instant::now();
        let mut span = self.span("chm_copy_ids_from_foo");

        let resp = self.r.ch_chshard_client
            .request(::reqwest::Method::POST, &self.r.ch_chshard_url)
            .query(&[
                ("database", &self.r.ch_chshard_database),
                ("param_test_arg", &test_arg.to_string()),
            ])
            .body(the_query)
            .send()
            .await
            .map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            })?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {
            let err_text = resp.text().await.unwrap_or_default();
            span.set_status(::opentelemetry::trace::Status::error(err_text.clone()));
            return Err(ChInteractionError::HttpResponseError {
                expected_status_code: 200,
                actual_status_code: status_code,
                body: err_text,
            });
        }
        METRIC_CHM_CHSHARD_COPY_IDS_FROM_FOO.observe((::std::time::Instant::now() - pre).as_secs_f64());
        Ok(())
    }

    pub async fn ch_insert_into_chshard_foo(&self, rows: &[ChInsTestchRowFoo]) -> Result<(), ChInteractionError> {
        use ::std::fmt::Write;
        let pre = ::std::time::Instant::now();

        let span = self.span("ch_insert_into_chshard_foo_serialize");

        let mut payload = "INSERT INTO foo(a, b, c, f, id) VALUES ".to_string();
        let mut row_idx = 0usize;
        for row in rows {
            payload += "(";

            payload += "base64Decode('";
            base64::prelude::BASE64_STANDARD.encode_string(&row.a, &mut payload);
            payload += "')";

            payload += ",";

            if let Some(row_val) = &row.b {
                payload += "base64Decode('";
                base64::prelude::BASE64_STANDARD.encode_string(row_val, &mut payload);
                payload += "')";
            } else { payload += "DEFAULT" }

            payload += ",";

            if let Some(row_val) = &row.c {
                payload += "base64Decode('";
                base64::prelude::BASE64_STANDARD.encode_string(row_val, &mut payload);
                payload += "')";
            } else { payload += "DEFAULT" }

            payload += ",";

            if let Some(row_val) = &row.f {
                payload += "base64Decode('";
                base64::prelude::BASE64_STANDARD.encode_string(row_val, &mut payload);
                payload += "')";
            } else { payload += "DEFAULT" }

            payload += ",";

            write!(&mut payload, "{}", row.id).unwrap();

            payload += ")";
            if row_idx < rows.len() - 1 {
                payload += ",";
            }
            row_idx += 1;
        }
        drop(span);

        let mut span = self.span("ch_insert_into_chshard_foo_send");
        let resp = self.r.ch_chshard_client
            .request(::reqwest::Method::POST, &self.r.ch_chshard_url)
            .query(&[
                ("database", &self.r.ch_chshard_database),
            ])
            .body(payload)
            .send()
            .await
            .map_err(|e| {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                ChInteractionError::HttpError(e)
            })?;

        let status_code = resp.status().as_u16();
        if status_code != 200 {
            let e = ChInteractionError::HttpResponseError {
                expected_status_code: 200,
                actual_status_code: status_code,
                body: resp.text().await.unwrap_or_default(),
            };
            span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            return Err(e);
        }
        METRIC_CHI_CHSHARD_FOO.observe((::std::time::Instant::now() - pre).as_secs_f64());
        Ok(())
    }

    pub async fn jetstream_publish_some_test_stream_producer(&self, input: &BwTypeTestVtype, subject: &str) -> Result<::async_nats::jetstream::context::PublishAckFuture, ::async_nats::error::Error<::async_nats::jetstream::context::PublishErrorKind>> {
        let pre = ::std::time::Instant::now();
        let mut span = self.span("js_publish_some_test_stream_producer");
        let trace_id = span.span_context().trace_id().to_string();
        let payload: ::std::string::String = bw_type_test_vtype_serialize_json(input);
        let payload_size = payload.len();
        let mut headers = ::async_nats::HeaderMap::new();
        headers.insert("trace-id", trace_id.as_str());
        let subject = format!("{}.{}", self.r.nats_stream_some_test_stream_producer, subject);
        let res = self.r.nats_conns[self.r.nats_conn_id_some_test_stream_producer].publish_with_headers(subject, headers, payload.into()).await;
        match &res {
            Ok(_res) => {
                METRIC_NATS_PUBLISH_LATENCY_SOME_TEST_STREAM_PRODUCER.observe((::std::time::Instant::now() - pre).as_secs_f64());
                METRIC_NATS_PUBLISH_BYTES_SOME_TEST_STREAM_PRODUCER.inc_by(payload_size.try_into().unwrap_or_default());
            }
            Err(e) => {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            }
        };
        res
    }

    pub async fn jetstream_publish_simple_msg_stream(&self, input: &BwTypeSimple) -> Result<::async_nats::jetstream::context::PublishAckFuture, ::async_nats::error::Error<::async_nats::jetstream::context::PublishErrorKind>> {
        let pre = ::std::time::Instant::now();
        let mut span = self.span("js_publish_simple_msg_stream");
        let trace_id = span.span_context().trace_id().to_string();
        let payload: ::std::string::String = bw_type_simple_serialize_json(input);
        let payload_size = payload.len();
        let mut headers = ::async_nats::HeaderMap::new();
        headers.insert("trace-id", trace_id.as_str());
        let subject = self.r.nats_stream_simple_msg_stream.clone();
        let res = self.r.nats_conns[self.r.nats_conn_id_simple_msg_stream].publish_with_headers(subject, headers, payload.into()).await;
        match &res {
            Ok(_res) => {
                METRIC_NATS_PUBLISH_LATENCY_SIMPLE_MSG_STREAM.observe((::std::time::Instant::now() - pre).as_secs_f64());
                METRIC_NATS_PUBLISH_BYTES_SIMPLE_MSG_STREAM.inc_by(payload_size.try_into().unwrap_or_default());
            }
            Err(e) => {
                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
            }
        };
        res
    }

    pub fn s3_storage(&self) -> &::s3::Bucket {
        &self.r.s3_storage
    }

    pub fn cfg_some_string(&self) -> &str {
        self.r.cfg_some_string.as_str()
    }

    pub fn cfg_some_int(&self) -> i64 {
        self.r.cfg_some_int
    }

    pub fn cfg_some_float(&self) -> f64 {
        self.r.cfg_some_float
    }

    pub fn cfg_some_bool(&self) -> bool {
        self.r.cfg_some_bool
    }
}


struct TransactionState {
    // extend lifetimes to static because we have trouble with travelling builder pattern
    trx: ::tokio_postgres::Transaction<'static>,
    _conn: Box<::bb8::PooledConnection<'static, ::bb8_postgres::PostgresConnectionManager<::tokio_postgres::NoTls>>>,
    _r: ::std::sync::Arc<AppResources>,
    start_time: ::std::time::Instant,
    context: ::opentelemetry::Context,
}
pub struct TrxDefaultAllAtOnceS1 {
    state: TransactionState,
}

impl TrxDefaultAllAtOnceS1 {
    pub async fn pgmq_insert_id_returning(self, test_arg: i32) -> Result<TrxDefaultAllAtOnceS2, PgInteractionError> {
        let the_query = r#"INSERT INTO foo(id) VALUES($1) RETURNING id"#;
        let pre = ::std::time::Instant::now();
        let mut span = self.state._r.tracer.start_with_context("pgmq_insert_id_returning", &self.state.context);
        let rows = self.state.trx.query(the_query, &[&test_arg]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_TRX_PGQ_TESTDB_INSERT_ID_RETURNING.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(PgqTestdbRowInsertIdReturning {
                id: r.try_get::<usize, i32>(0).map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::DeserializationError(e.to_string()) })?,
            });
        }

        Ok(TrxDefaultAllAtOnceS2 {
            r_insert_id_returning: res,
            state: self.state,
        })
    }

}

pub struct TrxDefaultAllAtOnceS2 {
    state: TransactionState,
    r_insert_id_returning: Vec<PgqTestdbRowInsertIdReturning>,
}

impl TrxDefaultAllAtOnceS2 {
    pub async fn pgq_max_id_from_foo(&mut self, test_arg: i32) -> Result<Vec<PgqTestdbRowMaxIdFromFoo>, PgInteractionError> {
        let the_query = r#"SELECT max(id) AS max_id FROM foo WHERE $1 > 0"#;
        let pre = ::std::time::Instant::now();
        let mut span = self.state._r.tracer.start_with_context("pgq_max_id_from_foo", &self.state.context);
        let rows = self.state.trx.query(the_query, &[&test_arg]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_TRX_PGQ_TESTDB_MAX_ID_FROM_FOO.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(PgqTestdbRowMaxIdFromFoo {
                max_id: r.try_get::<usize, Option<i32>>(0).map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::DeserializationError(e.to_string()) })?,
            });
        }

        Ok(res)
    }

    pub fn advance(self) -> TrxDefaultAllAtOnceS3 {
        TrxDefaultAllAtOnceS3 {
            state: self.state,
            r_insert_id_returning: self.r_insert_id_returning,
        }
    }

    #[allow(dead_code)]
    pub fn r_insert_id_returning(&self) -> &[PgqTestdbRowInsertIdReturning] {
        &self.r_insert_id_returning
    }
}

pub struct TrxDefaultAllAtOnceS3 {
    state: TransactionState,
    r_insert_id_returning: Vec<PgqTestdbRowInsertIdReturning>,
}

impl TrxDefaultAllAtOnceS3 {
    pub async fn pgm_insert_id(&mut self, test_arg: i32) -> Result<u64, PgInteractionError> {
        let the_query = r#"INSERT INTO foo(id) VALUES($1)"#;
        let pre = ::std::time::Instant::now();
        let mut span = self.state._r.tracer.start_with_context("pgm_insert_id", &self.state.context);
        let res = self.state.trx.execute(the_query, &[&test_arg]).await.map_err(|e| { span.set_status(::opentelemetry::trace::Status::error(e.to_string())); PgInteractionError::PostgresError(e) })?;
        METRIC_TRX_PGM_TESTDB_INSERT_ID.observe((::std::time::Instant::now() - pre).as_secs_f64());

        Ok(res)
    }

    pub fn advance(self) -> TrxDefaultAllAtOnceS4 {
        TrxDefaultAllAtOnceS4 {
            state: self.state,
            r_insert_id_returning: self.r_insert_id_returning,
        }
    }

    #[allow(dead_code)]
    pub fn r_insert_id_returning(&self) -> &[PgqTestdbRowInsertIdReturning] {
        &self.r_insert_id_returning
    }
}

pub struct TrxDefaultAllAtOnceS4 {
    state: TransactionState,
    r_insert_id_returning: Vec<PgqTestdbRowInsertIdReturning>,
}

pub struct TrxDefaultAllAtOnceOutput {
    pub r_insert_id_returning: Vec<PgqTestdbRowInsertIdReturning>,
}

impl TrxDefaultAllAtOnceS4 {

    pub async fn commit(self) -> Result<TrxDefaultAllAtOnceOutput, PgInteractionError> {
        let _ = self.state.trx.commit().await.map_err(|e| {
            PgInteractionError::PostgresError(e)
        })?;

        METRIC_PG_TRX_DEFAULT_ALL_AT_ONCE.observe((::std::time::Instant::now() - self.state.start_time).as_secs_f64());

        Ok(TrxDefaultAllAtOnceOutput {
            r_insert_id_returning: self.r_insert_id_returning,
        })
    }
    #[allow(dead_code)]
    pub fn r_insert_id_returning(&self) -> &[PgqTestdbRowInsertIdReturning] {
        &self.r_insert_id_returning
    }
}

pub struct ChInsTestchRowFoo {
    pub a: String,
    pub b: Option<String>,
    pub c: Option<String>,
    pub f: Option<String>,
    pub id: i64,
}
pub struct ChqTestchRowMaxIdFromFoo {
    pub max_id: i64,
    pub v_string: String,
    pub v_i32: i32,
    pub v_i64: i64,
    pub v_f32: f32,
    pub v_f64: f64,
    pub v_bool_t: bool,
    pub v_bool_f: bool,
}
pub struct ChqTestchRowMaxIdFromFooIds {
    pub max_id: i64,
}
pub struct ChqTestchRowMaxIdFromImp {
    pub max_id: i64,
}
pub struct PgqTestdbRowInsertIdReturning {
    pub id: i32,
}
pub struct PgqTestdbRowMaxIdFromFoo {
    pub max_id: Option<i32>,
}
#[::async_trait::async_trait]
pub trait AppRequirements {
    async fn http_endpoint_hello_world(&self, api: &AppApi, payload: HttpEndpointPayloadHelloWorld) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_example(&self, api: &AppApi, payload: HttpEndpointPayloadExample) -> Result<BwTypeTestVtype, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_mutate_test_1(&self, api: &AppApi, payload: HttpEndpointPayloadMutateTest1) -> Result<BwTypeTestOutputType, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_read_test_1(&self, api: &AppApi, payload: HttpEndpointPayloadReadTest1) -> Result<BwTypeTestOutputType, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_dummy(&self, api: &AppApi, payload: HttpEndpointPayloadDummy) -> Result<BwTypeTestVtype, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_download_file(&self, api: &AppApi, payload: HttpEndpointPayloadDownloadFile) -> Result<::actix_web::HttpResponse, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_upload_file(&self, api: &AppApi, payload: HttpEndpointPayloadUploadFile) -> Result<::actix_web::HttpResponse, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_upload_file_multipart(&self, api: &AppApi, payload: HttpEndpointPayloadUploadFileMultipart) -> Result<::actix_web::HttpResponse, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_configs_test(&self, api: &AppApi, payload: HttpEndpointPayloadConfigsTest) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_bg_job_counter(&self, api: &AppApi, payload: HttpEndpointPayloadBgJobCounter) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_ch_insert_select(&self, api: &AppApi, payload: HttpEndpointPayloadChInsertSelect) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_nats_ch_sink(&self, api: &AppApi, payload: HttpEndpointPayloadNatsChSink) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn http_endpoint_test_ch_mutator(&self, api: &AppApi, payload: HttpEndpointPayloadTestChMutator) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>>;
    async fn jetstream_consume_some_test_stream_consumer(&self, api: &AppApi, payload: BwTypeTestVtype, subject: &str) -> Result<(), Box<dyn ::std::error::Error + Send + Sync>>;
    async fn bg_job_incrementer(&self, api: AppApi) -> Result<(), Box<dyn ::std::error::Error + Send + Sync>>;
}

#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV1 {
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV2 {
    pub other_field: f64,
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV3Aux1 {
    pub x: f64,
    pub y: f64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV3 {
    pub coordinates: Option<BwTypeTestVtypeV3Aux1>,
    pub other_field: f64,
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV4Aux1 {
    pub x: f64,
    pub y: f64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV4 {
    pub coordinates: Option<BwTypeTestVtypeV4Aux1>,
    pub is_good: bool,
    pub nickname: String,
    pub other_field: f64,
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV5Aux1 {
    pub x: f64,
    pub y: f64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV5 {
    pub coordinates: Option<BwTypeTestVtypeV5Aux1>,
    pub is_nice: bool,
    pub other_field: f64,
    pub some_field: i64,
}
pub type BwTypeTestVtype = BwTypeTestVtypeV5;

fn bw_type_test_vtype_serialize_json(input: &BwTypeTestVtype) -> String {
    ::serde_json::to_string(input).expect("should never happen")
}

fn bw_type_test_vtype_deserialize_json(input: &[u8]) -> Result<BwTypeTestVtype, JsonDeserializationError> {
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV5>(input) {
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV4>(input) {
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV3>(input) {
        let the_val = bw_type_test_vtype_v3_to_v4(the_val);
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV2>(input) {
        let the_val = bw_type_test_vtype_v2_to_v3(the_val);
        let the_val = bw_type_test_vtype_v3_to_v4(the_val);
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV1>(input) {
        let the_val = bw_type_test_vtype_v1_to_v2(the_val);
        let the_val = bw_type_test_vtype_v2_to_v3(the_val);
        let the_val = bw_type_test_vtype_v3_to_v4(the_val);
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    return Err(JsonDeserializationError::UnknownType);
}


fn bw_type_test_vtype_v1_to_v2(input: BwTypeTestVtypeV1) -> BwTypeTestVtypeV2 {
    BwTypeTestVtypeV2 {
        other_field: 1.23,
        some_field: input.some_field,
    }
}
fn bw_type_test_vtype_v2_to_v3(input: BwTypeTestVtypeV2) -> BwTypeTestVtypeV3 {
    BwTypeTestVtypeV3 {
        coordinates: None,
        other_field: input.other_field,
        some_field: input.some_field,
    }
}
fn bw_type_test_vtype_v3_to_v4(input: BwTypeTestVtypeV3) -> BwTypeTestVtypeV4 {
    BwTypeTestVtypeV4 {
        coordinates: input.coordinates.map(|input| BwTypeTestVtypeV4Aux1 {
            x: input.x,
            y: input.y,
        }),
        is_good: true,
        nickname: r#"who knows"#.to_string(),
        other_field: input.other_field,
        some_field: input.some_field,
    }
}
fn bw_type_test_vtype_v4_to_v5(input: BwTypeTestVtypeV4) -> BwTypeTestVtypeV5 {
    BwTypeTestVtypeV5 {
        coordinates: input.coordinates.map(|input| BwTypeTestVtypeV5Aux1 {
            x: input.x,
            y: input.y,
        }),
        is_nice: input.is_good,
        other_field: input.other_field,
        some_field: input.some_field,
    }
}

#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestOutputTypeV1 {
    pub output_field: i64,
}
pub type BwTypeTestOutputType = BwTypeTestOutputTypeV1;

fn bw_type_test_output_type_serialize_json(input: &BwTypeTestOutputType) -> String {
    ::serde_json::to_string(input).expect("should never happen")
}

#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeSimpleV1 {
    pub some_field: i64,
    pub some_text: String,
}
pub type BwTypeSimple = BwTypeSimpleV1;

fn bw_type_simple_serialize_json(input: &BwTypeSimple) -> String {
    ::serde_json::to_string(input).expect("should never happen")
}



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


pub struct HttpEndpointPayloadHelloWorld {
    pub headers: ::actix_web::http::header::HeaderMap,
    pub arg: String,
    pub more: bool,
    pub qa_floot: Vec<f64>,
    pub qa_other: Option<i64>,
}

#[::actix_web::get("/hello_world/{arg}/{more}")]
async fn http_endpoint_hello_world(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, path: ::actix_web::web::Path<(String, bool)>) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_hello_world", http_fetch_trace_id(&req));
    let (arg, more) = path.into_inner();
    let headers = req.headers().clone();
    let qstring = ::qstring::QString::from(req.query_string());
    let mut qa_floot: Vec<f64> = Vec::new();
    let mut qa_other: Option<i64> = None;
    for (k, v) in qstring.to_pairs() {
        match k {
            "floot" => { qa_floot.push(v.parse::<f64>().map_err(|_| EdenPlatformHttpError::CannotParseQueryArgument)?) }
            "other" => { if qa_other.is_none() { qa_other = Some(v.parse::<i64>().map_err(|_| EdenPlatformHttpError::CannotParseQueryArgument)?) } }
            _ => {},
        }
    }
    let payload = HttpEndpointPayloadHelloWorld {
        headers,
        arg,
        more,
        qa_floot,
        qa_other,
    };
    let span = app_api.span("impl_http_hello_world");
    let output = servicer_data.app_implementation.http_endpoint_hello_world(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_HELLO_WORLD.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"hello_world\"", route: "\"/hello_world/{arg:TEXT}/{more:BOOL}?{other:INT}&{floot:FLOAT[]}\"" });
        new_e
    })?;
    drop(span);
    let output = output.into_string();
    METRIC_HTTP_LATENCY_HELLO_WORLD.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_HELLO_WORLD.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadExample {
    pub input_body: BwTypeTestVtype,
}

#[::actix_web::post("/example")]
async fn http_endpoint_example(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, payload: ::actix_web::web::Payload) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_example", http_fetch_trace_id(&req));
    match req.headers().get("content-type") {
        Some(h) => {
            if h.as_bytes() != "application/json".as_bytes() {
                return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: application/json"));
            }
        }
        None => {
            return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: application/json"));
        }
    }

    let input_body = fetch_body_with_limit(payload, 262144).await?;
    let input_body = bw_type_test_vtype_deserialize_json(input_body.as_ref()).map_err(|e| EdenPlatformHttpError::InputPayloadJsonDeserializationError(e))?;
    let payload = HttpEndpointPayloadExample {
        input_body,
    };
    let span = app_api.span("impl_http_example");
    let output = servicer_data.app_implementation.http_endpoint_example(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_EXAMPLE.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"example\"", route: "\"/example\"" });
        new_e
    })?;
    drop(span);
    let output = bw_type_test_vtype_serialize_json(&output);
    METRIC_HTTP_LATENCY_EXAMPLE.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_EXAMPLE.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadMutateTest1 {
    pub input_body: BwTypeTestVtype,
}

#[::actix_web::post("/mutate_test_1")]
async fn http_endpoint_mutate_test_1(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, payload: ::actix_web::web::Payload) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_mutate_test_1", http_fetch_trace_id(&req));
    match req.headers().get("content-type") {
        Some(h) => {
            if h.as_bytes() != "application/json".as_bytes() {
                return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: application/json"));
            }
        }
        None => {
            return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: application/json"));
        }
    }

    let input_body = fetch_body_with_limit(payload, 262144).await?;
    let input_body = bw_type_test_vtype_deserialize_json(input_body.as_ref()).map_err(|e| EdenPlatformHttpError::InputPayloadJsonDeserializationError(e))?;
    let payload = HttpEndpointPayloadMutateTest1 {
        input_body,
    };
    let span = app_api.span("impl_http_mutate_test_1");
    let output = servicer_data.app_implementation.http_endpoint_mutate_test_1(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_MUTATE_TEST_1.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"mutate_test_1\"", route: "\"/mutate_test_1\"" });
        new_e
    })?;
    drop(span);
    let output = bw_type_test_output_type_serialize_json(&output);
    METRIC_HTTP_LATENCY_MUTATE_TEST_1.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_MUTATE_TEST_1.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadReadTest1 {
}

#[::actix_web::get("/rt_1")]
async fn http_endpoint_read_test_1(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_read_test_1", http_fetch_trace_id(&req));
    let payload = HttpEndpointPayloadReadTest1 {
    };
    let span = app_api.span("impl_http_read_test_1");
    let output = servicer_data.app_implementation.http_endpoint_read_test_1(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_READ_TEST_1.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"read_test_1\"", route: "\"/rt_1\"" });
        new_e
    })?;
    drop(span);
    let output = bw_type_test_output_type_serialize_json(&output);
    METRIC_HTTP_LATENCY_READ_TEST_1.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_READ_TEST_1.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadDummy {
    pub qa_floatv_arg: Vec<f64>,
    pub qa_int_arg: Option<i64>,
    pub input_body: BwTypeTestVtype,
}

#[::actix_web::post("/dummy")]
async fn http_endpoint_dummy(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, payload: ::actix_web::web::Payload) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_dummy", http_fetch_trace_id(&req));
    match req.headers().get("content-type") {
        Some(h) => {
            if h.as_bytes() != "application/json".as_bytes() {
                return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: application/json"));
            }
        }
        None => {
            return Ok(::actix_web::HttpResponse::BadRequest().body("Expected Content-Type: application/json"));
        }
    }

    let qstring = ::qstring::QString::from(req.query_string());
    let mut qa_floatv_arg: Vec<f64> = Vec::new();
    let mut qa_int_arg: Option<i64> = None;
    for (k, v) in qstring.to_pairs() {
        match k {
            "floatv_arg" => { qa_floatv_arg.push(v.parse::<f64>().map_err(|_| EdenPlatformHttpError::CannotParseQueryArgument)?) }
            "int_arg" => { if qa_int_arg.is_none() { qa_int_arg = Some(v.parse::<i64>().map_err(|_| EdenPlatformHttpError::CannotParseQueryArgument)?) } }
            _ => {},
        }
    }
    let input_body = fetch_body_with_limit(payload, 262144).await?;
    let input_body = bw_type_test_vtype_deserialize_json(input_body.as_ref()).map_err(|e| EdenPlatformHttpError::InputPayloadJsonDeserializationError(e))?;
    let payload = HttpEndpointPayloadDummy {
        input_body,
        qa_floatv_arg,
        qa_int_arg,
    };
    let span = app_api.span("impl_http_dummy");
    let output = servicer_data.app_implementation.http_endpoint_dummy(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_DUMMY.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"dummy\"", route: "\"/dummy?{int_arg:INT}&{floatv_arg:FLOAT[]}\"" });
        new_e
    })?;
    drop(span);
    let output = bw_type_test_vtype_serialize_json(&output);
    METRIC_HTTP_LATENCY_DUMMY.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_DUMMY.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadDownloadFile {
    pub path: String,
}

#[::actix_web::get("/files/{path}")]
async fn http_endpoint_download_file(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, path: ::actix_web::web::Path<String>) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_download_file", http_fetch_trace_id(&req));
    let path = path.into_inner();
    let payload = HttpEndpointPayloadDownloadFile {
        path,
    };
    let span = app_api.span("impl_http_download_file");
    let output = servicer_data.app_implementation.http_endpoint_download_file(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_DOWNLOAD_FILE.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"download_file\"", route: "\"/files/{path:TEXT}\"" });
        new_e
    })?;
    drop(span);
    METRIC_HTTP_LATENCY_DOWNLOAD_FILE.observe((::std::time::Instant::now() - pre).as_secs_f64());
    Ok(output)
}


pub struct HttpEndpointPayloadUploadFile {
    pub path: String,
    pub input_body: Vec<u8>,
}

#[::actix_web::post("/files/{path}")]
async fn http_endpoint_upload_file(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, payload: ::actix_web::web::Payload, path: ::actix_web::web::Path<String>) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_upload_file", http_fetch_trace_id(&req));
    let path = path.into_inner();
    let input_body = fetch_body_with_limit(payload, 262144).await?.to_vec();
    let payload = HttpEndpointPayloadUploadFile {
        input_body,
        path,
    };
    let span = app_api.span("impl_http_upload_file");
    let output = servicer_data.app_implementation.http_endpoint_upload_file(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_UPLOAD_FILE.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"upload_file\"", route: "\"/files/{path:TEXT}\"" });
        new_e
    })?;
    drop(span);
    METRIC_HTTP_LATENCY_UPLOAD_FILE.observe((::std::time::Instant::now() - pre).as_secs_f64());
    Ok(output)
}


pub struct HttpEndpointPayloadUploadFileMultipart {
    pub path: String,
    pub input_body: ::tokio::sync::mpsc::Receiver<Result<Vec<u8>, ::actix_web::error::PayloadError>>,
}

#[::actix_web::post("/files-m/{path}")]
async fn http_endpoint_upload_file_multipart(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, payload: ::actix_web::web::Payload, path: ::actix_web::web::Path<String>) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_upload_file_multipart", http_fetch_trace_id(&req));
    let path = path.into_inner();
    let mut payload_to_process = payload;
    let (tx, input_body) = ::tokio::sync::mpsc::channel(64);
    let payload = HttpEndpointPayloadUploadFileMultipart {
        input_body,
        path,
    };
    let span = app_api.span("impl_http_upload_file_multipart");
    let (output, dump) = ::futures_util::join!(
        servicer_data.app_implementation.http_endpoint_upload_file_multipart(&app_api, payload),
        dump_body_to_channel(&mut payload_to_process, tx),
    );
    dump?;
    let output = output.map_err(|e| {
        METRIC_HTTP_ERRORS_UPLOAD_FILE_MULTIPART.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"upload_file_multipart\"", route: "\"/files-m/{path:TEXT}\"" });
        new_e
    })?;
    drop(span);
    METRIC_HTTP_LATENCY_UPLOAD_FILE_MULTIPART.observe((::std::time::Instant::now() - pre).as_secs_f64());
    Ok(output)
}


pub struct HttpEndpointPayloadConfigsTest {
}

#[::actix_web::get("/configs_test")]
async fn http_endpoint_configs_test(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_configs_test", http_fetch_trace_id(&req));
    let payload = HttpEndpointPayloadConfigsTest {
    };
    let span = app_api.span("impl_http_configs_test");
    let output = servicer_data.app_implementation.http_endpoint_configs_test(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_CONFIGS_TEST.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"configs_test\"", route: "\"/configs_test\"" });
        new_e
    })?;
    drop(span);
    let output = output.into_string();
    METRIC_HTTP_LATENCY_CONFIGS_TEST.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_CONFIGS_TEST.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadBgJobCounter {
}

#[::actix_web::get("/bg_job_counter")]
async fn http_endpoint_bg_job_counter(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_bg_job_counter", http_fetch_trace_id(&req));
    let payload = HttpEndpointPayloadBgJobCounter {
    };
    let span = app_api.span("impl_http_bg_job_counter");
    let output = servicer_data.app_implementation.http_endpoint_bg_job_counter(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_BG_JOB_COUNTER.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"bg_job_counter\"", route: "\"/bg_job_counter\"" });
        new_e
    })?;
    drop(span);
    let output = output.into_string();
    METRIC_HTTP_LATENCY_BG_JOB_COUNTER.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_BG_JOB_COUNTER.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadChInsertSelect {
    pub id: i64,
}

#[::actix_web::get("/ch_foo_insert_select/{id}")]
async fn http_endpoint_ch_insert_select(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, path: ::actix_web::web::Path<i64>) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_ch_insert_select", http_fetch_trace_id(&req));
    let id = path.into_inner();
    let payload = HttpEndpointPayloadChInsertSelect {
        id,
    };
    let span = app_api.span("impl_http_ch_insert_select");
    let output = servicer_data.app_implementation.http_endpoint_ch_insert_select(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_CH_INSERT_SELECT.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"ch_insert_select\"", route: "\"/ch_foo_insert_select/{id:INT}\"" });
        new_e
    })?;
    drop(span);
    let output = output.into_string();
    METRIC_HTTP_LATENCY_CH_INSERT_SELECT.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_CH_INSERT_SELECT.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadNatsChSink {
    pub id: i64,
}

#[::actix_web::get("/nats_publish_get_max/{id}")]
async fn http_endpoint_nats_ch_sink(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, path: ::actix_web::web::Path<i64>) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_nats_ch_sink", http_fetch_trace_id(&req));
    let id = path.into_inner();
    let payload = HttpEndpointPayloadNatsChSink {
        id,
    };
    let span = app_api.span("impl_http_nats_ch_sink");
    let output = servicer_data.app_implementation.http_endpoint_nats_ch_sink(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_NATS_CH_SINK.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"nats_ch_sink\"", route: "\"/nats_publish_get_max/{id:INT}\"" });
        new_e
    })?;
    drop(span);
    let output = output.into_string();
    METRIC_HTTP_LATENCY_NATS_CH_SINK.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_NATS_CH_SINK.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(output)
    )
}


pub struct HttpEndpointPayloadTestChMutator {
    pub min_id: i64,
}

#[::actix_web::get("/test_ch_mutator/{min_id}")]
async fn http_endpoint_test_ch_mutator(servicer_data: ::actix_web::web::Data<AppServicerApi>, #[allow(unused_variables)] req: ::actix_web::HttpRequest, path: ::actix_web::web::Path<i64>) -> Result<::actix_web::HttpResponse, EdenPlatformHttpError> {
    let pre = ::std::time::Instant::now();
    let app_api = build_app_api(&servicer_data, "gen_http_test_ch_mutator", http_fetch_trace_id(&req));
    let min_id = path.into_inner();
    let payload = HttpEndpointPayloadTestChMutator {
        min_id,
    };
    let span = app_api.span("impl_http_test_ch_mutator");
    let output = servicer_data.app_implementation.http_endpoint_test_ch_mutator(&app_api, payload).await.map_err(|e| {
        METRIC_HTTP_ERRORS_TEST_CH_MUTATOR.inc();
        let new_e = EdenPlatformHttpError::InternalError(e.to_string());
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"test_ch_mutator\"", route: "\"/test_ch_mutator/{min_id:INT}\"" });
        new_e
    })?;
    drop(span);
    let output = output.into_string();
    METRIC_HTTP_LATENCY_TEST_CH_MUTATOR.observe((::std::time::Instant::now() - pre).as_secs_f64());
    METRIC_HTTP_BYTES_TEST_CH_MUTATOR.inc_by(output.len().try_into().unwrap_or_default());
    Ok(
        ::actix_web::HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(output)
    )
}

async fn setup_jetstream_consumers_and_publishers(servicer_data: AppServicerApi) -> Result<(), ::async_nats::Error> {
    use ::futures_util::StreamExt;

    // initialize not yet initialized producer stream
    let _ = servicer_data.r.nats_conns[servicer_data.r.nats_conn_id_some_test_stream_producer].get_or_create_stream(::async_nats::jetstream::stream::Config {
        name: servicer_data.r.nats_stream_some_test_stream_producer.clone(),
        ..Default::default()
    }).await?;
    // initialize not yet initialized producer stream
    let _ = servicer_data.r.nats_conns[servicer_data.r.nats_conn_id_simple_msg_stream].get_or_create_stream(::async_nats::jetstream::stream::Config {
        name: servicer_data.r.nats_stream_simple_msg_stream.clone(),
        ..Default::default()
    }).await?;
    {
        let stream_name = &servicer_data.r.nats_stream_some_test_stream_consumer;
        let consumer_name = &servicer_data.r.deployment_name;

        let stream = servicer_data.r.nats_conns[servicer_data.r.nats_conn_id_some_test_stream_consumer].get_or_create_stream(::async_nats::jetstream::stream::Config {
            name: stream_name.clone(),
            ..Default::default()
        }).await?;
        let consumer = stream.get_or_create_consumer(consumer_name, ::async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(consumer_name.clone()),
            ..Default::default()
        }).await?;

        let stream_name = stream_name.clone();


        tokio::spawn(async move {
            loop {
                match consumer.messages().await {
                    Ok(mut messages) => {
                        loop {
                            match messages.next().await {
                                Some(msg) => {
                                    match msg {
                                        Ok(msg) => {
                                            let app_api = build_app_api(&servicer_data, "gen_js_input_some_test_stream_consumer", nats_fetch_trace_id(&msg));
                                            let bytes = msg.message.payload.as_ref();
                                            let payload_size = bytes.len();
                                            let pre = ::std::time::Instant::now();
                                            let deserialized_message = bw_type_test_vtype_deserialize_json(bytes);
                                            let stripped_subject = msg.subject.as_str().strip_prefix(stream_name.as_str()).expect("Can't strip subject name");
                                            assert!(stripped_subject.len() > 0);
                                            let subject = &stripped_subject[1..];
                                            match deserialized_message {
                                                Ok(input) => {
                                                    let mut span = app_api.span("impl_js_input_some_test_stream_consumer");
                                                    let res = servicer_data.app_implementation.jetstream_consume_some_test_stream_consumer(&app_api, input, &subject).await;
                                                    METRIC_NATS_CONSUMER_LATENCY_SOME_TEST_STREAM_CONSUMER.observe((::std::time::Instant::now() - pre).as_secs_f64());
                                                    match res {
                                                        Ok(()) => {
                                                            if let Err(e) = msg.ack().await {
                                                                span.set_status(::opentelemetry::trace::Status::error(e.to_string()));
                                                                ::log::error!("Error while acking successfully processed message: {:?}", e);
                                                                break;
                                                            };
                                                            METRIC_NATS_CONSUMER_BYTES_SOME_TEST_STREAM_CONSUMER.inc_by(payload_size.try_into().unwrap_or_default());
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


    Ok(())
}


::lazy_static::lazy_static! {
    static ref EPL_DEPLOYMENT_NAME: String = ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured");
    static ref EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER: String = ::std::env::var("EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER").expect("Mandatory environment variable EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER is not configured");
    static ref EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER: String = ::std::env::var("EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER").expect("Mandatory environment variable EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER is not configured");
    static ref EPL_NATS_STREAM_SIMPLE_MSG_STREAM: String = ::std::env::var("EPL_NATS_STREAM_SIMPLE_MSG_STREAM").expect("Mandatory environment variable EPL_NATS_STREAM_SIMPLE_MSG_STREAM is not configured");

    static ref METRIC_CHI_CHSHARD_FOO: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_ch_query_time",
        "Clickhouse query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "application_ch_shard".to_string() => "chshard".to_string(),
            "application_ch_schema".to_string() => "testch".to_string(),
            "query".to_string() => "foo".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_inserter".to_string() => "false".to_string(),
        },
    )).unwrap();

    static ref METRIC_CHM_CHSHARD_COPY_IDS_FROM_FOO: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_ch_query_time",
        "Clickhouse query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "application_ch_shard".to_string() => "chshard".to_string(),
            "application_ch_schema".to_string() => "testch".to_string(),
            "query".to_string() => "copy_ids_from_foo".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_inserter".to_string() => "true".to_string(),
        },
    )).unwrap();

    static ref METRIC_CHQ_CHSHARD_MAX_ID_FROM_FOO: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_ch_query_time",
        "Clickhouse query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "application_ch_shard".to_string() => "chshard".to_string(),
            "application_ch_schema".to_string() => "testch".to_string(),
            "query".to_string() => "max_id_from_foo".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_inserter".to_string() => "true".to_string(),
        },
    )).unwrap();

    static ref METRIC_CHQ_CHSHARD_MAX_ID_FROM_FOO_IDS: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_ch_query_time",
        "Clickhouse query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "application_ch_shard".to_string() => "chshard".to_string(),
            "application_ch_schema".to_string() => "testch".to_string(),
            "query".to_string() => "max_id_from_foo_ids".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_inserter".to_string() => "true".to_string(),
        },
    )).unwrap();

    static ref METRIC_CHQ_CHSHARD_MAX_ID_FROM_IMP: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_ch_query_time",
        "Clickhouse query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "application_ch_shard".to_string() => "chshard".to_string(),
            "application_ch_schema".to_string() => "testch".to_string(),
            "query".to_string() => "max_id_from_imp".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_inserter".to_string() => "true".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_BG_JOB_COUNTER: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "bg_job_counter",
            "http_method" => "GET",
            "http_path" => "/bg_job_counter",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_CH_INSERT_SELECT: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "ch_insert_select",
            "http_method" => "GET",
            "http_path" => "/ch_foo_insert_select/{id:INT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_CONFIGS_TEST: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "configs_test",
            "http_method" => "GET",
            "http_path" => "/configs_test",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_DUMMY: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "dummy",
            "http_method" => "POST",
            "http_path" => "/dummy?{int_arg:INT}&{floatv_arg:FLOAT[]}",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_EXAMPLE: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "example",
            "http_method" => "POST",
            "http_path" => "/example",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_HELLO_WORLD: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "hello_world",
            "http_method" => "GET",
            "http_path" => "/hello_world/{arg:TEXT}/{more:BOOL}?{other:INT}&{floot:FLOAT[]}",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_MUTATE_TEST_1: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "mutate_test_1",
            "http_method" => "POST",
            "http_path" => "/mutate_test_1",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_NATS_CH_SINK: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "nats_ch_sink",
            "http_method" => "GET",
            "http_path" => "/nats_publish_get_max/{id:INT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_READ_TEST_1: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "read_test_1",
            "http_method" => "GET",
            "http_path" => "/rt_1",
        },
    )).unwrap();

    static ref METRIC_HTTP_BYTES_TEST_CH_MUTATOR: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_sent_bytes",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "test_ch_mutator",
            "http_method" => "GET",
            "http_path" => "/test_ch_mutator/{min_id:INT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_BG_JOB_COUNTER: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "bg_job_counter",
            "http_method" => "GET",
            "http_path" => "/bg_job_counter",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_CH_INSERT_SELECT: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "ch_insert_select",
            "http_method" => "GET",
            "http_path" => "/ch_foo_insert_select/{id:INT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_CONFIGS_TEST: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "configs_test",
            "http_method" => "GET",
            "http_path" => "/configs_test",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_DOWNLOAD_FILE: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "download_file",
            "http_method" => "GET",
            "http_path" => "/files/{path:TEXT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_DUMMY: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "dummy",
            "http_method" => "POST",
            "http_path" => "/dummy?{int_arg:INT}&{floatv_arg:FLOAT[]}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_EXAMPLE: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "example",
            "http_method" => "POST",
            "http_path" => "/example",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_HELLO_WORLD: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "hello_world",
            "http_method" => "GET",
            "http_path" => "/hello_world/{arg:TEXT}/{more:BOOL}?{other:INT}&{floot:FLOAT[]}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_MUTATE_TEST_1: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "mutate_test_1",
            "http_method" => "POST",
            "http_path" => "/mutate_test_1",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_NATS_CH_SINK: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "nats_ch_sink",
            "http_method" => "GET",
            "http_path" => "/nats_publish_get_max/{id:INT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_READ_TEST_1: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "read_test_1",
            "http_method" => "GET",
            "http_path" => "/rt_1",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_TEST_CH_MUTATOR: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "test_ch_mutator",
            "http_method" => "GET",
            "http_path" => "/test_ch_mutator/{min_id:INT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_UPLOAD_FILE: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "upload_file",
            "http_method" => "POST",
            "http_path" => "/files/{path:TEXT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_ERRORS_UPLOAD_FILE_MULTIPART: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "upload_file_multipart",
            "http_method" => "POST",
            "http_path" => "/files-m/{path:TEXT}",
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_BG_JOB_COUNTER: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "bg_job_counter".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/bg_job_counter".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_CH_INSERT_SELECT: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "ch_insert_select".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/ch_foo_insert_select/{id:INT}".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_CONFIGS_TEST: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "configs_test".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/configs_test".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_DOWNLOAD_FILE: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "download_file".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/files/{path:TEXT}".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_DUMMY: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "dummy".to_string(),
            "http_method".to_string() => "POST".to_string(),
            "http_path".to_string() => "/dummy?{int_arg:INT}&{floatv_arg:FLOAT[]}".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_EXAMPLE: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "example".to_string(),
            "http_method".to_string() => "POST".to_string(),
            "http_path".to_string() => "/example".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_HELLO_WORLD: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "hello_world".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/hello_world/{arg:TEXT}/{more:BOOL}?{other:INT}&{floot:FLOAT[]}".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_MUTATE_TEST_1: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "mutate_test_1".to_string(),
            "http_method".to_string() => "POST".to_string(),
            "http_path".to_string() => "/mutate_test_1".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_NATS_CH_SINK: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "nats_ch_sink".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/nats_publish_get_max/{id:INT}".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_READ_TEST_1: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "read_test_1".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/rt_1".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_TEST_CH_MUTATOR: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "test_ch_mutator".to_string(),
            "http_method".to_string() => "GET".to_string(),
            "http_path".to_string() => "/test_ch_mutator/{min_id:INT}".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_UPLOAD_FILE: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "upload_file".to_string(),
            "http_method".to_string() => "POST".to_string(),
            "http_path".to_string() => "/files/{path:TEXT}".to_string(),
        },
    )).unwrap();

    static ref METRIC_HTTP_LATENCY_UPLOAD_FILE_MULTIPART: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "upload_file_multipart".to_string(),
            "http_method".to_string() => "POST".to_string(),
            "http_path".to_string() => "/files-m/{path:TEXT}".to_string(),
        },
    )).unwrap();

    static ref METRIC_NATS_CONSUMER_BYTES_SOME_TEST_STREAM_CONSUMER: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_nats_js_processed_bytes",
        "Bytes processed successfully from nats stream",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "nats_stream" => EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER.as_str(),
            "app_stream" => "some_test_stream_consumer",
        },
    )).unwrap();

    static ref METRIC_NATS_CONSUMER_LATENCY_SOME_TEST_STREAM_CONSUMER: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_nats_js_message_process_time",
        "Time in which nats message was processed for stream",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "nats_stream".to_string() => EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER.clone(),
            "app_stream".to_string() => "some_test_stream_consumer".to_string(),
        },
    )).unwrap();

    static ref METRIC_NATS_PUBLISH_BYTES_SIMPLE_MSG_STREAM: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_nats_js_published_bytes",
        "Bytes sent to nats stream",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "nats_stream" => EPL_NATS_STREAM_SIMPLE_MSG_STREAM.as_str(),
            "app_stream" => "simple_msg_stream",
        },
    )).unwrap();

    static ref METRIC_NATS_PUBLISH_BYTES_SOME_TEST_STREAM_PRODUCER: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_nats_js_published_bytes",
        "Bytes sent to nats stream",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "nats_stream" => EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER.as_str(),
            "app_stream" => "some_test_stream_producer",
        },
    )).unwrap();

    static ref METRIC_NATS_PUBLISH_LATENCY_SIMPLE_MSG_STREAM: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_nats_js_publish_time",
        "Time in which nats message was published to stream",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "nats_stream".to_string() => EPL_NATS_STREAM_SIMPLE_MSG_STREAM.clone(),
            "app_stream".to_string() => "simple_msg_stream".to_string(),
        },
    )).unwrap();

    static ref METRIC_NATS_PUBLISH_LATENCY_SOME_TEST_STREAM_PRODUCER: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_nats_js_publish_time",
        "Time in which nats message was published to stream",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "nats_stream".to_string() => EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER.clone(),
            "app_stream".to_string() => "some_test_stream_producer".to_string(),
        },
    )).unwrap();

    static ref METRIC_PGM_TESTDB_INSERT_ID: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_pg_query_time",
        "Postgres query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "pg_database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "none".to_string(),
        },
    )).unwrap();

    static ref METRIC_PGQ_TESTDB_INSERT_ID_RETURNING: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_pg_query_time",
        "Postgres query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "pg_database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id_returning".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "none".to_string(),
        },
    )).unwrap();

    static ref METRIC_PGQ_TESTDB_MAX_ID_FROM_FOO: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_pg_query_time",
        "Postgres query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "pg_database".to_string() => "testdb".to_string(),
            "query".to_string() => "max_id_from_foo".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "false".to_string(),
            "transaction".to_string() => "none".to_string(),
        },
    )).unwrap();

    static ref METRIC_PG_CONN_DEFAULT: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_connpool_time",
        "Time for acquiring database connection from connection pool",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "default".to_string(),
        },
    )).unwrap();

    static ref METRIC_PG_TRX_DEFAULT_ALL_AT_ONCE: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_transaction_time",
        "Database transaction time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "default".to_string(),
            "transaction".to_string() => "all_at_once".to_string(),
            "is_read_only".to_string() => "false".to_string(),
        },
    )).unwrap();

    static ref METRIC_TRX_PGM_TESTDB_INSERT_ID: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_pg_query_time",
        "Postgres query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "pg_database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "all_at_once".to_string(),
        },
    )).unwrap();

    static ref METRIC_TRX_PGQ_TESTDB_INSERT_ID_RETURNING: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_pg_query_time",
        "Postgres query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "pg_database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id_returning".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "all_at_once".to_string(),
        },
    )).unwrap();

    static ref METRIC_TRX_PGQ_TESTDB_MAX_ID_FROM_FOO: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_pg_query_time",
        "Postgres query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "pg_database".to_string() => "testdb".to_string(),
            "query".to_string() => "max_id_from_foo".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "false".to_string(),
            "transaction".to_string() => "all_at_once".to_string(),
        },
    )).unwrap();
}
