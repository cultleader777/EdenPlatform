use std::collections::{BTreeSet, HashSet};

use convert_case::Casing;

use crate::{database::Database, static_analysis::{L1Projections, server_runtime::{ServerRuntime, IntegrationTest}}, codegen::l1_provisioning::dns::reverse_ip};

pub fn dns_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    all_servers_resolve(db, l1proj, runtime);
    ns_servers_resolve(db, l1proj, runtime);
    ns_records_exist_for_region(db, l1proj, runtime);
    internal_ptr_records_exist_for_all_servers(db, l1proj, runtime);
    root_dns_servers_resolve_ip(db, l1proj, runtime);
    internal_dns_sec_works(db, l1proj, runtime);
    external_dns_sec_works(db, l1proj, runtime);
    public_dns_works_for_lb(db, l1proj, runtime);
    public_dns_ptr_works_for_lb(db, l1proj, runtime);
    root_ns_records_resolve(db, l1proj, runtime);
    root_ns_records_resolve_to_ips(db, l1proj, runtime);
}

fn external_dns_sec_works(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut domain_ingress_set: BTreeSet<String> = BTreeSet::new();
    for f_ing in db.frontend_application_deployment_ingress().rows_iter() {
        let sub = db.frontend_application_deployment_ingress().c_subdomain(f_ing);
        let tld = db.frontend_application_deployment_ingress().c_tld(f_ing);
        let tld = db.tld().c_domain(tld);
        let _ = domain_ingress_set.insert(format!("{sub}.{tld}"));
    }
    for b_ing in db.backend_application_deployment_ingress().rows_iter() {
        let sub = db.backend_application_deployment_ingress().c_subdomain(b_ing);
        let tld = db.backend_application_deployment_ingress().c_tld(b_ing);
        let tld = db.tld().c_domain(tld);
        let _ = domain_ingress_set.insert(format!("{sub}.{tld}"));
    }

    let mut master_dns_servers: Vec<String> = Vec::new();
    let mut slave_dns_servers: Vec<String> = Vec::new();
    if let Some(ms) = &l1proj.dns_checks.all_regions.master_server {
        if let Some(iface) = l1proj.internet_network_iface.get(ms) {
            let ip = db.network_interface().c_if_ip(*iface);
            master_dns_servers.push(ip.clone());
        }
    }
    for ss in &l1proj.dns_checks.all_regions.slave_servers {
        if let Some(iface) = l1proj.internet_network_iface.get(ss) {
            let ip = db.network_interface().c_if_ip(*iface);
            slave_dns_servers.push(ip.clone());
        }
    }

    let domains: Vec<_> = domain_ingress_set.into_iter().collect();

    runtime.add_integration_test(
        format!("dns_public_dnssec_works_from_master"),
        IntegrationTest::DnsSecWorksExternal {
            target_servers: master_dns_servers.clone(),
            dns_to_lookup: domains.clone(),
        }
    );

    runtime.add_integration_test(
        format!("dns_public_dnssec_works_from_slave"),
        IntegrationTest::DnsSecWorksExternal {
            target_servers: slave_dns_servers.clone(),
            dns_to_lookup: domains.clone(),
        }
    );
}

fn root_ns_records_resolve_to_ips(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let root_dns_servers = l1proj.dns_checks.all_regions.all_servers();
    let mut master_dns_servers: Vec<String> = Vec::new();
    let mut slave_dns_servers: Vec<String> = Vec::new();
    if let Some(ms) = &l1proj.dns_checks.all_regions.master_server {
        if let Some(iface) = l1proj.internet_network_iface.get(ms) {
            let ip = db.network_interface().c_if_ip(*iface);
            master_dns_servers.push(format!("{ip}:53"));
        }
    }
    for ss in &l1proj.dns_checks.all_regions.slave_servers {
        if let Some(iface) = l1proj.internet_network_iface.get(ss) {
            let ip = db.network_interface().c_if_ip(*iface);
            slave_dns_servers.push(format!("{ip}:53"));
        }
    }

    let mut cnt = 0;
    for tld in db.tld().rows_iter() {
        let mut queries: Vec<(String, Vec<String>)> = Vec::new();
        let tld = db.tld().c_domain(tld);
        let tld_name_snake = tld.to_case(convert_case::Case::Snake).replace(".", "_");
        for srv in &root_dns_servers {
            cnt += 1;
            if let Some(iface) = l1proj.internet_network_iface.get(srv) {
                let ip = db.network_interface().c_if_ip(*iface);
                let ns_record = format!("ns{cnt}.{tld}");
                queries.push((ns_record, vec![ip.clone()]));
            }
        }

        runtime.add_integration_test(
            format!("dns_ns_records_resolve_to_ips_in_master_servers_{tld_name_snake}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: master_dns_servers.clone(),
                queries: queries.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_ns_records_resolve_to_ips_in_slave_servers_{tld_name_snake}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: slave_dns_servers.clone(),
                queries: queries.clone(),
            }
        );
    }
}

