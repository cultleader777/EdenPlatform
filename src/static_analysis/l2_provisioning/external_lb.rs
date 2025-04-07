use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use convert_case::Casing;

use crate::codegen::l1_provisioning::dns::sanitize_dns_name;
use crate::database::{TableRowPointerRegion, TableRowPointerServer, TableRowPointerTld};
use crate::static_analysis::{L1Projections, server_runtime, get_global_settings};
use crate::static_analysis::docker_images::image_handle_from_pin;
use crate::static_analysis::networking::{prometheus_metric_exists_test, check_servers_regional_distribution};
use crate::static_analysis::server_runtime::{epl_architecture_to_nomad_architecture, VaultSecretHandle};
use crate::{
    database::Database,
    static_analysis::{
        http_endpoints::{HttpPathRoute, PageMethod, ValidHttpPathSegment},
        server_runtime::{
            NomadJobKind, NomadJobStage, RouteData, ServerRuntime, SystemServerVolume,
        },
        PlatformValidationError,
    },
};

struct TldCert {
    chain: VaultSecretHandle,
    private_key: VaultSecretHandle,
}

pub fn deploy_external_lb(
    db: &Database,
    region: TableRowPointerRegion,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    let gs = get_global_settings(db);
    let docker_image_pin = db.region().c_docker_image_external_lb(region);
    let prometheus_lua_module =
        String::from_utf8(include_bytes!("lb_lua_modules/prometheus.lua").to_vec())
            .expect("Must be valid string");
    let prometheus_lua_module_keys =
        String::from_utf8(include_bytes!("lb_lua_modules/prometheus_keys.lua").to_vec())
            .expect("Must be valid string");
    let prometheus_lua_module_counter =
        String::from_utf8(include_bytes!("lb_lua_modules/prometheus_resty_counter.lua").to_vec())
            .expect("Must be valid string");
    let region_name = db.region().c_region_name(region);

    let metrics_expose_subnet = db
        .network()
        .rows_iter()
        .find(|n| db.network().c_network_name(*n) == "lan")
        .map(|i| db.network().c_cidr(i).clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let mut basic_auth_files = Vec::new();
    let site_config_file = generate_lb_site_conf(
        db, gs.admin_tld,
        "/local/admin_htpasswd_file",
        runtime, region,
        &mut basic_auth_files,
    );

    let admin_site = generate_admin_site(runtime);
    let service_slug = "epl-external-lb";
    let nomad_job_name = "external-lb";
    let ngx_conf_abs =
        runtime.consul_kv_write(
            region,
            "epl-kv/external-elb-conf".to_string(),
            generate_lb_nginx_conf(&site_config_file, "/secrets/prom-server.conf")
        );

    let admin_html_config =
        runtime.consul_kv_write(region, "epl-kv/admin-site-html".to_string(), admin_site.clone());
    let region_servers =
        db.region()
          .c_referrers_datacenter__region(region)
          .iter()
          .map(|i| db.datacenter().c_referrers_server__dc(*i))
          .flatten();
    let ingress_servers: Vec<TableRowPointerServer> = region_servers
        .filter_map(|srv| {
            if db.server().c_is_ingress(*srv) {
                Some(*srv)
            } else { None }
        }).collect::<Vec<_>>();

    check_servers_regional_distribution(
        db,
        region,
        ingress_servers.iter().cloned(),
        format!("External Load balancer for region {region_name}")
    )?;

    let mut ingress_server_archs: BTreeSet<String> = BTreeSet::new();
    for is in &ingress_servers {
        let sk = l1proj.server_kinds.value(*is);
        if !ingress_server_archs.contains(db.server_kind().c_architecture(*sk)) {
            let _ = ingress_server_archs.insert(db.server_kind().c_architecture(*sk).clone());
        }
    }

    let http_port = runtime.lock_port_all_servers_duplicate_ok(80, "External LB HTTP".to_string())?;
    let https_port = runtime.lock_port_all_servers_duplicate_ok(443, "External LB HTTPS".to_string())?;
    // Duplicate ok because locks come from multiple regions
    let memory =
        runtime.reserve_memory_every_server_mb(db, "External load balancer".to_string(), 64)?;

    let mon_cluster = l1proj.monitoring_clusters.region_default(region);
    for arch in &ingress_server_archs {
        let docker_image = image_handle_from_pin(db, &arch, docker_image_pin, "openresty")?;
        let server_lock = if ingress_servers.len() > 0 {
            // println!("{:?} {:?}", region, ingress_servers);
            Some(runtime.lock_servers_with_label(db, format!("epl-ingress-{region_name}"), &ingress_servers)?)
        } else { None };
        let cert_volume = runtime.system_volume_all_servers_read_lock(
            db,
            SystemServerVolume::TlsCertificates,
            "External LB certificates for serving external HTTPS traffic".to_string(),
        )?;

        let mut secrets = runtime.issue_vault_secret_renew_from_source(region, "ext-lb");

        let mut cert_map: BTreeMap<TableRowPointerTld, TldCert> = BTreeMap::new();

        for tld in db.tld().rows_iter() {
            if db.tld().c_automatic_certificates(tld) {
                let domain = db.tld().c_domain(tld);
                let domain_kebab = domain.replace(".", "-");
                let sanitized = sanitize_dns_name(&domain);
                let cert_name = format!("certs/{domain_kebab}");
                let sec_handle_chain = {
                    let handle = VaultSecretHandle::from_epl_kv_secret(&cert_name, "full_chain", server_runtime::VaultSecretRequest::Pem);
                    let output_key_chain = format!("full_chain_{sanitized}");
                    secrets.request_secret(
                        region,
                        &output_key_chain,
                        server_runtime::VaultSecretRequest::ExistingVaultSecret {
                            handle: Box::new(handle.clone()), sprintf: None
                        },
                    )
                };

                let sec_handle_key = {
                    let handle = VaultSecretHandle::from_epl_kv_secret(&cert_name, "key", server_runtime::VaultSecretRequest::Pem);
                    let output_key_chain = format!("key_{sanitized}");
                    secrets.request_secret(
                        region,
                        &output_key_chain,
                        server_runtime::VaultSecretRequest::ExistingVaultSecret {
                            handle: Box::new(handle.clone()), sprintf: None
                        },
                    )
                };

                assert!(cert_map.insert(tld, TldCert {
                    chain: sec_handle_chain,
                    private_key: sec_handle_key,
                }).is_none());
            }
        }

        let finalized_secrets =
            if !cert_map.is_empty() {
              Some(secrets.finalize())
            } else { None };

        let consul_service_metrics = runtime.instantiate_and_seal_consul_service(region, service_slug);

        let job = runtime.fetch_nomad_job(
            l1proj.epl_nomad_namespace,
            nomad_job_name.to_string(),
            region,
            NomadJobKind::SystemStateless,
            NomadJobStage::SystemJob,
        );
        if let Some(finalized_secrets) = finalized_secrets {
            job.assign_vault_secrets(finalized_secrets);
        }
        job.add_replacement_macro(
            "ADMIN_PANEL_HTPASSWD_FILE".to_string(),
            server_runtime::ReplaceWith::EplSecretKeyValue("admin_panel_htpasswd_file".to_string())
        );

        let tg_name = format!("external-lb-{arch}");
        let tg = job.fetch_task_group(tg_name);
        tg.constrain_architecture(epl_architecture_to_nomad_architecture(&arch));
        if let Some(server_lock) = server_lock {
            tg.assign_server_lock(server_lock);
        }
        tg.add_locked_port("http", http_port.clone());
        tg.add_locked_port("https", https_port.clone());
        tg.expose_port_as_tcp_service("http", &consul_service_metrics);
        if let Some(mon_cluster) = &mon_cluster {
            tg.collect_prometheus_metrics(&consul_service_metrics, *mon_cluster, None);
        }

        let task = tg.fetch_task("external-lb".to_string(), docker_image);
        let _admin_htpasswd_path =
            task.add_local_file("admin_htpasswd_file".to_string(), "ADMIN_PANEL_HTPASSWD_FILE".to_string());
        task.add_local_file("prometheus.lua".to_string(), prometheus_lua_module.clone());
        task.add_local_file(
            "prometheus_keys.lua".to_string(),
            prometheus_lua_module_keys.clone(),
        );
        task.add_local_file(
            "prometheus_resty_counter.lua".to_string(),
            prometheus_lua_module_counter.clone(),
        );

        for (tld, the_val) in &cert_map {
            let domain = db.tld().c_domain(*tld);
            let mut texpr = the_val.chain.template_expression();
            texpr += "\n";
            task.add_secure_config_wchange_signal(format!("tls_cert_{domain}.pem"), texpr, server_runtime::ChangeSignal::SIGHUP);
            let mut texpr = the_val.private_key.template_expression();
            texpr += "\n";
            task.add_secure_config_wchange_signal(format!("tls_key_{domain}.pem"), texpr, server_runtime::ChangeSignal::SIGHUP);
        }

        task.bind_volume(cert_volume, "/etc/ssl".to_string());
        task.add_memory(memory.clone());
        let prom_conf = lua_prometheus_server(&metrics_expose_subnet);
        // need to interpret {{ meta.private_ip in nomad }}
        let _prom_conf_path =
            task.add_secure_config("prom-server.conf".to_string(), prom_conf);
        let onchange_conf_path =
            task.add_executable_local_file(
                "onchange-conf".to_ascii_lowercase(),
                change_sites_script().to_string(),
            );
        let onchange_index_html_path =
            task.add_executable_local_file(
                "onchange-index-html".to_ascii_lowercase(),
                change_admin_html_script().to_string(),
            );
        let start_path =
            task.add_executable_local_file(
                "start".to_ascii_lowercase(),
                startup_script(
                    "/secrets/nginx.conf",
                    &onchange_conf_path,
                    &onchange_index_html_path
                ).to_string(),
            );
        let _ = task.add_consul_kv_file_with_change_script(
            "admin-index.html.src".to_string(),
            admin_html_config.clone(),
            Some(onchange_index_html_path.to_string()),
        );
        let _ = task.add_consul_kv_file_with_change_script(
            "nginx.conf.src".to_string(),
            ngx_conf_abs.clone(),
            Some(onchange_conf_path.to_string()),
        );

        // TODO: we could unload all basic auth files via one command
        // from consul key value store so it gets unloaded once
        // and then we don't need to change the job declaration.
        //
        // compile time gzip base64 for the filesystem of files wanted?
        for (fname, contents) in &basic_auth_files {
            task.add_secure_config(fname.clone(), contents.clone());
        }

        task.set_entrypoint(vec![start_path]);
    }

    external_lb_tests(db, l1proj, runtime, region);

    Ok(())
}

fn startup_script(
    nginx_conf: &str,
    on_change_nginx_conf_path: &str,
    on_change_index_html_path: &str,
) -> String {
    format!(r#"#!/bin/sh

{on_change_nginx_conf_path} noreload
{on_change_index_html_path}

exec /usr/bin/openresty -g 'daemon off;' -c {nginx_conf}
"#)
}

fn change_sites_script() -> &'static str {
    r#"#!/bin/sh

NO_RELOAD=$1

# we don't worry about failures because we assume
# lb config is tested well enough in compile time
gunzip -c /secrets/nginx.conf.src > /secrets/nginx.conf.tmp
mv -f /secrets/nginx.conf.tmp /secrets/nginx.conf

if [ -z "$NO_RELOAD" ]
then
  /usr/bin/openresty -s reload -c /secrets/nginx.conf
fi
"#
}

fn change_admin_html_script() -> &'static str {
    r#"#!/bin/sh

NO_RELOAD=$1

# we don't worry about failures because we assume
# lb config is tested well enough in compile time
gunzip -c /secrets/admin-index.html.src > /secrets/admin-index.html
mkdir -p /local/www
mv -f /secrets/admin-index.html /local/www/index.html
"#
}

