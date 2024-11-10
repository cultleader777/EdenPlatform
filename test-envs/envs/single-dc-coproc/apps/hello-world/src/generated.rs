#[allow(unused_imports)]
use serde::{Serialize, Deserialize};
#[allow(unused_imports)]
use opentelemetry::trace::{TraceContextExt, Tracer};


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
    dbconn_default: ::bb8::Pool<::bb8_postgres::PostgresConnectionManager<::tokio_postgres::NoTls>>,
    nats_conn_id_some_test_stream_producer: usize,
    nats_stream_some_test_stream_producer: String,
    nats_conn_id_some_test_stream_consumer: usize,
    nats_stream_some_test_stream_consumer: String,
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
        let epl_dbconn_default = ::std::env::var("EPL_DBCONN_DEFAULT").expect("Mandatory environment variable EPL_DBCONN_DEFAULT not configured");
        let epl_nats_conn_some_test_stream_producer = ::std::env::var("EPL_NATS_CONN_SOME_TEST_STREAM_PRODUCER").expect("Mandatory environment variable EPL_NATS_CONN_SOME_TEST_STREAM_PRODUCER not configured");
        let nats_stream_some_test_stream_producer = ::std::env::var("EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER").expect("Mandatory environment variable EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER not configured").to_string();
        let epl_nats_conn_some_test_stream_consumer = ::std::env::var("EPL_NATS_CONN_SOME_TEST_STREAM_CONSUMER").expect("Mandatory environment variable EPL_NATS_CONN_SOME_TEST_STREAM_CONSUMER not configured");
        let nats_stream_some_test_stream_consumer = ::std::env::var("EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER").expect("Mandatory environment variable EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER not configured").to_string();
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

        let manager = ::bb8_postgres::PostgresConnectionManager::new(
            ::tokio_postgres::Config::from_str(&epl_dbconn_default)?, ::tokio_postgres::NoTls
        );
        let dbconn_default = ::bb8::Pool::builder()
            .build(manager)
            .await?;


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
            dbconn_default,
            nats_conn_id_some_test_stream_producer,
            nats_stream_some_test_stream_producer,
            nats_conn_id_some_test_stream_consumer,
            nats_stream_some_test_stream_consumer,
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
enum EplTracingContext {
    Context(::opentelemetry::Context),
    NoContext,
}

