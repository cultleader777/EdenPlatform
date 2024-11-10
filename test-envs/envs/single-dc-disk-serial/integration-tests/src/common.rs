
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
