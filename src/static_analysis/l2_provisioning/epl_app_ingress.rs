use std::collections::{BTreeSet, BTreeMap, HashMap};

use regex::Regex;

use crate::{
    database::{
        Database, TableRowPointerBackendApplicationDeploymentIngress,
        TableRowPointerBackendHttpEndpoint, TableRowPointerFrontendPage, TableRowPointerTld, TableRowPointerRegion,
    },
    static_analysis::{
        http_endpoints::{CheckedHttpEndpoint, CorePathSegment, PageMethod, PathArgs},
        projections::Projection,
        server_runtime::{RouteContent, RouteData, ServerRuntime},
        PlatformValidationError, get_global_settings, SyncChecksOutputs,
    },
};

#[cfg(test)]
use crate::static_analysis::http_endpoints::ValidHttpPrimitiveType;

#[derive(Default, Clone)]
pub struct IpsCollection {
    pub ipv4: Vec<std::net::Ipv4Addr>,
    pub ipv6: Vec<std::net::Ipv6Addr>,
}

pub fn ingress_static_analysis(
    db: &Database,
    sync_checks: &SyncChecksOutputs,
) -> Result<(BTreeMap<TableRowPointerTld, BTreeMap<String, IpsCollection>>, HashMap<TableRowPointerRegion, IpsCollection>), PlatformValidationError> {
    let settings = get_global_settings(db);
    for i in db.frontend_application_deployment().rows_iter() {
        let ingress_count = db
            .frontend_application_deployment()
            .c_referrers_frontend_application_deployment_ingress__deployment(i)
            .len();
        if ingress_count > 1 {
            return Err(
                PlatformValidationError::FrontendApplicationDeploymentHasMoreThanOneIngress {
                    application_name: db
                        .frontend_application()
                        .c_application_name(
                            db.frontend_application_deployment().c_application_name(i),
                        )
                        .clone(),
                    deployment_name: db
                        .frontend_application_deployment()
                        .c_deployment_name(i)
                        .clone(),
                    maximum_allowed: 1,
                    actual: ingress_count,
                },
            );
        }
    }

    if !settings.disable_deployment_min_server_tests {
        let mut regions_with_deployments = BTreeSet::new();
        for deployment in db.frontend_application_deployment().rows_iter() {
            let region = db.frontend_application_deployment().c_region(deployment);
            regions_with_deployments.insert(region);
        }
        for deployment in db.backend_application_deployment().rows_iter() {
            let region = db.backend_application_deployment().c_region(deployment);
            regions_with_deployments.insert(region);
        }

        for region in &regions_with_deployments {
            let dcs = db.region().c_referrers_datacenter__region(*region);
            let mut servers = Vec::new();
            for dc in dcs {
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    servers.push(*server);
                }
            }
            if servers.len() < 4 {
                return Err(
                    PlatformValidationError::RegionWithDeploymentsHasLessThanFourServers {
                        region: db.region().c_region_name(*region).clone(),
                        servers: servers.iter().map(|i| {
                            db.server().c_hostname(*i).clone()
                        }).collect::<Vec<_>>(),
                        minimum: 4,
                    },
                );
            }
        }
    }

    let mut dns_table: BTreeMap<TableRowPointerTld, BTreeMap<String, TableRowPointerRegion>> =
        BTreeMap::new();
    let mut final_dns_table: BTreeMap<TableRowPointerTld, BTreeMap<String, IpsCollection>> =
        BTreeMap::new();
    let mut region_ingresses: HashMap<TableRowPointerRegion, IpsCollection> =
        HashMap::new();

    if !settings.disable_deployment_min_ingress_tests {
        let mut regions_with_ingresses = BTreeSet::new();

        for fe_ingress in db.frontend_application_deployment_ingress().rows_iter() {
            let deployment = db.frontend_application_deployment_ingress().c_deployment(fe_ingress);
            let region = db.frontend_application_deployment().c_region(deployment);
            regions_with_ingresses.insert(region);

            let tld = db.frontend_application_deployment_ingress().c_tld(fe_ingress);
            let e = dns_table.entry(tld).or_default();
            let subdomain = db.frontend_application_deployment_ingress().c_subdomain(fe_ingress);
            if let Some(used_region) = e.get(subdomain) {
                if region != *used_region {
                    panic!(
                        "Multiple regions targeted with single fqdn {}.{} is not supported yet (frontend deployment {} is trying to do)",
                        db.frontend_application_deployment_ingress().c_subdomain(fe_ingress),
                        db.tld().c_domain(tld),
                        db.frontend_application_deployment().c_deployment_name(deployment),
                    )
                }
            } else {
                let _ = e.insert(subdomain.clone(), region);
            }
        }

        for be_ingress in db.backend_application_deployment_ingress().rows_iter() {
            let deployment = db.backend_application_deployment_ingress().c_deployment(be_ingress);
            let region = db.backend_application_deployment().c_region(deployment);
            regions_with_ingresses.insert(region);

            let tld = db.backend_application_deployment_ingress().c_tld(be_ingress);
            let e = dns_table.entry(tld).or_default();
            let subdomain = db.backend_application_deployment_ingress().c_subdomain(be_ingress);
            if let Some(used_region) = e.get(subdomain) {
                if region != *used_region {
                    panic!(
                        "Multiple regions targeted with single fqdn {}.{} is not supported yet (backend deployment {} is trying to do)",
                        db.backend_application_deployment_ingress().c_subdomain(be_ingress),
                        db.tld().c_domain(tld),
                        db.backend_application_deployment().c_deployment_name(deployment),
                    )
                }
            } else {
                let _ = e.insert(subdomain.clone(), region);
            }
        }

        for bb_ingress in db.blackbox_deployment_ingress().rows_iter() {
            let service_reg = db.blackbox_deployment_ingress().c_service(bb_ingress);
            let deployment = db.blackbox_deployment_service_registration().c_parent(service_reg);
            let region = db.blackbox_deployment().c_region(deployment);
            regions_with_ingresses.insert(region);
            let tld = db.blackbox_deployment_ingress().c_tld(bb_ingress);
            let e = dns_table.entry(tld).or_default();
            let subdomain = db.blackbox_deployment_ingress().c_subdomain(bb_ingress);
            if let Some(used_region) = e.get(subdomain) {
                if region != *used_region {
                    panic!(
                        "Multiple regions targeted with single fqdn {}.{} is not supported yet (blackbox deployment {} is trying to do)",
                        db.blackbox_deployment_ingress().c_subdomain(bb_ingress),
                        db.tld().c_domain(tld),
                        db.blackbox_deployment().c_deployment_name(deployment),
                    )
                }
            } else {
                let _ = e.insert(subdomain.clone(), region);
            }
        }

        for region in db.region().rows_iter() {
            let mut ingress_servers = Vec::new();
            let mut ipv6_ingress_servers = Vec::new();
            for dc in db.region().c_referrers_datacenter__region(region) {
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    if db.server().c_is_ingress(*server) {
                        ingress_servers.push(*server);
                        let iter =
                            db.server().c_children_network_interface(*server)
                                .iter().map(|i| {
                                    db.network().c_network_name(db.network_interface().c_if_network(*i))
                                });

                        if iter.clone().find(|i| *i == "internet").is_none() {
                            return Err(
                                PlatformValidationError::ServerMarkedAsIngressHasNoPublicIpInterface {
                                    datacenter: db.datacenter().c_dc_name(*dc).clone(),
                                    server: db.server().c_hostname(*server).clone(),
                                    existing_network_interfaces: iter.map(|i| i.clone()).collect(),
                                    missing_network_interface: "internet".to_string(),
                                },
                            );
                        }

                        let re = region_ingresses.entry(region).or_default();
                        for net_if in db.server().c_children_network_interface(*server) {
                            if "internet" == db.network().c_network_name(db.network_interface().c_if_network(*net_if)) {
                                re.ipv4.push(db.network_interface().c_if_ip(*net_if).parse::<std::net::Ipv4Addr>().unwrap());
                            }
                        }

                        if settings.enable_ipv6 {
                            if let Some(ipv6_addr) = sync_checks.network.node_public_ipv6_addrs.get(&server) {
                                re.ipv6.push(ipv6_addr.clone());
                                ipv6_ingress_servers.push(*server);
                            }
                        }
                    }
                }

            }

            if ingress_servers.len() < 2 {
                // two ingress per DC is  must
                return Err(
                    PlatformValidationError::RegionWithIngressesHasLessThanTwoIngressServers {
                        region: db.region().c_region_name(region).clone(),
                        ingress_servers: ingress_servers.iter().map(|i| {
                            db.server().c_hostname(*i).clone()
                        }).collect::<Vec<_>>(),
                        minimum: 2,
                    },
                );
            }

            // either we have some ipv6 ingress, then we should have ipv6 for all
            // or ipv6 is forced so we must check that we have ipv6 on every ingress server
            let expect_ipv6_ingress = ipv6_ingress_servers.len() > 0 || settings.force_ipv6;
            if settings.enable_ipv6 && expect_ipv6_ingress {
                // not all ingress servers have ipv6 address
                if ipv6_ingress_servers.len() < ingress_servers.len() {
                    return Err(
                        PlatformValidationError::RegionWithIngressesHasInconsistentIpV6IngressCount {
                            region: db.region().c_region_name(region).clone(),
                            ipv4_ingress_servers: ingress_servers.iter().map(|i| {
                                db.server().c_hostname(*i).clone()
                            }).collect::<Vec<_>>(),
                            ipv6_ingress_servers: ipv6_ingress_servers.iter().map(|i| {
                                db.server().c_hostname(*i).clone()
                            }).collect::<Vec<_>>(),
                            expected_ipv6_ingress_count: ingress_servers.len(),
                            is_ipv6_support_enabled: settings.enable_ipv6,
                            is_ipv6_support_forced: settings.force_ipv6,
                        },
                    );
                }
            }
        }
    }

    for (tld, doms) in &dns_table {
        let hosts = final_dns_table.entry(*tld).or_default();
        for (dom, dc) in doms {
            let mut subdomain = "".to_string();
            if !dom.is_empty() {
                subdomain = format!("{}.", dom);
            }
            let fqdn = format!("{}{}.", subdomain, db.tld().c_domain(*tld));
            let _ = hosts.insert(fqdn, region_ingresses.get(dc).unwrap().clone());
        }
    }

    Ok((final_dns_table, region_ingresses))
}