fn external_lb_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime, region: TableRowPointerRegion) {
    if let Some(mon_cluster) = l1proj.monitoring_clusters.region_default(region) {
        let region_snake = db.region().c_region_name(region).to_case(convert_case::Case::Snake);
        runtime.add_integration_test(
            format!("external_lb_region_{region_snake}_metrics_exist"),
            prometheus_metric_exists_test(db, l1proj, mon_cluster, "nginx_http_requests_total")
        );
    }
}

fn ensure_no_html_text(input: &str) {
    assert!(!input.contains('<') && !input.contains('>'));
}

fn generate_admin_site(runtime: &ServerRuntime) -> String {
    let mut res = String::new();

    res += "<html>\n";
    res += "<head>\n";
    res += "</head>\n";
    res += "<body>\n";
    res += "<h1>Eden Platform admin panel</h1>\n";
    for (k, v) in runtime.admin_links() {
        ensure_no_html_text(k);
        res += "<h3>";
        res += k;
        res += "</h3>\n";
        res += "<ul>\n";
        for link in v {
            res += "<li>";
            res += link;
            res += "</li>";
        }
        res += "</ul>\n";
    }
    res += "</body>\n";
    res += "</html>\n";

    res
}

fn generate_lb_site_conf(
    db: &Database,
    admin_tld: TableRowPointerTld,
    admin_htpasswd_file_path: &str,
    runtime: &ServerRuntime,
    region: TableRowPointerRegion,
    // files where we're asked to make basic auth
    basic_auth_files: &mut Vec<(String, String)>,
) -> String {
    let settings = get_global_settings(db);

    let (maybe_ipv6_http, maybe_ipv6_https) =
        if settings.enable_ipv6 {
            ("
    listen [::]:80;",
             "
    listen [::]:443 ssl;")
        } else { ("", "") };

    let mut res = format!(
        r#"
server {{
    listen 80;{maybe_ipv6_http}
    server_name _;
    return 301 https://$host$request_uri;
}}
"#
    );

    { // admin tld
        let certs = if db.tld().c_automatic_certificates(admin_tld) {
            let domain = db.tld().c_domain(admin_tld);
            format!(r#"
    ssl_certificate /secrets/tls_cert_{domain}.pem;
    ssl_certificate_key /secrets/tls_key_{domain}.pem;
"#)
        } else {
            r#"
    ssl_certificate /etc/ssl/public_tls_cert.pem;
    ssl_certificate_key /etc/ssl/public_tls_key.pem;
"#.to_string()
        };
        let admin_tld = db.tld().c_domain(admin_tld);

        write!(&mut res,
               r#"
server {{
    listen 80;{maybe_ipv6_http}
    server_name admin.{admin_tld};
    return 301 https://$host$request_uri;
}}

server {{
    resolver 127.0.0.1:53 valid=10s;

    listen 443 ssl;{maybe_ipv6_https}

    auth_basic "Administrator Area";
    auth_basic_user_file {admin_htpasswd_file_path};

    server_name admin.{admin_tld};

    ssl_ciphers ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA256;
{certs}

    location / {{
        root /local/www/;
    }}
}}
"#
        ).unwrap();
    }

    for (k, v) in runtime.frontend_lb_routes(region) {
        for (subd, rc) in &v.subdomains {
            let certs = if db.tld().c_automatic_certificates(*k) {
                let domain = db.tld().c_domain(*k);
                format!(r#"
    ssl_certificate /secrets/tls_cert_{domain}.pem;
    ssl_certificate_key /secrets/tls_key_{domain}.pem;
"#)
            } else {
                r#"
    ssl_certificate /etc/ssl/public_tls_cert.pem;
    ssl_certificate_key /etc/ssl/public_tls_key.pem;
"#.to_string()
            };
            let subdomain = if !subd.subdomain.is_empty() {
                format!("{}.", subd.subdomain)
            } else { "".to_string() };
            write!(&mut res, r#"
server {{
    listen 80;{maybe_ipv6_http}
    server_name {}{};
    return 301 https://$host$request_uri;
}}

"#, subdomain, db.tld().c_domain(*k)).unwrap();
            res += "server {\n";
            res += "    resolver 127.0.0.1:53 valid=10s;\n";
            res += "    listen 443 ssl;";
            res += maybe_ipv6_https;
            res += "\n";
            res += &format!(
                "    server_name {}{};\n",
                subdomain,
                db.tld().c_domain(*k)
            );
            // grafana has its own auth
            // minio has its own
            let already_have_auth =
                subd.subdomain.starts_with("adm-grafana")
                || subd.subdomain.starts_with("adm-minio");
            if subd.subdomain.starts_with("adm-") && !already_have_auth {
                write!(&mut res, r#"
    auth_basic "Administrator Area";
    auth_basic_user_file {admin_htpasswd_file_path};
"#).unwrap();
            }
            res += "    ssl_ciphers ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA256;\n";
            res += &certs;
            res += "\n";

            let fwd_routes = rc.routes.flat_pages();
            let mut routes_map: BTreeMap<Vec<ValidHttpPathSegment>, Vec<HttpPathRoute<RouteData>>> =
                Default::default();
            for r in &fwd_routes {
                let k = r.source_path.clone();
                let e = routes_map.entry(k).or_default();
                e.push(r.clone());
            }

            let mut sorted_routes = routes_map.values().collect::<Vec<_>>();
            // most specific routes are first, then forward to root routes
            sorted_routes.sort_by_key(|i| i[0].source_path.len());
            sorted_routes.reverse();

            for route in sorted_routes {
                let mut methods: BTreeSet<PageMethod> = BTreeSet::new();
                for i in route {
                    assert!(
                        methods.insert(i.method.clone()),
                        "We expect these to be variations of different methods?"
                    );
                    for j in route {
                        assert_eq!(i.value, j.value, "Content is assumed to be equal to ensure same proxy pass rule applies everywhere");
                    }
                }
                let nginx_regex = methods
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join("|");
                let full_condition = format!(" $request_method !~ ^({nginx_regex})$ ");
                let target_route = &route[0];
                let content = &target_route.value;

                let is_epl_app = content.content.is_epl_app();
                let location_path = mk_nginx_path(target_route);
                res += "    location ~ ^";
                res += &location_path;
                if is_epl_app {
                    res += "$";
                }

                res += " {\n";

                if !content.basic_auth.is_empty() {
                    // no downtime reload screwed
                    let new_pwd_file = format!("basic_auth_{}.htpasswd", basic_auth_files.len() + 1);
                    write!(&mut res, r#"
        auth_basic "Restricted Area";
        auth_basic_user_file /secrets/{new_pwd_file};
"#).unwrap();
                    let mut content = content.basic_auth.clone();
                    if !content.ends_with("\n") {
                        content.push('\n');
                    }
                    basic_auth_files.push((new_pwd_file, content));
                }

                match &content.content {
                    crate::static_analysis::server_runtime::RouteContent::InternalUpstream {
                        consul_service,
                        port,
                        is_https,
                        target_path,
                        unlimited_body,
                    } => {
                        let is_nomad = consul_service.service_name() == "nomad-servers";
                        let is_consul = consul_service.service_name() == "consul";
                        // TODO: why vault returns 500 errors when cache is on like nomad/consul?
                        // let is_vault = consul_service.service_name() == "vault";
                        if is_epl_app {
                            // restruct only certain routes
                            res += "        if (";
                            res += &full_condition;
                            res += ") {\n";
                            res += "            return 405;\n";
                            res += "        }\n";
                        }

                        // nomad special https://developer.hashicorp.com/nomad/tutorials/manage-clusters/reverse-proxy-ui
                        if is_nomad {
                            res += "        proxy_buffering off;
        proxy_read_timeout 310s;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header Origin \"${scheme}://${proxy_host}\";
";
                        }

                        if *unlimited_body {
                            res += "
        client_max_body_size 0;
";
                        }

                        res += "        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_set_header Host $host;
";
                        assert!(target_path.starts_with('/'));
                        let protocol = if *is_https { "https" } else { "http" };

                        res += "        set $dummy_var ";
                        res += &consul_service.service_fqdn();
                        res += ":";
                        res += &port.to_string();
                        res += ";\n";
                        // if we have prefix forward as is
                        if &location_path != target_path {
                            res += "        rewrite ";
                            res += &location_path;
                            res += " ";
                            res += target_path;
                            res += " break;\n";
                        }
                        // This trick with $dummy_var is needed if service is down and nginx
                        // can't resolve service on startup, so it wouldn't fail
                        // https://serverfault.com/questions/700894/make-nginx-ignore-site-config-when-its-upstream-cannot-be-reached
                        writeln!(res, "        proxy_pass {}://$dummy_var;", protocol)
                            .expect("Should work");
                        // If say wasm frontend is rolled out and newer hash javascript is not found
                        // then after getting 404 try next upstream which is a newer version and
                        // should have the newer file
                        if target_path.contains("epl-app-") {
                            writeln!(res, "        proxy_next_upstream error timeout http_404 http_500;")
                                .expect("Should work");
                        }

                        // cache assets for fast loading
                        if is_consul {
                            write!(&mut res, "
        location ~* ^/ui/assets/.* {{
            expires 30d;
            add_header Cache-Control private;
            proxy_pass {protocol}://$dummy_var;
        }}
").expect("should work");
                        }
                    }
                }
                res += "    }\n\n";
            }

            res += "}\n";
            res += "\n";
        }
    }

    res
}

fn mk_nginx_path<T: Clone>(route: &HttpPathRoute<T>) -> String {
    let mut res = String::new();
    for i in &route.source_path {
        match i {
            crate::static_analysis::http_endpoints::ValidHttpPathSegment::StaticPath(p) => {
                res += p;
            }
            crate::static_analysis::http_endpoints::ValidHttpPathSegment::Prefix(p) => {
                res += p;
                res += "(.+)";
            }
            crate::static_analysis::http_endpoints::ValidHttpPathSegment::Argument => {
                res += "(.+)";
            }
            crate::static_analysis::http_endpoints::ValidHttpPathSegment::Slash => {
                res += "/";
            }
        }
    }
    res
}

fn generate_lb_nginx_conf(site_contents: &str, prom_server_path: &str) -> String {
    let lua_prom = lua_prometheus_wiring();

    format!(
        r#"
pcre_jit on;

worker_processes 4;
worker_rlimit_nofile 12288;

events {{
    worker_connections  1024;
}}

http {{
    include       /etc/openresty/mime.types;
    default_type  application/octet-stream;
    # started failing in actual env
    server_names_hash_bucket_size 128;

    # for websockets
    map $http_upgrade $connection_upgrade {{
        default upgrade;
        '' close;
    }}

    # Log in JSON Format
    log_format nginxlog_json escape=json '{{ "@timestamp": "$time_iso8601", '
         '"remote_addr": "$remote_addr", '
         '"body_bytes_sent": $body_bytes_sent, '
         '"gzip_ratio": "$gzip_ratio", '
         '"request_time": $request_time, '
         '"upstream_response_time": $upstream_response_time, '
         '"response_status": $status, '
         '"request": "$request", '
         '"request_method": "$request_method", '
         '"host": "$host",'
         '"upstream_addr": "$upstream_addr",'
         '"http_x_forwarded_for": "$http_x_forwarded_for",'
         '"http_referrer": "$http_referer", '
         '"http_user_agent": "$http_user_agent", '
         '"http_version": "$server_protocol", '
         '"server_port": "$server_port"}}';
    access_log /dev/stdout nginxlog_json;

    # See Move default writable paths to a dedicated directory (#119)
    # https://github.com/openresty/docker-openresty/issues/119
    client_body_temp_path /var/run/openresty/nginx-client-body;
    proxy_temp_path       /var/run/openresty/nginx-proxy;
    fastcgi_temp_path     /var/run/openresty/nginx-fastcgi;
    uwsgi_temp_path       /var/run/openresty/nginx-uwsgi;
    scgi_temp_path        /var/run/openresty/nginx-scgi;

    sendfile        on;

    keepalive_timeout  65;

    gzip  on;
    gzip_proxied no-cache no-store private expired auth;
    gzip_types application/json text/html;

    include {prom_server_path};

{lua_prom}
{site_contents}
}}
"#
    )
}

// As per https://github.com/knyar/nginx-lua-prometheus docs
fn lua_prometheus_server(subnet: &str) -> String {
    format!(
        r#"
server {{
  listen {{{{ env "meta.private_ip" }}}}:80;
  allow {subnet};
  deny all;
  location /metrics {{
    content_by_lua_block {{
      metric_connections:set(ngx.var.connections_reading, {{"reading"}})
      metric_connections:set(ngx.var.connections_waiting, {{"waiting"}})
      metric_connections:set(ngx.var.connections_writing, {{"writing"}})
      prometheus:collect()
    }}
  }}

  location / {{
    return 404;
  }}
}}
"#
    )
}

fn lua_prometheus_wiring() -> &'static str {
    r#"
    lua_shared_dict prometheus_metrics 10M;
    lua_package_path '/local/?.lua;;';

    init_worker_by_lua_block {
      prometheus = require("prometheus").init("prometheus_metrics")

      metric_requests = prometheus:counter(
        "nginx_http_requests_total", "Number of HTTP requests", {"host", "status"})
      metric_latency = prometheus:histogram(
        "nginx_http_request_duration_seconds", "HTTP request latency", {"host"})
      metric_connections = prometheus:gauge(
        "nginx_http_connections", "Number of HTTP connections", {"state"})
      metric_response_sizes = prometheus:histogram(
        "nginx_http_response_size_bytes", "Size of HTTP responses", nil,
        {10,100,1000,10000,100000,1000000})
    }

    log_by_lua_block {
      metric_requests:inc(1, {ngx.var.server_name, ngx.var.status})
      metric_latency:observe(tonumber(ngx.var.request_time), {ngx.var.server_name})
      metric_response_sizes:observe(tonumber(ngx.var.bytes_sent))
    }
"#
}
