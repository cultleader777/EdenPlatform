use std::{fmt::Write, collections::BTreeMap};
use crate::static_analysis::{CheckedDB, server_runtime::{IntegrationTest, IntegrationTestCredentials}};

use super::Directory;


pub(crate) fn generate_integration_test_dir(
    db: &CheckedDB,
    test_dir: &mut Directory,
) {
    test_dir.create_file("Cargo.toml", cargo_toml());
    let src_dir = test_dir.create_directory("src");
    src_dir.create_file("common.rs", common_lib());
    src_dir.create_file("main.rs", main_rs());
    src_dir.create_file("generated.rs", generated_tests(db));
    let manual_dir = src_dir.create_directory("manual");
    manual_dir.create_file_if_not_exists("mod.rs", "".to_string());
}

fn generated_tests(
    db: &CheckedDB,
) -> String {
    let mut res = String::new();

    write!(&mut res, r#"
use std::error::Error;
use crate::common::*;
use base64::prelude::*;

"#).unwrap();

    for (test_name, test) in db.projections.server_runtime.all_integration_tests() {
        match test {
            IntegrationTest::DnsResolutionWorksARecords { target_servers, queries } => {
                generate_dns_a_record_test(&mut res, test_name, &target_servers, &queries);
            }
            IntegrationTest::DnsResolutionARecordCount { target_servers, queries } => {
                generate_dns_a_record_count_test(&mut res, test_name, &target_servers, &queries);
            }
            IntegrationTest::DnsResolutionWorksNsRecords { target_servers, queries } => {
                generate_dns_ns_record_test(&mut res, test_name, &target_servers, &queries);
            }
            IntegrationTest::DnsResolutionWorksPtrRecords { target_servers, queries } => {
                generate_dns_ptr_record_test(&mut res, test_name, &target_servers, &queries);
            }
            IntegrationTest::DnsSecWorksInternal { target_servers, source_ip, server_to_lookup, server_to_lookup_ip, region, tld } => {
                generate_internal_dnssec_test(&mut res, test_name, target_servers, source_ip, server_to_lookup, server_to_lookup_ip, region, tld);
            }
            IntegrationTest::DnsSecWorksExternal { target_servers, dns_to_lookup } => {
                generate_external_dnssec_test(&mut res, test_name, target_servers, dns_to_lookup);
            }
            IntegrationTest::TcpSocketsOpen { target_sockets } => {
                generate_tcp_sockets_open_test(&mut res, test_name, target_sockets);
            }
            IntegrationTest::PrometheusMetricExists { prometheus_server_ip, prometheus_server_port, metric, should_exist } => {
                generate_prometheus_metric_exists_test(&mut res, test_name, &prometheus_server_ip, *prometheus_server_port, metric, *should_exist);
            }
            IntegrationTest::HttpGetRespondsOk { server_ips, http_server_port, path } => {
                generate_http_get_ok_test(&mut res, test_name, server_ips, *http_server_port, path);
            }
            IntegrationTest::HttpGetRespondsString { server_ips, hostname, http_server_port, path, is_https, expected_string, use_admin_panel_credentials } => {
                generate_http_get_expected_string_test(&mut res, test_name, hostname, server_ips, *http_server_port, path, *is_https, &expected_string, use_admin_panel_credentials);
            }
            IntegrationTest::PingWorks { server_ips } => {
                generate_ping_test(&mut res, test_name, server_ips);
            }
            IntegrationTest::LokiWriterReaderTest { dns_server, reader_dns_name, writer_dns_name, reader_port, writer_port } => {
                generate_loki_writer_reader_test(&mut res, test_name, dns_server, &reader_dns_name, &writer_dns_name, *reader_port, *writer_port);
            }
            IntegrationTest::LokiStreamExists { dns_server, reader_dns_name, reader_port, stream_identifiers } => {
                generate_loki_stream_exists_test(&mut res, test_name, dns_server, &reader_dns_name, *reader_port, &stream_identifiers);
            }
            IntegrationTest::TempoSpansWritable { dns_server, service_name, push_port, query_port } => {
                generate_tempo_writer_reader_test(&mut res, test_name, dns_server, &service_name, *push_port, *query_port);
            }
            IntegrationTest::InsideNodeDnsAResolutionWorks { server_ips } => {
                generate_ssh_dns_a_record_test(&mut res, test_name, server_ips);
            }
            IntegrationTest::InsideNodePingWorks { server_ips } => {
                generate_ssh_ping_test(&mut res, test_name, server_ips);
            }
            IntegrationTest::CrossDcSourceIpCheck {
                port_range_start,
                server_to_run_iperf_server_from_with_private_ip,
                servers_to_run_iperf_client_from_with_expected_ips,
            } => {
                let (iperf_ssh_ip, iperf_bind_ip) = server_to_run_iperf_server_from_with_private_ip;
                generate_ssh_iperf_source_ip_test(&mut res, test_name, *port_range_start, &iperf_ssh_ip, &iperf_bind_ip, &servers_to_run_iperf_client_from_with_expected_ips);
            }
        }
    }

    res
}

fn generate_loki_stream_exists_test(
    res: &mut String,
    test_name: &str,
    dns_server: &str,
    reader_dns: &str,
    reader_port: i64,
    stream_identifiers: &[(String, String)],
) {
    let streams = stream_identifiers.iter().map(|(sname, svalue)| {
        format!("{sname}=\"{svalue}\"")
    }).collect::<Vec<_>>();
    let joined = streams.join(",");
    let expected_value = format!(r#"{{{joined}}}"#);
    let expected_value = expected_value.replace("\"", "\\\"");
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    assert!(check_if_loki_stream_exists("{dns_server}", "{reader_dns}", {reader_port}, "{expected_value}").await?);

    Ok(())
}}
"#).unwrap();
}