pub fn deploy_epl_backend_applications(
    db: &Database,
    runtime: &mut ServerRuntime,
    checked_http_endpoints: &Projection<TableRowPointerBackendHttpEndpoint, CheckedHttpEndpoint>,
    endpoints_list: &Projection<
        TableRowPointerBackendApplicationDeploymentIngress,
        BTreeSet<TableRowPointerBackendHttpEndpoint>,
    >,
) -> Result<(), PlatformValidationError> {
    for ingress in db.backend_application_deployment_ingress().rows_iter() {
        let deployment = db
            .backend_application_deployment_ingress()
            .c_deployment(ingress);
        let region = db.backend_application_deployment().c_region(deployment);
        let tld = db.backend_application_deployment_ingress().c_tld(ingress);
        let subdomain = db
            .backend_application_deployment_ingress()
            .c_subdomain(ingress);
        let mountpoint = db
            .backend_application_deployment_ingress()
            .c_mountpoint(ingress);
        let application = db
            .backend_application_deployment()
            .c_application_name(deployment);

        let mountpoint_segments = is_valid_mountpoint(mountpoint.as_str());
        if mountpoint_segments.is_none() {
            return Err(PlatformValidationError::AppIngressInvalidMountpoint {
                deployment: db.backend_application_deployment().c_deployment_name(deployment).clone(),
                application_name: db.backend_application().c_application_name(application).clone(),
                mountpoint: mountpoint.to_string(),
                explanation: "Mountpoint must start and end with a slash and its path segments must be alphanumeric symbols",
            });
        }
        let mountpoint_segments = mountpoint_segments.unwrap();
        let endpoints = endpoints_list.value(ingress);

        for e in endpoints {
            let checked = checked_http_endpoints.value(*e);
            let nginx_path = mk_nginx_path(&mountpoint_segments, &checked.path_args.required_args);
            let service = runtime.fetch_existing_consul_service(region, &format!(
                "epl-app-{}",
                db.backend_application_deployment()
                    .c_deployment_name(deployment)
            ));
            let upstream = RouteContent::InternalUpstream {
                is_https: false,
                consul_service: service,
                unlimited_body: checked.receive_body_as_stream,
                port: db
                    .backend_application_deployment()
                    .c_http_port(deployment)
                    .try_into()
                    .unwrap(),
                target_path: nginx_path.downstream_path.clone(),
            };
            let rd = RouteData { content: upstream, basic_auth: "".to_string() };
            runtime.expose_route_in_tld_for_app(
                region,
                tld,
                subdomain,
                &checked.path_args.required_args,
                rd,
                from_str_page_method(
                    db.http_methods()
                        .c_http_method_name(db.backend_http_endpoint().c_http_method(*e)),
                ),
                &db.backend_application_deployment()
                    .c_deployment_name(deployment)
                    .clone(),
                db.backend_application().c_application_name(application),
                &mountpoint_segments,
                mountpoint.as_str(),
            )?;
        }
    }

    Ok(())
}

