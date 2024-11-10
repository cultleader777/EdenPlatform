
use std::error::Error;

use reqwest::StatusCode;

use crate::common::*;


#[derive(serde::Deserialize)]
struct Test1Response {
    output_field: i64,
}

#[tokio::test]
async fn epl_app_mutation() -> Result<(), Box<dyn Error>> {
    let resp = http_post(HttpPostInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.12".to_string()),
        port: 443,
        path: "/muh/app/mutate_test_1",
        is_https: true,
        input_body: "{\"some_field\":1}".to_owned().into_bytes(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let r1: Test1Response = serde_json::from_slice(&resp.body)?;

    let resp = http_post(HttpPostInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: "/muh/app/mutate_test_1",
        is_https: true,
        input_body: "{\"some_field\":1}".to_owned().into_bytes(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let r2: Test1Response = serde_json::from_slice(&resp.body)?;

    assert!(r2.output_field > r1.output_field);

    Ok(())
}

#[tokio::test]
async fn epl_app_db_read() -> Result<(), Box<dyn Error>> {
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.12".to_string()),
        port: 443,
        path: "/muh/app/rt_1".to_string(),
        is_https: true,
        headers: Vec::new(),
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let _r1: Test1Response = serde_json::from_slice(&resp.body)?;
    Ok(())
}

#[tokio::test]
async fn epl_app_prometheus_metrics_gathered() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "epl_http_endpoint_sent_bytes").await?);

    Ok(())
}

#[tokio::test]
async fn epl_app_upload_and_download() -> Result<(), Box<dyn Error>> {
    use rand::Rng;
    let file_id: u64 = rand::thread_rng().gen();
    let content: u64 = rand::thread_rng().gen();
    let file_name = format!("{file_id}.txt");
    let file_content = content.to_string();
    let path = format!("/muh/app/files/{file_name}");
    let path_static: &'static String = Box::leak(Box::new(path.clone()));

    let resp = http_post(HttpPostInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.12".to_string()),
        port: 443,
        path: path_static.as_str(),
        is_https: true,
        input_body: file_content.clone().into_bytes(),
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::CREATED);

    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: path.clone(),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(&resp.body, file_content.as_bytes());

    Ok(())
}

#[tokio::test]
async fn epl_app_upload_multipart_and_download() -> Result<(), Box<dyn Error>> {
    use rand::Rng;
    let file_id: u64 = rand::thread_rng().gen();
    let file_name = format!("{file_id}.txt");
    let file_size = 1024 * 1024 * 16;
    let mut content: Vec<u8> = Vec::with_capacity(file_size);
    for i in 0..file_size {
        content.push((i % 256).try_into().unwrap());
    }
    let path = format!("/muh/app/files/{file_name}");
    let path_mul = format!("/muh/app/files-m/{file_name}");
    let path_static: &'static String = Box::leak(Box::new(path_mul.clone()));

    let resp = http_post(HttpPostInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.12".to_string()),
        port: 443,
        path: path_static.as_str(),
        is_https: true,
        input_body: content.clone(),
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::CREATED);

    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: path.clone(),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(&resp.body, &content);

    Ok(())
}

#[tokio::test]
async fn epl_app_config() -> Result<(), Box<dyn Error>> {
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: "/muh/app/configs_test".to_string(),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(
        String::from_utf8(resp.body).unwrap(),
        "<body><p>some_string henlo bois</p><p>some_int 42</p><p>some_float 3.14</p><p>some_bool true</p></body>"
    );

    Ok(())
}

#[tokio::test]
async fn epl_app_bg_jobs() -> Result<(), Box<dyn Error>> {
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: "/muh/app/bg_job_counter".to_string(),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let counter_a = String::from_utf8(resp.body).unwrap().parse::<i32>().unwrap();
    let _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: "/muh/app/bg_job_counter".to_string(),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let counter_b = String::from_utf8(resp.body).unwrap().parse::<i32>().unwrap();

    assert!(counter_b > counter_a, "Counter should have increased after sleep");

    Ok(())
}

#[tokio::test]
async fn epl_app_clickhouse_integration_and_ch_mutator() -> Result<(), Box<dyn Error>> {
    let start = std::time::SystemTime::now();
    let ts = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards").as_secs();
    // inside this endpoint clickhouse insert and then select are performed
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: format!("/muh/app/ch_foo_insert_select/{ts}"),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let body_string = String::from_utf8(resp.body).unwrap();
    assert_eq!(body_string, format!("Max id: {ts}|a !@#$%^&amp;*()_+|123|123|12.3|12.3|true|false"));

    // inside this endpoint clickhouse mutation and then select are performed
    // first post two times larger timestamp to filter out current insertion
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: format!("/muh/app/test_ch_mutator/{}", ts * 2),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let body_num: u64 = String::from_utf8(resp.body).unwrap().parse().unwrap();
    assert!(body_num < ts);

    // now do same with 0 to get max id
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: "/muh/app/test_ch_mutator/0".to_string(),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let body_num: u64 = String::from_utf8(resp.body).unwrap().parse().unwrap();
    assert_eq!(body_num, ts);

    Ok(())
}

#[tokio::test]
async fn epl_app_nats_to_clickhouse_integration() -> Result<(), Box<dyn Error>> {
    let start = std::time::SystemTime::now();
    let ts = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards").as_secs();
    // inside this endpoint clickhouse insert and then select are performed
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        ip: Some("10.17.0.13".to_string()),
        port: 443,
        path: format!("/muh/app/nats_publish_get_max/{ts}"),
        is_https: true,
        headers: Vec::new()
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    let body_string = String::from_utf8(resp.body).unwrap();
    assert_eq!(body_string, format!("Max imp table: {ts}"));

    Ok(())
}