fn root_ns_records_resolve(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let all_dns_servers = l1proj.dns_checks.all_regions.all_servers();
    let mut master_dns_servers: Vec<String> = Vec::new();
    let mut slave_dns_servers: Vec<String> = Vec::new();
    if let Some(ms) = &l1proj.dns_checks.all_regions.master_server {
        if let Some(iface) = l1proj.internet_network_iface.get(ms) {
            let ip = db.network_interface().c_if_ip(*iface);
            master_dns_servers.push(format!("{ip}:53"));
        }
    }
    for ss in &l1proj.dns_checks.all_regions.slave_servers {
        if let Some(iface) = l1proj.internet_network_iface.get(ss) {
            let ip = db.network_interface().c_if_ip(*iface);
            slave_dns_servers.push(format!("{ip}:53"));
        }
    }

    for tld in db.tld().rows_iter() {
        let mut ns_records: Vec<(String, Vec<String>)> = Vec::new();
        let tld_name = db.tld().c_domain(tld);
        let tld_name_snake = tld_name.to_case(convert_case::Case::Snake).replace(".", "_");
        let query_domain = format!("{tld_name}");
        let mut expected_ns: Vec<String> = Vec::new();

        let mut ns_count = 0;
        for _ in &all_dns_servers {
            ns_count += 1;
            expected_ns.push(format!("ns{ns_count}.{tld_name}."));
        }

        ns_records.push((query_domain, expected_ns));

        runtime.add_integration_test(
            format!("dns_ns_records_exist_for_root_in_master_servers_{tld_name_snake}"),
            IntegrationTest::DnsResolutionWorksNsRecords {
                target_servers: master_dns_servers.clone(),
                queries: ns_records.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_ns_records_exist_for_root_in_slave_servers_{tld_name_snake}"),
            IntegrationTest::DnsResolutionWorksNsRecords {
                target_servers: slave_dns_servers.clone(),
                queries: ns_records.clone(),
            }
        );
    }
}

fn public_dns_ptr_works_for_lb(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    // TODO: if multiple ptr figure out which is most important
    // have priority?
    let mut domain_ingress_set: BTreeSet<String> = BTreeSet::new();
    for f_ing in db.frontend_application_deployment_ingress().rows_iter() {
        let sub = db.frontend_application_deployment_ingress().c_subdomain(f_ing);
        let tld = db.frontend_application_deployment_ingress().c_tld(f_ing);
        let tld = db.tld().c_domain(tld);
        let _ = domain_ingress_set.insert(format!("{sub}.{tld}"));
    }
    for b_ing in db.backend_application_deployment_ingress().rows_iter() {
        let sub = db.backend_application_deployment_ingress().c_subdomain(b_ing);
        let tld = db.backend_application_deployment_ingress().c_tld(b_ing);
        let tld = db.tld().c_domain(tld);
        let _ = domain_ingress_set.insert(format!("{sub}.{tld}"));
    }

    let mut all_master_dns_servers: Vec<String> = Vec::new();
    let mut all_slave_dns_servers: Vec<String> = Vec::new();
    if let Some(master_server) = &l1proj.dns_checks.all_regions.master_server {
        let iface = l1proj.internet_network_iface.get(master_server).unwrap();
        let ip = db.network_interface().c_if_ip(*iface);
        all_master_dns_servers.push(format!("{ip}:53"));
    }

    for slave_server in &l1proj.dns_checks.all_regions.slave_servers {
        let iface = l1proj.internet_network_iface.get(slave_server).unwrap();
        let ip = db.network_interface().c_if_ip(*iface);
        all_slave_dns_servers.push(format!("{ip}:53"));
    }

    let mut tagged_ips: HashSet<String> = HashSet::new();
    let mut ingress_servers: Vec<String> = Vec::new();
    let mut queries: Vec<(String, Vec<String>)> = Vec::new();
    if let Some(first_domain) = domain_ingress_set.iter().next() {
        for server in db.server().rows_iter() {
            if db.server().c_is_ingress(server) {
                let net_if = l1proj.internet_network_iface.get(&server).unwrap();
                let internet_ip = db.network_interface().c_if_ip(*net_if);
                let rev_ip = reverse_ip(internet_ip);
                let in_addr = format!("{rev_ip}.in-addr.arpa.");
                if tagged_ips.insert(in_addr.clone()) {
                    queries.push((in_addr.clone(), vec![format!("{first_domain}.")]));
                }
                ingress_servers.push(internet_ip.clone());
            }
        }
    }

    runtime.add_integration_test(
        format!("dns_public_ingress_records_have_rev_in_master"),
        IntegrationTest::DnsResolutionWorksPtrRecords {
            target_servers: all_master_dns_servers,
            queries: queries.clone(),
        }
    );
    runtime.add_integration_test(
        format!("dns_public_ingress_records_have_rev_in_slave"),
        IntegrationTest::DnsResolutionWorksPtrRecords {
            target_servers: all_slave_dns_servers,
            queries: queries.clone(),
        }
    );
}

fn public_dns_works_for_lb(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut domain_ingress_set: BTreeSet<String> = BTreeSet::new();
    for f_ing in db.frontend_application_deployment_ingress().rows_iter() {
        let sub = db.frontend_application_deployment_ingress().c_subdomain(f_ing);
        let tld = db.frontend_application_deployment_ingress().c_tld(f_ing);
        let tld = db.tld().c_domain(tld);
        let _ = domain_ingress_set.insert(format!("{sub}.{tld}"));
    }
    for b_ing in db.backend_application_deployment_ingress().rows_iter() {
        let sub = db.backend_application_deployment_ingress().c_subdomain(b_ing);
        let tld = db.backend_application_deployment_ingress().c_tld(b_ing);
        let tld = db.tld().c_domain(tld);
        let _ = domain_ingress_set.insert(format!("{sub}.{tld}"));
    }

    let mut all_master_dns_servers: Vec<String> = Vec::new();
    let mut all_slave_dns_servers: Vec<String> = Vec::new();
    if let Some(master_server) = &l1proj.dns_checks.all_regions.master_server {
        let iface = l1proj.internet_network_iface.get(master_server).unwrap();
        let ip = db.network_interface().c_if_ip(*iface);
        all_master_dns_servers.push(format!("{ip}:53"));
    }

    for slave_server in &l1proj.dns_checks.all_regions.slave_servers {
        let iface = l1proj.internet_network_iface.get(slave_server).unwrap();
        let ip = db.network_interface().c_if_ip(*iface);
        all_slave_dns_servers.push(format!("{ip}:53"));
    }

    let mut ingress_servers: Vec<String> = Vec::new();
    for server in db.server().rows_iter() {
        if db.server().c_is_ingress(server) {
            let net_if = l1proj.internet_network_iface.get(&server).unwrap();
            let internet_ip = db.network_interface().c_if_ip(*net_if);
            ingress_servers.push(internet_ip.clone());
        }
    }

    let mut queries: Vec<(String, Vec<String>)> = Vec::new();
    for dom in &domain_ingress_set {
        queries.push((dom.clone(), ingress_servers.clone()));
    }

    runtime.add_integration_test(
        format!("dns_public_ingress_records_resolve_from_master"),
        IntegrationTest::DnsResolutionWorksARecords {
            target_servers: all_master_dns_servers,
            queries: queries.clone(),
        }
    );
    runtime.add_integration_test(
        format!("dns_public_ingress_records_resolve_from_slave"),
        IntegrationTest::DnsResolutionWorksARecords {
            target_servers: all_slave_dns_servers,
            queries: queries.clone(),
        }
    );
}

fn internal_ptr_records_exist_for_all_servers(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    for r in db.region().rows_iter() {
        let tld = db.region().c_tld(r);
        let tld_name = db.tld().c_domain(tld);
        let reg_name = db.region().c_region_name(r);
        let uc_region = reg_name.to_case(convert_case::Case::Snake);
        let mut all_servers_with_dns: Vec<(String, Vec<String>)> = Vec::new();
        let mut master_servers: Vec<String> = Vec::new();
        let mut slave_servers: Vec<String> = Vec::new();
        let mut other_servers: Vec<String> = Vec::new();
        for dc in db.region().c_referrers_datacenter__region(r) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                let lan_iface = l1proj.consul_network_iface.value(*server);
                let lan_ip = db.network_interface().c_if_ip(*lan_iface);
                let reversed = reverse_ip(&lan_ip);
                let arp_addr = format!("{reversed}.in-addr.arpa.");
                let fqdn = l1proj.server_fqdns.value(*server);
                if l1proj.dns_checks.regions.get(&r).is_none() {
                    continue;
                }
                let dns_srvs = l1proj.dns_checks.regions.get(&r).unwrap();
                let ns_no = dns_srvs.ns_number(*server);
                let rev_addr =
                    if ns_no >= 0 {
                        format!("ns{ns_no}.{reg_name}.{tld_name}.")
                    } else {
                        format!("{fqdn}.")
                    };
                all_servers_with_dns.push((arp_addr, vec![rev_addr]));

                let this_dns_srv = format!("{lan_ip}:53");
                if db.server().c_is_dns_master(*server) {
                    master_servers.push(this_dns_srv);
                } else if db.server().c_is_dns_slave(*server) {
                    slave_servers.push(this_dns_srv);
                } else {
                    other_servers.push(this_dns_srv);
                }
            }
        }

        runtime.add_integration_test(
            format!("dns_ptr_records_resolve_from_master_{uc_region}"),
            IntegrationTest::DnsResolutionWorksPtrRecords {
                target_servers: master_servers,
                queries: all_servers_with_dns.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_ptr_records_resolve_from_slave_{uc_region}"),
            IntegrationTest::DnsResolutionWorksPtrRecords {
                target_servers: slave_servers,
                queries: all_servers_with_dns.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_ptr_records_resolve_from_others_{uc_region}"),
            IntegrationTest::DnsResolutionWorksPtrRecords {
                target_servers: other_servers,
                queries: all_servers_with_dns.clone(),
            }
        );
    }
}

fn ns_records_exist_for_region(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut master_dns_servers: Vec<String> = Vec::new();
    let master_servers = l1proj.dns_checks.all_regions.all_servers();
    for ms in &master_servers {
        let iface = l1proj.consul_network_iface.value(*ms);
        let ip = db.network_interface().c_if_ip(*iface);
        master_dns_servers.push(format!("{ip}:53"));
    }

    for tld in db.tld().rows_iter() {
        let mut ns_records: Vec<(String, Vec<String>)> = Vec::new();
        let tld_name = db.tld().c_domain(tld);
        let tld_name_snake = tld_name.to_case(convert_case::Case::Snake).replace(".", "_");
        for region in db.tld().c_referrers_region__tld(tld) {
            let region_name = db.region().c_region_name(*region);
            let query_domain = format!("{region_name}.{tld_name}");
            let mut expected_ns: Vec<String> = Vec::new();

            let mut ns_count = 0;
            for _ in &master_dns_servers {
                ns_count += 1;
                expected_ns.push(format!("ns{ns_count}.{region_name}.{tld_name}."));
            }

            ns_records.push((query_domain, expected_ns));
        }

        runtime.add_integration_test(
            format!("dns_ns_records_exist_for_regions_{tld_name_snake}"),
            IntegrationTest::DnsResolutionWorksNsRecords {
                target_servers: master_dns_servers.clone(),
                queries: ns_records.clone(),
            }
        );
    }
}

fn ns_servers_resolve(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    for r in db.region().rows_iter() {
        let rname = db.region().c_region_name(r);
        let tld = db.region().c_tld(r);
        let domain = db.tld().c_domain(tld);
        let uc_region = db.region().c_region_name(r).to_case(convert_case::Case::Snake);
        let mut all_servers_with_dns: Vec<(String, Vec<String>)> = Vec::new();
        let mut all_dns_servers: Vec<String> = Vec::new();
        if let Some(dchecks) = l1proj.dns_checks.regions.get(&r) {
            let mut ns_count = 0;
            if let Some(m) = &dchecks.master_server {
                ns_count += 1;
                let lan_iface = l1proj.consul_network_iface.value(*m);
                let lan_ip = db.network_interface().c_if_ip(*lan_iface);
                all_servers_with_dns.push((format!("ns{ns_count}.{rname}.{domain}"), vec![lan_ip.clone()]));
            }

            for s in &dchecks.slave_servers {
                ns_count += 1;
                let lan_iface = l1proj.consul_network_iface.value(*s);
                let lan_ip = db.network_interface().c_if_ip(*lan_iface);
                all_servers_with_dns.push((format!("ns{ns_count}.{rname}.{domain}"), vec![lan_ip.clone()]));
            }

            for dc in db.region().c_referrers_datacenter__region(r) {
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    let lan_iface = l1proj.consul_network_iface.value(*server);
                    let lan_ip = db.network_interface().c_if_ip(*lan_iface);

                    let this_dns_srv = format!("{lan_ip}:53");
                    all_dns_servers.push(this_dns_srv);
                }
            }

            runtime.add_integration_test(
                format!("dns_ns_servers_resolve_from_all_{uc_region}"),
                IntegrationTest::DnsResolutionWorksARecords {
                    target_servers: all_dns_servers,
                    queries: all_servers_with_dns.clone(),
                }
            );
        }
    }
}