fn generate_loki_writer_reader_test(
    res: &mut String,
    test_name: &str,
    dns_server: &str,
    reader_dns: &str,
    writer_dns: &str,
    reader_port: i64,
    writer_port: i64,
) {
    let expected_value = r#""result":[{{"stream":{{"foo":"{rnd_label}"}},"values":[["{time_nanos}","henlo {rnd_label}"]]}}]"#;
    let expected_value = expected_value.replace("\"", "\\\"");
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    use rand::{{seq::SliceRandom, Rng}};
    use serde_json::json;

    let reader_dns = "{reader_dns}";
    let writer_dns = "{writer_dns}";
    let reader_port = {reader_port};
    let writer_port = {writer_port};
    let reader = resolve_ip_custom_dns("{dns_server}", reader_dns).await?;
    let writer = resolve_ip_custom_dns("{dns_server}", writer_dns).await?;
    assert!(reader.len() > 0 && writer.len() > 0);

    let writer = writer.choose(&mut rand::thread_rng()).unwrap();

    let rnd_label: u64 = rand::thread_rng().gen();
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let time_nanos = since_the_epoch.as_nanos();
    let payload = json!(
        {{
            "streams": [
                {{
                    "stream": {{
                        "foo": rnd_label.to_string(),
                    }},
                    "values": [
                        [
                            format!("{{time_nanos}}"),
                            format!("henlo {{rnd_label}}"),
                        ]
                    ]
                }}
            ]
        }}
    );

    let res = http_post(HttpPostInput {{
        dns_name: writer_dns,
        ip: Some(writer.to_string()),
        port: writer_port,
        path: "/loki/api/v1/push",
        is_https: false,
        input_body: serde_json::to_vec(&payload).unwrap(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ],
    }}).await?;

    assert!(res.status.as_u16() == 204);

    let res = http_post(HttpPostInput {{
        dns_name: writer_dns,
        ip: Some(writer.to_string()),
        port: writer_port,
        path: "/flush",
        is_https: false,
        input_body: Vec::new(),
        headers: Vec::new(),
    }}).await?;

    assert!(res.status.as_u16() == 204);

    let expected_value = format!("{expected_value}");

    // let params = urlen
    let params = urlencoding::encode(&format!("{{{{foo=\"{{rnd_label}}\"}}}}")).into_owned();
    let path = format!("/loki/api/v1/query_range?query={{params}}");

    // repeat few times
    for _ in 0..10 {{
        let reader = reader.choose(&mut rand::thread_rng()).unwrap();
        println!("Reader instance: {{reader}}");
        let res = http_get(HttpGetInput {{
            dns_name: reader_dns,
            ip: Some(reader.to_string()),
            port: reader_port,
            path: path.clone(),
            is_https: false,
            headers: Vec::new(),
        }}).await?;

        assert!(res.status.is_success());

        if res.body_to_string().contains(&expected_value) {{
            return Ok(())
        }}

        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }}

    panic!("Test failed to contain expected value: {{expected_value}}");
}}
"#).unwrap();
}