pub fn deploy_epl_frontend_applications(
    db: &Database,
    runtime: &mut ServerRuntime,
    checked_frontend_endpoints: &Projection<TableRowPointerFrontendPage, PathArgs>,
) -> Result<(), PlatformValidationError> {
    // DONE: iterate all frontend applications and forward their
    // http paths to the downstream application
    // 1 to 1 paths, downstream nginx instance
    // must be able to handle multiple pages (just serve same page under different path)
    // hmmm... Or maybe frontend is ALWAYS mounted to root?.. because frontend must have full path?
    // no, frontend doesn't need full path only relative path and we can make correct relative
    // path behaviour an assumption in the codebase

    for fr_ing in db.frontend_application_deployment_ingress().rows_iter() {
        let depl = db
            .frontend_application_deployment_ingress()
            .c_deployment(fr_ing);
        let region = db.frontend_application_deployment().c_region(depl);
        let tld = db.frontend_application_deployment_ingress().c_tld(fr_ing);
        let subdomain = db
            .frontend_application_deployment_ingress()
            .c_subdomain(fr_ing);
        let app = db
            .frontend_application_deployment()
            .c_application_name(depl);
        let mountpoint = db
            .frontend_application_deployment_ingress()
            .c_mountpoint(fr_ing);
        let mountpoint_segments = is_valid_mountpoint(mountpoint.as_str());
        if mountpoint_segments.is_none() {
            return Err(PlatformValidationError::AppIngressInvalidMountpoint {
                deployment: db.frontend_application_deployment().c_deployment_name(depl).clone(),
                application_name: db.frontend_application().c_application_name(app).clone(),
                mountpoint: mountpoint.to_string(),
                explanation: "Mountpoint must start and end with a slash and its path segments must be alphanumeric symbols",
            });
        }
        let mountpoint_segments = mountpoint_segments.unwrap();

        let service = runtime.fetch_existing_consul_service(region, &format!(
            "epl-app-{}",
            db.frontend_application_deployment().c_deployment_name(depl)
        ));

        let resources_upstream = RouteContent::InternalUpstream {
            is_https: false,
            consul_service: service.clone(),
            port: db
                .frontend_application_deployment()
                .c_http_port(depl)
                .try_into()
                .unwrap(),
            target_path: "/epl-app-$1".to_string(),
            unlimited_body: false,
        };

        runtime.expose_prefix_in_tld_for_frontend_app(
            region,
            tld,
            subdomain,
            "epl-app-",
            RouteData {
                content: resources_upstream,
                basic_auth: "".to_string(),
            },
            db.frontend_application_deployment()
                .c_deployment_name(depl)
                .as_str(),
            db.frontend_application().c_application_name(app).as_str(),
            &mountpoint_segments,
        )?;

        for page in db.frontend_application().c_children_frontend_page(app) {
            let path_args = checked_frontend_endpoints.value(*page);
            let nginx_path = mk_nginx_path(&mountpoint_segments, &path_args.required_args);
            let upstream = RouteContent::InternalUpstream {
                is_https: false,
                consul_service: service.clone(),
                port: db
                    .frontend_application_deployment()
                    .c_http_port(depl)
                    .try_into()
                    .unwrap(),
                target_path: nginx_path.downstream_path.clone(),
                unlimited_body: false,
            };
            let rd = RouteData { content: upstream, basic_auth: "".to_string() };
            runtime.expose_route_in_tld_for_app(
                region,
                tld,
                subdomain,
                &path_args.required_args,
                rd,
                PageMethod::GET,
                &db.frontend_application_deployment()
                    .c_deployment_name(depl)
                    .clone(),
                db.frontend_application().c_application_name(app),
                &mountpoint_segments,
                mountpoint.as_str(),
            )?;
        }
    }

    Ok(())
}

