use std::collections::{BTreeSet, BTreeMap, HashMap};

use crate::{database::{TableRowPointerServer, Database, TableRowPointerRegion}, static_analysis::{networking::check_servers_regional_distribution, get_global_settings}};

use super::super::{PlatformValidationError, projections::Projection, CheckedDB};

pub struct DnsChecksSingleInstance {
    pub master_server: Option<TableRowPointerServer>,
    pub slave_servers: BTreeSet<TableRowPointerServer>,
}

pub struct DnsChecks {
    pub regions: BTreeMap<TableRowPointerRegion, DnsChecksSingleInstance>,
    pub all_regions: DnsChecksSingleInstance,
    pub certs_needed: bool,
}

impl DnsChecksSingleInstance {
    pub fn total_dns_servers(&self) -> usize {
        let mut res = 0;
        if self.master_server.is_some() {
            res += 1;
        }
        res += self.slave_servers.len();
        res
    }

    pub fn contains(&self, server: TableRowPointerServer) -> bool {
        if let Some(this_server) = &self.master_server {
            if *this_server == server {
                return true;
            }
        }

        for this_server in &self.slave_servers {
            if *this_server == server {
                return true;
            }
        }

        false
    }

    pub fn ns_number(&self, server: TableRowPointerServer) -> i32 {
        let mut counter = 0;
        if let Some(this_server) = &self.master_server {
            counter += 1;
            if *this_server == server {
                return counter;
            }
        }

        for this_server in &self.slave_servers {
            counter += 1;
            if *this_server == server {
                return counter;
            }
        }

        -1
    }

    pub fn all_servers(&self) -> Vec<TableRowPointerServer> {
        let mut res = Vec::new();

        if let Some(this_server) = &self.master_server {
            res.push(*this_server);
        }

        for this_server in &self.slave_servers {
            res.push(*this_server);
        }

        res
    }

    pub fn master_lan_ip(&self, db: &CheckedDB) -> String {
        if let Some(ms) = &self.master_server {
            let iface = db.projections.consul_network_iface.value(*ms);
            return db.db.network_interface().c_if_ip(*iface).clone();
        }
        panic!("Should never happen here, all ips should have been settled")
    }

    pub fn master_internet_ip(&self, db: &CheckedDB) -> String {
        if let Some(ms) = &self.master_server {
            let iface = db.projections.internet_network_iface.get(ms).unwrap();
            return db.db.network_interface().c_if_ip(*iface).clone();
        }
        panic!("Should never happen here, all ips should have been settled")
    }

    pub fn slaves_lan_ips(&self, db: &CheckedDB) -> Vec<String> {
        let mut res = Vec::with_capacity(2);
        for slave in &self.slave_servers {
            let iface = db.projections.consul_network_iface.value(*slave);
            res.push(db.db.network_interface().c_if_ip(*iface).clone());
        }
        res
    }

    pub fn slaves_internet_ips(&self, db: &CheckedDB) -> Vec<String> {
        let mut res = Vec::with_capacity(2);
        for slave in &self.slave_servers {
            let iface = db.projections.internet_network_iface.get(slave).unwrap();
            res.push(db.db.network_interface().c_if_ip(*iface).clone());
        }
        res
    }
}

pub fn dns_checks(db: &Database, server_fqdns: &Projection<TableRowPointerServer, String>) -> Result<DnsChecks, PlatformValidationError> {
    let mut all_ingresses_subdomains: HashMap<String, &'static str> = HashMap::new();
    let res = dns_checks_for_servers(db, &mut all_ingresses_subdomains, server_fqdns)?;

    dns_checks_for_misc_records(db, &mut all_ingresses_subdomains)?;

    Ok(res)
}