fn generate_tempo_writer_reader_test(
    res: &mut String,
    test_name: &str,
    dns_server: &str,
    consul_service: &str,
    push_port: i64,
    query_port: i64,
) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    use rand::seq::SliceRandom;
    use serde_json::json;
    use std::ops::Add;

    let dns = "{consul_service}";
    let push_port = {push_port};
    let query_port = {query_port};
    let instances = resolve_ip_custom_dns("{dns_server}", dns).await?;
    assert!(instances.len() > 0);

    let trace_id = generate_tempo_trace_id();
    let span_id = generate_tempo_span_id();

    let instance = instances.choose(&mut rand::thread_rng()).unwrap();
    let start = std::time::SystemTime::now();
    let start_time = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let end_time = start_time.add(std::time::Duration::from_secs(1));
    let start_time_nanos = start_time.as_nanos();
    let end_time_nanos = end_time.as_nanos();
    let payload = json!({{
        "resourceSpans": [{{
        "resource": {{
                "attributes": [{{
                "key": "service.name",
                "value": {{
                        "stringValue": "my.service"
                }}
                }}]
        }},
        "scopeSpans": [{{
                "scope": {{
                "name": "my.library",
                "version": "1.0.0",
                "attributes": [{{
                        "key": "my.scope.attribute",
                        "value": {{
                            "stringValue": "some scope attribute"
                        }}
                }}]
                }},
                "spans": [
                {{
                "traceId": trace_id,
                "spanId": span_id,
                "name": "I am a span!",
                "startTimeUnixNano": start_time_nanos,
                "endTimeUnixNano": end_time_nanos,
                "kind": 2,
                "attributes": [
                {{
                        "key": "my.span.attr",
                        "value": {{
                        "stringValue": "some value"
                        }}
                }}]
                }}]
        }}]
        }}]
    }});

    let res = http_post(HttpPostInput {{
        dns_name: dns,
        ip: Some(instance.to_string()),
        port: push_port,
        path: "/v1/traces",
        is_https: false,
        input_body: serde_json::to_vec(&payload).unwrap(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ],
    }}).await?;

    assert!(res.status.as_u16() == 200);

    let res = http_post(HttpPostInput {{
        dns_name: dns,
        ip: Some(instance.to_string()),
        port: query_port,
        path: "/flush",
        is_https: false,
        input_body: Vec::new(),
        headers: Vec::new(),
    }}).await?;

    assert!(res.status.as_u16() == 204);

    let path = format!("/api/traces/{{trace_id}}");

    // repeat few times
    for _ in 0..10 {{
        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let reader = instances.choose(&mut rand::thread_rng()).unwrap();
        println!("Reader instance: {{reader}}");
        let res = http_get(HttpGetInput {{
            dns_name: dns,
            ip: Some(reader.to_string()),
            port: query_port,
            path: path.clone(),
            is_https: false,
            headers: Vec::new(),
        }}).await?;

        if !res.status.is_success() {{
            continue;
        }}

        if res.status.as_u16() == 200 {{
            return Ok(());
        }}
    }}

    panic!("Test failed to find the trace id {{trace_id}}");
}}
"#).unwrap();
}

