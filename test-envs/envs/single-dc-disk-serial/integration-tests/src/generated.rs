
use std::error::Error;
use crate::common::*;
use base64::prelude::*;


#[tokio::test]
async fn admin_panel_responds_responds() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<h1>Eden Platform admin panel</h1>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "admin.epl-infra.net",
            port: 443,
            path: "/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn consul_servers_available_from_region_us_west_to_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("consul.service.us-west.consul", &["10.17.0.10", "10.17.0.11", "10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn consul_servers_available_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("consul.service.consul", &["10.17.0.10", "10.17.0.11", "10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn consul_ui_external_responds_us_west() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Consul by HashiCorp</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/ui/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "adm-consul-us-west.epl-infra.net",
            port: 443,
            path: "/ui/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn consul_ui_responds_us_west() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "10.17.0.10",
        "10.17.0.11",
        "10.17.0.12",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Consul by HashiCorp</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:8501/ui/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "service",
            port: 8501,
            path: "/ui/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn dns_all_servers_resolve_from_master_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("server-a.us-west.epl-infra.net", &["10.17.0.10"]),
        ("server-b.us-west.epl-infra.net", &["10.17.0.11"]),
        ("server-c.us-west.epl-infra.net", &["10.17.0.12"]),
        ("server-d.us-west.epl-infra.net", &["10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.13:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_all_servers_resolve_from_others_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("server-a.us-west.epl-infra.net", &["10.17.0.10"]),
        ("server-b.us-west.epl-infra.net", &["10.17.0.11"]),
        ("server-c.us-west.epl-infra.net", &["10.17.0.12"]),
        ("server-d.us-west.epl-infra.net", &["10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",
        "10.17.0.11:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_all_servers_resolve_from_slave_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("server-a.us-west.epl-infra.net", &["10.17.0.10"]),
        ("server-b.us-west.epl-infra.net", &["10.17.0.11"]),
        ("server-c.us-west.epl-infra.net", &["10.17.0.12"]),
        ("server-d.us-west.epl-infra.net", &["10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.12:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ns_records_exist_for_regions_epl_infra_net() -> Result<(), Box<dyn Error>> {
    let names_and_ns_srvs = [
        ("us-west.epl-infra.net", ["ns1.us-west.epl-infra.net.", "ns2.us-west.epl-infra.net."]),

    ];
    let dns_servers = [
        "10.17.0.13:53",
        "10.17.0.12:53",

    ];

    for server in dns_servers {
        for (dns, recs) in &names_and_ns_srvs {
            let res = resolve_ip_custom_dns_nsrecords(server, dns).await?;
            assert_eq!(sorted(res.as_slice()), sorted(recs.as_slice()));
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ns_records_exist_for_root_in_master_servers_epl_infra_net() -> Result<(), Box<dyn Error>> {
    let names_and_ns_srvs = [
        ("epl-infra.net", ["ns1.epl-infra.net.", "ns2.epl-infra.net."]),

    ];
    let dns_servers = [
        "77.77.77.13:53",

    ];

    for server in dns_servers {
        for (dns, recs) in &names_and_ns_srvs {
            let res = resolve_ip_custom_dns_nsrecords(server, dns).await?;
            assert_eq!(sorted(res.as_slice()), sorted(recs.as_slice()));
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ns_records_exist_for_root_in_slave_servers_epl_infra_net() -> Result<(), Box<dyn Error>> {
    let names_and_ns_srvs = [
        ("epl-infra.net", ["ns1.epl-infra.net.", "ns2.epl-infra.net."]),

    ];
    let dns_servers = [
        "77.77.77.12:53",

    ];

    for server in dns_servers {
        for (dns, recs) in &names_and_ns_srvs {
            let res = resolve_ip_custom_dns_nsrecords(server, dns).await?;
            assert_eq!(sorted(res.as_slice()), sorted(recs.as_slice()));
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ns_records_resolve_to_ips_in_master_servers_epl_infra_net() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("ns1.epl-infra.net", &["77.77.77.13"]),
        ("ns2.epl-infra.net", &["77.77.77.12"]),

    ];
    let dns_servers = [
        "77.77.77.13:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ns_records_resolve_to_ips_in_slave_servers_epl_infra_net() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("ns1.epl-infra.net", &["77.77.77.13"]),
        ("ns2.epl-infra.net", &["77.77.77.12"]),

    ];
    let dns_servers = [
        "77.77.77.12:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ns_servers_resolve_from_all_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("ns1.us-west.epl-infra.net", &["10.17.0.13"]),
        ("ns2.us-west.epl-infra.net", &["10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",
        "10.17.0.11:53",
        "10.17.0.12:53",
        "10.17.0.13:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ptr_records_resolve_from_master_us_west() -> Result<(), Box<dyn Error>> {
    let ptr_and_dns: &[(&str, &[&str])] = &[
        ("10.0.17.10.in-addr.arpa.", &["server-a.us-west.epl-infra.net."]),
        ("11.0.17.10.in-addr.arpa.", &["server-b.us-west.epl-infra.net."]),
        ("12.0.17.10.in-addr.arpa.", &["ns2.us-west.epl-infra.net."]),
        ("13.0.17.10.in-addr.arpa.", &["ns1.us-west.epl-infra.net."]),

    ];
    let dns_servers = [
        "10.17.0.13:53",

    ];

    for server in dns_servers {
        for (dns, recs) in ptr_and_dns {
            let res = resolve_ip_custom_dns_ptr_records(server, dns).await?;
            assert_eq!(res, *recs);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ptr_records_resolve_from_others_us_west() -> Result<(), Box<dyn Error>> {
    let ptr_and_dns: &[(&str, &[&str])] = &[
        ("10.0.17.10.in-addr.arpa.", &["server-a.us-west.epl-infra.net."]),
        ("11.0.17.10.in-addr.arpa.", &["server-b.us-west.epl-infra.net."]),
        ("12.0.17.10.in-addr.arpa.", &["ns2.us-west.epl-infra.net."]),
        ("13.0.17.10.in-addr.arpa.", &["ns1.us-west.epl-infra.net."]),

    ];
    let dns_servers = [
        "10.17.0.10:53",
        "10.17.0.11:53",

    ];

    for server in dns_servers {
        for (dns, recs) in ptr_and_dns {
            let res = resolve_ip_custom_dns_ptr_records(server, dns).await?;
            assert_eq!(res, *recs);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_ptr_records_resolve_from_slave_us_west() -> Result<(), Box<dyn Error>> {
    let ptr_and_dns: &[(&str, &[&str])] = &[
        ("10.0.17.10.in-addr.arpa.", &["server-a.us-west.epl-infra.net."]),
        ("11.0.17.10.in-addr.arpa.", &["server-b.us-west.epl-infra.net."]),
        ("12.0.17.10.in-addr.arpa.", &["ns2.us-west.epl-infra.net."]),
        ("13.0.17.10.in-addr.arpa.", &["ns1.us-west.epl-infra.net."]),

    ];
    let dns_servers = [
        "10.17.0.12:53",

    ];

    for server in dns_servers {
        for (dns, recs) in ptr_and_dns {
            let res = resolve_ip_custom_dns_ptr_records(server, dns).await?;
            assert_eq!(res, *recs);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_public_dnssec_works_from_master() -> Result<(), Box<dyn Error>> {
    let ips = [
        "77.77.77.13",

    ];
    let domains: &[&str] = &[

    ];
    for ip in ips {
        for domain in domains {
            let res = execute_command_stdout(&format!("dig @{ip} {domain} +dnssec +short")).await;
            assert!(res.contains("A 15 3"));
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_public_dnssec_works_from_slave() -> Result<(), Box<dyn Error>> {
    let ips = [
        "77.77.77.12",

    ];
    let domains: &[&str] = &[

    ];
    for ip in ips {
        for domain in domains {
            let res = execute_command_stdout(&format!("dig @{ip} {domain} +dnssec +short")).await;
            assert!(res.contains("A 15 3"));
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_public_ingress_records_have_rev_in_master() -> Result<(), Box<dyn Error>> {
    let ptr_and_dns: &[(&str, &[&str])] = &[

    ];
    let dns_servers = [
        "77.77.77.13:53",

    ];

    for server in dns_servers {
        for (dns, recs) in ptr_and_dns {
            let res = resolve_ip_custom_dns_ptr_records(server, dns).await?;
            assert_eq!(res, *recs);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_public_ingress_records_have_rev_in_slave() -> Result<(), Box<dyn Error>> {
    let ptr_and_dns: &[(&str, &[&str])] = &[

    ];
    let dns_servers = [
        "77.77.77.12:53",

    ];

    for server in dns_servers {
        for (dns, recs) in ptr_and_dns {
            let res = resolve_ip_custom_dns_ptr_records(server, dns).await?;
            assert_eq!(res, *recs);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_public_ingress_records_resolve_from_master() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[

    ];
    let dns_servers = [
        "77.77.77.13:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_public_ingress_records_resolve_from_slave() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[

    ];
    let dns_servers = [
        "77.77.77.12:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_root_servers_resolve_from_master_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("ns1.epl-infra.net", &["10.17.0.13"]),
        ("ns2.epl-infra.net", &["10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.13:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_root_servers_resolve_from_others_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("ns1.epl-infra.net", &["10.17.0.13"]),
        ("ns2.epl-infra.net", &["10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",
        "10.17.0.11:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_root_servers_resolve_from_slave_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("ns1.epl-infra.net", &["10.17.0.13"]),
        ("ns2.epl-infra.net", &["10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.12:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn dns_sec_works_from_master_epl_infra_net_us_west() -> Result<(), Box<dyn Error>> {
    let ips = [
        "10.17.0.13",

    ];
    let routes = execute_command_stdout("ip route").await;
    let all_addrs = execute_command_stdout("ip addr").await;
    // if we have admin wireguard route use that
    // if libvirt just use the intrface
    let maybe_source_ip =
      if !routes.contains("10.0.0.0/8 dev wg") && all_addrs.contains("inet 10.17.0.1") {
        "-b 10.17.0.1"
      } else { "" };
    for ip in ips {
        let res = execute_command_stdout(&format!("dig {maybe_source_ip} @{ip} server-a.us-west.epl-infra.net +dnssec +short")).await;
        assert!(res.contains("10.17.0.10"));
        assert!(res.contains("us-west.epl-infra.net."));

        let res = execute_command_stdout(&format!("dig {maybe_source_ip} @{ip} us-west.epl-infra.net DS +dnssec | grep -E \"IN\\s+RRSIG\\s+DS\\s+15\"")).await;
        assert!(res.contains("RRSIG"));
    }

    Ok(())
}

#[tokio::test]
async fn dns_sec_works_from_others_epl_infra_net_us_west() -> Result<(), Box<dyn Error>> {
    let ips = [
        "10.17.0.10",
        "10.17.0.11",

    ];
    let routes = execute_command_stdout("ip route").await;
    let all_addrs = execute_command_stdout("ip addr").await;
    // if we have admin wireguard route use that
    // if libvirt just use the intrface
    let maybe_source_ip =
      if !routes.contains("10.0.0.0/8 dev wg") && all_addrs.contains("inet 10.17.0.1") {
        "-b 10.17.0.1"
      } else { "" };
    for ip in ips {
        let res = execute_command_stdout(&format!("dig {maybe_source_ip} @{ip} server-a.us-west.epl-infra.net +dnssec +short")).await;
        assert!(res.contains("10.17.0.10"));
        assert!(res.contains("us-west.epl-infra.net."));

        let res = execute_command_stdout(&format!("dig {maybe_source_ip} @{ip} us-west.epl-infra.net DS +dnssec | grep -E \"IN\\s+RRSIG\\s+DS\\s+15\"")).await;
        assert!(res.contains("RRSIG"));
    }

    Ok(())
}

#[tokio::test]
async fn dns_sec_works_from_slave_epl_infra_net_us_west() -> Result<(), Box<dyn Error>> {
    let ips = [
        "10.17.0.12",

    ];
    let routes = execute_command_stdout("ip route").await;
    let all_addrs = execute_command_stdout("ip addr").await;
    // if we have admin wireguard route use that
    // if libvirt just use the intrface
    let maybe_source_ip =
      if !routes.contains("10.0.0.0/8 dev wg") && all_addrs.contains("inet 10.17.0.1") {
        "-b 10.17.0.1"
      } else { "" };
    for ip in ips {
        let res = execute_command_stdout(&format!("dig {maybe_source_ip} @{ip} server-a.us-west.epl-infra.net +dnssec +short")).await;
        assert!(res.contains("10.17.0.10"));
        assert!(res.contains("us-west.epl-infra.net."));

        let res = execute_command_stdout(&format!("dig {maybe_source_ip} @{ip} us-west.epl-infra.net DS +dnssec | grep -E \"IN\\s+RRSIG\\s+DS\\s+15\"")).await;
        assert!(res.contains("RRSIG"));
    }

    Ok(())
}

#[tokio::test]
async fn docker_registry_us_west_dns_exists() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("epl-docker-registry.service.consul", &["10.17.0.10", "10.17.0.11", "10.17.0.12", "10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn docker_registry_us_west_healthcheck_responds_ok() -> Result<(), Box<dyn Error>> {

    let targets = [
        "10.17.0.10",
        "10.17.0.11",
        "10.17.0.12",
        "10.17.0.13",

    ];

    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:5000/");
        println!("Calling {url}");
        let res = http_get(HttpGetInput {
            dns_name: "service",
            is_https: false,
            ip: Some(t.to_string()),
            port: 5000,
            path: "/".to_string(),
            headers: Vec::new(),
        }).await?;
        if !res.status.is_success() {
            eprintln!("Http get to http://{t}:5000/ failed");
            some_failed = true;
        }
    }
    if some_failed {
        panic!("Some http checks have failed");
    }

    Ok(())
}

#[tokio::test]
async fn external_lb_region_us_west_metrics_exist() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "nginx_http_requests_total").await?);

    Ok(())
}

#[tokio::test]
async fn grafana_dns_record_expected_count_main() -> Result<(), Box<dyn Error>> {
    let ips_and_dns = [
        ("epl-grafana-main.service.consul", 2),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, expect) in &ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?.len();
            assert_eq!(res, *expect as usize);
        }
    }

    Ok(())
}

#[tokio::test]
async fn grafana_external_admin_panel_responds_main() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("GRAFANA_MAIN_ADMIN_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Grafana</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "adm-grafana-main.epl-infra.net",
            port: 443,
            path: "/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn grafana_node_exporter_dashboard_loaded_main() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("GRAFANA_MAIN_ADMIN_PASSWORD").unwrap())))),

    ];

    let expected_string = "\"title\":\"Node Exporter Full\"";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/api/dashboards/uid/rYdddlPWk");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "adm-grafana-main.epl-infra.net",
            port: 443,
            path: "/api/dashboards/uid/rYdddlPWk".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn internal_nodes_have_internet_region_us_west_dc_dc_1() -> Result<(), Box<dyn Error>> {
    let servers_and_ips: &[(&str, &[&str])] = &[
        ("10.17.0.10", &["1.1.1.1"]),
        ("10.17.0.11", &["1.1.1.1"]),

    ];

    let mut futures = Vec::new();
    let mut command_and_ssh_server = Vec::new();
    for (server, ips) in servers_and_ips {
        let mut to_join = Vec::new();
        for ip in *ips {
            to_join.push(format!("( ping -W 1 -c 1 {ip} )"));
        }
        let joined = to_join.join(" && ");
        command_and_ssh_server.push(format!("{server}: {joined}"));
        futures.push(ssh_command_stdout(server, joined));
    }

    let res = futures::future::join_all(futures).await;
    let mut failed = false;
    for (idx, r) in res.iter().enumerate() {
        if r.is_empty() {
            failed = true;
            eprintln!("Ping failed {}", command_and_ssh_server[idx]);
        }
    }

    if failed {
        panic!("Some pings failed");
    }

    Ok(())
}

#[tokio::test]
async fn internal_nodes_resolve_public_dns_region_us_west_dc_dc_1() -> Result<(), Box<dyn Error>> {
    let servers_and_domains: &[(&str, &[&str])] = &[
        ("10.17.0.10", &["example.com"]),
        ("10.17.0.11", &["example.com"]),

    ];

    let mut futures = Vec::new();
    let mut command_and_ssh_server = Vec::new();
    for (server, domains) in servers_and_domains {
        let mut to_join = Vec::new();
        for domain in *domains {
            to_join.push(format!("( dig {domain} +short | grep -F . )"));
        }
        let joined = to_join.join(" && ");
        command_and_ssh_server.push(format!("{server}: {joined}"));
        futures.push(ssh_command_stdout(server, joined));
    }

    let res = futures::future::join_all(futures).await;
    let mut failed = false;
    for (idx, r) in res.iter().enumerate() {
        if r.is_empty() {
            failed = true;
            eprintln!("Dns lookup failed {}", command_and_ssh_server[idx]);
        }
    }

    if failed {
        panic!("Some dns lookups failed");
    }

    Ok(())
}

#[tokio::test]
async fn intra_dc_dc_1_inside_subnet_ping_10_17_0_0p24_works() -> Result<(), Box<dyn Error>> {
    let servers_and_ips: &[(&str, &[&str])] = &[
        ("10.17.0.10", &["10.17.0.12"]),
        ("77.77.77.12", &["10.17.0.10"]),

    ];

    let mut futures = Vec::new();
    let mut command_and_ssh_server = Vec::new();
    for (server, ips) in servers_and_ips {
        let mut to_join = Vec::new();
        for ip in *ips {
            to_join.push(format!("( ping -W 1 -c 1 {ip} )"));
        }
        let joined = to_join.join(" && ");
        command_and_ssh_server.push(format!("{server}: {joined}"));
        futures.push(ssh_command_stdout(server, joined));
    }

    let res = futures::future::join_all(futures).await;
    let mut failed = false;
    for (idx, r) in res.iter().enumerate() {
        if r.is_empty() {
            failed = true;
            eprintln!("Ping failed {}", command_and_ssh_server[idx]);
        }
    }

    if failed {
        panic!("Some pings failed");
    }

    Ok(())
}

#[tokio::test]
async fn loki_cluster_main_has_journald_stream() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"journald\"}").await?);

    Ok(())
}

#[tokio::test]
async fn loki_cluster_main_has_l1_provisioning_stream() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"l1_provisioning\"}").await?);

    Ok(())
}

#[tokio::test]
async fn loki_cluster_main_has_l2_provisioning_stream() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"l2_provisioning\"}").await?);

    Ok(())
}

#[tokio::test]
async fn loki_cluster_main_read_write_test() -> Result<(), Box<dyn Error>> {
    use rand::{seq::SliceRandom, Rng};
    use serde_json::json;

    let reader_dns = "epl-loki-main-loki-reader.service.consul";
    let writer_dns = "epl-loki-main-loki-writer.service.consul";
    let reader_port = 3012;
    let writer_port = 3010;
    let reader = resolve_ip_custom_dns("10.17.0.10:53", reader_dns).await?;
    let writer = resolve_ip_custom_dns("10.17.0.10:53", writer_dns).await?;
    assert!(reader.len() > 0 && writer.len() > 0);

    let writer = writer.choose(&mut rand::thread_rng()).unwrap();

    let rnd_label: u64 = rand::thread_rng().gen();
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let time_nanos = since_the_epoch.as_nanos();
    let payload = json!(
        {
            "streams": [
                {
                    "stream": {
                        "foo": rnd_label.to_string(),
                    },
                    "values": [
                        [
                            format!("{time_nanos}"),
                            format!("henlo {rnd_label}"),
                        ]
                    ]
                }
            ]
        }
    );

    let res = http_post(HttpPostInput {
        dns_name: writer_dns,
        ip: Some(writer.to_string()),
        port: writer_port,
        path: "/loki/api/v1/push",
        is_https: false,
        input_body: serde_json::to_vec(&payload).unwrap(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ],
    }).await?;

    assert!(res.status.as_u16() == 204);

    let res = http_post(HttpPostInput {
        dns_name: writer_dns,
        ip: Some(writer.to_string()),
        port: writer_port,
        path: "/flush",
        is_https: false,
        input_body: Vec::new(),
        headers: Vec::new(),
    }).await?;

    assert!(res.status.as_u16() == 204);

    let expected_value = format!("\"result\":[{{\"stream\":{{\"foo\":\"{rnd_label}\"}},\"values\":[[\"{time_nanos}\",\"henlo {rnd_label}\"]]}}]");

    // let params = urlen
    let params = urlencoding::encode(&format!("{{foo=\"{rnd_label}\"}}")).into_owned();
    let path = format!("/loki/api/v1/query_range?query={params}");

    // repeat few times
    for _ in 0..10 {
        let reader = reader.choose(&mut rand::thread_rng()).unwrap();
        println!("Reader instance: {reader}");
        let res = http_get(HttpGetInput {
            dns_name: reader_dns,
            ip: Some(reader.to_string()),
            port: reader_port,
            path: path.clone(),
            is_https: false,
            headers: Vec::new(),
        }).await?;

        assert!(res.status.is_success());

        if res.body_to_string().contains(&expected_value) {
            return Ok(())
        }

        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    panic!("Test failed to contain expected value: {expected_value}");
}

#[tokio::test]
async fn loki_cluster_main_reader_dns_exist() -> Result<(), Box<dyn Error>> {
    let ips_and_dns = [
        ("epl-loki-main-loki-reader.service.consul", 2),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, expect) in &ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?.len();
            assert_eq!(res, *expect as usize);
        }
    }

    Ok(())
}

#[tokio::test]
async fn loki_cluster_main_reader_prometheus_metrics_exist() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "loki_querier_tail_active{job='epl-loki-main-loki-reader'}").await?);

    Ok(())
}

#[tokio::test]
async fn loki_cluster_main_writer_dns_exist() -> Result<(), Box<dyn Error>> {
    let ips_and_dns = [
        ("epl-loki-main-loki-writer.service.consul", 3),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, expect) in &ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?.len();
            assert_eq!(res, *expect as usize);
        }
    }

    Ok(())
}

#[tokio::test]
async fn loki_cluster_main_writer_prometheus_metrics_exist() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "loki_distributor_ingester_appends_total{job='epl-loki-main-loki-writer'}").await?);

    Ok(())
}

#[tokio::test]
async fn minio_cluster_global_healthcheck_responds_ok() -> Result<(), Box<dyn Error>> {

    let targets = [
        "10.17.0.10",
        "10.17.0.11",
        "10.17.0.12",
        "10.17.0.13",

    ];

    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9000/minio/health/live");
        println!("Calling {url}");
        let res = http_get(HttpGetInput {
            dns_name: "service",
            is_https: false,
            ip: Some(t.to_string()),
            port: 9000,
            path: "/minio/health/live".to_string(),
            headers: Vec::new(),
        }).await?;
        if !res.status.is_success() {
            eprintln!("Http get to http://{t}:9000/minio/health/live failed");
            some_failed = true;
        }
    }
    if some_failed {
        panic!("Some http checks have failed");
    }

    Ok(())
}

#[tokio::test]
async fn minio_cluster_global_instances_available_in_dns() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("epl-minio-global.service.consul", &["10.17.0.10", "10.17.0.11", "10.17.0.12", "10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn minio_cluster_global_prometheus_metrics_gathered() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "minio_s3_requests_total").await?);

    Ok(())
}

#[tokio::test]
async fn minio_external_admin_panel_responds_global() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>MinIO Console</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "adm-minio-global.epl-infra.net",
            port: 443,
            path: "/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_alertmanager_healthcheck_ok() -> Result<(), Box<dyn Error>> {

    let targets = [
        "10.17.0.10",
        "10.17.0.12",

    ];

    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9092/-/healthy");
        println!("Calling {url}");
        let res = http_get(HttpGetInput {
            dns_name: "service",
            is_https: false,
            ip: Some(t.to_string()),
            port: 9092,
            path: "/-/healthy".to_string(),
            headers: Vec::new(),
        }).await?;
        if !res.status.is_success() {
            eprintln!("Http get to http://{t}:9092/-/healthy failed");
            some_failed = true;
        }
    }
    if some_failed {
        panic!("Some http checks have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_alertmanager_peer_ports() -> Result<(), Box<dyn Error>> {
    let sockets = [
        "10.17.0.10:9093",
        "10.17.0.12:9093",

    ];

    let mut res = Vec::new();

    for s in sockets {
        res.push(test_tcp_socket(s.to_string()));
    }

    for f in res {
        f.await?;
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_alertmanager_ui() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "10.17.0.10",
        "10.17.0.12",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Alertmanager</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9092/#/alerts");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "service",
            port: 9092,
            path: "/#/alerts".to_string(),
            is_https: false,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_cadvisor_metrics_exist() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "container_cpu_user_seconds_total").await?);

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_default_prometheus_alert_exists() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "10.17.0.10",
        "10.17.0.12",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "\"name\":\"FilesystemSpaceLow\"";
    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9090/api/v1/rules");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "service",
            port: 9090,
            path: "/api/v1/rules".to_string(),
            is_https: false,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_dns_exists() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("epl-mon-default-prometheus.service.consul", &["10.17.0.10", "10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_has_epl_l1_provisioning_id_metrics() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "epl_l1_provisioning_id").await?);

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_has_vector_metrics() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "vector_buffer_events").await?);

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_no_scrape_targets_are_down() -> Result<(), Box<dyn Error>> {
    assert!(!does_prometheus_metric_exist("10.17.0.10", 9090, "up == 0").await?);

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_node_exporter_metrics_exist() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "node_cpu_seconds_total").await?);

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_prometheus_exposes_metrics() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "10.17.0.10",
        "10.17.0.12",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "prometheus_notifications_queue_capacity 10000";
    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9090/metrics");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "service",
            port: 9090,
            path: "/metrics".to_string(),
            is_https: false,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_prometheus_healthcheck_ok() -> Result<(), Box<dyn Error>> {

    let targets = [
        "10.17.0.10",
        "10.17.0.12",

    ];

    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9090/-/healthy");
        println!("Calling {url}");
        let res = http_get(HttpGetInput {
            dns_name: "service",
            is_https: false,
            ip: Some(t.to_string()),
            port: 9090,
            path: "/-/healthy".to_string(),
            headers: Vec::new(),
        }).await?;
        if !res.status.is_success() {
            eprintln!("Http get to http://{t}:9090/-/healthy failed");
            some_failed = true;
        }
    }
    if some_failed {
        panic!("Some http checks have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_prometheus_metrics_exist() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "prometheus_target_scrape_pool_targets").await?);

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_prometheus_ui_works() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "10.17.0.10",
        "10.17.0.12",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Prometheus Time Series Collection and Processing Server</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9090/graph");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "service",
            port: 9090,
            path: "/graph".to_string(),
            is_https: false,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_default_victoriametrics_healthcheck_ok() -> Result<(), Box<dyn Error>> {

    let targets = [
        "10.17.0.10",
        "10.17.0.12",

    ];

    let mut some_failed = false;
    for t in targets {
        let url = format!("http://{t}:9091/health");
        println!("Calling {url}");
        let res = http_get(HttpGetInput {
            dns_name: "service",
            is_https: false,
            ip: Some(t.to_string()),
            port: 9091,
            path: "/health".to_string(),
            headers: Vec::new(),
        }).await?;
        if !res.status.is_success() {
            eprintln!("Http get to http://{t}:9091/health failed");
            some_failed = true;
        }
    }
    if some_failed {
        panic!("Some http checks have failed");
    }

    Ok(())
}

#[tokio::test]
async fn monitoring_cluster_external_admin_panel_responds_default() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Prometheus Time Series Collection and Processing Server</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/graph");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "adm-prometheus-default.epl-infra.net",
            port: 443,
            path: "/graph".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn nomad_clients_available_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("nomad-clients.service.consul", &["10.17.0.10", "10.17.0.11", "10.17.0.12", "10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn nomad_job_region_us_west_job_grafana_main_logs_in_main() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"nomad_docker\",job_name=\"grafana-main\"}").await?);

    Ok(())
}

#[tokio::test]
async fn nomad_job_region_us_west_job_loki_main_logs_in_main() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"nomad_docker\",job_name=\"loki-main\"}").await?);

    Ok(())
}

#[tokio::test]
async fn nomad_job_region_us_west_job_minio_global_logs_in_main() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"nomad_docker\",job_name=\"minio-global\"}").await?);

    Ok(())
}

#[tokio::test]
async fn nomad_job_region_us_west_job_pg_testdb_logs_in_main() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"nomad_docker\",job_name=\"pg-testdb\"}").await?);

    Ok(())
}

#[tokio::test]
async fn nomad_job_region_us_west_job_tempo_us_west_logs_in_main() -> Result<(), Box<dyn Error>> {
    assert!(check_if_loki_stream_exists("10.17.0.10:53", "epl-loki-main-loki-reader.service.consul", 3012, "{source_type=\"nomad_docker\",job_name=\"tempo-us-west\"}").await?);

    Ok(())
}

#[tokio::test]
async fn nomad_region_us_west_prometheus_metrics_gathered() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "nomad_client_allocs_cpu_allocated").await?);

    Ok(())
}

#[tokio::test]
async fn nomad_servers_available_from_region_us_west_to_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("nomad-servers.service.us-west.consul", &["10.17.0.10", "10.17.0.12", "10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn nomad_servers_available_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("nomad-servers.service.consul", &["10.17.0.10", "10.17.0.12", "10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn nomad_ui_external_responds_us_west() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Nomad</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "adm-nomad-us-west.epl-infra.net",
            port: 443,
            path: "/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn nomad_ui_responds_us_west() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "10.17.0.10",
        "10.17.0.12",
        "10.17.0.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Nomad</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:4646/ui/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "service",
            port: 4646,
            path: "/ui/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn pg_deployment_testdb_db_exists_grafana() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "pg_locks_count{job='epl-pg-testdb-pg-exp',datname='grafana'}").await?);

    Ok(())
}

#[tokio::test]
async fn pg_deployment_testdb_db_exists_postgres() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "pg_locks_count{job='epl-pg-testdb-pg-exp',datname='postgres'}").await?);

    Ok(())
}

#[tokio::test]
async fn pg_deployment_testdb_dns_exists() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("epl-pg-testdb.service.consul", &["10.17.0.10", "10.17.0.11", "10.17.0.12"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn pg_deployment_testdb_prometheus_metrics_gathered() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "pg_locks_count{job='epl-pg-testdb-pg-exp'}").await?);

    Ok(())
}

#[tokio::test]
async fn pg_deployment_testdb_sockets_open() -> Result<(), Box<dyn Error>> {
    let sockets = [
        "10.17.0.10:5432",
        "10.17.0.10:5433",
        "10.17.0.10:5434",
        "10.17.0.10:5435",
        "10.17.0.11:5432",
        "10.17.0.11:5433",
        "10.17.0.11:5434",
        "10.17.0.11:5435",
        "10.17.0.12:5432",
        "10.17.0.12:5433",
        "10.17.0.12:5434",
        "10.17.0.12:5435",

    ];

    let mut res = Vec::new();

    for s in sockets {
        res.push(test_tcp_socket(s.to_string()));
    }

    for f in res {
        f.await?;
    }

    Ok(())
}

#[tokio::test]
async fn private_ips_ping() -> Result<(), Box<dyn Error>> {

    let targets: &[&str] = &[
        "10.17.0.10",
        "10.17.0.11",
        "10.17.0.12",
        "10.17.0.13",

    ];

    for t in targets {
        assert!(ping_server(t).await);
    }

    Ok(())
}

#[tokio::test]
async fn public_ips_ping() -> Result<(), Box<dyn Error>> {

    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];

    for t in targets {
        assert!(ping_server(t).await);
    }

    Ok(())
}

#[tokio::test]
async fn public_nodes_have_internet_region_us_west_dc_dc_1() -> Result<(), Box<dyn Error>> {
    let servers_and_ips: &[(&str, &[&str])] = &[
        ("77.77.77.12", &["1.1.1.1"]),
        ("77.77.77.13", &["1.1.1.1"]),

    ];

    let mut futures = Vec::new();
    let mut command_and_ssh_server = Vec::new();
    for (server, ips) in servers_and_ips {
        let mut to_join = Vec::new();
        for ip in *ips {
            to_join.push(format!("( ping -W 1 -c 1 {ip} )"));
        }
        let joined = to_join.join(" && ");
        command_and_ssh_server.push(format!("{server}: {joined}"));
        futures.push(ssh_command_stdout(server, joined));
    }

    let res = futures::future::join_all(futures).await;
    let mut failed = false;
    for (idx, r) in res.iter().enumerate() {
        if r.is_empty() {
            failed = true;
            eprintln!("Ping failed {}", command_and_ssh_server[idx]);
        }
    }

    if failed {
        panic!("Some pings failed");
    }

    Ok(())
}

#[tokio::test]
async fn public_nodes_resolve_public_dns_region_us_west_dc_dc_1() -> Result<(), Box<dyn Error>> {
    let servers_and_domains: &[(&str, &[&str])] = &[
        ("77.77.77.12", &["example.com"]),
        ("77.77.77.13", &["example.com"]),

    ];

    let mut futures = Vec::new();
    let mut command_and_ssh_server = Vec::new();
    for (server, domains) in servers_and_domains {
        let mut to_join = Vec::new();
        for domain in *domains {
            to_join.push(format!("( dig {domain} +short | grep -F . )"));
        }
        let joined = to_join.join(" && ");
        command_and_ssh_server.push(format!("{server}: {joined}"));
        futures.push(ssh_command_stdout(server, joined));
    }

    let res = futures::future::join_all(futures).await;
    let mut failed = false;
    for (idx, r) in res.iter().enumerate() {
        if r.is_empty() {
            failed = true;
            eprintln!("Dns lookup failed {}", command_and_ssh_server[idx]);
        }
    }

    if failed {
        panic!("Some dns lookups failed");
    }

    Ok(())
}

#[tokio::test]
async fn tempo_cluster_us_west_storing_and_querying_traces_works() -> Result<(), Box<dyn Error>> {
    use rand::seq::SliceRandom;
    use serde_json::json;
    use std::ops::Add;

    let dns = "epl-tempo-us-west.service.consul";
    let push_port = 4313;
    let query_port = 4310;
    let instances = resolve_ip_custom_dns("10.17.0.10:53", dns).await?;
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
    let payload = json!({
        "resourceSpans": [{
        "resource": {
                "attributes": [{
                "key": "service.name",
                "value": {
                        "stringValue": "my.service"
                }
                }]
        },
        "scopeSpans": [{
                "scope": {
                "name": "my.library",
                "version": "1.0.0",
                "attributes": [{
                        "key": "my.scope.attribute",
                        "value": {
                            "stringValue": "some scope attribute"
                        }
                }]
                },
                "spans": [
                {
                "traceId": trace_id,
                "spanId": span_id,
                "name": "I am a span!",
                "startTimeUnixNano": start_time_nanos,
                "endTimeUnixNano": end_time_nanos,
                "kind": 2,
                "attributes": [
                {
                        "key": "my.span.attr",
                        "value": {
                        "stringValue": "some value"
                        }
                }]
                }]
        }]
        }]
    });

    let res = http_post(HttpPostInput {
        dns_name: dns,
        ip: Some(instance.to_string()),
        port: push_port,
        path: "/v1/traces",
        is_https: false,
        input_body: serde_json::to_vec(&payload).unwrap(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ],
    }).await?;

    assert!(res.status.as_u16() == 200);

    let res = http_post(HttpPostInput {
        dns_name: dns,
        ip: Some(instance.to_string()),
        port: query_port,
        path: "/flush",
        is_https: false,
        input_body: Vec::new(),
        headers: Vec::new(),
    }).await?;

    assert!(res.status.as_u16() == 204);

    let path = format!("/api/traces/{trace_id}");

    // repeat few times
    for _ in 0..10 {
        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let reader = instances.choose(&mut rand::thread_rng()).unwrap();
        println!("Reader instance: {reader}");
        let res = http_get(HttpGetInput {
            dns_name: dns,
            ip: Some(reader.to_string()),
            port: query_port,
            path: path.clone(),
            is_https: false,
            headers: Vec::new(),
        }).await?;

        if !res.status.is_success() {
            continue;
        }

        if res.status.as_u16() == 200 {
            return Ok(());
        }
    }

    panic!("Test failed to find the trace id {trace_id}");
}

#[tokio::test]
async fn vault_active_server_available_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns = [
        ("active.vault.service.consul", 1),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, expect) in &ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?.len();
            assert_eq!(res, *expect as usize);
        }
    }

    Ok(())
}

#[tokio::test]
async fn vault_region_us_west_prometheus_metrics_gathered() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "vault_barrier_list_count").await?);

    Ok(())
}

