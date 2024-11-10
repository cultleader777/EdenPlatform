use actix_web::HttpResponse;
use async_trait::async_trait;
use maud::html;
use crate::generated::*;

pub struct AppImplementation {
    counter: ::std::sync::atomic::AtomicI32,
}

impl AppImplementation {
    pub fn new() -> AppImplementation { AppImplementation { counter: std::sync::atomic::AtomicI32::new(0) } }
}

#[async_trait]
impl AppRequirements for AppImplementation {
    async fn http_endpoint_hello_world(&self, api: &AppApi, payload: HttpEndpointPayloadHelloWorld) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>> {
        let stuff = api.pgq_max_id_from_foo(1).await?;

        let moar = payload.more.to_string();
        let floots = format!("{:?}", payload.qa_floot);
        let other = format!("{:?}", payload.qa_other);
        let html = html! {
            html {
                @for i in &stuff {
                    body {
                        p { "Hello world for " (i.max_id.unwrap_or(123)) "!" }
                        p { "Payload arg:" (payload.arg)
                             " more:" (moar)
                             " qa_floot:" (floots)
                             " qa_other:" (other) }
                    }
                }
            }
        };

        Ok(html)
    }

    async fn http_endpoint_example(&self, api: &AppApi, payload: HttpEndpointPayloadExample) -> Result<BwTypeTestVtypeV5, Box<dyn ::std::error::Error + Send + Sync>> {
        use rand::Rng;
        let next = rand::thread_rng().gen::<i32>();

        let mut res =
            api.pgtrx_begin_all_at_once().await?
               .pgmq_insert_id_returning(123).await?;

        for i in 1..10 {
            let _output = res.pgq_max_id_from_foo(i).await?;
        }

        let mut res = res.advance();

        for i in next..next+10 {
            let _output = res.pgm_insert_id(i).await?;
        }

        res.advance().commit().await?;

        let published = api.jetstream_publish_some_test_stream_producer(&payload.input_body, "soobject").await?.await?;
        let _ = api.pgmq_insert_id_returning((published.sequence + 1000) as i32).await?;

        Ok(payload.input_body)
    }

    /// Increment insert existing max id in the table by provided payload
    async fn http_endpoint_mutate_test_1(&self, api: &AppApi, payload: HttpEndpointPayloadMutateTest1) -> Result<BwTypeTestOutputTypeV1, Box<dyn ::std::error::Error + Send + Sync>> {
        let mid = api.pgq_max_id_from_foo(1).await?;
        let res = mid.get(0).map(|i| i.max_id.unwrap_or(0)).unwrap_or(0) as i64;
        let new_id = res + payload.input_body.some_field;
        let _ = api.pgmq_insert_id_returning(new_id.try_into()?).await?;
        Ok(BwTypeTestOutputTypeV1 { output_field: new_id as i64 })
    }

    async fn http_endpoint_read_test_1(&self, api: &AppApi, _payload: HttpEndpointPayloadReadTest1) -> Result<BwTypeTestOutputTypeV1, Box<dyn ::std::error::Error + Send + Sync>> {
        let mid = api.pgq_max_id_from_foo(1).await?;
        let res = mid.get(0).map(|i| i.max_id.unwrap_or(0)).unwrap_or(0) as i64;
        Ok(BwTypeTestOutputTypeV1 { output_field: res })
    }

    async fn jetstream_consume_some_test_stream_consumer(&self, api: &AppApi, payload: BwTypeTestVtypeV5, sub: &str) -> Result<(), Box<dyn ::std::error::Error + Send + Sync>> {
        log::info!("Got message from subject {}", sub);
        let _res = api.pgm_insert_id(payload.some_field as i32).await?;
        Ok(())
    }

    async fn http_endpoint_dummy(&self, _api: &AppApi, mut payload: HttpEndpointPayloadDummy) -> Result<BwTypeTestVtypeV5, Box<dyn ::std::error::Error + Send + Sync>> {
        if let Some(a) = &payload.qa_int_arg {
            payload.input_body.some_field += *a;
        }

        for fl in &payload.qa_floatv_arg {
            payload.input_body.other_field += *fl;
        }

        Ok(payload.input_body)
    }