pub fn deploy_epl_blackbox_applications(
    db: &Database,
    runtime: &mut ServerRuntime,
) -> Result<(), PlatformValidationError> {
    for ingress in db.blackbox_deployment_ingress().rows_iter() {
        let service = db.blackbox_deployment_ingress().c_service(ingress);
        let port = db.blackbox_deployment_ingress().c_port(ingress);
        let deployment = db.blackbox_deployment_service_registration().c_parent(service);
        let region = db.blackbox_deployment().c_region(deployment);
        let tld = db.blackbox_deployment_ingress().c_tld(ingress);
        let subdomain = db
            .blackbox_deployment_ingress()
            .c_subdomain(ingress);

        let mut found_ports = 0;
        for reg in db.blackbox_deployment_service_registration().c_referrers_blackbox_deployment_service_instance__service_registration(service) {
            let inst_port = db.blackbox_deployment_service_instance().c_port(*reg);
            let inst_port_value = db.blackbox_deployment_port().c_port(inst_port);
            if port == inst_port_value {
                if db.blackbox_deployment_port().c_protocol(inst_port) != "http" {
                    return Err(PlatformValidationError::BlackboxIngressPortInvalidProtocol {
                        bb_deployment: db.blackbox_deployment().c_deployment_name(deployment).clone(),
                        port: inst_port_value,
                        protocol: db.blackbox_deployment_port().c_protocol(inst_port).to_string(),
                        expected_protocol: "http".to_string(),
                    });
                }

                found_ports += 1;
            }
        }

        if found_ports == 0 {
            return Err(PlatformValidationError::BlackboxIngressPortNotFound {
                ingress_tld: db.tld().c_domain(tld).clone(),
                ingress_subdomain: db.blackbox_deployment_ingress().c_subdomain(ingress).clone(),
                bb_deployment: db.blackbox_deployment().c_deployment_name(deployment).clone(),
                port,
            });
        }

        let service = runtime.fetch_existing_consul_service(
            region, &db.blackbox_deployment_service_registration().c_service_name(service)
        );
        let upstream = RouteContent::InternalUpstream {
            is_https: false,
            consul_service: service,
            unlimited_body: false,
            port: port.try_into()
                .unwrap(),
            target_path: "/".to_string(),
        };
        runtime.expose_root_route_in_tld(
            db,
            region,
            tld,
            subdomain,
            RouteData {
                content: upstream,
                basic_auth: db.blackbox_deployment_ingress().c_basic_auth_credentials(ingress).clone(),
            },
        )?;
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct NginxTargetPath {
    frontend_path: String,
    downstream_path: String,
}

fn mk_nginx_path(mountpoint: &[String], current_path: &[CorePathSegment]) -> NginxTargetPath {
    use std::fmt::Write;

    let mut f_path = '/'.to_string();
    for i in mountpoint {
        f_path += i;
        f_path.push('/');
    }

    let mut t_path = String::new();

    let mut arg_count = 1;
    for cp in current_path {
        match cp {
            CorePathSegment::Text(p) => {
                if !f_path.ends_with('/') {
                    f_path.push('/');
                }
                f_path += p;

                if !t_path.ends_with('/') {
                    t_path.push('/');
                }
                t_path += p;
            }
            CorePathSegment::Argument(_, _) => {
                let arg = arg_count;
                arg_count += 1;
                if !f_path.ends_with('/') {
                    f_path.push('/');
                }
                f_path += "(.*)";

                if !t_path.ends_with('/') {
                    t_path.push('/');
                }
                t_path.push('$');
                write!(t_path, "{}", arg).unwrap();
            }
            CorePathSegment::LastSlash => {
                f_path.push('/');
                t_path.push('/');
            }
        }
    }

    NginxTargetPath {
        frontend_path: f_path,
        downstream_path: t_path,
    }
}

fn from_str_page_method(input: &str) -> PageMethod {
    match input {
        "GET" => PageMethod::GET,
        "POST" => PageMethod::POST,
        "PUT" => PageMethod::PUT,
        m => panic!("Unexpected page method: {m}"),
    }
}

lazy_static! {
    static ref VALID_MOUNTPOINT_SEGMENT_REGEX: Regex = Regex::new("^[a-z0-9_-]+$").unwrap();
}

fn is_valid_mountpoint(input: &str) -> Option<Vec<String>> {
    if !input.starts_with('/') {
        return None;
    }

    if !input.ends_with('/') {
        return None;
    }

    let segments = input.split('/').collect::<Vec<_>>();
    assert!(segments.len() >= 2);

    let to_check = if segments.len() == 2 {
        // / route case, return empty slice of segments to check
        &segments[0..0]
    } else {
        // /a/b/c/ route case, return all strings between slashes
        &segments[1..segments.len() - 1]
    };

    for segment in to_check {
        if !VALID_MOUNTPOINT_SEGMENT_REGEX.is_match(*segment) {
            return None;
        }
    }

    Some(to_check.iter().map(|i| i.to_string()).collect::<Vec<_>>())
}

#[test]
fn test_valid_egress_mountpoints() {
    assert_eq!(is_valid_mountpoint("/"), Some(vec![]));
    assert_eq!(
        is_valid_mountpoint("/hello/"),
        Some(vec!["hello".to_string()])
    );
    assert_eq!(
        is_valid_mountpoint("/hello/world/"),
        Some(vec!["hello".to_string(), "world".to_string()])
    );
    assert_eq!(is_valid_mountpoint("/hello/world"), None);
    assert_eq!(is_valid_mountpoint("hello/world/"), None);
    assert_eq!(is_valid_mountpoint("other"), None);
    assert_eq!(is_valid_mountpoint("/@/world/"), None);
}

#[test]
fn test_projecting_nginx_paths() {
    assert_eq!(
        mk_nginx_path(
            &["a".to_string(), "b".to_string()],
            &[
                CorePathSegment::Text("hello".to_string()),
                CorePathSegment::Argument("arg1".to_string(), ValidHttpPrimitiveType::Bool),
                CorePathSegment::Text("world".to_string()),
                CorePathSegment::Argument("arg2".to_string(), ValidHttpPrimitiveType::Bool),
            ],
        ),
        NginxTargetPath {
            frontend_path: "/a/b/hello/(.*)/world/(.*)".to_string(),
            downstream_path: "/hello/$1/world/$2".to_string(),
        }
    )
}

#[test]
fn test_projecting_nginx_paths_2() {
    assert_eq!(
        mk_nginx_path(
            &[],
            &[
                CorePathSegment::Text("hello".to_string()),
                CorePathSegment::Argument("arg1".to_string(), ValidHttpPrimitiveType::Bool),
                CorePathSegment::Text("world".to_string()),
                CorePathSegment::Argument("arg2".to_string(), ValidHttpPrimitiveType::Bool),
            ],
        ),
        NginxTargetPath {
            frontend_path: "/hello/(.*)/world/(.*)".to_string(),
            downstream_path: "/hello/$1/world/$2".to_string(),
        }
    )
}
