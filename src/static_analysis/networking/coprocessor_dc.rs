use std::collections::{BTreeSet, BTreeMap};

use crate::{database::{TableRowPointerServer, TableRowPointerDatacenter, TableRowPointerRegion}, static_analysis::PlatformValidationError};

pub struct CoprocessorRegionData {
    pub gateways: BTreeSet<TableRowPointerServer>,
    pub coprocessor_dc: TableRowPointerDatacenter,
}

pub fn coprocessor_analysis(
    db: &crate::database::Database,
) -> Result<BTreeMap<TableRowPointerRegion, CoprocessorRegionData>, PlatformValidationError> {
    let mut res = BTreeMap::new();
    for region in db.region().rows_iter() {
        let region_name = db.region().c_region_name(region);
        let region_avail_mode = db.region().c_availability_mode(region);
        let has_coproc_dc = db.region().c_has_coprocessor_dc(region);
        let is_single_dc = db.region().c_availability_mode(region) == "single_dc";
        let is_multi_dc = db.region().c_availability_mode(region) == "multi_dc";
        assert!(is_single_dc ^ is_multi_dc, "dc avail mode should be either single dc or multi dc");

        let mut region_coproc_gws: BTreeSet<TableRowPointerServer> = BTreeSet::new();
        let mut region_coproc_dcs: BTreeSet<TableRowPointerDatacenter> = BTreeSet::new();

        for dc in db.region().c_referrers_datacenter__region(region) {
            let dc_impl = db.datacenter().c_implementation(*dc);
            let is_coproc_dc = dc_impl == "coprocessor";
            if is_coproc_dc {
                assert!(region_coproc_dcs.insert(*dc));
            }

            let dc_name = db.datacenter().c_dc_name(*dc);
            let mut coproc_gws: BTreeSet<TableRowPointerServer> = BTreeSet::new();
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                if db.server().c_is_coprocessor_gateway(*server) {
                    assert!(coproc_gws.insert(*server));
                }
            }

            // single dc: must have two coproc gateways
            // multi dc: must have two coproc gateways across the region

            if has_coproc_dc {
                if is_single_dc {
                    if !is_coproc_dc && coproc_gws.len() != 2 {
                        // error, must be two gateways
                        return Err(PlatformValidationError::CoprocessorDcMustHaveTwoGateways {
                            region: region_name.clone(),
                            region_availability_mode: region_avail_mode.clone(),
                            region_dc: dc_name.clone(),
                            region_expected_coprocessor_gateways: 2,
                            region_found_coprocessor_gateways: coproc_gws.len(),
                            region_found_coprocessor_gateways_servers: coproc_gws.iter().map(|srv| {
                                db.server().c_hostname(*srv).clone()
                            }).collect::<Vec<_>>(),
                            region_has_coprocessor_dc: has_coproc_dc,
                        });
                    }
                } else if is_multi_dc {
                    if coproc_gws.len() > 1 {
                        // error, must not be more than one in DC
                        return Err(PlatformValidationError::CoprocessorCannotHaveMoreThanOneGatewayInDatacenter {
                            region: region_name.clone(),
                            region_availability_mode: region_avail_mode.clone(),
                            region_dc: dc_name.clone(),
                            region_expected_maximum_coprocessor_gateways_per_dc: 1,
                            region_found_coprocessor_gateways: coproc_gws.len(),
                            region_found_coprocessor_gateways_servers: coproc_gws.iter().map(|srv| {
                                db.server().c_hostname(*srv).clone()
                            }).collect::<Vec<_>>(),
                            region_has_coprocessor_dc: has_coproc_dc,
                        });
                    }
                } else { panic!("huh") }
            } else {
                if coproc_gws.len() > 0 {
                    // error, no coprocessor gws should exist as region not enabled
                    return Err(PlatformValidationError::CoprocessorDcIsNotEnabledInRegionButCoprocessorServersExist {
                        region: region_name.clone(),
                        region_availability_mode: region_avail_mode.clone(),
                        region_dc: dc_name.clone(),
                        region_expected_coprocessor_gateways: 0,
                        region_found_coprocessor_gateways: coproc_gws.len(),
                        region_found_coprocessor_gateways_servers: coproc_gws.iter().map(|srv| {
                            db.server().c_hostname(*srv).clone()
                        }).collect::<Vec<_>>(),
                        region_has_coprocessor_dc: has_coproc_dc,
                    });
                }
            }

            if is_coproc_dc {
                if coproc_gws.len() > 0 {
                    return Err(PlatformValidationError::CoprocessorDatacenterMustNotHaveCoprocessorGateways {
                        region: region_name.clone(),
                        datacenter: dc_name.clone(),
                        dc_expected_coprocessor_gateways: 0,
                        dc_found_coprocessor_gateways: coproc_gws.len(),
                        dc_found_coprocessor_gateways_servers: coproc_gws.iter().map(|srv| {
                            db.server().c_hostname(*srv).clone()
                        }).collect::<Vec<_>>(),
                        dc_implementation: dc_impl.clone(),
                    });
                }

                // we don't want coprocessor servers partake in consensus
                // because they depend on the core region network,
                // we don't want to put nomad/consul masters into leaf networks
                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    let forbidden_roles = [
                        ("is_consul_master", db.server().c_is_consul_master(*server)),
                        ("is_nomad_master", db.server().c_is_nomad_master(*server)),
                        ("is_vault_instance", db.server().c_is_vault_instance(*server)),
                        ("is_dns_master", db.server().c_is_dns_master(*server)),
                        ("is_dns_slave", db.server().c_is_dns_slave(*server)),
                        ("is_vpn_gateway", db.server().c_is_vpn_gateway(*server)),
                        ("is_coprocessor_gateway", db.server().c_is_coprocessor_gateway(*server)),
                        ("is_router", db.server().c_is_router(*server)),
                    ];

                    for (role_name, is_part) in forbidden_roles {
                        if is_part {
                            return Err(PlatformValidationError::CoprocessorDatacenterServerHasForbiddenRole {
                                region: region_name.clone(),
                                datacenter: dc_name.clone(),
                                server_hostname: db.server().c_hostname(*server).clone(),
                                forbidden_role: role_name.to_string(),
                            });
                        }
                    }

                    let mut found_vpn_ifaces = Vec::new();
                    for ni in db.server().c_children_network_interface(*server) {
                        let if_name = db.network_interface().c_if_name(*ni);
                        let network = db.network_interface().c_if_network(*ni);
                        let network_name = db.network().c_network_name(network);
                        if network_name == "vpn" {
                            found_vpn_ifaces.push(*ni);
                            match if_name.as_str() {
                                "wg0" | "wg1" => {}
                                other => {
                                    return Err(PlatformValidationError::CoprocessorServerVpnInterfaceNamesMustBeWg0AndWg1 {
                                        region: region_name.clone(),
                                        datacenter: dc_name.clone(),
                                        server_hostname: db.server().c_hostname(*server).clone(),
                                        only_allowed_names: vec![
                                            "wg0".to_string(), "wg1".to_string(),
                                        ],
                                        vpn_interface_name: other.to_string(),
                                    });
                                }
                            }
                        }
                    }

                    if found_vpn_ifaces.len() != 2 {
                        return Err(PlatformValidationError::CoprocessorServerMustHaveTwoVpnInterfaces {
                            region: region_name.clone(),
                            datacenter: dc_name.clone(),
                            server_hostname: db.server().c_hostname(*server).clone(),
                            vpn_interfaces_expected: 2,
                            vpn_interfaces_found: found_vpn_ifaces.len(),
                            vpn_interfaces_found_names: found_vpn_ifaces.iter().map(|i| {
                                db.network_interface().c_if_name(*i).clone()
                            }).collect::<Vec<_>>(),
                        });
                    }
                }
            }

            region_coproc_gws.extend(coproc_gws);
        }

        if has_coproc_dc {
            if region_coproc_dcs.is_empty() {
                return Err(PlatformValidationError::CoprocessorRegionDoesntHaveCoprocessorDc {
                    region: region_name.clone(),
                    coprocessor_dcs_found: region_coproc_dcs.len(),
                    coprocessor_dcs_expected: 1,
                    coprocessor_dcs: Vec::new(),
                });
            }

            if region_coproc_dcs.len() > 1 {
                return Err(PlatformValidationError::CoprocessorRegionHasMoreThanOneCoprocessorDc {
                    region: region_name.clone(),
                    coprocessor_dcs_found: region_coproc_dcs.len(),
                    coprocessor_dcs_expected: 1,
                    coprocessor_dcs: region_coproc_dcs.iter().map(|i| {
                        db.datacenter().c_dc_name(*i).clone()
                    }).collect::<Vec<_>>(),
                });
            }

            if is_single_dc {
                assert_eq!(region_coproc_gws.len(), 2, "Should have been caught earlier");
            } else if is_multi_dc {
                if region_coproc_gws.len() != 2 {
                    // error must be two coproc gateways in region
                    return Err(PlatformValidationError::CoprocessorRegionMustHaveTwoCoprocessorGateways {
                        region: region_name.clone(),
                        region_availability_mode: region_avail_mode.clone(),
                        region_expected_coprocessor_gateways: 2,
                        region_found_coprocessor_gateways: region_coproc_gws.len(),
                        region_found_coprocessor_gateways_servers: region_coproc_gws.iter().map(|srv| {
                            db.server().c_hostname(*srv).clone()
                        }).collect::<Vec<_>>(),
                        region_has_coprocessor_dc: has_coproc_dc,
                    });
                }
            } else { panic!("huh") }

            // at this point we have a region which must have coprocessor gateway
            // now let's check that every server is vpn gateway and has internet
            for coproc_gw in &region_coproc_gws {
                if !db.server().c_is_vpn_gateway(*coproc_gw) {
                    let dc = db.server().c_dc(*coproc_gw);
                    let dc_name = db.datacenter().c_dc_name(dc);
                    return Err(PlatformValidationError::CoprocessorServerMustBeVPNGateway {
                        region: region_name.clone(),
                        datacenter: dc_name.clone(),
                        is_vpn_gateway: db.server().c_is_vpn_gateway(*coproc_gw),
                        is_coprocessor_gateway: true,
                        server: db.server().c_hostname(*coproc_gw).clone(),
                    });
                }
            }

            let coprocessor_dc = *region_coproc_dcs.iter().next().unwrap();

            assert!(res.insert(region, CoprocessorRegionData {
                gateways: region_coproc_gws,
                coprocessor_dc,
            }).is_none());
        } else {
            if !region_coproc_dcs.is_empty() {
                return Err(PlatformValidationError::RegionNotMarkedAsCoprocessorHasCoprocessorDatacenters {
                    region: region_name.clone(),
                    coprocessor_dcs_found: region_coproc_dcs.len(),
                    region_has_coprocessor_dc: has_coproc_dc,
                    coprocessor_dcs_expected: 0,
                    coprocessor_dcs: region_coproc_dcs.iter().map(|i| {
                        db.datacenter().c_dc_name(*i).clone()
                    }).collect::<Vec<_>>(),
                });
            }
        }
    }

    // at this point every coprocessor region has two servers as coprocessor gateways
    // and these servers also must be VPN instaces

    Ok(res)
}
