
use std::{error::Error, ffi::OsString, collections::HashSet};

use headless_chrome::Browser;
use reqwest::StatusCode;

use crate::common::*;

#[cfg(test)]
pub fn are_bytes_gzip(input: &[u8]) -> bool {
    input.len() >= 2 && input[0] == 0x1f && input[1] == 0x8b
}

#[tokio::test]
async fn epl_frontends_exist() -> Result<(), Box<dyn Error>> {
    let res = resolve_ip_custom_dns("10.17.0.10:53", "epl-app-frontend-test.service.consul").await?;
    assert_eq!(res.len(), 3);

    Ok(())
}

#[tokio::test]
async fn internal_frontend_returns() -> Result<(), Box<dyn Error>> {
    let res = resolve_ip_custom_dns("10.17.0.10:53", "epl-app-frontend-test.service.consul").await?;
    assert!(res.len() > 0);
    let res = http_get(HttpGetInput {
        dns_name: "epl-app-frontend-test.service.consul",
        ip: Some(res[0].to_string()),
        port: 7437,
        path: "/".to_string(),
        is_https: false,
        headers: Vec::new(),
    }).await?;

    assert_eq!(res.status, StatusCode::OK);
    assert!(res.body_to_string().contains("<title>Trunk App</title>"));

    Ok(())
}

#[tokio::test]
async fn external_frontend_returns() -> Result<(), Box<dyn Error>> {
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        port: 443,
        path: "/".to_string(),
        is_https: true,
        ip: Some("10.17.0.12".to_string()),
        headers: Vec::new(),
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    assert!(resp.body_to_string().contains("<title>Trunk App</title>"));

    Ok(())
}