#[tokio::test]
async fn vault_servers_available_from_region_us_west_to_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("vault.service.us-west.consul", &["10.17.0.11", "10.17.0.12", "10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn vault_servers_available_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns: &[(&str, &[&str])] = &[
        ("vault.service.consul", &["10.17.0.11", "10.17.0.12", "10.17.0.13"]),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, ip) in ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?;
            let expect = to_ipv4_vec(ip);
            assert_eq!(res, expect);
        }
    }

    Ok(())
}

#[tokio::test]
async fn vault_standby_server_available_region_us_west() -> Result<(), Box<dyn Error>> {
    let ips_and_dns = [
        ("standby.vault.service.consul", 2),

    ];
    let dns_servers = [
        "10.17.0.10:53",

    ];

    for server in dns_servers {
        for (dns, expect) in &ips_and_dns {
            let res = resolve_ip_custom_dns(server, dns).await?.len();
            assert_eq!(res, *expect as usize);
        }
    }

    Ok(())
}

#[tokio::test]
async fn vault_ui_external_responds_us_west() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "77.77.77.12",
        "77.77.77.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Vault</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:443/ui/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "adm-vault-us-west.epl-infra.net",
            port: 443,
            path: "/ui/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}

#[tokio::test]
async fn vault_ui_responds_us_west() -> Result<(), Box<dyn Error>> {
    let targets: &[&str] = &[
        "10.17.0.11",
        "10.17.0.12",
        "10.17.0.13",

    ];
    let headers = vec![
      ("Authorization".to_string(), format!("Basic {}", BASE64_STANDARD.encode(&format!("admin:{}", std::env::var("ADMIN_PANEL_PASSWORD").unwrap())))),

    ];

    let expected_string = "<title>Vault</title>";
    let mut some_failed = false;
    for t in targets {
        let url = format!("https://{t}:8200/ui/");
        println!("Calling {url}");
        let resp = http_get(HttpGetInput {
            dns_name: "service",
            port: 8200,
            path: "/ui/".to_string(),
            is_https: true,
            ip: Some(t.to_string()),
            headers: headers.clone(),
        }).await?;
        if resp.status != reqwest::StatusCode::OK {
            eprintln!("Request {url} did not return 200");
            some_failed = true;
        }
        let body = resp.body_to_string();
        if !body.contains(expected_string) {
            eprintln!("Response body {url} does not contain expected string '{expected_string}' body: '{body}'");
            some_failed = true;
        }
    }

    if some_failed {
        panic!("Some http requests have failed");
    }

    Ok(())
}
