use async_trait::async_trait;
use maud::html;
use crate::generated::*;

pub struct AppImplementation;

impl AppImplementation {
    pub fn new() -> AppImplementation { AppImplementation }
}

#[async_trait]
impl AppRequirements for AppImplementation {
    async fn http_endpoint_hello_world(&self, api: &AppApi, payload: HttpEndpointPayloadHelloWorld) -> Result<::maud::Markup, Box<dyn ::std::error::Error + Send + Sync>> {
        let stuff = api.dbq_max_id_from_foo(1).await?;

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
        let mut res =
            api.dbtrx_begin_all_at_once().await?
               .dbmq_insert_id_returning(123).await?;

        for i in 1..10 {
            let _output = res.dbq_max_id_from_foo(i).await?;
        }

        let mut res = res.advance();

        for i in 1..10 {
            let _output = res.dbm_insert_id(i).await?;
        }

        res.advance().commit().await?;

        let published = api.jetstream_publish_some_test_stream_producer(&payload.input_body).await?;
        let _ = api.dbmq_insert_id_returning((published.sequence + 1000) as i32).await?;

        Ok(payload.input_body)
    }

    /// Increment insert existing max id in the table by provided payload
    async fn http_endpoint_mutate_test_1(&self, api: &AppApi, payload: HttpEndpointPayloadMutateTest1) -> Result<BwTypeTestOutputTypeV1, Box<dyn ::std::error::Error + Send + Sync>> {
        let mid = api.dbq_max_id_from_foo(1).await?;
        let res = mid.get(0).map(|i| i.max_id.unwrap_or(0)).unwrap_or(0) as i64;
        let new_id = res + payload.input_body.some_field;
        let _ = api.dbmq_insert_id_returning(new_id.try_into()?).await?;
        Ok(BwTypeTestOutputTypeV1 { output_field: new_id as i64 })
    }

    async fn http_endpoint_read_test_1(&self, api: &AppApi, _payload: HttpEndpointPayloadReadTest1) -> Result<BwTypeTestOutputTypeV1, Box<dyn ::std::error::Error + Send + Sync>> {
        let mid = api.dbq_max_id_from_foo(1).await?;
        let res = mid.get(0).map(|i| i.max_id.unwrap_or(0)).unwrap_or(0) as i64;
        Ok(BwTypeTestOutputTypeV1 { output_field: res })
    }

    async fn jetstream_consume_some_test_stream_consumer(&self, api: &AppApi, payload: BwTypeTestVtypeV5) -> Result<(), Box<dyn ::std::error::Error + Send + Sync>> {
        let _res = api.dbm_insert_id(payload.some_field as i32).await?;
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
}

#[test]
fn dummy_test() {
    assert_eq!(7, 7);
}
