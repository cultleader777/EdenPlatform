use std::collections::{BTreeSet, BTreeMap};

use serde::{Deserialize, Serialize};

use crate::{database::{Database, TableRowPointerDatacenter}, static_analysis::{PlatformValidationError, networking::first_three_octets}};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BmSimpleDatacenterArguments {
    pub gateway_ip: std::net::Ipv4Addr,
}

pub fn compute_bm_simple_topology(
    db: &Database,
) -> Result<BTreeMap<TableRowPointerDatacenter, BmSimpleDatacenterArguments>, PlatformValidationError> {
    let dc_implementation = "bm_simple";
    let mut res = BTreeMap::new();
    for region in db.region().rows_iter() {
        for dc in db.region().c_referrers_datacenter__region(region) {
            if db.datacenter().c_implementation(*dc) == dc_implementation {
                let dc_name = db.datacenter().c_dc_name(*dc);
                let mut subnet_map: BTreeSet<String> = BTreeSet::new();

                let dc_net = db.datacenter().c_network_cidr(*dc);
                let dc_net_parsed: ipnet::Ipv4Net = dc_net.parse().unwrap();

                let settings = db.datacenter().c_implementation_settings(*dc);
                let arguments: BmSimpleDatacenterArguments = serde_yaml::from_str(&settings)
                    .map_err(|e| {
                        let ex = example_settings();
                        PlatformValidationError::DatacenterImplementationInvalidSettings {
                            dc: dc_name.clone(),
                            dc_implementation: dc_implementation.to_string(),
                            current_settings: settings.clone(),
                            parsing_error: e.to_string(),
                            example_settings: serde_yaml::to_string(&ex).unwrap(),
                        }
                    })?;

                if !dc_net_parsed.contains(&arguments.gateway_ip) {
                    return Err(PlatformValidationError::BmSimpleGatewayIpIsOutsideDatacenterNetwork {
                        bm_simple_datacenter: dc_name.clone(),
                        gateway_ip: arguments.gateway_ip.to_string(),
                        dc_network: dc_net.clone(),
                    });
                }

                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    for net_if in db.server().c_children_network_interface(*server) {
                        let net_ptr = db.network_interface().c_if_network(*net_if);
                        match db.network().c_network_name(net_ptr).as_str() {
                            "dcrouter" => {
                                // this is needed to uniformly set public ips inside edendb source
                                panic!("Should never happen, bmsimple shouldn't have dcrouter nodes");
                            }
                            "lan" => {
                                let lan_ip = db.network_interface().c_if_ip(*net_if);
                                let subnet_id = first_three_octets(lan_ip);
                                let _ = subnet_map.insert(subnet_id);
                            }
                            "internet" => {}
                            "vpn" => {},
                            unexpected => {
                                panic!("Unexpected network {unexpected}")
                            }
                        }
                    }
                }

                if subnet_map.len() > 1 {
                    panic!("Should have been validated at networking layer");
                }

                if let Some(subnet) = subnet_map.iter().next() {
                    let subnet_str = format!("{subnet}.0/24");
                    let subnet_net: ipnet::Ipv4Net = subnet_str.parse().unwrap();
                    if !subnet_net.contains(&arguments.gateway_ip) {
                        return Err(PlatformValidationError::BmSimpleOnlySubnetDoesntContainGatewayIp {
                            bm_simple_datacenter: dc_name.clone(),
                            gateway_ip: arguments.gateway_ip.to_string(),
                            only_dc_subnet: subnet_str,
                        });
                    }
                }

                assert!(res.insert(*dc, arguments).is_none());
            }
        }
    }

    Ok(res)
}

fn example_settings() -> BmSimpleDatacenterArguments {
    BmSimpleDatacenterArguments {
        gateway_ip: "10.12.17.1".parse().unwrap(),
    }
}