/// This struct carries context for one app request or message processing
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

    pub async fn dbq_max_id_from_foo(&self, test_arg: i32) -> Result<Vec<DbqRowMaxIdFromFoo>, DbInteractionError> {
        let pre = ::std::time::Instant::now();
        let span = self.span("dbconn_get");
        let conn = self.r.dbconn_default.get().await.map_err(|e| {
                DbInteractionError::ConnectionPoolError(e)
            })?;
        drop(span);
        METRIC_DB_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());
        let the_query = r#"SELECT max(id) AS max_id FROM foo WHERE $1 > 0"#;
        let pre = ::std::time::Instant::now();
        let _span = self.span("dbq_max_id_from_foo");
        let rows = conn.query(the_query, &[&test_arg]).await.map_err(|e| DbInteractionError::PostgresError(e))?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_DBQ_TESTDB_MAX_ID_FROM_FOO.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(DbqRowMaxIdFromFoo {
                max_id: r.try_get::<usize, Option<i32>>(0).map_err(|e| DbInteractionError::DeserializationError(e.to_string()))?,
            });
        }
        Ok(res)
    }

    pub async fn dbmq_insert_id_returning(&self, test_arg: i32) -> Result<Vec<DbqRowInsertIdReturning>, DbInteractionError> {
        let pre = ::std::time::Instant::now();
        let span = self.span("dbconn_get");
        let conn = self.r.dbconn_default.get().await.map_err(|e| {
                DbInteractionError::ConnectionPoolError(e)
            })?;
        drop(span);
        METRIC_DB_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());
        let the_query = r#"INSERT INTO foo(id) VALUES($1) RETURNING id"#;
        let pre = ::std::time::Instant::now();
        let _span = self.span("dbmq_insert_id_returning");
        let rows = conn.query(the_query, &[&test_arg]).await.map_err(|e| DbInteractionError::PostgresError(e))?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_DBQ_TESTDB_INSERT_ID_RETURNING.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(DbqRowInsertIdReturning {
                id: r.try_get::<usize, i32>(0).map_err(|e| DbInteractionError::DeserializationError(e.to_string()))?,
            });
        }
        Ok(res)
    }

    pub async fn dbm_insert_id(&self, test_arg: i32) -> Result<u64, DbInteractionError> {
        let pre = ::std::time::Instant::now();
        let span = self.span("dbconn_get");
        let conn = self.r.dbconn_default.get().await.map_err(|e| {
                DbInteractionError::ConnectionPoolError(e)
            })?;
        drop(span);
        METRIC_DB_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());
        let the_query = r#"INSERT INTO foo(id) VALUES($1)"#;
        let pre = ::std::time::Instant::now();
        let _span = self.span("dbm_insert_id");
        let res = conn.execute(the_query, &[&test_arg]).await.map_err(|e| DbInteractionError::PostgresError(e))?;
        METRIC_DBM_TESTDB_INSERT_ID.observe((::std::time::Instant::now() - pre).as_secs_f64());
        Ok(res)
    }

    pub async fn dbtrx_begin_all_at_once(&self) -> Result<TrxDefaultAllAtOnceS1, DbInteractionError> {
        let pre = ::std::time::Instant::now();
        let context = ::opentelemetry::Context::current_with_span(self.span("dbtrx_all_at_once"));
        let span = self.r.tracer.start_with_context("dbconn_get", &context);
        let conn = self.r.dbconn_default.get().await.map_err(|e| {
            DbInteractionError::ConnectionPoolError(e)
        })?;
        drop(span);
        METRIC_DB_CONN_DEFAULT.observe((::std::time::Instant::now() - pre).as_secs_f64());

        // make lifetime longer to survive builder pattern
        let conn: ::bb8::PooledConnection<'static, ::bb8_postgres::PostgresConnectionManager<::tokio_postgres::NoTls>> = unsafe {
            ::std::mem::transmute(conn)
        };
        // pin conn in memory so that transaction could rely on its location
        let mut conn = Box::new(conn);
        let trx = conn.transaction().await.map_err(|e| {
            DbInteractionError::PostgresError(e)
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

    pub async fn jetstream_publish_some_test_stream_producer(&self, input: &BwTypeTestVtype) -> Result<::async_nats::jetstream::context::PublishAckFuture, ::async_nats::error::Error<::async_nats::jetstream::context::PublishErrorKind>> {
        use opentelemetry::trace::Span;        let pre = ::std::time::Instant::now();
        let span = self.span("js_publish_some_test_stream_producer");
        let trace_id = span.span_context().trace_id().to_string();
        let payload: ::bytes::Bytes = bw_type_test_vtype_serialize_bin(input).into();
        let payload_size = payload.len();
        let mut headers = ::async_nats::HeaderMap::new();
        headers.insert("trace-id", trace_id.as_str());
        let res = self.r.nats_conns[self.r.nats_conn_id_some_test_stream_producer].publish_with_headers(self.r.nats_stream_some_test_stream_producer.clone(), headers, payload).await;
        if res.is_ok() {
            METRIC_NATS_PUBLISH_LATENCY_SOME_TEST_STREAM_PRODUCER.observe((::std::time::Instant::now() - pre).as_secs_f64());
            METRIC_NATS_PUBLISH_BYTES_SOME_TEST_STREAM_PRODUCER.inc_by(payload_size.try_into().unwrap_or_default());
        }
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
    pub async fn dbmq_insert_id_returning(self, test_arg: i32) -> Result<TrxDefaultAllAtOnceS2, DbInteractionError> {
        let the_query = r#"INSERT INTO foo(id) VALUES($1) RETURNING id"#;
        let pre = ::std::time::Instant::now();
        let _span = self.state._r.tracer.start_with_context("dbmq_insert_id_returning", &self.state.context);
        let rows = self.state.trx.query(the_query, &[&test_arg]).await.map_err(|e| DbInteractionError::PostgresError(e))?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_TRX_DBQ_TESTDB_INSERT_ID_RETURNING.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(DbqRowInsertIdReturning {
                id: r.try_get::<usize, i32>(0).map_err(|e| DbInteractionError::DeserializationError(e.to_string()))?,
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
    r_insert_id_returning: Vec<DbqRowInsertIdReturning>,
}

impl TrxDefaultAllAtOnceS2 {
    pub async fn dbq_max_id_from_foo(&mut self, test_arg: i32) -> Result<Vec<DbqRowMaxIdFromFoo>, DbInteractionError> {
        let the_query = r#"SELECT max(id) AS max_id FROM foo WHERE $1 > 0"#;
        let pre = ::std::time::Instant::now();
        let _span = self.state._r.tracer.start_with_context("dbq_max_id_from_foo", &self.state.context);
        let rows = self.state.trx.query(the_query, &[&test_arg]).await.map_err(|e| DbInteractionError::PostgresError(e))?;
        let mut res = Vec::with_capacity(rows.len());
        METRIC_TRX_DBQ_TESTDB_MAX_ID_FROM_FOO.observe((::std::time::Instant::now() - pre).as_secs_f64());
        for r in rows {
            res.push(DbqRowMaxIdFromFoo {
                max_id: r.try_get::<usize, Option<i32>>(0).map_err(|e| DbInteractionError::DeserializationError(e.to_string()))?,
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
    pub fn r_insert_id_returning(&self) -> &[DbqRowInsertIdReturning] {
        &self.r_insert_id_returning
    }
}

pub struct TrxDefaultAllAtOnceS3 {
    state: TransactionState,
    r_insert_id_returning: Vec<DbqRowInsertIdReturning>,
}

impl TrxDefaultAllAtOnceS3 {
    pub async fn dbm_insert_id(&mut self, test_arg: i32) -> Result<u64, DbInteractionError> {
        let the_query = r#"INSERT INTO foo(id) VALUES($1)"#;
        let pre = ::std::time::Instant::now();
        let _span = self.state._r.tracer.start_with_context("dbm_insert_id", &self.state.context);
        let res = self.state.trx.execute(the_query, &[&test_arg]).await.map_err(|e| DbInteractionError::PostgresError(e))?;
        METRIC_TRX_DBM_TESTDB_INSERT_ID.observe((::std::time::Instant::now() - pre).as_secs_f64());

        Ok(res)
    }

    pub fn advance(self) -> TrxDefaultAllAtOnceS4 {
        TrxDefaultAllAtOnceS4 {
            state: self.state,
            r_insert_id_returning: self.r_insert_id_returning,
        }
    }

    #[allow(dead_code)]
    pub fn r_insert_id_returning(&self) -> &[DbqRowInsertIdReturning] {
        &self.r_insert_id_returning
    }
}

pub struct TrxDefaultAllAtOnceS4 {
    state: TransactionState,
    r_insert_id_returning: Vec<DbqRowInsertIdReturning>,
}

pub struct TrxDefaultAllAtOnceOutput {
    pub r_insert_id_returning: Vec<DbqRowInsertIdReturning>,
}

impl TrxDefaultAllAtOnceS4 {

    pub async fn commit(self) -> Result<TrxDefaultAllAtOnceOutput, DbInteractionError> {
        let _ = self.state.trx.commit().await.map_err(|e| {
            DbInteractionError::PostgresError(e)
        })?;

        METRIC_DB_TRX_DEFAULT_ALL_AT_ONCE.observe((::std::time::Instant::now() - self.state.start_time).as_secs_f64());

        Ok(TrxDefaultAllAtOnceOutput {
            r_insert_id_returning: self.r_insert_id_returning,
        })
    }
    #[allow(dead_code)]
    pub fn r_insert_id_returning(&self) -> &[DbqRowInsertIdReturning] {
        &self.r_insert_id_returning
    }
}

pub struct DbqRowInsertIdReturning {
    pub id: i32,
}
pub struct DbqRowMaxIdFromFoo {
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
    async fn jetstream_consume_some_test_stream_consumer(&self, api: &AppApi, payload: BwTypeTestVtype) -> Result<(), Box<dyn ::std::error::Error + Send + Sync>>;
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

fn bw_type_test_vtype_serialize_bin(input: &BwTypeTestVtype) -> Vec<u8> {
    let mut output = Vec::with_capacity(32);
    ::bincode::serialize_into(&mut output, &(0x0005c2f622b97ae7 as u64)).expect("should never happen");
    ::bincode::serialize_into(&mut output, input).expect("should never happen");
    output
}

fn bw_type_test_vtype_deserialize_bin(input: &[u8]) -> Result<BwTypeTestVtype, BinaryDeserializationError> {
    let cursor_size = input.len();
    let mut cursor = ::std::io::Cursor::new(input);
    let header: u64 = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::MessageTooShort)?;
    let version: u16 = ((header & 0xffff0000000000)>>48).try_into().unwrap();
    match version {
        5 => {
            if header != 1621737283877607 { return Err(BinaryDeserializationError::VersionHashMismatch { expected: 1621737283877607, actual: header }) }
            let the_val: BwTypeTestVtypeV5 = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::CorruptedData)?;
            if (cursor.position() as usize) < cursor_size { return Err(BinaryDeserializationError::ExtraBytesLeft) }
            Ok(the_val)
        }
        4 => {
            if header != 1396335038581596 { return Err(BinaryDeserializationError::VersionHashMismatch { expected: 1396335038581596, actual: header }) }
            let the_val: BwTypeTestVtypeV4 = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::CorruptedData)?;
            if (cursor.position() as usize) < cursor_size { return Err(BinaryDeserializationError::ExtraBytesLeft) }
            let the_val = bw_type_test_vtype_v4_to_v5(the_val);
            Ok(the_val)
        }
        3 => {
            if header != 962412026069000 { return Err(BinaryDeserializationError::VersionHashMismatch { expected: 962412026069000, actual: header }) }
            let the_val: BwTypeTestVtypeV3 = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::CorruptedData)?;
            if (cursor.position() as usize) < cursor_size { return Err(BinaryDeserializationError::ExtraBytesLeft) }
            let the_val = bw_type_test_vtype_v3_to_v4(the_val);
            let the_val = bw_type_test_vtype_v4_to_v5(the_val);
            Ok(the_val)
        }
        2 => {
            if header != 703544960600620 { return Err(BinaryDeserializationError::VersionHashMismatch { expected: 703544960600620, actual: header }) }
            let the_val: BwTypeTestVtypeV2 = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::CorruptedData)?;
            if (cursor.position() as usize) < cursor_size { return Err(BinaryDeserializationError::ExtraBytesLeft) }
            let the_val = bw_type_test_vtype_v2_to_v3(the_val);
            let the_val = bw_type_test_vtype_v3_to_v4(the_val);
            let the_val = bw_type_test_vtype_v4_to_v5(the_val);
            Ok(the_val)
        }
        1 => {
            if header != 408199628205529 { return Err(BinaryDeserializationError::VersionHashMismatch { expected: 408199628205529, actual: header }) }
            let the_val: BwTypeTestVtypeV1 = ::bincode::deserialize_from(&mut cursor).map_err(|_| BinaryDeserializationError::CorruptedData)?;
            if (cursor.position() as usize) < cursor_size { return Err(BinaryDeserializationError::ExtraBytesLeft) }
            let the_val = bw_type_test_vtype_v1_to_v2(the_val);
            let the_val = bw_type_test_vtype_v2_to_v3(the_val);
            let the_val = bw_type_test_vtype_v3_to_v4(the_val);
            let the_val = bw_type_test_vtype_v4_to_v5(the_val);
            Ok(the_val)
        }
        unknown_version => {
            if unknown_version > 5 { return Err(BinaryDeserializationError::UnsupportedVersionYet(unknown_version)) }
            Err(BinaryDeserializationError::UnknownVersion(unknown_version))
        }
    }
}


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
pub enum DbInteractionError {
    PostgresError(::tokio_postgres::Error),
    ConnectionPoolError(::bb8::RunError<::tokio_postgres::Error>),
    DeserializationError(String),
}

impl std::fmt::Display for DbInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "database interaction error: {:?}", self)
    }
}

impl std::error::Error for DbInteractionError {}




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
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"hello_world\"", route: "\"/hello_world/<arg:TEXT>/<more:BOOL>?<other:INT>&<floot:FLOAT[]>\"" });
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
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"dummy\"", route: "\"/dummy?<int_arg:INT>&<floatv_arg:FLOAT[]>\"" });
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
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"download_file\"", route: "\"/files/<path:TEXT>\"" });
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
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"upload_file\"", route: "\"/files/<path:TEXT>\"" });
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
          ::kv_log_macro::error!("Error serving http request", { error: format!("\"{}\"", e).as_str(), endpoint_name: "\"upload_file_multipart\"", route: "\"/files-m/<path:TEXT>\"" });
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