fn internal_dns_sec_works(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    for tld in db.tld().rows_iter() {
        let tld_name = db.tld().c_domain(tld);
        let tld_snake = tld_name.to_case(convert_case::Case::Snake).replace(".", "_");
        for r in db.tld().c_referrers_region__tld(tld) {
            let uc_region = db.region().c_region_name(*r).to_case(convert_case::Case::Snake);
            let mut master_servers: Vec<String> = Vec::new();
            let mut slave_servers: Vec<String> = Vec::new();
            let mut other_servers: Vec<String> = Vec::new();
            let mut first_server = None;
            for dc in db.region().c_referrers_datacenter__region(*r) {
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    if first_server.is_none() {
                        first_server = Some(server);
                    }
                    let lan_iface = l1proj.consul_network_iface.value(*server);
                    let lan_ip = db.network_interface().c_if_ip(*lan_iface);

                    let this_dns_srv = lan_ip.clone();
                    if db.server().c_is_dns_master(*server) {
                        master_servers.push(this_dns_srv);
                    } else if db.server().c_is_dns_slave(*server) {
                        slave_servers.push(this_dns_srv);
                    } else {
                        other_servers.push(this_dns_srv);
                    }
                }
            }

            if let Some(first_server) = first_server {
                let first_server_lan_iface = l1proj.consul_network_iface.value(*first_server);
                let first_server_lan_ip = db.network_interface().c_if_ip(*first_server_lan_iface);
                let libvirt_gw_ip =
                    format!("{first_server_lan_ip}/24")
                    .parse::<ipnet::Ipv4Net>()
                    .unwrap().hosts().next().unwrap().to_string();
                runtime.add_integration_test(
                    format!("dns_sec_works_from_master_{tld_snake}_{uc_region}"),
                    IntegrationTest::DnsSecWorksInternal {
                        target_servers: master_servers,
                        source_ip: libvirt_gw_ip.clone(),
                        server_to_lookup: db.server().c_hostname(*first_server).clone(),
                        server_to_lookup_ip: first_server_lan_ip.clone(),
                        region: db.region().c_region_name(*r).clone(),
                        tld: tld_name.to_string(),
                    }
                );

                runtime.add_integration_test(
                    format!("dns_sec_works_from_slave_{tld_snake}_{uc_region}"),
                    IntegrationTest::DnsSecWorksInternal {
                        target_servers: slave_servers,
                        source_ip: libvirt_gw_ip.clone(),
                        server_to_lookup: db.server().c_hostname(*first_server).clone(),
                        server_to_lookup_ip: first_server_lan_ip.clone(),
                        region: db.region().c_region_name(*r).clone(),
                        tld: tld_name.to_string(),
                    }
                );

                runtime.add_integration_test(
                    format!("dns_sec_works_from_others_{tld_snake}_{uc_region}"),
                    IntegrationTest::DnsSecWorksInternal {
                        target_servers: other_servers,
                        source_ip: libvirt_gw_ip.clone(),
                        server_to_lookup: db.server().c_hostname(*first_server).clone(),
                        server_to_lookup_ip: first_server_lan_ip.clone(),
                        region: db.region().c_region_name(*r).clone(),
                        tld: tld_name.to_string(),
                    }
                );
            }
        }
    }
}