fn generate_ping_test(
    res: &mut String,
    test_name: &str,
    server_ips: &[String],
) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{

    let targets: &[&str] = &[
"#).unwrap();
    for ip in server_ips {
        write!(res, r#"        "{ip}",
"#).unwrap();
    }
    write!(res, r#"
    ];

    for t in targets {{
        assert!(ping_server(t).await);
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_http_get_expected_string_test(
    res: &mut String,
    test_name: &str,
    hostname: &Option<String>,
    server_ips: &[String],
    http_server_port: i64,
    path: &str,
    is_https: bool,
    expected_string: &str,
    use_admin_panel_credentials: &Option<IntegrationTestCredentials>,
) {
    let expected_string = expected_string.replace("\"", "\\\"");
    let hostname = hostname.as_ref().map(|i| i.as_str()).unwrap_or("service");
    let protocol = if is_https { "https" } else { "http" };
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let targets: &[&str] = &[
"#).unwrap();
    for ip in server_ips {
        write!(res, r#"        "{ip}",
"#).unwrap();
    }
    write!(res, r#"
    ];
    let headers = vec![
"#).unwrap();
    if let Some(creds) = use_admin_panel_credentials {
        match creds {
            IntegrationTestCredentials::AdminPanel => {
                write!(res, "      (\"Authorization\".to_string(), format!(\"Basic {{}}\", BASE64_STANDARD.encode(&format!(\"admin:{{}}\", std::env::var(\"ADMIN_PANEL_PASSWORD\").unwrap())))),\n").unwrap();
            }
            IntegrationTestCredentials::GrafanaCluster(gc_name) => {
                let env_name = gc_name.to_uppercase().replace("-", "_");
                write!(res, "      (\"Authorization\".to_string(), format!(\"Basic {{}}\", BASE64_STANDARD.encode(&format!(\"admin:{{}}\", std::env::var(\"GRAFANA_{env_name}_ADMIN_PASSWORD\").unwrap())))),\n").unwrap();
            }
        }
    }
    write!(res, r#"
    ];

    let expected_string = "{expected_string}";
    let mut some_failed = false;
    for t in targets {{
        let url = format!("{protocol}://{{t}}:{http_server_port}{path}");
        println!("Calling {{url}}");
        let resp = http_get(HttpGetInput {{
            dns_name: "{hostname}",
            port: {http_server_port},
            path: "{path}".to_string(),
            is_https: {is_https},
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }}).await?;
        if resp.status != reqwest::StatusCode::OK {{
            eprintln!("Request {{url}} did not return 200");
            some_failed = true;
        }}
        let body = resp.body_to_string();
        if !body.contains(expected_string) {{
            eprintln!("Response body {{url}} does not contain expected string '{{expected_string}}' body: '{{body}}'");
            some_failed = true;
        }}
    }}

    if some_failed {{
        panic!("Some http requests have failed");
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_http_get_ok_test(
    res: &mut String,
    test_name: &str,
    server_ips: &[String],
    http_server_port: i64,
    path: &str,
) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{

    let targets = [
"#).unwrap();
    for ip in server_ips {
        write!(res, r#"        "{ip}",
"#).unwrap();
    }
    write!(res, r#"
    ];

    let mut some_failed = false;
    for t in targets {{
        let url = format!("http://{{t}}:{http_server_port}{path}");
        println!("Calling {{url}}");
        let res = http_get(HttpGetInput {{
            dns_name: "service",
            is_https: false,
            ip: Some(t.to_string()),
            port: {http_server_port},
            path: "{path}".to_string(),
            headers: Vec::new(),
        }}).await?;
        if !res.status.is_success() {{
            eprintln!("Http get to http://{{t}}:{http_server_port}{path} failed");
            some_failed = true;
        }}
    }}
    if some_failed {{
        panic!("Some http checks have failed");
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_prometheus_metric_exists_test(
    res: &mut String,
    test_name: &str,
    prom_ip: &str,
    prom_port: i64,
    metric: &str,
    should_exist: bool,
) {
    let maybe_negate = if should_exist { "" } else { "!" };
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    assert!({maybe_negate}does_prometheus_metric_exist("{prom_ip}", {prom_port}, "{metric}").await?);

    Ok(())
}}
"#).unwrap();
}

fn generate_tcp_sockets_open_test(
    res: &mut String,
    test_name: &str,
    target_sockets: &[String],
) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let sockets = [
"#).unwrap();
    for ts in target_sockets {
        write!(res, r#"        "{ts}",
"#).unwrap();
    }
    write!(res, r#"
    ];

    let mut res = Vec::new();

    for s in sockets {{
        res.push(test_tcp_socket(s.to_string()));
    }}

    for f in res {{
        f.await?;
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_external_dnssec_test(
    res: &mut String,
    test_name: &str,
    target_servers: &[String],
    dns_to_lookup: &[String],
) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let ips = [
"#).unwrap();
    for server in target_servers {
        write!(res, r#"        "{server}",
"#).unwrap();
    }
    write!(res, r#"
    ];
    let domains: &[&str] = &[
"#).unwrap();
    for domain in dns_to_lookup {
        write!(res, r#"        "{domain}",
"#).unwrap();
    }
    write!(res, r#"
    ];
    for ip in ips {{
        for domain in domains {{
            let res = execute_command_stdout(&format!("dig @{{ip}} {{domain}} +dnssec +short")).await;
            assert!(res.contains("A 15 3"));
        }}
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_internal_dnssec_test(
    res: &mut String,
    test_name: &str,
    target_servers: &[String],
    source_ip: &str,
    server_to_lookup: &str,
    server_to_lookup_ip: &str,
    region: &str,
    tld: &str
) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let ips = [
"#).unwrap();
    for server in target_servers {
        write!(res, r#"        "{server}",
"#).unwrap();
    }
    write!(res, r#"
    ];
    let routes = execute_command_stdout("ip route").await;
    let all_addrs = execute_command_stdout("ip addr").await;
    // if we have admin wireguard route use that
    // if libvirt just use the intrface
    let maybe_source_ip =
      if !routes.contains("10.0.0.0/8 dev wg") && all_addrs.contains("inet {source_ip}") {{
        "-b {source_ip}"
      }} else {{ "" }};
    for ip in ips {{
        let res = execute_command_stdout(&format!("dig {{maybe_source_ip}} @{{ip}} {server_to_lookup}.{region}.{tld} +dnssec +short")).await;
        assert!(res.contains("{server_to_lookup_ip}"));
        assert!(res.contains("{region}.{tld}."));

        let res = execute_command_stdout(&format!("dig {{maybe_source_ip}} @{{ip}} {region}.{tld} DS +dnssec | grep -E \"IN\\s+RRSIG\\s+DS\\s+15\"")).await;
        assert!(res.contains("RRSIG"));
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_dns_ptr_record_test(res: &mut String, test_name: &str, target_servers: &[String], queries: &[(String, Vec<String>)]) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let ptr_and_dns: &[(&str, &[&str])] = &[
"#).unwrap();

    for (dns_name, target_ip) in queries {
        let mut ips: Vec<_> = target_ip.iter().map(|i| format!("\"{i}\"")).collect();
        ips.sort();
        let ips = ips.join(", ");
        write!(res, r#"        ("{dns_name}", &[{ips}]),
"#).unwrap()
    }

    write!(res, r#"
    ];
    let dns_servers = [
"#).unwrap();

    for server in target_servers {
        write!(res, r#"        "{server}",
"#).unwrap()
    }

    write!(res, r#"
    ];

    for server in dns_servers {{
        for (dns, recs) in ptr_and_dns {{
            let res = resolve_ip_custom_dns_ptr_records(server, dns).await?;
            assert_eq!(res, *recs);
        }}
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_dns_ns_record_test(res: &mut String, test_name: &str, target_servers: &[String], queries: &[(String, Vec<String>)]) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let names_and_ns_srvs = [
"#).unwrap();

    for (dns_name, target_ip) in queries {
        let mut ips: Vec<_> = target_ip.iter().map(|i| format!("\"{i}\"")).collect();
        ips.sort();
        let ips = ips.join(", ");
        write!(res, r#"        ("{dns_name}", [{ips}]),
"#).unwrap()
    }

    write!(res, r#"
    ];
    let dns_servers = [
"#).unwrap();

    for server in target_servers {
        write!(res, r#"        "{server}",
"#).unwrap()
    }

    write!(res, r#"
    ];

    for server in dns_servers {{
        for (dns, recs) in &names_and_ns_srvs {{
            let res = resolve_ip_custom_dns_nsrecords(server, dns).await?;
            assert_eq!(sorted(res.as_slice()), sorted(recs.as_slice()));
        }}
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_dns_a_record_test(res: &mut String, test_name: &str, target_servers: &[String], queries: &[(String, Vec<String>)]) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let ips_and_dns: &[(&str, &[&str])] = &[
"#).unwrap();

    let mut queries = queries.to_vec();
    for (_, q) in queries.iter_mut() {
        q.sort();
    }

    for (dns_name, target_ip) in queries {
        let ips: Vec<_> = target_ip.iter().map(|i| format!("\"{i}\"")).collect();
        let ips = ips.join(", ");
        write!(res, r#"        ("{dns_name}", &[{ips}]),
"#).unwrap()
    }

    write!(res, r#"
    ];
    let dns_servers = [
"#).unwrap();

    for server in target_servers {
        write!(res, r#"        "{server}",
"#).unwrap()
    }

    write!(res, r#"
    ];

    for server in dns_servers {{
        for (dns, ip) in ips_and_dns {{
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }}
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_ssh_dns_a_record_test(res: &mut String, test_name: &str, server_ips: &BTreeMap<String, Vec<String>>) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let servers_and_domains: &[(&str, &[&str])] = &[
"#).unwrap();

    for (server_to_ssh, domains_to_test) in server_ips {
        let domains: Vec<_> = domains_to_test.iter().map(|i| format!("\"{i}\"")).collect();
        let domains = domains.join(", ");
        write!(res, r#"        ("{server_to_ssh}", &[{domains}]),
"#).unwrap()
    }

    write!(res, r#"
    ];

    let mut futures = Vec::new();
    let mut command_and_ssh_server = Vec::new();
    for (server, domains) in servers_and_domains {{
        let mut to_join = Vec::new();
        for domain in *domains {{
            to_join.push(format!("( dig {{domain}} +short | grep -F . )"));
        }}
        let joined = to_join.join(" && ");
        command_and_ssh_server.push(format!("{{server}}: {{joined}}"));
        futures.push(ssh_command_stdout(server, joined));
    }}

    let res = futures::future::join_all(futures).await;
    let mut failed = false;
    for (idx, r) in res.iter().enumerate() {{
        if r.is_empty() {{
            failed = true;
            eprintln!("Dns lookup failed {{}}", command_and_ssh_server[idx]);
        }}
    }}

    if failed {{
        panic!("Some dns lookups failed");
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_ssh_ping_test(res: &mut String, test_name: &str, server_ips: &BTreeMap<String, Vec<String>>) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let servers_and_ips: &[(&str, &[&str])] = &[
"#).unwrap();

    for (server_to_ssh, domains_to_test) in server_ips {
        let domains: Vec<_> = domains_to_test.iter().map(|i| format!("\"{i}\"")).collect();
        let domains = domains.join(", ");
        write!(res, r#"        ("{server_to_ssh}", &[{domains}]),
"#).unwrap()
    }

    write!(res, r#"
    ];

    let mut futures = Vec::new();
    let mut command_and_ssh_server = Vec::new();
    for (server, ips) in servers_and_ips {{
        let mut to_join = Vec::new();
        for ip in *ips {{
            to_join.push(format!("( ping -W 1 -c 1 {{ip}} )"));
        }}
        let joined = to_join.join(" && ");
        command_and_ssh_server.push(format!("{{server}}: {{joined}}"));
        futures.push(ssh_command_stdout(server, joined));
    }}

    let res = futures::future::join_all(futures).await;
    let mut failed = false;
    for (idx, r) in res.iter().enumerate() {{
        if r.is_empty() {{
            failed = true;
            eprintln!("Ping failed {{}}", command_and_ssh_server[idx]);
        }}
    }}

    if failed {{
        panic!("Some pings failed");
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_ssh_iperf_source_ip_test(
    res: &mut String,
    test_name: &str,
    port_range_start: usize,
    iperf_server_ssh_ip: &str,
    iperf_server_bind_ip: &str,
    servers_to_connect_from: &[(String, String)],
) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    use std::collections::BTreeMap;
    use regex::Regex;

    let servers_to_connect_from: &[(&str, &str)] = &[
"#).unwrap();

    for (ssh_ip, expected_ip) in servers_to_connect_from {
        write!(res, r#"        ("{ssh_ip}", "{expected_ip}"),
"#).unwrap()
    }

    write!(res, r#"
    ];

    let ip_and_port_re = Regex::new(r".* local (\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}) port (\d+) connected to (\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}) port (\d+)$").unwrap();
    let mut command_and_ssh_server = Vec::new();
    let iperf_port_initial = {port_range_start};
    let mut ports_map: BTreeMap<i64, &str> = BTreeMap::new();
    let server_log_file = format!("/tmp/epl-iperf-{{iperf_port_initial}}.log");
    let iperf_kill_command = format!("( ps aux | grep 'iperf -s -p {{iperf_port_initial}}' | grep -v grep | awk '{{{{print $2}}}}' | xargs -r kill || true )");
    let iperf_server_command = format!("{{iperf_kill_command}}; tmux new-session -d 'rm -f {{server_log_file}}; timeout 120s iperf -s -p {{iperf_port_initial}} --logfile {{server_log_file}} -B {iperf_server_bind_ip}'");

    let _ = ssh_command_stdout("{iperf_server_ssh_ip}", iperf_server_command).await;

    let mut client_port_no = iperf_port_initial;
    for (server_ssh_ip, server_expected_ip) in servers_to_connect_from {{
        client_port_no += 1;
        assert!(ports_map.insert(client_port_no, *server_expected_ip).is_none());
        let command = format!("iperf -c {iperf_server_bind_ip} --port {{iperf_port_initial}} -n1 --connect-timeout 2000 -B {{server_expected_ip}} --cport {{client_port_no}} --bitrate=1");
        command_and_ssh_server.push(format!("{{server_ssh_ip}}: {{command}}"));
        ssh_command_stdout(server_ssh_ip, command).await;
    }}

    let output = ssh_command_stdout("{iperf_server_ssh_ip}", format!("{{iperf_kill_command}}; cat {{server_log_file}}")).await;
    for line in output.lines() {{
        if let Some(captures) = ip_and_port_re.captures(line) {{
            let ip_address = captures.get(3).unwrap().as_str();
            let port = captures.get(4).unwrap().as_str().parse::<i64>().unwrap();
            if let Some(expected) = ports_map.get(&port) {{
                if expected != &ip_address {{
                    panic!("Wrong source ip address for iperf connection for port {{port}}, expected source ip to be {{expected}}, got {{ip_address}}");
                }}
                let _ = ports_map.remove(&port);
            }} else {{
                panic!("Unexpected iperf connection: {{line}}");
            }}
        }}
    }}

    if !ports_map.is_empty() {{
        for (port, expected_ip) in ports_map {{
            eprintln!("iperf client connection not found from {{expected_ip}}:{{port}}");
        }}
        panic!("Some iperf ports were not found in iperf server log");
    }}

    Ok(())
}}
"#).unwrap();
}

fn generate_dns_a_record_count_test(res: &mut String, test_name: &str, target_servers: &[String], queries: &[(String, usize)]) {
    write!(res, r#"
#[tokio::test]
async fn {test_name}() -> Result<(), Box<dyn Error>> {{
    let ips_and_dns = [
"#).unwrap();

    for (dns_name, count) in queries {
        write!(res, r#"        ("{dns_name}", {count}),
"#).unwrap()
    }

    write!(res, r#"
    ];
    let dns_servers = [
"#).unwrap();

    for server in target_servers {
        write!(res, r#"        "{server}",
"#).unwrap()
    }

    write!(res, r#"
    ];

    for server in dns_servers {{
        for (dns, expect) in &ips_and_dns {{
            let res = resolve_ip_custom_dns(server, dns).await?.len();
            assert_eq!(res, *expect as usize);
        }}
    }}

    Ok(())
}}
"#).unwrap();
}

fn main_rs() -> String {
r#"
#[cfg(test)]
mod common;
#[cfg(test)]
mod manual;
#[cfg(test)]
mod generated;

#[tokio::main]
async fn main() {
    panic!("Run 'cargo test' instead of this executable.");
}
"#.to_string()
}

fn cargo_toml() -> String {
r#"
[package]
name = "integration-tests"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
tokio = { version = "1.28.0", features = ["full"] }
reqwest = { version = "0.11.16", default-features = false, features = ["rustls-tls-webpki-roots"] }
rsdns = { version = "0.15.0", features = ["net-tokio"] }
rand = "0.8.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0.96"
urlencoding = "2.1.2"
headless_chrome = "1.0.5"
regex = "1.8.4"
futures = "0.3.30"
base64 = "0.21.7"
hex = "0.4.3"
"#.to_string()
}

// TODO: think of what to do with resolve ip
// per region. add regions? disable hardcodings
fn common_lib() -> String {
r#"
use std::{str::FromStr, error::Error, net::{SocketAddr, SocketAddrV4, Ipv4Addr}, collections::BTreeMap};
use rand::{seq::SliceRandom, Rng};
use reqwest::StatusCode;

pub fn generate_tempo_trace_id() -> String {
    hex::encode(rand::thread_rng().gen::<[u8; 16]>())
}

pub fn generate_tempo_span_id() -> String {
    hex::encode(rand::thread_rng().gen::<[u8; 8]>())
}

pub async fn resolve_ip_custom_dns(dns_server: &str, qname: &str) -> Result<Vec<std::net::Ipv4Addr>, Box<dyn Error>> {
    let nameserver = std::net::SocketAddr::from_str(dns_server)?;
    let config = rsdns::clients::ClientConfig::with_nameserver(nameserver);
    let mut client = rsdns::clients::tokio::Client::new(config).await?;
    let rrset = client.query_rrset::<rsdns::records::data::A>(qname, rsdns::constants::Class::In).await?;
    let mut res: Vec<std::net::Ipv4Addr> = rrset.rdata.iter().map(|i| {
        i.address
    }).collect();

    res.sort();

    Ok(res)
}

pub async fn resolve_ip_custom_dns_nsrecords(dns_server: &str, qname: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let nameserver = std::net::SocketAddr::from_str(dns_server)?;
    let config = rsdns::clients::ClientConfig::with_nameserver(nameserver);
    let mut client = rsdns::clients::tokio::Client::new(config).await?;
    let rrset = client.query_rrset::<rsdns::records::data::Ns>(qname, rsdns::constants::Class::In).await?;
    let res = rrset.rdata.iter().map(|i| {
        i.nsdname.as_str().to_string()
    }).collect();

    Ok(res)
}

pub async fn resolve_ip_custom_dns_ptr_records(dns_server: &str, qname: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let nameserver = std::net::SocketAddr::from_str(dns_server)?;
    let config = rsdns::clients::ClientConfig::with_nameserver(nameserver);
    let mut client = rsdns::clients::tokio::Client::new(config).await?;
    let rrset = client.query_rrset::<rsdns::records::data::Ptr>(qname, rsdns::constants::Class::In).await?;
    let res = rrset.rdata.iter().map(|i| {
        i.ptrdname.as_str().to_string()
    }).collect();

    Ok(res)
}

pub fn sorted<T: Ord>(inp: &[T]) -> Vec<&T> {
    let mut res = Vec::with_capacity(inp.len());
    for i in inp {
        res.push(i);
    }
    res.sort();
    res
}

pub fn to_ipv4_vec(input: &[&str]) -> Vec<std::net::Ipv4Addr> {
    input.iter().map(|i| {
        std::net::Ipv4Addr::from_str(*i).expect("Invalid ip address passed")
    }).collect::<Vec<_>>()
}

pub async fn test_tcp_socket(sock_addr: String) -> Result<(), Box<dyn Error>> {
    let addr = sock_addr.parse().unwrap();

    let socket = tokio::net::TcpSocket::new_v4()?;
    let _stream = socket.connect(addr).await?;

    Ok(())
}

pub struct HttpGetInput {
    pub dns_name: &'static str,
    pub ip: Option<String>,
    pub port: u16,
    pub path: String,
    pub is_https: bool,
    pub headers: Vec<(String, String)>,
}

pub struct HttpPostInput {
    pub dns_name: &'static str,
    pub ip: Option<String>,
    pub port: u16,
    pub path: &'static str,
    pub is_https: bool,
    pub input_body: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

pub struct HttpOutput {
    pub body: Vec<u8>,
    pub headers: BTreeMap<String, String>,
    pub status: StatusCode,
}

impl HttpOutput {
    pub fn body_to_string(&self) -> String {
        String::from_utf8(self.body.clone()).expect("Cannot convert body from utf-8 string")
    }
}

pub async fn http_get(input: HttpGetInput) -> Result<HttpOutput, Box<dyn Error>> {
    assert!(input.path.starts_with("/"));

    let target_ip = if let Some(ip) = input.ip {
        Ipv4Addr::from_str(ip.as_str()).expect("Cannot parse ipv4 ip")
    } else {
        panic!("Explicit ip must be specified")
    };
    let sock_addr = SocketAddr::V4(SocketAddrV4::new(target_ip, input.port));
    let client = reqwest::Client::builder()
        .timeout(tokio::time::Duration::from_millis(5000))
        .danger_accept_invalid_certs(true)
        .resolve(input.dns_name, sock_addr)
        .build()?;
    let proto = if input.is_https { "https" } else { "http" };
    let mut req = client.get(format!("{}://{}:{}{}", proto, input.dns_name, input.port, input.path));
    for (k, v) in &input.headers {
        req = req.header(k, v);
    }
    let mut headers = BTreeMap::new();
    let resp = req.send().await?;
    let status = resp.status();
    for (k, v) in resp.headers() {
        // we don't care about exotic header values when testing
        if let Ok(v) = v.to_str() {
            assert!(headers.insert(k.to_string(), v.to_string()).is_none());
        }
    }
    let body = resp.bytes().await?.to_vec();
    Ok(HttpOutput {
        body,
        status,
        headers,
    })
}

pub async fn http_post(input: HttpPostInput) -> Result<HttpOutput, Box<dyn Error>> {
    assert!(input.path.starts_with("/"));

    let target_ip = if let Some(ip) = input.ip {
        Ipv4Addr::from_str(ip.as_str()).expect("Cannot parse ipv4 ip")
    } else {
        panic!("Explicit ip must be specified")
    };
    let sock_addr = SocketAddr::V4(SocketAddrV4::new(target_ip, input.port));
    let client = reqwest::Client::builder()
        .timeout(tokio::time::Duration::from_millis(5000))
        .danger_accept_invalid_certs(true)
        .resolve(input.dns_name, sock_addr)
        .build()?;
    let proto = if input.is_https { "https" } else { "http" };
    let mut req = client.post(format!("{}://{}:{}{}", proto, input.dns_name, input.port, input.path));

    for (k, v) in &input.headers {
        req = req.header(k, v);
    }

    let mut headers = BTreeMap::new();
    let resp = req
        .body(input.input_body.clone())
        .send().await?;
    let status = resp.status();
    for (k, v) in resp.headers() {
        // we don't care about exotic header values when testing
        if let Ok(v) = v.to_str() {
            assert!(headers.insert(k.to_string(), v.to_string()).is_none());
        }
    }
    let body = resp.bytes().await?.to_vec();
    Ok(HttpOutput {
        body,
        status,
        headers,
    })
}

#[derive(Debug)]
struct InvalidPrometheusResponse;

impl std::fmt::Display for InvalidPrometheusResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"invalid prometheus response")
    }
}

impl std::error::Error for InvalidPrometheusResponse {
    fn description(&self) -> &str {
        "invalid prometheus response"
    }
}

pub async fn does_prometheus_metric_exist(ip: &str, port: u16, metric: &str) -> Result<bool, Box<dyn Error>> {
    let metric = urlencoding::encode(metric).to_string();
    let path = format!("/api/v1/query?query={metric}");
    let res = http_get(HttpGetInput {
        dns_name: "prometheus", ip: Some(ip.to_string()), port, path, is_https: false, headers: Vec::new(),
    }).await?;

    let body = res.body_to_string();
    if !body.contains("\"status\":\"success\"") {
        return Err(Box::new(InvalidPrometheusResponse));
    }

    return Ok(!body.contains("\"result\":[]"))
}

pub async fn execute_command_stdout(script: &str) -> String {
    use tokio::process::Command;
    println!("Executing command: {script}");
    let mut cmd = Command::new("sh");
    cmd.arg("-c");
    cmd.arg(script);
    let res = cmd.output().await.expect("Failed executing command");
    assert_eq!(res.status.code().unwrap(), 0);

    let output = String::from_utf8(res.stdout.clone()).expect("Invalid utf8 in command output");
    println!("Output: {output}");
    output
}

#[cfg(test)]
pub async fn ssh_command_stdout(server: &str, command: String) -> String {
    use tokio::process::Command;
    println!("Executing ssh command on server {server}: {command}");
    let mut cmd = Command::new("sh");
    cmd
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());
    let replaced = command.replace("\"", "\\\"");
    let script = format!(
        "ssh -o ConnectTimeout=5 -o ConnectionAttempts=1 -o ServerAliveCountMax=3 -o ServerAliveInterval=5 -o StrictHostKeyChecking=false -o UserKnownHostsFile=/dev/null -i ../servers/aux/root_ssh_key admin@{server} \"{replaced}\""
    );
    println!("{script}");
    cmd.arg("-c");
    cmd.arg(script);

    let timeout_seconds = 20;
    tokio::select! {
        res = cmd.output() => {
            let res = res.expect("Failed executing command");
            assert_eq!(res.status.code().unwrap(), 0);

            let output = String::from_utf8(res.stdout.clone()).expect("Invalid utf8 in command output");
            println!("Output: {output}");
            output
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(timeout_seconds)) => {
            panic!("ssh command timeout of {timeout_seconds} exceeded")
        },
    }
}

pub async fn ping_server(ip: &str) -> bool {
    use tokio::process::Command;

    let mut cmd = Command::new("ping");
    cmd.arg("-c");
    cmd.arg("1");
    cmd.arg("-W");
    cmd.arg("1");
    cmd.arg(ip);
    let res = cmd.output().await.expect("Failed executing command");

    res.status.code().unwrap() == 0
}

#[cfg(test)]
pub async fn check_if_loki_stream_exists(dns: &str, loki_service: &'static str, port: u16, stream: &str) -> Result<bool, Box<dyn Error>> {
    let reader = resolve_ip_custom_dns(dns, loki_service).await?;
    assert!(reader.len() > 0);

    let params = urlencoding::encode(stream).into_owned();
    let path = format!("/loki/api/v1/query_range?query={params}&limit=1");
    // at least one value contained
    let expected_value = "\"result\":[{";

    let reader = reader.choose(&mut rand::thread_rng()).unwrap();
    println!("Reader instance: {reader}");

    let res = http_get(HttpGetInput {
        dns_name: loki_service,
        ip: Some(reader.to_string()),
        port,
        path: path.clone(),
        is_https: false,
        headers: Vec::new(),
    }).await?;

    assert!(res.status.is_success());

    println!("{}", res.body_to_string());

    return Ok(res.body_to_string().contains(&expected_value));
}
"#.to_string()
}