async fn setup_jetstream_consumers_and_publishers(servicer_data: AppServicerApi) -> Result<(), ::async_nats::Error> {
    use ::futures_util::StreamExt;

    // initialize not yet initialized producer stream
    let _ = servicer_data.r.nats_conns[servicer_data.r.nats_conn_id_some_test_stream_producer].get_or_create_stream(::async_nats::jetstream::stream::Config {
        name: servicer_data.r.nats_stream_some_test_stream_producer.clone(),
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
                                            let deserialized_message = bw_type_test_vtype_deserialize_bin(bytes);
                                            match deserialized_message {
                                                Ok(input) => {
                                                    let span = app_api.span("impl_js_input_some_test_stream_consumer");
                                                    let res = servicer_data.app_implementation.jetstream_consume_some_test_stream_consumer(&app_api, input).await;
                                                    drop(span);
                                                    METRIC_NATS_CONSUMER_LATENCY_SOME_TEST_STREAM_CONSUMER.observe((::std::time::Instant::now() - pre).as_secs_f64());
                                                    match res {
                                                        Ok(()) => {
                                                            if let Err(e) = msg.ack().await {
                                                                ::log::error!("Error while acking successfully processed message: {:?}", e);
                                                                break;
                                                            };
                                                            METRIC_NATS_CONSUMER_BYTES_SOME_TEST_STREAM_CONSUMER.inc_by(payload_size.try_into().unwrap_or_default());
                                                        },
                                                        Err(e) => {
                                                            ::log::error!("Got error while processing message: {:?}", e);
                                                            break;
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    ::log::error!("Error during deserialization of binary data: {:?}", e);
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

    static ref METRIC_DBM_TESTDB_INSERT_ID: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_query_time",
        "Database query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "none".to_string(),
        },
    )).unwrap();

    static ref METRIC_DBQ_TESTDB_INSERT_ID_RETURNING: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_query_time",
        "Database query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id_returning".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "none".to_string(),
        },
    )).unwrap();

    static ref METRIC_DBQ_TESTDB_MAX_ID_FROM_FOO: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_query_time",
        "Database query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "testdb".to_string(),
            "query".to_string() => "max_id_from_foo".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "false".to_string(),
            "transaction".to_string() => "none".to_string(),
        },
    )).unwrap();

    static ref METRIC_DB_CONN_DEFAULT: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_connpool_time",
        "Time for acquiring database connection from connection pool",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "default".to_string(),
        },
    )).unwrap();

    static ref METRIC_DB_TRX_DEFAULT_ALL_AT_ONCE: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
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
            "http_path" => "/dummy?<int_arg:INT>&<floatv_arg:FLOAT[]>",
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
            "http_path" => "/hello_world/<arg:TEXT>/<more:BOOL>?<other:INT>&<floot:FLOAT[]>",
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
            "http_path" => "/files/<path:TEXT>",
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
            "http_path" => "/dummy?<int_arg:INT>&<floatv_arg:FLOAT[]>",
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
            "http_path" => "/hello_world/<arg:TEXT>/<more:BOOL>?<other:INT>&<floot:FLOAT[]>",
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

    static ref METRIC_HTTP_ERRORS_UPLOAD_FILE: ::prometheus::IntCounter = ::prometheus::register_int_counter!(::prometheus::opts!(
        "epl_http_endpoint_errors",
        "Body bytes sent to the user",
        ::prometheus::labels! {
            "deployment_name" => EPL_DEPLOYMENT_NAME.as_str(),
            "application" => "hello-world",
            "endpoint_name" => "upload_file",
            "http_method" => "POST",
            "http_path" => "/files/<path:TEXT>",
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
            "http_path" => "/files-m/<path:TEXT>",
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
            "http_path".to_string() => "/files/<path:TEXT>".to_string(),
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
            "http_path".to_string() => "/dummy?<int_arg:INT>&<floatv_arg:FLOAT[]>".to_string(),
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
            "http_path".to_string() => "/hello_world/<arg:TEXT>/<more:BOOL>?<other:INT>&<floot:FLOAT[]>".to_string(),
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

    static ref METRIC_HTTP_LATENCY_UPLOAD_FILE: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_http_endpoint_process_time",
        "Time in which http endpoint was processed and is ready to send bytes back to the user",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "endpoint_name".to_string() => "upload_file".to_string(),
            "http_method".to_string() => "POST".to_string(),
            "http_path".to_string() => "/files/<path:TEXT>".to_string(),
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
            "http_path".to_string() => "/files-m/<path:TEXT>".to_string(),
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

    static ref METRIC_TRX_DBM_TESTDB_INSERT_ID: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_query_time",
        "Database query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id".to_string(),
            "is_query".to_string() => "false".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "all_at_once".to_string(),
        },
    )).unwrap();

    static ref METRIC_TRX_DBQ_TESTDB_INSERT_ID_RETURNING: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_query_time",
        "Database query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "testdb".to_string(),
            "query".to_string() => "insert_id_returning".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "true".to_string(),
            "transaction".to_string() => "all_at_once".to_string(),
        },
    )).unwrap();

    static ref METRIC_TRX_DBQ_TESTDB_MAX_ID_FROM_FOO: ::prometheus::Histogram = ::prometheus::register_histogram!(::prometheus::histogram_opts!(
        "epl_database_query_time",
        "Database query time",
        vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        ::prometheus::labels! {
            "deployment_name".to_string() => ::std::env::var("EPL_DEPLOYMENT_NAME").expect("Mandatory environment variable EPL_DEPLOYMENT_NAME is not configured"),
            "application".to_string() => "hello-world".to_string(),
            "database".to_string() => "testdb".to_string(),
            "query".to_string() => "max_id_from_foo".to_string(),
            "is_query".to_string() => "true".to_string(),
            "is_mutating".to_string() => "false".to_string(),
            "transaction".to_string() => "all_at_once".to_string(),
        },
    )).unwrap();
}