fn root_dns_servers_resolve_ip(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let master_srvs = l1proj.dns_checks.all_regions.all_servers();
    let mut all_servers_with_dns: Vec<(String, Vec<String>)> = Vec::new();
    let mut ns_count = 0;
    for tld in db.tld().rows_iter() {
        let domain = db.tld().c_domain(tld);
        for m in &master_srvs {
            ns_count += 1;
            let lan_iface = l1proj.consul_network_iface.value(*m);
            let lan_ip = db.network_interface().c_if_ip(*lan_iface);
            all_servers_with_dns.push((format!("ns{ns_count}.{domain}"), vec![lan_ip.clone()]))
        }
    }

    for r in db.region().rows_iter() {
        let uc_region = db.region().c_region_name(r).to_case(convert_case::Case::Snake);
        let mut master_servers: Vec<String> = Vec::new();
        let mut slave_servers: Vec<String> = Vec::new();
        let mut other_servers: Vec<String> = Vec::new();
        for dc in db.region().c_referrers_datacenter__region(r) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                let lan_iface = l1proj.consul_network_iface.value(*server);
                let lan_ip = db.network_interface().c_if_ip(*lan_iface);

                let this_dns_srv = format!("{lan_ip}:53");
                if db.server().c_is_dns_master(*server) {
                    master_servers.push(this_dns_srv);
                } else if db.server().c_is_dns_slave(*server) {
                    slave_servers.push(this_dns_srv);
                } else {
                    other_servers.push(this_dns_srv);
                }
            }
        }

        runtime.add_integration_test(
            format!("dns_root_servers_resolve_from_master_{uc_region}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: master_servers,
                queries: all_servers_with_dns.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_root_servers_resolve_from_slave_{uc_region}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: slave_servers,
                queries: all_servers_with_dns.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_root_servers_resolve_from_others_{uc_region}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: other_servers,
                queries: all_servers_with_dns.clone(),
            }
        );
    }
}