    async fn http_endpoint_upload_file(&self, api: &AppApi, payload: HttpEndpointPayloadUploadFile) -> Result<::actix_web::HttpResponse, Box<dyn ::std::error::Error + Send + Sync>> {
        let bucket = api.s3_storage();
        let _ = bucket.put_object(&payload.path, &payload.input_body).await?;
        Ok(HttpResponse::Created().finish())
    }

    async fn http_endpoint_upload_file_multipart(&self, api: &AppApi, mut payload: HttpEndpointPayloadUploadFileMultipart) -> Result<::actix_web::HttpResponse, Box<dyn ::std::error::Error + Send + Sync>> {
        let bucket = api.s3_storage();

        let mut buffer = Vec::with_capacity(1024 * 1024 * 2);
        while let Some(bytes) = payload.input_body.recv().await {
            let bytes = bytes?;
            buffer.extend_from_slice(&bytes);
        }

        bucket.put_object(&payload.path, &buffer).await?;

        Ok(HttpResponse::Created().finish())
    }

    async fn http_endpoint_download_file(&self, api: &AppApi, payload: HttpEndpointPayloadDownloadFile) -> Result<::actix_web::HttpResponse, Box<dyn ::std::error::Error + Send + Sync>> {
        let bucket = api.s3_storage();
        let obj = bucket.get_object(payload.path).await;
        match obj {
            Ok(data) => {
                Ok(
                    HttpResponse::Ok()
                        .append_header(("Content-Type", "application/octet-stream"))
                        .body(data.to_vec())
                )
            }
            Err(_) => {
                Ok(HttpResponse::NotFound().finish())
            }
        }
    }

    async fn http_endpoint_configs_test(&self, api: &AppApi, _payload: HttpEndpointPayloadConfigsTest) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>>  {
        Ok(html! {
            body {
                p { "some_string " (api.cfg_some_string()) }
                p { "some_int " (api.cfg_some_int()) }
                p { "some_float " (api.cfg_some_float()) }
                p { "some_bool " (api.cfg_some_bool().to_string()) }
            }
        })
    }

    async fn bg_job_incrementer(&self, _api: AppApi) -> Result<(), Box<dyn ::std::error::Error + Send + Sync>> {
        loop {
            let _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let _ = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    async fn http_endpoint_bg_job_counter(&self, _api: &AppApi, _payload: HttpEndpointPayloadBgJobCounter) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>> {
        let current: i32 = self.counter.load(std::sync::atomic::Ordering::SeqCst);
        Ok(html! { (current) })
    }

    async fn http_endpoint_ch_insert_select(&self, api: &AppApi, payload: HttpEndpointPayloadChInsertSelect) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>> {
        let _ = api.ch_insert_into_chshard_foo(&[
            ChInsTestchRowFoo {
                id: payload.id,
                a: format!("Hello {}", payload.id),
                b: None,
                c: None,
                f: None,
            }
        ]).await?;
        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        let res = api.chq_max_id_from_foo(payload.id - 1).await?;
        if res.len() > 0 {
            Ok(html! { "Max id: " (res[0].max_id)
            "|" (res[0].v_string)
            "|" (res[0].v_i32)
            "|" (res[0].v_i64)
            "|" (res[0].v_f32)
            "|" (res[0].v_f64)
            "|" (if res[0].v_bool_t { "true" } else { "false" })
            "|" (if res[0].v_bool_f { "true" } else { "false" })
        })
        } else {
            Ok(html! { "No max id found" })
        }
    }

    async fn http_endpoint_nats_ch_sink(&self, api: &AppApi, payload: HttpEndpointPayloadNatsChSink) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>> {
        let _ = api.jetstream_publish_simple_msg_stream(&BwTypeSimple {
            some_field: payload.id,
            some_text: "Henlo".to_string(),
        }).await?.await?;

        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        let max_id = api.chq_max_id_from_imp().await?;
        if max_id.len() > 0 {
            Ok(html! { "Max imp table: " (max_id[0].max_id) })
        } else {
            Ok(html! { "No max id found" })
        }
    }

    async fn http_endpoint_test_ch_mutator(&self, api: &AppApi, payload: HttpEndpointPayloadTestChMutator) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>> {
        api.chm_copy_ids_from_foo(payload.min_id).await?;
        let max_ids = api.chq_max_id_from_foo_ids().await?;

        Ok(html!({ (max_ids[0].max_id) }))
    }
}

#[test]
fn dummy_test() {
    assert_eq!(7, 7);
}