#[test]
fn headless_browser_hello_world() -> Result<(), Box<dyn Error>> {
    // We have chromium from nix environment ungoogled-chromium package
    let mut args = Vec::new();
    let os_str = OsString::from("--host-resolver-rules=MAP www.epl-infra.net 10.17.0.13");
    args.push(os_str.as_os_str());
    let browser = Browser::new(headless_chrome::LaunchOptions {
        headless: true,
        sandbox: true,
        ignore_certificate_errors: true,
        args,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    let _ = tab.navigate_to("https://www.epl-infra.net/")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Hello world!</h1>"));

    let _ = tab.navigate_to("https://www.epl-infra.net/single/42")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Single Arg id:42</h1>"));

    let _ = tab.navigate_to("https://www.epl-infra.net/next/42/7.77?arg_a=false&arg_b=true&arg_c=1&arg_d=2&arg_d=3&arg_e=4.2&arg_f=7.7&arg_f=1.2&arg_f=1.7&arg_g=hello&arg_h=foo&arg_h=bar")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Next Page id:42 farg:7.77</h1>"));
    assert!(content.contains(r#"<ul><li>arg_a:Some(false)</li><li>arg_b:[true]</li><li>arg_c:Some(1)</li><li>arg_d:[2, 3]</li><li>arg_e:Some(4.2)</li><li>arg_f:[7.7, 1.2, 1.7]</li><li>arg_g:Some("hello")</li><li>arg_h:["foo", "bar"]</li></ul>"#));

    Ok(())
}

#[tokio::test]
async fn external_frontend_returns_compressed() -> Result<(), Box<dyn Error>> {
    let mut size_set: HashSet<usize> = HashSet::new();
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        port: 443,
        path: "/".to_string(),
        is_https: true,
        ip: Some("10.17.0.12".to_string()),
        headers: vec![
            ("Accept-Encoding".to_string(), "gzip".to_string()),
        ],
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);
    assert!(are_bytes_gzip(&resp.body));

    // this way we are reasonably sure files are unique
    // because any mistake in any page will redirect to index
    assert!(size_set.insert(resp.body.len()));

    let resp_uncompressed = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        port: 443,
        path: "/".to_string(),
        is_https: true,
        ip: Some("10.17.0.13".to_string()),
        headers: vec![],
    }).await?;

    let body = resp_uncompressed.body_to_string();
    let links_regex = regex::Regex::new(" href=\"(.*?\\.[a-z]+)\"").unwrap();
    for caps in links_regex.captures_iter(&body) {
        let resource = caps.get(1).unwrap();
        let path = format!("/{}", resource.as_str());
        println!("Getting path {path}");
        let resp = http_get(HttpGetInput {
            dns_name: "www.epl-infra.net",
            port: 443,
            path: path.clone(),
            is_https: true,
            ip: Some("10.17.0.12".to_string()),
            headers: vec![
                ("Accept-Encoding".to_string(), "gzip".to_string()),
            ],
        }).await?;
        assert_eq!(resp.status, StatusCode::OK);
        // ensure this unique page
        println!("{} -> {}", path, resp.body.len());
        assert!(size_set.insert(resp.body.len()));
        assert!(are_bytes_gzip(&resp.body));
    }

    Ok(())
}

#[tokio::test]
async fn external_frontend_returns_cached() -> Result<(), Box<dyn Error>> {
    let resp = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        port: 443,
        path: "/".to_string(),
        is_https: true,
        ip: Some("10.17.0.13".to_string()),
        headers: vec![
            ("Accept-Encoding".to_string(), "gzip".to_string()),
        ],
    }).await?;

    assert_eq!(resp.status, StatusCode::OK);

    let resp_uncompressed = http_get(HttpGetInput {
        dns_name: "www.epl-infra.net",
        port: 443,
        path: "/".to_string(),
        is_https: true,
        ip: Some("10.17.0.12".to_string()),
        headers: vec![],
    }).await?;

    assert_eq!(resp_uncompressed.headers.get("Cache-Control"), None);

    let body = resp_uncompressed.body_to_string();
    let links_regex = regex::Regex::new(" href=\"(.*?\\.[a-z]+)\"").unwrap();
    for caps in links_regex.captures_iter(&body) {
        let resource = caps.get(1).unwrap();
        let path = format!("/{}", resource.as_str());
        println!("Getting path {path}");
        let resp = http_get(HttpGetInput {
            dns_name: "www.epl-infra.net",
            port: 443,
            path: path.clone(),
            is_https: true,
            ip: Some("10.17.0.13".to_string()),
            headers: vec![
                ("Accept-Encoding".to_string(), "gzip".to_string()),
            ],
        }).await?;
        assert_eq!(resp.status, StatusCode::OK);
        println!("Headers {:#?}", resp.headers);
        assert!(resp.headers.get("etag").is_some());
        assert_eq!(resp.headers.get("cache-control"), Some("max-age=315360000".to_string()).as_ref());
    }

    Ok(())
}