fn all_servers_resolve(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    for r in db.region().rows_iter() {
        let uc_region = db.region().c_region_name(r).to_case(convert_case::Case::Snake);
        let mut all_servers_with_dns: Vec<(String, Vec<String>)> = Vec::new();
        let mut master_servers: Vec<String> = Vec::new();
        let mut slave_servers: Vec<String> = Vec::new();
        let mut other_servers: Vec<String> = Vec::new();
        for dc in db.region().c_referrers_datacenter__region(r) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                let lan_iface = l1proj.consul_network_iface.value(*server);
                let lan_ip = db.network_interface().c_if_ip(*lan_iface);
                let fqdn = l1proj.server_fqdns.value(*server);
                all_servers_with_dns.push((fqdn.clone(), vec![lan_ip.clone()]));

                let this_dns_srv = format!("{lan_ip}:53");
                if db.server().c_is_dns_master(*server) {
                    master_servers.push(this_dns_srv);
                } else if db.server().c_is_dns_slave(*server) {
                    slave_servers.push(this_dns_srv);
                } else {
                    other_servers.push(this_dns_srv);
                }
            }
        }

        runtime.add_integration_test(
            format!("dns_all_servers_resolve_from_master_{uc_region}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: master_servers,
                queries: all_servers_with_dns.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_all_servers_resolve_from_slave_{uc_region}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: slave_servers,
                queries: all_servers_with_dns.clone(),
            }
        );

        runtime.add_integration_test(
            format!("dns_all_servers_resolve_from_others_{uc_region}"),
            IntegrationTest::DnsResolutionWorksARecords {
                target_servers: other_servers,
                queries: all_servers_with_dns.clone(),
            }
        );
    }
}