fn dns_checks_for_misc_records(db: &Database, all_ingresses_subdomains: &mut HashMap<String, &'static str>) -> Result<(), PlatformValidationError> {
    lazy_static! {
        pub static ref TXT_RECORD_REGEX: regex::Regex = regex::Regex::new(r#"^[a-zA-Z0-9_]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9_])?$"#).unwrap();
        pub static ref NORMAL_RECORD_REGEX: regex::Regex = regex::Regex::new(r#"^[a-zA-Z0-9]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9_]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?)*$"#).unwrap();
        pub static ref VALID_TXT_VALUE: regex::Regex = regex::Regex::new(r#"^([ -~]{1,253})$"#).unwrap();
        pub static ref VALID_FQDN_VALUE: regex::Regex = regex::Regex::new(r#"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*\.?$"#).unwrap();
    }

    let gs = get_global_settings(db);

    let max_values = 30;
    for tld in db.tld().rows_iter() {
        let tld_value = db.tld().c_domain(tld);
        for txt_rec in db.tld().c_children_tld_txt_record(tld) {
            let subdomain = db.tld_txt_record().c_subdomain(*txt_rec);
            if subdomain.contains(tld_value) {
                return Err(PlatformValidationError::DnsMiscSubdomainFullTldIsRedundant {
                    record_type: "TXT".to_string(),
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    explanation: "Your subdomain should never include the full tld domain, it is already assumed".to_string(),
                });
            }

            if subdomain == "_acme-challenge" {
                return Err(PlatformValidationError::DnsForbiddenMiscTxtRecord {
                    record_type: "TXT".to_string(),
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    reserved_values: vec![
                        "_acme-challenge".to_string(),
                    ]
                });
            }

            if subdomain != "@" {
                if !TXT_RECORD_REGEX.is_match(&subdomain) {
                    return Err(PlatformValidationError::DnsInvalidMiscTxtRecordSubdomain {
                        failed_regex_check: TXT_RECORD_REGEX.to_string(),
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                    });
                }
            }

            let values = db.tld_txt_record().c_children_tld_txt_record_value(*txt_rec).len();
            if values == 0 {
                return Err(PlatformValidationError::DnsMiscTxtRecordNoValues {
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    values_found: values,
                    values_min: 1,
                });
            }

            if values > max_values {
                return Err(PlatformValidationError::DnsMiscTxtRecordTooManyValues {
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    values_found: values,
                    values_max: max_values,
                });
            }

            for txt_value in db.tld_txt_record().c_children_tld_txt_record_value(*txt_rec) {
                let the_value = db.tld_txt_record_value().c_value(*txt_value);
                if !VALID_TXT_VALUE.is_match(&the_value) {
                    return Err(PlatformValidationError::DnsMiscTxtRecordInvalidValue {
                        failed_regex_check: VALID_TXT_VALUE.to_string(),
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                        value: db.tld_txt_record_value().c_value(*txt_value).clone(),
                    });
                }
            }
        }

        for cname_rec in db.tld().c_children_tld_cname_record(tld) {
            // we don't use any cnames so anything added is misc
            let subdomain = db.tld_cname_record().c_subdomain(*cname_rec);
            if subdomain.contains(tld_value) {
                return Err(PlatformValidationError::DnsMiscSubdomainFullTldIsRedundant {
                    record_type: "CNAME".to_string(),
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    explanation: "Your subdomain should never include the full tld domain, it is already assumed".to_string(),
                });
            }

            if subdomain == "@" {
                // having @ CNAME would clash with other records we set for the zone,
                // potentially clashing with functionality of ingresses and so on
                // better just forbid it now until we can figure out use case where it is
                // actually needed
                return Err(PlatformValidationError::DnsForbiddenMiscCnameRecordSubdomain {
                    forbidden_value: "@".to_string(),
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                });
            } else {
                if !NORMAL_RECORD_REGEX.is_match(&subdomain) {
                    return Err(PlatformValidationError::DnsInvalidMiscCnameRecordSubdomain {
                        failed_regex_check: NORMAL_RECORD_REGEX.to_string(),
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                    });
                }

                if gs.admin_tld == tld {
                    if subdomain.starts_with("adm-") {
                        // if someone ever needs this we can do abstraction where we detect
                        // all ingress reservations, now we just know that epl
                        // admin services start with adm-
                        // besides, this check is only on admin_tld domain
                        return Err(PlatformValidationError::DnsMiscCnameRecordReservedPrefix {
                            tld: db.tld().c_domain(tld).clone(),
                            subdomain: subdomain.clone(),
                            forbidden_prefix: "adm-".to_string(),
                        });
                    }
                }

                let fqdn = format!("{}.{}.", subdomain, db.tld().c_domain(tld));
                if let Some(existing_value) = all_ingresses_subdomains.insert(fqdn, "tld_cname_record") {
                    return Err(PlatformValidationError::DnsMiscCnameRecordClashSubdomain {
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                        previous_source: existing_value.to_string(),
                    });
                }
            }

            let values = db.tld_cname_record().c_children_tld_cname_record_value(*cname_rec).len();
            if values == 0 {
                return Err(PlatformValidationError::DnsMiscCnameRecordNoValues {
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    values_found: values,
                    values_min: 1,
                });
            }

            if values > max_values {
                return Err(PlatformValidationError::DnsMiscCnameRecordTooManyValues {
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    values_found: values,
                    values_max: max_values,
                });
            }

            for cname_value in db.tld_cname_record().c_children_tld_cname_record_value(*cname_rec) {
                let the_value = db.tld_cname_record_value().c_value(*cname_value);
                if !VALID_FQDN_VALUE.is_match(&the_value) {
                    return Err(PlatformValidationError::DnsMiscCnameRecordInvalidValue {
                        failed_regex_check: VALID_FQDN_VALUE.to_string(),
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                        value: db.tld_cname_record_value().c_value(*cname_value).clone(),
                    });
                }

                // the idea is if we have domain name like u12341.uwueqw.sendgrid.net.
                // we have suspicion that this is fqdn but is not ended with a period
                // so bind9 will implicitly add our top level zone at the end
                let period_count = the_value.match_indices(".").count();
                if period_count >= 2 && !the_value.ends_with(".") {
                    return Err(PlatformValidationError::DnsMiscCnameRecordWithAtLeastTwoPeriodsMustBeFqdn {
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                        value: the_value.clone(),
                        periods_found: period_count,
                        value_expected_at_the_end: ".".to_string(),
                    });
                }
            }
        }

        for mx_rec in db.tld().c_children_tld_mx_record(tld) {
            // we don't use any cnames so anything added is misc
            let subdomain = db.tld_mx_record().c_subdomain(*mx_rec);
            if subdomain.contains(tld_value) {
                return Err(PlatformValidationError::DnsMiscSubdomainFullTldIsRedundant {
                    record_type: "MX".to_string(),
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    explanation: "Your subdomain should never include the full tld domain, it is already assumed".to_string(),
                });
            }

            if subdomain != "@" {
                if !NORMAL_RECORD_REGEX.is_match(&subdomain) {
                    return Err(PlatformValidationError::DnsInvalidMiscTxtRecordSubdomain {
                        failed_regex_check: NORMAL_RECORD_REGEX.to_string(),
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                    });
                }
            }

            let values = db.tld_mx_record().c_children_tld_mx_record_value(*mx_rec).len();
            if values == 0 {
                return Err(PlatformValidationError::DnsMiscMxRecordNoValues {
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    values_found: values,
                    values_min: 1,
                });
            }

            if values > max_values {
                return Err(PlatformValidationError::DnsMiscMxRecordTooManyValues {
                    tld: db.tld().c_domain(tld).clone(),
                    subdomain: subdomain.clone(),
                    values_found: values,
                    values_max: max_values,
                });
            }

            for mx_value in db.tld_mx_record().c_children_tld_mx_record_value(*mx_rec) {
                let the_value = db.tld_mx_record_value().c_value(*mx_value);
                if !VALID_FQDN_VALUE.is_match(&the_value) {
                    return Err(PlatformValidationError::DnsMiscMxRecordInvalidValue {
                        failed_regex_check: VALID_FQDN_VALUE.to_string(),
                        priority: db.tld_mx_record_value().c_priority(*mx_value),
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                        value: db.tld_mx_record_value().c_value(*mx_value).clone(),
                    });
                }

                // the idea is if we have domain name like u12341.uwueqw.sendgrid.net.
                // we have suspicion that this is fqdn but is not ended with a period
                // so bind9 will implicitly add our top level zone at the end, which might be unexpected
                let period_count = the_value.match_indices(".").count();
                if period_count >= 2 && !the_value.ends_with(".") {
                    return Err(PlatformValidationError::DnsMiscMxRecordWithAtLeastTwoPeriodsMustBeFqdn {
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.clone(),
                        value: the_value.clone(),
                        periods_found: period_count,
                        value_expected_at_the_end: ".".to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}


fn dns_checks_for_servers(
    db: &Database,
    all_ingresses_subdomains: &mut HashMap<String, &'static str>,
    server_fqdns: &Projection<TableRowPointerServer, String>
) -> Result<DnsChecks, PlatformValidationError> {
    let mut res = DnsChecks {
        regions: BTreeMap::new(),
        all_regions: DnsChecksSingleInstance {
            master_server: None,
            slave_servers: BTreeSet::new(),
        },
        certs_needed: false,
    };

    let settings = get_global_settings(db);
    if settings.disable_dns_quorum_tests {
        return Ok(res);
    }

    let is_single_region_mode = db.region().len() == 1;
    let is_multi_region_mode = !is_single_region_mode;

    for tld in db.tld().rows_iter() {
        if db.tld().c_automatic_certificates(tld) {
            res.certs_needed = true;
            break;
        }
    }

    let mut slave_regions = Vec::new();

    // frontend and backend ingresses can clash because EPL
    // should ensure they're working together in the same subdomain
    for i in db.frontend_application_deployment_ingress().rows_iter() {
        let tld = db.tld().c_domain(db.frontend_application_deployment_ingress().c_tld(i));
        let subdomain = db.frontend_application_deployment_ingress().c_subdomain(i);
        let fqdn = format!("{}.{}.", subdomain, tld);
        let _ = all_ingresses_subdomains.insert(fqdn.clone(), "frontend_application_deployment_ingress");
    }
    for i in db.backend_application_deployment_ingress().rows_iter() {
        let tld = db.tld().c_domain(db.backend_application_deployment_ingress().c_tld(i));
        let subdomain = db.backend_application_deployment_ingress().c_subdomain(i);
        let fqdn = format!("{}.{}.", subdomain, tld);
        let _ = all_ingresses_subdomains.insert(fqdn.clone(), "backend_application_deployment_ingress");
    }

    let admin_fqdn = format!("admin.{}.", db.tld().c_domain(settings.admin_tld));
    if let Some(clash_source_table) = all_ingresses_subdomains.insert(admin_fqdn.clone(), "admin_domain") {
        return Err(PlatformValidationError::DnsRegionNameAndIngressSubdomainFqdnClash {
            fqdn: admin_fqdn,
            clash_source_table,
        });
    }


    for region in db.region().rows_iter() {
        let tld = db.tld().c_domain(settings.admin_tld);
        let fqdn = format!("{}.{}.", db.region().c_region_name(region), tld);
        if let Some(clash_source_table) = all_ingresses_subdomains.insert(fqdn.clone(), "region_tld_name") {
            return Err(PlatformValidationError::DnsRegionNameAndIngressSubdomainFqdnClash {
                fqdn,
                clash_source_table,
            });
        }

        let mut master_server = None;
        let mut slave_servers = BTreeSet::new();
        let is_master_region = db.region().c_is_dns_master(region);
        let is_slave_region = db.region().c_is_dns_slave(region);

        let mut region_dns_servers = Vec::new();

        if db.region().c_is_dns_slave(region) {
            slave_regions.push(db.region().c_region_name(region).clone());
        }

        for dc in db.region().c_referrers_datacenter__region(region) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                let found_interfaces =
                    db.server()
                    .c_children_network_interface(*server)
                    .iter()
                    .map(|i| {
                        db.network().c_network_name(db.network_interface().c_if_network(*i)).clone()
                    })
                    .collect::<Vec<_>>();

                let has_if = |interface: &str| {
                    found_interfaces.iter().find(|i| *i == interface).is_some()
                };

                if db.server().c_is_dns_master(*server) {
                    region_dns_servers.push(*server);

                    if !has_if("internet") {
                        return Err(PlatformValidationError::DnsMasterMustHaveInternetNetworkInterface {
                            server: server_fqdns.value(*server).clone(),
                            found_interfaces,
                        });
                    }

                    if let Some(master_server) = &master_server {
                        return Err(PlatformValidationError::DnsHasMoreThanOneMaster {
                            first_master_server: server_fqdns.value(*master_server).clone(),
                            second_master_server: server_fqdns.value(*server).clone(),
                        });
                    }

                    master_server = Some(*server);

                    if is_master_region {
                        if let Some(prev_master_server) = &res.all_regions.master_server {
                            // should have been caught earlier
                            let prev_master_dc = db.server().c_dc(*prev_master_server);
                            let prev_master_region = db.datacenter().c_region(prev_master_dc);

                            return Err(PlatformValidationError::DnsMoreThanOneMasterRegion {
                                first_master_region: db.region().c_region_name(prev_master_region).clone(),
                                first_master_server: server_fqdns.value(*prev_master_server).clone(),
                                second_master_region: db.region().c_region_name(region).clone(),
                                second_master_server: server_fqdns.value(*server).clone(),
                            });
                        }

                        res.all_regions.master_server = Some(*server);
                    }

                    if is_slave_region && is_multi_region_mode {
                        // this is master dns server but slave dc, add this server as slave for root zone
                        res.all_regions.slave_servers.insert(*server);

                        if res.all_regions.slave_servers.len() > 2 {
                            return Err(PlatformValidationError::DnsMoreThanTwoSlaveRegions {
                                slave_servers: res.all_regions.slave_servers.iter().map(|i| server_fqdns.value(*i).clone()).collect(),
                                slave_regions: res.all_regions.slave_servers.iter().map(|i| {
                                    let dc = db.server().c_dc(*i);
                                    let region = db.datacenter().c_region(dc);
                                    db.region().c_region_name(region).clone()
                                }).collect(),
                            });
                        }
                    }
                }

                if db.server().c_is_dns_slave(*server) {
                    region_dns_servers.push(*server);

                    if !has_if("internet") {
                        return Err(PlatformValidationError::DnsSlaveMustHaveInternetNetworkInterface {
                            server: server_fqdns.value(*server).clone(),
                            found_interfaces,
                        });
                    }

                    assert!(slave_servers.insert(*server));

                    if slave_servers.len() > 2 {
                        return Err(PlatformValidationError::DnsHasMoreThanTwoSlaves {
                            slave_servers: slave_servers.iter().map(|i| server_fqdns.value(*i).clone()).collect(),
                        });
                    }

                    // in single dc mode masters and slaves are equivalent
                    if is_single_region_mode {
                        assert!(res.all_regions.slave_servers.insert(*server));
                    }
                }
            }
        }

        if master_server.is_none() {
            return Err(PlatformValidationError::DnsNoMasterServerSpecifiedInRegion {
                explanation: "There must be exactly one server specified with `is_dns_master` column set to true inside single region".to_string(),
            });
        }

        if slave_servers.is_empty() {
            return Err(PlatformValidationError::DnsNoSlaveServersSpecifiedInRegion {
                explanation: "There must be from one to two slave dns servers specified with `is_dns_slave` column set to true inside single region".to_string(),
            });
        }

        assert!(res.regions.insert(region, DnsChecksSingleInstance { master_server, slave_servers }).is_none());

        check_servers_regional_distribution(
            db, region, region_dns_servers.into_iter(),
            format!("DNS servers for region {}", db.region().c_region_name(region))
        )?;
    }

    // there must be one master all dc server regardless of mode
    if res.all_regions.master_server.is_none() {
        return Err(PlatformValidationError::DnsNoMasterRegionSpecified {
            explanation: "There must be exactly one region specified with `is_dns_master` column set to true".to_string(),
        });
    }

    // there must be one slave dc server regardless of mode
    if res.all_regions.slave_servers.is_empty() {
        return Err(PlatformValidationError::DnsNoSlaveRegionSpecified {
            explanation: "There must be from one to two regions specified with `is_dns_slave` column set to true".to_string(),
        });
    }

    if is_multi_region_mode && slave_regions.len() > 2 {
        return Err(PlatformValidationError::DnsMoreThanTwoSlaveRegions {
            slave_servers: Vec::new(),
            slave_regions,
        });
    }

    Ok(res)
}

pub fn server_fqdns(db: &Database) -> Projection<TableRowPointerServer, String> {
    let settings = get_global_settings(db);
    Projection::create(db.server().rows_iter(), |server| {
        let dc = db.server().c_dc(server);
        let region = db.datacenter().c_region(dc);
        let tld = settings.admin_tld;
        format!("{}.{}.{}", db.server().c_hostname(server), db.region().c_region_name(region), db.tld().c_domain(tld))
    })
}