#[test]
fn headless_browser_navigation() -> Result<(), Box<dyn Error>> {
    // We have chromium from nix environment ungoogled-chromium package
    let mut args = Vec::new();
    let os_str = OsString::from("--host-resolver-rules=MAP www.epl-infra.net 10.17.0.12");
    args.push(os_str.as_os_str());
    let browser = Browser::new(headless_chrome::LaunchOptions {
        headless: true,
        sandbox: true,
        ignore_certificate_errors: true,
        args,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    let _ = tab.navigate_to("https://www.epl-infra.net/")?.wait_until_navigated()?;
    assert!(tab.get_content()?.contains("<h1>Hello world!</h1>"));

    tab.wait_until_navigated()?;
    tab.find_element(".button_a")?.click()?;
    assert!(tab.get_content()?.contains("<h1>Single Arg id:123</h1>"));

    tab.find_element(".button_c")?.click()?;
    tab.wait_until_navigated()?;
    assert!(tab.get_content()?.contains("<h1>Hello world!</h1>"));

    tab.find_element(".button_b")?.click()?;
    tab.wait_until_navigated()?;
    assert!(tab.get_content()?.contains("<ul><li>arg_a:Some(true)</li><li>arg_b:[]</li><li>arg_c:Some(42)</li><li>arg_d:[]</li><li>arg_e:Some(1e-13)</li><li>arg_f:[]</li><li>arg_g:None</li><li>arg_h:[\"salookie\", \"dookie\", \"хелло\", \"#*#@#@!$)@#*%#^)_\"]</li></ul>"));

    tab.find_element(".button_d")?.click()?;
    tab.wait_until_navigated()?;
    assert!(tab.get_content()?.contains("<h1>Hello world!</h1>"));

    Ok(())
}

#[test]
fn headless_browser_other_frontend() -> Result<(), Box<dyn Error>> {
    // We have chromium from nix environment ungoogled-chromium package
    let mut args = Vec::new();
    let os_str = OsString::from("--host-resolver-rules=MAP www.epl-infra.net 10.17.0.13");
    args.push(os_str.as_os_str());
    let browser = Browser::new(headless_chrome::LaunchOptions {
        headless: true,
        sandbox: true,
        ignore_certificate_errors: true,
        args,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    let _ = tab.navigate_to("https://www.epl-infra.net/other/")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Other app home</h1>"));

    let _ = tab.navigate_to("https://www.epl-infra.net/other/single/42")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Other app42</h1>"));

    let _ = tab.navigate_to("https://www.epl-infra.net/other/other/")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Other dummy page</h1>"));

    Ok(())
}

#[test]
fn headless_browser_http_request() -> Result<(), Box<dyn Error>> {
    // We have chromium from nix environment ungoogled-chromium package
    let mut args = Vec::new();
    let os_str = OsString::from("--host-resolver-rules=MAP www.epl-infra.net 10.17.0.12");
    args.push(os_str.as_os_str());
    let browser = Browser::new(headless_chrome::LaunchOptions {
        headless: true,
        sandbox: true,
        ignore_certificate_errors: true,
        args,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    let _ = tab.navigate_to("https://www.epl-infra.net/rest_test/")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Rest test</h1>"));
    assert!(content.contains("<p>is nice: false, of: -1.23, sf: -123</p>"));

    let _ = tab.wait_for_element("button")?.click()?;
    let _ = tab.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<p>is nice: true, of: 14.4, sf: 794</p>"));

    Ok(())
}

#[test]
fn headless_browser_backend_link_test() -> Result<(), Box<dyn Error>> {
    // We have chromium from nix environment ungoogled-chromium package
    let mut args = Vec::new();
    let os_str = OsString::from("--host-resolver-rules=MAP www.epl-infra.net 10.17.0.13");
    args.push(os_str.as_os_str());
    let browser = Browser::new(headless_chrome::LaunchOptions {
        headless: true,
        sandbox: true,
        ignore_certificate_errors: true,
        args,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    let _ = tab.navigate_to("https://www.epl-infra.net/links_test/")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Links test</h1>"));

    let _ = tab.wait_for_element(".to_backend")?.click()?;
    let _ = tab.wait_for_element("p")?;
    let content = tab.get_content()?;
    assert!(content.contains("<p>Payload arg:hello more:true qa_floot:[0.7, 7.7] qa_other:Some(777)</p>"));

    Ok(())
}

#[test]
fn headless_browser_frontend_link_test() -> Result<(), Box<dyn Error>> {
    // We have chromium from nix environment ungoogled-chromium package
    let mut args = Vec::new();
    let os_str = OsString::from("--host-resolver-rules=MAP www.epl-infra.net 10.17.0.12");
    args.push(os_str.as_os_str());
    let browser = Browser::new(headless_chrome::LaunchOptions {
        headless: true,
        sandbox: true,
        ignore_certificate_errors: true,
        args,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    let _ = tab.navigate_to("https://www.epl-infra.net/links_test/")?.wait_until_navigated()?;
    let content = tab.get_content()?;
    assert!(content.contains("<h1>Links test</h1>"));

    let _ = tab.wait_for_element(".to_frontend")?.click()?;
    let _ = tab.wait_for_element("h1")?;
    let content = tab.get_content()?;
    eprintln!("HUH[{content}]");
    assert!(content.contains("<h1>All arg id:7 name:some%20name opt_i:17 opt_m:0.77.7</h1>"));

    Ok(())
}
