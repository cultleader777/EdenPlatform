use self::coprocessor_dc::CoprocessorRegionData;
use super::{errors::PlatformValidationError, projections::Projection, server_runtime::{IntegrationTest, ServerRuntime}, L1Projections, get_global_settings};
use crate::database::{Database, TableRowPointerNetworkInterface, TableRowPointerServer, TableRowPointerNetwork, TableRowPointerDatacenter, TableRowPointerRegion, TableRowPointerLokiCluster, TableRowPointerMonitoringCluster, TableRowPointerMinioBucket, TableRowPointerServerZfsDataset, TableRowPointerServerXfsVolume, TableRowPointerTempoCluster, TableRowPointerBackendApplicationDeployment, TableRowPointerBackendApplicationS3Bucket};
use convert_case::Casing;
use ipnet::Ipv4Net;
use std::{collections::{BTreeMap, HashMap, HashSet, BTreeSet}, str::FromStr, cmp::Ordering};

mod coprocessor_dc;

pub struct SubnetWithHosts {
    pub network: TableRowPointerNetwork,
    pub interfaces: Vec<TableRowPointerNetworkInterface>,
    pub dc: TableRowPointerDatacenter,
}

pub struct NetworkAnalysisOutput {
    pub subnets_to_interfaces_map: BTreeMap<Ipv4Net, SubnetWithHosts>,
    pub consul_masters: BTreeMap<TableRowPointerRegion, Vec<TableRowPointerServer>>,
    pub nomad_masters: BTreeMap<TableRowPointerRegion, Vec<TableRowPointerServer>>,
    pub vault_masters: BTreeMap<TableRowPointerRegion, Vec<TableRowPointerServer>>,
    pub region_coprocessor_gws: BTreeMap<TableRowPointerRegion, CoprocessorRegionData>,
    pub node_public_ipv6_addrs: BTreeMap<TableRowPointerServer, std::net::Ipv6Addr>,
    pub libvirt_network_topology: LibvirtNetworkTopology,
    pub cross_dc_connectivity: BTreeMap<(TableRowPointerDatacenter, TableRowPointerDatacenter), CrossDcConnectivity>,
    pub wireguard_across_dc_needed: bool,
    pub networking_answers: NetworkingAnswers,
    pub test_docker_registry_gw_address: Option<String>,
}

#[derive(PartialEq, Eq)]
pub enum CrossDcConnectivity {
    Wireguard,
    CloudNative
}

#[derive(Clone)]
pub struct LibvirtServer {
    pub mac: eui48::MacAddress,
    pub ip: std::net::Ipv4Addr,
}

pub struct LibvirtNetwork {
    pub libvirt_name: String,
    pub servers: BTreeMap<TableRowPointerServer, LibvirtServer>,
    pub network: TableRowPointerNetwork,
    pub gw_ip: std::net::Ipv4Addr,
    pub ip_net: ipnet::Ipv4Net,
    pub dhcp_enable: bool,
}

pub struct LibvirtNetworkTopology {
    pub networks: BTreeMap<String, LibvirtNetwork>,
}

#[derive(Clone)]
pub struct VpnGateway {
    pub internet_interface: Option<TableRowPointerNetworkInterface>,
    pub vpn_interface: TableRowPointerNetworkInterface,
}

pub struct DcVpnGateways {
    pub servers: BTreeMap<TableRowPointerServer, VpnGateway>,
}

#[derive(Clone)]
pub struct DcParameters {
    // networking
    pub is_epl_responsible_for_inter_subnet_routing: bool,
    pub is_epl_responsible_for_internal_node_internet: bool,
    pub is_same_dcimpl_connection_managed_by_provider: bool,
    pub provides_admin_vpn_access: bool,
    pub are_floating_ips_available_in_subnets: bool,
    pub are_public_ips_hidden: bool,
    pub use_l3_hop_for_vpn_gateways: bool,
    pub can_have_more_than_one_subnet: bool,
    // disks
    pub disk_id_transform: Option<String>,
    pub interfaces_need_vlan: bool,
}

pub struct NetworkingAnswers {
    pub dcs: BTreeMap<TableRowPointerDatacenter, DcNetworkingAnswers>,
}

pub struct DcNetworkingAnswers {
    pub params: DcParameters,
    pub is_subnet_routing_needed: bool,
    // does node with private IP need NAT
    pub is_private_node_manual_internet_routing_needed: bool,
    // do we use consul VRRP routing directly in nodes
    pub is_consul_vrrp: bool,
    pub is_hardware_vrrp: bool,
    pub is_ospf_routing_needed: bool,
    pub is_floating_subnet_ip_needed: bool,
    pub is_vpn_node_same_as_routing_node: bool,
    pub should_overshadow_ospf_dc_blackhole_route: bool,
    pub is_private_node_to_gw_routing_needed: bool,
    pub subnets: BTreeMap<Ipv4Net, SubnetNetworkingAnswers>,
    pub has_managed_routing_to_other_dcs: bool,
    pub routers_with_internet_interfaces: Vec<TableRowPointerNetworkInterface>,
}

#[derive(Clone)]
pub struct ServerInterfaces {
    pub server: TableRowPointerServer,
    pub lan_iface: TableRowPointerNetworkInterface,
    pub vpn_iface: Option<TableRowPointerNetworkInterface>,
    pub dcrouting_iface: Option<TableRowPointerNetworkInterface>,
    pub internet_iface: Option<TableRowPointerNetworkInterface>,
}

pub struct SubnetNetworkingAnswers {
    pub vpn_interfaces: Vec<ServerInterfaces>,
    pub routing_interfaces: Vec<ServerInterfaces>,
    pub floating_ip: Option<std::net::Ipv4Addr>,
}

pub fn validations(
    db: &crate::database::Database,
) -> Result<NetworkAnalysisOutput, super::PlatformValidationError> {
    dc_analysis(db)?;
    regions_analysis(db)?;
    let res = compute_cross_dc_connectivity(db);
    let subnets_to_interfaces_map = subnets_analysis(db, res.wireguard_across_dc_needed)?;
    let node_public_ipv6_addrs = ipv6_analysis(db)?;
    let libvirt_network_topology = libvirt_network_topology(db, &subnets_to_interfaces_map);
    let consul_masters = consul_analysis(db)?;
    let nomad_masters = nomad_analysis(db)?;
    let vault_masters = vault_analysis(db)?;
    let networking_answers = networking_answers(
        db,
        &subnets_to_interfaces_map,
        res.wireguard_across_dc_needed,
        &res.datacenters_with_native_routing
    )?;
    ensure_all_hidden_internet_interfaces_are_void(db, &networking_answers)?;
    let test_docker_registry_gw_address = subnets_to_interfaces_map
        .iter().next().map(|i| i.0.hosts().take(1).next().unwrap().to_string());
    let region_coprocessor_gws = coprocessor_dc::coprocessor_analysis(db)?;

    Ok(NetworkAnalysisOutput {
        subnets_to_interfaces_map,
        consul_masters,
        nomad_masters,
        vault_masters,
        libvirt_network_topology,
        node_public_ipv6_addrs,
        cross_dc_connectivity: res.cross_dc_connectivity,
        wireguard_across_dc_needed: res.wireguard_across_dc_needed,
        networking_answers,
        test_docker_registry_gw_address,
        region_coprocessor_gws,
    })
}

struct CrossDcConnResult {
    pub cross_dc_connectivity: BTreeMap<(TableRowPointerDatacenter, TableRowPointerDatacenter), CrossDcConnectivity>,
    pub wireguard_across_dc_needed: bool,
    pub datacenters_with_native_routing: BTreeSet<TableRowPointerDatacenter>,
}

fn ensure_all_hidden_internet_interfaces_are_void(
    db: &crate::database::Database, answers: &NetworkingAnswers
) -> Result<(), PlatformValidationError> {

    for dc in db.datacenter().rows_iter() {
        let net_ans = answers.dcs.get(&dc).unwrap();
        if net_ans.params.are_public_ips_hidden {
            for server in db.datacenter().c_referrers_server__dc(dc) {
                for ni in db.server().c_children_network_interface(*server) {
                    let network = db.network().c_network_name(db.network_interface().c_if_network(*ni));
                    let if_name = db.network_interface().c_if_name(*ni);
                    if network == "internet" && if_name != "void" {
                        return Err(PlatformValidationError::ForThisDatacenterImplementationAllInternetInterfaceNamesMustBeVoid {
                            dc: db.datacenter().c_dc_name(dc).clone(),
                            dc_implementation: db.datacenter().c_implementation(dc).clone(),
                            server: db.server().c_hostname(*server).clone(),
                            network: "internet".to_string(),
                            network_interface_name: if_name.clone(),
                            network_interface_only_allowed_name: "void".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn compute_cross_dc_connectivity(db: &crate::database::Database) -> CrossDcConnResult {
    let mut res = BTreeMap::new();
    let mut datacenters_with_native_routing = BTreeSet::new();
    let mut wireguard_across_dc_needed = false;
    for dc_a in db.datacenter().rows_iter() {
        for dc_b in db.datacenter().rows_iter() {
            if dc_a != dc_b {
                let is_same_cloud = |cloud: &str| {
                    db.datacenter().c_implementation(dc_a) == cloud && db.datacenter().c_implementation(dc_b) == cloud
                };
                if is_same_cloud("aws") || is_same_cloud("gcloud")
                {
                    // aws uses transit gateway
                    // gcloud uses native networking
                    assert!(res.insert((dc_a, dc_b), CrossDcConnectivity::CloudNative).is_none());
                    let _ = datacenters_with_native_routing.insert(dc_a);
                    let _ = datacenters_with_native_routing.insert(dc_b);
                } else {
                    wireguard_across_dc_needed = true;
                    assert!(res.insert((dc_a, dc_b), CrossDcConnectivity::Wireguard).is_none());
                }
            }
        }
    }
    CrossDcConnResult {
        cross_dc_connectivity: res,
        wireguard_across_dc_needed,
        datacenters_with_native_routing,
    }
}

fn regions_analysis(
    db: &crate::database::Database,
) -> Result<(), PlatformValidationError> {

    for region in db.region().rows_iter() {
        let dcs = || {
            db.region().c_referrers_datacenter__region(region)
                .iter()
                .filter(|i| {
                    db.datacenter().c_implementation(**i) != "coprocessor"
                })
                .map(|dc| {
                    db.datacenter().c_dc_name(*dc).clone()
                })
                .collect::<Vec<_>>()
        };
        // we don't count coprocessor DC as these are considered branches to main DCs
        let non_coproc_dc_count =
            db.region().c_referrers_datacenter__region(region).iter().filter(|i| {
                db.datacenter().c_implementation(**i) != "coprocessor"
            }).count();
        match db.region().c_availability_mode(region).as_str() {
            "single_dc" => {
                if non_coproc_dc_count != 1 {
                    return Err(PlatformValidationError::SingleDcRegionMustHaveOnlyOneDatacenter {
                        region: db.region().c_region_name(region).clone(),
                        expected_count: 1,
                        found_datacenters: dcs(),
                    });
                }
            }
            "multi_dc" => {
                if non_coproc_dc_count < 3 {
                    return Err(PlatformValidationError::MultiDcRegionMustHaveAtLeastThreeDatacenters {
                        region: db.region().c_region_name(region).clone(),
                        expected_at_least: 3,
                        found_datacenters: dcs(),
                    });
                }
            }
            _ => panic!("Should never be reached")
        }
    }

    Ok(())
}

fn dc_analysis(
    db: &crate::database::Database,
) -> Result<(), PlatformValidationError> {
    // don't allow mixing testvms with production dc implementations
    let mut prod_implementations: Vec<TableRowPointerDatacenter> = Vec::new();
    let mut dev_implementations: Vec<TableRowPointerDatacenter> = Vec::new();
    for dc in db.datacenter().rows_iter() {
        if db.datacenter().c_implementation(dc) == "testvms" {
            dev_implementations.push(dc);
        } else if db.datacenter().c_implementation(dc) != "coprocessor" {
            prod_implementations.push(dc);
        }
    }

    if !dev_implementations.is_empty() && !prod_implementations.is_empty() {

        return Err(PlatformValidationError::TestVmsDatacentersCannotBeMixedWithProductionDatacenters {
            test_vm_datacenters: dev_implementations.iter().map(|i| db.datacenter().c_dc_name(*i).clone()).collect(),
            production_datacenters: prod_implementations.iter().map(|i| db.datacenter().c_dc_name(*i).clone()).collect(),
        });
    }

    Ok(())
}

fn ipv6_analysis(db: &crate::database::Database) ->
    Result<BTreeMap<TableRowPointerServer, std::net::Ipv6Addr>, PlatformValidationError>
{
    let mut res = BTreeMap::new();
    let private_ip_net = ipnet::Ipv6Net::from_str("fc00::/7").expect("cmon boi");
    for server in db.server().rows_iter() {
        let ipv6_addr = db.server().c_public_ipv6_address(server);
        if !ipv6_addr.is_empty() {
            let addr = std::net::Ipv6Addr::from_str(ipv6_addr.as_str())
                .map_err(|e| {
                    PlatformValidationError::InvalidPublicIpV6AddressForNode {
                        server_name: db.server().c_hostname(server).clone(),
                        ipv6_address: ipv6_addr.clone(),
                        parsing_error: e.to_string(),
                    }
                })?;

            if addr.is_loopback() {
                return Err(
                    PlatformValidationError::PublicIpV6AddressForNodeIsLoopback {
                        server_name: db.server().c_hostname(server).clone(),
                        ipv6_address: ipv6_addr.clone(),
                    }
                );
            }

            if addr.is_multicast() {
                return Err(
                    PlatformValidationError::PublicIpV6AddressForNodeIsMulticast {
                        server_name: db.server().c_hostname(server).clone(),
                        ipv6_address: ipv6_addr.clone(),
                    }
                );
            }

            if private_ip_net.contains(&addr) {
                return Err(
                    PlatformValidationError::PublicIpV6AddressIsPrivate {
                        server_name: db.server().c_hostname(server).clone(),
                        ipv6_address: ipv6_addr.clone(),
                    }
                );
            }

            let has_internet_iface =
                db.server().c_children_network_interface(server)
                .iter().filter(|ni| {
                    db.network().c_network_name(db.network_interface().c_if_network(**ni)) == "internet"
                }).next().is_some();

            if !has_internet_iface {
                return Err(
                    PlatformValidationError::ServerHasPublicIpV6AddressButDoesntHaveIpV4PublicAddress {
                        server_name: db.server().c_hostname(server).clone(),
                        ipv6_address: ipv6_addr.clone(),
                        ipv4_network_interfaces:
                            db.server().c_children_network_interface(server)
                              .iter()
                              .map(|ni| {
                                  db.network().c_network_name(db.network_interface().c_if_network(*ni)).clone()
                              }).collect(),
                    }
                );
            }

            // we pick only one address for the interface
            // to leave the rest in the server for VMs
            // or anything else
            if !ipv6_addr.ends_with(":1") {
                return Err(
                    PlatformValidationError::PublicIpV6AddressDoesNotEndWithOne {
                        server_name: db.server().c_hostname(server).clone(),
                        ipv6_address: ipv6_addr.clone(),
                    }
                );
            }

            assert!(res.insert(server, addr).is_none());
        }
    }

    Ok(res)
}

fn subnets_analysis(
    db: &crate::database::Database,
    wireguard_across_dc_needed: bool,
) -> Result<BTreeMap<Ipv4Net, SubnetWithHosts>, PlatformValidationError> {
    let mut subnet_to_interfaces_map: BTreeMap<Ipv4Net, SubnetWithHosts> =
        BTreeMap::new();
    let mut vpn_found = false;
    for net_ptr in db.network().rows_iter() {
        let network_name = db.network().c_network_name(net_ptr);
        let is_internet = network_name == "internet";
        let is_vpn = network_name == "vpn";
        vpn_found |= is_vpn;
        // We only need to ensure that in this case
        // that ips are not duplicate inside datacenter
        let tolerate_duplicate_ips = network_name == "dcrouter";
        let mut range: iprange::IpRange<ipnet::Ipv4Net> = iprange::IpRange::new();
        let subnet_val = db.network().c_cidr(net_ptr);
        let subnet_addr: ipnet::Ipv4Net =
            subnet_val
                .as_str()
                .parse()
                .map_err(
                    |e: ipnet::AddrParseError| PlatformValidationError::InvalidNetworkIpV4Subnet {
                        network_name: network_name.clone(),
                        subnet_value: subnet_val.clone(),
                        parsing_error: e.to_string(),
                    },
                )?;

        let trunc_subnet = subnet_addr.trunc();
        if subnet_addr != trunc_subnet {
            return Err(PlatformValidationError::NetworkSubnetIsNotTruncated {
                network_name: network_name.clone(),
                subnet_value: subnet_val.clone(),
                expected_value: trunc_subnet.to_string(),
            });
        }

        range.add(subnet_addr);

        let ifs = db
            .network()
            .c_referrers_network_interface__if_network(net_ptr);

        let mut network_ips_w_masks: Vec<(Ipv4Net, TableRowPointerNetworkInterface)> = Vec::new();
        for if_ptr in ifs {
            let ip_val = db.network_interface().c_if_ip(*if_ptr);
            let srv = db.network_interface().c_parent(*if_ptr);

            let if_name = db.network_interface().c_if_name(*if_ptr);
            let colon_counts = if_name.find(":").iter().count();
            let dc = db.server().c_dc(srv);
            let dc_impl = db.datacenter().c_implementation(dc);
            let is_coprocessor_dc = dc_impl == "coprocessor";

            if colon_counts > 1 {
                return Err(
                    PlatformValidationError::InvalidServerInterfaceName {
                        server_name: db.server().c_hostname(srv).clone(),
                        interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                        explanation: format!("Cannot have more than one colon in interface name, found {colon_counts}"),
                    },
                );
            } else if colon_counts == 1 {
                // we take this branch which loops over interfaces only if colon is found
                // 1. check that interface is interface of another in this server
                // 2. check that it starts from 0
                let split = if_name.split(":").collect::<Vec<_>>();
                assert_eq!(split.len(), 2);
                let idx = &split[1];
                let parsed_idx = idx.parse::<u8>().map_err(|e| {
                    let e = e.to_string();
                    PlatformValidationError::InvalidServerSubinterfaceIndex {
                        server_name: db.server().c_hostname(srv).clone(),
                        interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                        explanation: format!("Can't parse subinterface index, part '{idx}' in '{if_name}' is not a number: {e}"),
                    }
                })?;

                // start with this, if we ever need more we can add
                if parsed_idx != 1 {
                    return Err(
                        PlatformValidationError::InvalidServerSubinterfaceIndex {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            explanation: format!("Only allowed subinterface index is '1' got '{parsed_idx}' in '{if_name}'"),
                        }
                    );
                }

                let parent_iface_name = &split[0];
                let found = db.server().c_children_network_interface(srv).iter().filter(|other_iface| {
                    db.network_interface().c_if_name(**other_iface) == *parent_iface_name
                }).collect::<Vec<_>>();
                assert!(found.len() <= 1);
                if found.is_empty() {
                    return Err(
                        PlatformValidationError::InvalidServerSubinterface {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            explanation: format!("Parent interface '{parent_iface_name}' not found in server for subinterface '{if_name}'"),
                        }
                    );
                }

                if !is_internet {
                    return Err(
                        PlatformValidationError::InvalidServerSubinterface {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            explanation: format!(
                                "Only internet is allowed to be a subinterface, interface network: '{}'",
                                db.network().c_network_name(net_ptr).clone()
                            ),
                        }
                    );
                }

                let parent_iface_net = db.network_interface().c_if_network(*found[0]);
                if parent_iface_net == net_ptr {
                    return Err(
                        PlatformValidationError::InvalidServerSubinterfaceBelongsToSameParentNetwork {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            explanation: format!("Parent interface '{parent_iface_name}' belongs to the same network as subinterface '{if_name}'"),
                            parent_network: db.network().c_network_name(parent_iface_net).clone(),
                            parent_network_cidr: db.network().c_cidr(parent_iface_net).clone(),
                            subinterface_network: db.network().c_network_name(net_ptr).clone(),
                            subinterface_network_cidr: db.network().c_cidr(net_ptr).clone(),
                        }
                    );
                }

                if !is_coprocessor_dc {
                    return Err(
                        PlatformValidationError::InvalidServerSubinterfaceOnlyCoprocessorDcAllowsSubinterface {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            datacenter: db.datacenter().c_dc_name(dc).clone(),
                            datacenter_implementation: dc_impl.clone(),
                            only_allowed_datacenter_implementation: "coprocessor".to_string(),
                        }
                    );
                }
            }
            let ip = std::net::Ipv4Addr::from_str(ip_val).map_err(|e| {
                PlatformValidationError::InvalidIpV4Address {
                    server_name: db.server().c_hostname(srv).clone(),
                    interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                    value: ip_val.clone(),
                    parsing_error: e.to_string(),
                }
            })?;

            if is_internet && ip.is_private() {
                return Err(
                    PlatformValidationError::InternetNetworkCannotHavePrivateIpAddresses {
                        server_name: db.server().c_hostname(srv).clone(),
                        interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                        network_name: db.network().c_network_name(net_ptr).clone(),
                        ip_address: ip_val.clone(),
                        forbidden_ranges: vec![
                            "10.0.0.0/8".to_string(),
                            "172.16.0.0/12".to_string(),
                            "192.168.0.0/16".to_string(),
                        ],
                    },
                );
            }

            if !is_internet && !ip.is_private() {
                return Err(
                    PlatformValidationError::NonInternetNetworkCannotHavePublicIpAddresses {
                        server_name: db.server().c_hostname(srv).clone(),
                        interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                        network_name: db.network().c_network_name(net_ptr).clone(),
                        ip_address: ip_val.clone(),
                        allowed_ranges: vec![
                            "10.0.0.0/8".to_string(),
                            "172.16.0.0/12".to_string(),
                            "192.168.0.0/16".to_string(),
                        ],
                    },
                );
            }

            if is_vpn {
                if !is_coprocessor_dc && !db.server().c_is_vpn_gateway(srv) {
                    return Err(
                        PlatformValidationError::VpnInterfaceExistsButServerIsNotMarkedVpnGateway {
                            server_name: db.server().c_hostname(srv).clone(),
                            vpn_interface_ip: ip.to_string(),
                        },
                    );
                }

                let forbidden_vpn_ip = "172.21.7.254";
                if ip_val == forbidden_vpn_ip {
                    return Err(
                        PlatformValidationError::VpnAddressReservedForAdmin {
                            server_name: db.server().c_hostname(srv).clone(),
                            vpn_interface_ip: ip.to_string(),
                            forbidden_vpn_ip: forbidden_vpn_ip.to_string(),
                        },
                    );
                }
            }

            let prefix = db.network_interface().c_if_prefix(*if_ptr);
            let ip_wcidr = format!("{}/{}", &ip_val, prefix);
            if !range.contains(&ip) {
                return Err(PlatformValidationError::InterfaceIpIsNotInsideSubnet {
                    server_name: db.server().c_hostname(srv).clone(),
                    interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                    interface_ip: ip_wcidr,
                    network_name: db.network().c_network_name(net_ptr).clone(),
                    subnet_range: subnet_val.clone(),
                });
            }

            let subnet = ipnet::Ipv4Net::from_str(&ip_wcidr).unwrap().trunc();

            if prefix < 32 {
                if subnet.network() == ip {
                    return Err(
                        PlatformValidationError::ServerIpCannotBeNetworkAddress {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            interface_ip: ip.to_string(),
                            network_name: db.network().c_network_name(net_ptr).clone(),
                            subnet_range: subnet.to_string(),
                        },
                    );
                }

                if subnet.broadcast() == ip {
                    return Err(
                        PlatformValidationError::ServerIpCannotBeBroadcastAddress {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            interface_ip: ip.to_string(),
                            network_name: db.network().c_network_name(net_ptr).clone(),
                            subnet_range: subnet.to_string(),
                        },
                    );
                }

                let first_ip = subnet.hosts().take(1).next().unwrap();

                if first_ip == ip {
                    return Err(
                        PlatformValidationError::FirstIpInSubnetIsReservedToGateway {
                            server_name: db.server().c_hostname(srv).clone(),
                            interface_name: db.network_interface().c_if_name(*if_ptr).clone(),
                            interface_ip: ip_wcidr,
                            network_name: db.network().c_network_name(net_ptr).clone(),
                            subnet_range: subnet.to_string(),
                        },
                    );
                }
            }

            network_ips_w_masks.push((subnet, *if_ptr));
        }

        network_ips_w_masks.sort_by(|a, b| {
            let ord = a.0.cmp(&b.0);
            if ord != Ordering::Equal {
                return ord;
            }
            db.network_interface().c_if_ip(a.1).cmp(db.network_interface().c_if_ip(b.1))
        });

        for w in network_ips_w_masks.as_slice().windows(2) {
            let (a_ip, a_ptr) = w[0];
            let (b_ip, b_ptr) = w[1];
            let a_srv = db.network_interface().c_parent(a_ptr);
            let b_srv = db.network_interface().c_parent(b_ptr);

            if a_ip != b_ip {
                // subnets are different, there cannot be overlap between them
                if a_ip.contains(&b_ip) || b_ip.contains(&a_ip) {
                    return Err(PlatformValidationError::SubnetOverlapAcrossInterfaces {
                        server_a_name: db.server().c_hostname(a_srv).clone(),
                        interface_a_name: db.network_interface().c_if_name(a_ptr).clone(),
                        interface_a_ip: format!(
                            "{}/{}",
                            db.network_interface().c_if_ip(a_ptr),
                            db.network_interface().c_if_prefix(a_ptr)
                        ),
                        server_b_name: db.server().c_hostname(b_srv).clone(),
                        interface_b_name: db.network_interface().c_if_name(b_ptr).clone(),
                        interface_b_ip: format!(
                            "{}/{}",
                            db.network_interface().c_if_ip(b_ptr),
                            db.network_interface().c_if_prefix(b_ptr)
                        ),
                        subnet_name: db.network().c_network_name(net_ptr).clone(),
                        subnet_range: subnet_val.clone(),
                    });
                }
            }

            // for servers to be in same subnet subnets can only be the the same
            if !tolerate_duplicate_ips && db.network_interface().c_if_ip(a_ptr) == db.network_interface().c_if_ip(b_ptr) {
                return Err(PlatformValidationError::DuplicateIpFoundOnTheNetwork {
                    server_a_name: db.server().c_hostname(a_srv).clone(),
                    interface_a_name: db.network_interface().c_if_name(a_ptr).clone(),
                    interface_a_ip: format!(
                        "{}/{}",
                        db.network_interface().c_if_ip(a_ptr),
                        db.network_interface().c_if_prefix(a_ptr)
                    ),
                    server_b_name: db.server().c_hostname(b_srv).clone(),
                    interface_b_name: db.network_interface().c_if_name(b_ptr).clone(),
                    interface_b_ip: format!(
                        "{}/{}",
                        db.network_interface().c_if_ip(b_ptr),
                        db.network_interface().c_if_prefix(b_ptr)
                    ),
                    subnet_name: db.network().c_network_name(net_ptr).clone(),
                    subnet_range: subnet_val.clone(),
                });
            }
        }

        for (net, if_ptr) in &network_ips_w_masks {
            let v = subnet_to_interfaces_map.entry(*net).or_insert_with(|| {
                let srv = db.network_interface().c_parent(*if_ptr);
                let dc = db.server().c_dc(srv);
                SubnetWithHosts {
                    network: net_ptr,
                    interfaces: Vec::new(),
                    dc,
                }
            });
            v.interfaces.push(*if_ptr);
        }
    }

    if db.datacenter().len() > 1 && !vpn_found {
        return Err(PlatformValidationError::MoreThanOneDatacenterButNoVpnNetworkToConnectThem {
            datacenter_count: db.datacenter().len(),
            missing_network: "vpn".to_string(),
        });
    }

    // root subnets can never clash either
    for net in db
        .network()
        .rows_iter()
        .collect::<Vec<_>>()
        .as_slice()
        .windows(2)
    {
        let net_a = net[0];
        let net_b = net[1];

        let net_name_a = db.network().c_network_name(net_a);
        let net_name_b = db.network().c_network_name(net_b);
        let both_networks_not_internet = net_name_a != "internet" && net_name_b != "internet";
        let mut sorted = vec![net_name_a, net_name_b];
        sorted.sort();
        let are_lan_and_dcrouter = sorted[0] == "dcrouter" && sorted[1] == "lan";
        if both_networks_not_internet && !are_lan_and_dcrouter
        {
            let subnet_a = db
                .network()
                .c_cidr(net_a)
                .parse::<Ipv4Net>()
                .unwrap()
                .trunc();
            let subnet_b = db
                .network()
                .c_cidr(net_b)
                .parse::<Ipv4Net>()
                .unwrap()
                .trunc();

            if subnet_a.contains(&subnet_b) || subnet_b.contains(&subnet_a) {
                return Err(PlatformValidationError::SubnetOverlapAcrossNetworks {
                    network_a_name: db.network().c_network_name(net_a).clone(),
                    network_a_cidr: db.network().c_cidr(net_a).clone(),
                    network_b_name: db.network().c_network_name(net_b).clone(),
                    network_b_cidr: db.network().c_cidr(net_b).clone(),
                });
            }
        }
    }

    check_for_allowed_network_cidrs(db)?;
    check_for_servers_belonging_to_dc_cidrs(db, wireguard_across_dc_needed)?;

    Ok(subnet_to_interfaces_map)
}

fn check_for_allowed_network_cidrs(db: &Database) -> Result<(), PlatformValidationError> {
    for net_ptr in db.network().rows_iter() {
        let subnet_val = db.network().c_cidr(net_ptr);

        if db.network().c_network_name(net_ptr) == "lan" && db.network().c_cidr(net_ptr) != "10.0.0.0/8" {
            return Err(PlatformValidationError::NetworkLanDisallowedSubnetValue {
                network_name: db.network().c_network_name(net_ptr).clone(),
                subnet_value: subnet_val.clone(),
                only_allowed_value: "10.0.0.0/8".to_string(),
            });
        }

        if db.network().c_network_name(net_ptr) == "vpn" && db.network().c_cidr(net_ptr) != "172.21.0.0/16" {
            return Err(PlatformValidationError::NetworkVpnDisallowedSubnetValue {
                network_name: db.network().c_network_name(net_ptr).clone(),
                subnet_value: subnet_val.clone(),
                only_allowed_value: "172.21.0.0/16".to_string(),
            });
        }

        if db.network().c_network_name(net_ptr) == "dcrouter" && db.network().c_cidr(net_ptr) != "10.0.0.0/8" {
            return Err(PlatformValidationError::NetworkDcrouterDisallowedSubnetValue {
                network_name: db.network().c_network_name(net_ptr).clone(),
                subnet_value: subnet_val.clone(),
                only_allowed_value: "10.0.0.0/8".to_string(),
            });
        }
    }

    Ok(())
}

fn check_for_servers_belonging_to_dc_cidrs(
    db: &Database,
    wireguard_across_dc_needed: bool
) -> Result<(), PlatformValidationError> {
    let global_settings = get_global_settings(db);
    let find_net = |name| {
        db.network()
          .rows_iter()
          .find(|i| db.network().c_network_name(*i) == name)
          .map(|lan| {
              let subnet_addr: ipnet::Ipv4Net =
                  db.network().c_cidr(lan)
                      .as_str()
                      .parse()
                      .unwrap();
              (lan, subnet_addr)
          })
    };
    let lan: Option<(TableRowPointerNetwork, Ipv4Net)> = find_net("lan");
    let vpn: Option<(TableRowPointerNetwork, Ipv4Net)> = find_net("vpn");
    let dcrouter: Option<(TableRowPointerNetwork, Ipv4Net)> = find_net("dcrouter");

    let mut lan_set: HashMap<String, TableRowPointerDatacenter> = HashMap::new();

    for dc in db.datacenter().rows_iter() {
        let mut dc_subnets: BTreeMap<String, usize> = BTreeMap::new();
        let mut dcrouter_ips: HashMap<String, TableRowPointerServer> = HashMap::new();
        let subnet_val = db.datacenter().c_network_cidr(dc);
        let dc_impl = db.datacenter().c_implementation(dc);
        let is_coproc_dc = dc_impl == "coprocessor";
        let dc_params = get_dc_parameters(&dc_impl);
        let subnet_addr: ipnet::Ipv4Net =
            subnet_val
                .as_str()
                .parse()
                .map_err(
                    |e: ipnet::AddrParseError| PlatformValidationError::InvalidDcIpV4Subnet {
                        datacenter_name: db.datacenter().c_dc_name(dc).clone(),
                        subnet_value: subnet_val.clone(),
                        parsing_error: e.to_string(),
                    },
                )?;

        let mut subnet_vlan_ids: BTreeMap<&str, i64> = BTreeMap::new();
        // hetzner uses this, if someone needs more we
        // can extend later
        let min_vlan = 4000;
        let max_vlan = 4091;

        // 1. DONE lan cidr must be /16
        // 2. DONE lan cidr must belong to the lan network
        // 3. DONE dc lan cidrs must be unique
        // 4. all servers that refer to this datacenter must
        // have a lan interface that belongs to this subnet
        if subnet_addr.prefix_len() != 16 {
            return Err(PlatformValidationError::DatacenterLanNetworkMustHaveSlash16Prefix {
                datacenter_name: db.datacenter().c_dc_name(dc).clone(),
                network_cidr: subnet_val.clone(),
                expected_prefix: "/16".to_string(),
                actual_prefix: format!("/{}", subnet_addr.prefix_len()),
            });
        }

        if let Some((lan, lan_subnet)) = lan.as_ref() {
            if !lan_subnet.contains(&subnet_addr) {
                return Err(PlatformValidationError::DatacenterNetworkDoesntBelongToGlobalLan {
                    network_name: db.network().c_network_name(*lan).clone(),
                    network_cidr: db.network().c_cidr(*lan).clone(),
                    datacenter_name: db.datacenter().c_dc_name(dc).clone(),
                    datacenter_cidr: db.datacenter().c_network_cidr(dc).clone(),
                });
            }

            if let Some(prev_dc) = lan_set.insert(subnet_addr.network().to_string(), dc) {
                return Err(PlatformValidationError::DatacenterNetworkClash {
                    datacenter_a_name: db.datacenter().c_dc_name(prev_dc).clone(),
                    datacenter_a_network_cidr: db.datacenter().c_network_cidr(prev_dc).clone(),
                    datacenter_b_name: db.datacenter().c_dc_name(dc).clone(),
                    datacenter_b_network_cidr: db.datacenter().c_network_cidr(dc).clone(),
                });
            }

            for server in db.datacenter().c_referrers_server__dc(dc) {
                for net_if in db.server().c_children_network_interface(*server) {
                    let if_network = db.network_interface().c_if_network(*net_if);
                    let vlan_id = db.network_interface().c_if_vlan(*net_if);
                    let cidr = db.network_interface().c_if_prefix(*net_if);
                    if if_network == *lan {
                        if !is_coproc_dc {
                            if cidr != 24 {
                                return Err(PlatformValidationError::LanInterfaceCidrIsNot24 {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_cidr: cidr,
                                    expected_cidr: 24,
                                    interface_network: "lan".to_string(),
                                });
                            }
                        } else {
                            if cidr != 32 {
                                return Err(PlatformValidationError::LanInterfaceCidrIsNot32 {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_cidr: cidr,
                                    expected_cidr: 32,
                                    interface_network: "lan".to_string(),
                                    datacenter_implementation: dc_impl.clone(),
                                });
                            }
                        }

                        if dc_params.interfaces_need_vlan {
                            if vlan_id == -1 {
                                return Err(PlatformValidationError::VlanIdForInterfaceIsNotSet {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_vlan: vlan_id,
                                    interface_network: "lan".to_string(),
                                    datacenter_implementation: dc_impl.clone(),
                                });
                            }

                            if vlan_id < min_vlan || vlan_id > max_vlan {
                                return Err(PlatformValidationError::VlanIdForInterfaceIsInInvalidRange {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_vlan: vlan_id,
                                    interface_network: "lan".to_string(),
                                    min_vlan,
                                    max_vlan,
                                    datacenter_implementation: dc_impl.clone(),
                                });
                            }

                            if let Some(subnet_vlan_id) = subnet_vlan_ids.get(subnet_val.as_str()) {
                                if *subnet_vlan_id != vlan_id {
                                    return Err(PlatformValidationError::VlanSubnetHasDifferentIds {
                                        server_name: db.server().c_hostname(*server).clone(),
                                        interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                        interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                        interface_vlan: vlan_id,
                                        interface_network: "lan".to_string(),
                                        prev_vlan_id: *subnet_vlan_id,
                                        current_vlan_id: vlan_id,
                                        datacenter_implementation: dc_impl.clone(),
                                    });
                                }
                            } else {
                                assert!(
                                    subnet_vlan_ids.insert(subnet_val.as_str(), vlan_id).is_none(),
                                    "vlan id shouldn't exist yet"
                                );
                            }

                            let if_name = db.network_interface().c_if_name(*net_if);
                            let if_spl = if_name.split('.').collect::<Vec<_>>();

                            if if_spl.len() != 2 {
                                return Err(PlatformValidationError::VlanInvalidInterfaceName {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_network: "lan".to_string(),
                                    vlan_id,
                                    example_valid_interface_name: format!("eth0.{}", vlan_id),
                                    datacenter_implementation: dc_impl.clone(),
                                });
                            }

                            let if_vlan_id = if_spl[1].parse::<i64>().map_err(|e| {
                                PlatformValidationError::VlanInterfaceNameIdIsNotANumber {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_vlan: vlan_id,
                                    interface_network: "lan".to_string(),
                                    parsing_error: e.to_string(),
                                    example_valid_interface_name: format!("eth0.{}", vlan_id),
                                    datacenter_implementation: dc_impl.clone(),
                                }
                            })?;

                            if if_vlan_id != vlan_id {
                                return Err(PlatformValidationError::VlanInterfaceNameIdMismatchToVlanId {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_vlan: vlan_id,
                                    interface_network: "lan".to_string(),
                                    interface_name_vlan_id: if_vlan_id,
                                    example_valid_interface_name: format!("eth0.{}", vlan_id),
                                    datacenter_implementation: dc_impl.clone(),
                                });
                            }

                            let if_to_find = &if_spl[0];

                            // find vlan config which is attached to the interface
                            let mut found_if =
                                db.server().c_children_network_interface(*server).iter()
                                    .filter(|other_if| {
                                        *other_if != net_if &&
                                            db.network_interface().c_if_name(**other_if) == *if_to_find
                                    });

                            if found_if.next().is_none() {
                                return Err(PlatformValidationError::VlanCantFindVlanParentNetworkInterface {
                                    server_name: db.server().c_hostname(*server).clone(),
                                    interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                    interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                    interface_vlan: vlan_id,
                                    interface_network: "lan".to_string(),
                                    interfaces_on_server:
                                        db.server().c_children_network_interface(*server).iter()
                                            .filter_map(|i| {
                                                if i != net_if { Some(db.network_interface().c_if_name(*i).clone()) }
                                                else { None }
                                            })
                                            .collect(),
                                    example_valid_interface_name: format!("eth0.{}", vlan_id),
                                    datacenter_implementation: dc_impl.clone(),
                                });
                            }
                        } else if vlan_id != -1 {
                            return Err(PlatformValidationError::VlanDisabledInDcImplementationButSpecified {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_vlan: vlan_id,
                                interface_network: "lan".to_string(),
                                datacenter_implementation: dc_impl.clone(),
                            });
                        }

                        let ip = std::net::Ipv4Addr::from_str(
                            db.network_interface().c_if_ip(*net_if)
                        ).expect("All ips should have been parsed earlier");

                        if dc_params.use_l3_hop_for_vpn_gateways && ip.octets()[2] >= 128 {
                            return Err(PlatformValidationError::InterfaceIpThirdOctetIsTooLarge {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_network: db.network().c_network_name(if_network).clone(),
                                datacenter_name: db.datacenter().c_dc_name(dc).clone(),
                                datacenter_subnet: subnet_val.clone(),
                                explanation: format!("{}.128.0 and above ips are used for direct L3 to VPN routing hop GRE tunnel", first_two_octets(subnet_val)),
                            });
                        }

                        let dc_subnet = first_three_octets(db.network_interface().c_if_ip(*net_if));
                        *dc_subnets.entry(dc_subnet).or_default() += 1;

                        if !subnet_addr.contains(&ip) {
                            return Err(PlatformValidationError::InterfaceIpIsNotInsideDatacenterSubnet {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_network: db.network().c_network_name(if_network).clone(),
                                datacenter_name: db.datacenter().c_dc_name(dc).clone(),
                                datacenter_subnet: subnet_val.clone(),
                            });
                        }

                        let octets = db.network_interface().c_if_ip(*net_if).split(".").collect::<Vec<_>>();
                        let third_octet: i64 = octets[2].parse::<i64>().unwrap();
                        let is_in_lan_range = third_octet >= 0 && third_octet < 252;
                        if !is_in_lan_range {
                            let dcrouter_range =
                                format!("{}.252.0/22", first_two_octets(db.network_interface().c_if_ip(*net_if)));
                            return Err(PlatformValidationError::LanInterfaceIsInsideForbiddenDcrouterRange {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_network: "lan".to_string(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_cidr: cidr,
                                dcrouter_range,
                            });
                        }
                    }
                }
            }
        }

        if dc_subnets.len() > 1 {
            if !dc_params.can_have_more_than_one_subnet {
                return Err(PlatformValidationError::DatacenterImplementationDoesntAllowMoreThanOneSubnet {
                    dc: db.datacenter().c_dc_name(dc).clone(),
                    subnet_count: dc_subnets.len(),
                    max_subnets: 1,
                });
            }
        }

        let router_subnet_vlan_id = db.datacenter().c_router_subnet_vlan_id(dc);

        if dc_params.interfaces_need_vlan {
            if router_subnet_vlan_id == -1 {
                return Err(PlatformValidationError::VlanSubnetRouterVlanIdUnspecified {
                    subnet_count: dc_subnets.len(),
                    vlan_id: router_subnet_vlan_id,
                    datacenter_implementation: dc_impl.clone(),
                });
            }

            if router_subnet_vlan_id < min_vlan || router_subnet_vlan_id > max_vlan {
                return Err(PlatformValidationError::VlanIdForRouterSubnetIsInInvalidRange {
                    min_vlan,
                    max_vlan,
                    router_subnet_vlan_id,
                    datacenter_implementation: dc_impl.clone(),
                });
            }

            // hetzner inter dc vlan id must never clash with other subnets
            let is_hetzner = "hetzner" == dc_impl;
            for (subnet, vlan_id) in subnet_vlan_ids.iter() {
                if is_hetzner && *vlan_id == global_settings.hetzner_inter_dc_vlan_id {
                    return Err(PlatformValidationError::HetznerInterDcVlanIdClashesWithSubnetVlanIds {
                        interface_vlan: *vlan_id,
                        hetzner_inter_dc_vlan_id: global_settings.hetzner_inter_dc_vlan_id,
                        datacenter_implementation: dc_impl.clone(),
                    });
                }

                if *vlan_id == router_subnet_vlan_id {
                    return Err(PlatformValidationError::VlanSubnetRouterVlanIdClashesWithSubnetVlanId {
                        subnet_count: dc_subnets.len(),
                        router_subnet_vlan_id,
                        subnet_vlan_id: *vlan_id,
                        subnet: subnet.to_string(),
                        datacenter_implementation: dc_impl.clone(),
                    });
                }
            }
        } else {
            if router_subnet_vlan_id != -1 {
                return Err(PlatformValidationError::VlanSubnetRouterVlanIdSpecifiedButNotUsed {
                    subnet_count: dc_subnets.len(),
                    vlan_id: router_subnet_vlan_id,
                    datacenter_implementation: dc_impl.clone(),
                });
            }
        }

        let is_subnet_routing_needed = is_subnet_routing_needed(dc_subnets.len(), &dc_params, wireguard_across_dc_needed);
        if let Some((dcrouter, _)) = dcrouter.as_ref() {
            for server in db.datacenter().c_referrers_server__dc(dc) {
                let is_router = db.server().c_is_router(*server)
                    || (db.server().c_is_vpn_gateway(*server) && is_subnet_routing_needed);

                let mut router_if_found = false;
                for net_if in db.server().c_children_network_interface(*server) {
                    let if_network = db.network_interface().c_if_network(*net_if);
                    let if_ip = db.network_interface().c_if_ip(*net_if);
                    let cidr = db.network_interface().c_if_prefix(*net_if);

                    if if_network == *dcrouter {
                        if cidr != 22 {
                            return Err(PlatformValidationError::DcrouterInterfaceCidrIsNot22 {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_cidr: cidr,
                                expected_cidr: 22,
                                interface_network: "dcrouter".to_string(),
                            });
                        }

                        let ip = std::net::Ipv4Addr::from_str(
                            db.network_interface().c_if_ip(*net_if)
                        ).expect("All ips should have been parsed earlier");

                        if !subnet_addr.contains(&ip) {
                            return Err(PlatformValidationError::InterfaceIpIsNotInsideDatacenterSubnet {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_network: db.network().c_network_name(if_network).clone(),
                                datacenter_name: db.datacenter().c_dc_name(dc).clone(),
                                datacenter_subnet: subnet_val.clone(),
                            });
                        }

                        if let Some(prev) = dcrouter_ips.insert(if_ip.clone(), *server) {
                            return Err(PlatformValidationError::DcrouterDuplicateRouterIpInsideDatacenterDetected {
                                datacenter: db.datacenter().c_dc_name(dc).clone(),
                                previous_server_hostname: db.server().c_hostname(prev).clone(),
                                previous_server_interface_ip: if_ip.clone(),
                                duplicate_server_hostname: db.server().c_hostname(*server).clone(),
                                duplicate_server_interface_ip: if_ip.clone(),
                            });
                        }

                        if !is_router {
                            return Err(PlatformValidationError::DcrouterInterfaceExistsButServerIsNotMarkedAsRouter {
                                server_name: db.server().c_hostname(*server).clone(),
                                router_interface_ip: if_ip.clone(),
                            });
                        }

                        let octets = db.network_interface().c_if_ip(*net_if).split(".").collect::<Vec<_>>();
                        let third_octet: i64 = octets[2].parse::<i64>().unwrap();
                        let is_in_dcrouter_range = third_octet >= 252 && third_octet <= 255;
                        if !is_in_dcrouter_range {
                            let dcrouter_range =
                                format!("{}.252.0/22", first_two_octets(db.network_interface().c_if_ip(*net_if)));
                            return Err(PlatformValidationError::DcrouterInterfaceIsOutsideAllowedRange {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_network: "dcrouter".to_string(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_cidr: cidr,
                                dcrouter_range,
                            });
                        }

                        router_if_found = true;
                    }
                }

                if is_router && !router_if_found {
                    return Err(PlatformValidationError::DcrouterServerMustHaveDcrouterInterface {
                        server: db.server().c_hostname(*server).clone(),
                        server_dc: db.datacenter().c_dc_name(dc).clone(),
                        network_interfaces: db.server().c_children_network_interface(*server).iter().map(|i| {
                            db.network().c_network_name(db.network_interface().c_if_network(*i)).clone()
                        }).collect(),
                    });
                }
            }
        }

        if let Some((vpn, _vpn_subnet)) = vpn.as_ref() {
            for server in db.datacenter().c_referrers_server__dc(dc) {
                for net_if in db.server().c_children_network_interface(*server) {
                    let if_network = db.network_interface().c_if_network(*net_if);
                    let cidr = db.network_interface().c_if_prefix(*net_if);
                    if if_network == *vpn {
                        if cidr != 16 {
                            return Err(PlatformValidationError::VpnInterfaceCidrIsNot16 {
                                server_name: db.server().c_hostname(*server).clone(),
                                interface_name: db.network_interface().c_if_name(*net_if).clone(),
                                interface_ip: db.network_interface().c_if_ip(*net_if).clone(),
                                interface_cidr: cidr,
                                expected_cidr: 16,
                                interface_network: "vpn".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // No point in having many small subnets, however, this can
        // be overriden by user by setting allow_small_subnets to true
        // To have more than one subnet inside a DC you should have at least two hundred
        // servers to split them into twoo 100 host subnets
        if !db.datacenter().c_allow_small_subnets(dc) && dc_subnets.len() >= 2 {
            for (sub, cnt) in &dc_subnets {
                if *cnt < 100 {
                    return Err(PlatformValidationError::DcSubnetHasTooFewHosts {
                        dc: db.datacenter().c_dc_name(dc).clone(),
                        subnet: format!("{sub}.0/24"),
                        servers_count: *cnt,
                        minimum_count: 100,
                    });
                }
            }
        }
    }

    Ok(())
}

fn consul_analysis(db: &Database) -> Result<BTreeMap<TableRowPointerRegion, Vec<TableRowPointerServer>>, PlatformValidationError> {
    let mut res = BTreeMap::new();
    let settings = super::get_global_settings(db);
    let checks_enabled = !settings.disable_consul_quorum_tests;

    for region in db.region().rows_iter() {
        let mut found_consul_servers = Vec::new();
        let mut dc_buckets: BTreeMap<TableRowPointerDatacenter, Vec<String>> = BTreeMap::new();
        let is_multi_dc_mode = db.region().c_availability_mode(region) == "multi_dc";

        for dc in db.region().c_referrers_datacenter__region(region) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                if db.server().c_is_consul_master(*server) {
                    found_consul_servers.push(*server);
                    dc_buckets.entry(*dc).or_default().push(db.server().c_hostname(*server).clone());
                }
            }
        }

        if checks_enabled
            && found_consul_servers.len() != 3
            && found_consul_servers.len() != 5
        {
            return Err(
                PlatformValidationError::ConsulServerQuorumMustBeThreeOrFiveInRegion {
                    region: db.region().c_region_name(region).clone(),
                    found_consul_servers: found_consul_servers.iter().map(|i| {
                        db.server().c_hostname(*i).clone()
                    }).collect(),
                },
            );
        }

        if checks_enabled && is_multi_dc_mode {
            match found_consul_servers.len() {
                3 => {
                    for (dc, bucket) in &dc_buckets {
                        if bucket.len() > 1 {
                            return Err(
                                PlatformValidationError::ConsulServersQuorumTooManyServersInSingleDc {
                                    region: db.region().c_region_name(region).clone(),
                                    datacenter: db.datacenter().c_dc_name(*dc).clone(),
                                    total_consul_servers: 3,
                                    max_allowed_per_dc: 1,
                                    found_in_dc: bucket.len(),
                                    consul_servers_in_dc: bucket.clone(),
                                },
                            );
                        }
                    }
                }
                5 => {
                    for (dc, bucket) in &dc_buckets {
                        if bucket.len() > 2 {
                            return Err(
                                PlatformValidationError::ConsulServersQuorumTooManyServersInSingleDc {
                                    region: db.region().c_region_name(region).clone(),
                                    datacenter: db.datacenter().c_dc_name(*dc).clone(),
                                    total_consul_servers: 5,
                                    max_allowed_per_dc: 2,
                                    found_in_dc: bucket.len(),
                                    consul_servers_in_dc: bucket.clone(),
                                },
                            );
                        }
                    }
                }
                _ => {
                    panic!("Should never be reached here");
                },
            }
        }

        assert!(res.insert(region, found_consul_servers).is_none());
    }

    Ok(res)
}

fn nomad_analysis(db: &Database) -> Result<BTreeMap<TableRowPointerRegion, Vec<TableRowPointerServer>>, PlatformValidationError> {
    let mut res = BTreeMap::new();
    let settings = super::get_global_settings(db);
    let checks_enabled = !settings.disable_nomad_quorum_tests;

    for region in db.region().rows_iter() {
        let mut found_nomad_servers = Vec::new();
        let mut dc_buckets: BTreeMap<TableRowPointerDatacenter, Vec<String>> = BTreeMap::new();
        let is_multi_dc_mode = db.region().c_availability_mode(region) == "multi_dc";
        for dc in db.region().c_referrers_datacenter__region(region) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                if db.server().c_is_nomad_master(*server) {
                    found_nomad_servers.push(*server);
                    dc_buckets.entry(*dc).or_default().push(db.server().c_hostname(*server).clone());
                }
            }
        }

        if checks_enabled
            && found_nomad_servers.len() != 3
            && found_nomad_servers.len() != 5
        {
            return Err(
                PlatformValidationError::NomadServersQuorumMustBeThreeOrFiveInRegion {
                    region: db.region().c_region_name(region).clone(),
                    found_nomad_servers:
                        found_nomad_servers
                            .iter().map(|i| {
                                db.server().c_hostname(*i).clone()
                            }).collect(),
                },
            );
        }

        if checks_enabled && is_multi_dc_mode {
            match found_nomad_servers.len() {
                3 => {
                    for (dc, bucket) in &dc_buckets {
                        if bucket.len() > 1 {
                            return Err(
                                PlatformValidationError::NomadServersQuorumTooManyServersInSingleDc {
                                    region: db.region().c_region_name(region).clone(),
                                    datacenter: db.datacenter().c_dc_name(*dc).clone(),
                                    total_nomad_servers: 3,
                                    max_allowed_per_dc: 1,
                                    found_in_dc: bucket.len(),
                                    nomad_servers_in_dc: bucket.clone(),
                                },
                            );
                        }
                    }
                }
                5 => {
                    for (dc, bucket) in &dc_buckets {
                        if bucket.len() > 2 {
                            return Err(
                                PlatformValidationError::NomadServersQuorumTooManyServersInSingleDc {
                                    region: db.region().c_region_name(region).clone(),
                                    datacenter: db.datacenter().c_dc_name(*dc).clone(),
                                    total_nomad_servers: 5,
                                    max_allowed_per_dc: 2,
                                    found_in_dc: bucket.len(),
                                    nomad_servers_in_dc: bucket.clone(),
                                },
                            );
                        }
                    }
                }
                _ => {
                    panic!("Should never be reached here");
                },
            }
        }

        assert!(res.insert(region, found_nomad_servers).is_none());
    }

    Ok(res)
}

fn vault_analysis(db: &Database) -> Result<BTreeMap<TableRowPointerRegion, Vec<TableRowPointerServer>>, PlatformValidationError> {
    let mut res = BTreeMap::new();

    let settings = super::get_global_settings(db);
    let checks_enabled = !settings.disable_vault_quorum_tests;

    for region in db.region().rows_iter() {
        let mut found_vault_instances = Vec::new();
        let mut dc_buckets: BTreeMap<TableRowPointerDatacenter, Vec<String>> = BTreeMap::new();
        let is_multi_dc_mode = db.region().c_availability_mode(region) == "multi_dc";
        for dc in db.region().c_referrers_datacenter__region(region) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                if db.server().c_is_vault_instance(*server) {
                    found_vault_instances.push(*server);
                    dc_buckets.entry(*dc).or_default().push(db.server().c_hostname(*server).clone());
                }
            }
        }

        if checks_enabled
            && found_vault_instances.len() != 3
            && found_vault_instances.len() != 5
        {
            return Err(
                PlatformValidationError::VaultServersQuorumMustBeThreeOrFiveInRegion {
                    region: db.region().c_region_name(region).clone(),
                    found_vault_instances: found_vault_instances.iter().map(|i| {
                        db.server().c_hostname(*i).clone()
                    }).collect(),
                },
            );
        }

        // if it is three, there cannot be more than one per dc
        // if it is five, there cannot be more than two per dc
        if checks_enabled && is_multi_dc_mode {
            match found_vault_instances.len() {
                3 => {
                    for (dc, bucket) in &dc_buckets {
                        if bucket.len() > 1 {
                            return Err(
                                PlatformValidationError::VaultServersQuorumTooManyServersInSingleDc {
                                    region: db.region().c_region_name(region).clone(),
                                    datacenter: db.datacenter().c_dc_name(*dc).clone(),
                                    total_vault_servers: 3,
                                    max_allowed_per_dc: 1,
                                    found_in_dc: bucket.len(),
                                    vault_servers_in_dc: bucket.clone(),
                                },
                            );
                        }
                    }
                }
                5 => {
                    for (dc, bucket) in &dc_buckets {
                        if bucket.len() > 2 {
                            return Err(
                                PlatformValidationError::VaultServersQuorumTooManyServersInSingleDc {
                                    region: db.region().c_region_name(region).clone(),
                                    datacenter: db.datacenter().c_dc_name(*dc).clone(),
                                    total_vault_servers: 5,
                                    max_allowed_per_dc: 2,
                                    found_in_dc: bucket.len(),
                                    vault_servers_in_dc: bucket.clone(),
                                },
                            );
                        }
                    }
                }
                _ => {
                    panic!("Should never be reached here");
                },
            }
        }

        assert!(res.insert(region, found_vault_instances).is_none());
    }

    Ok(res)
}

pub fn find_consul_network_interfaces(
    db: &Database,
) -> Result<
    Projection<TableRowPointerServer, TableRowPointerNetworkInterface>,
    PlatformValidationError,
> {
    Projection::maybe_create(db.server().rows_iter(), |server| {
        // now we find by lan
        for ni in db.server().c_children_network_interface(server) {
            if db
                .network()
                .c_network_name(db.network_interface().c_if_network(*ni))
                .as_str()
                == "lan"
            {
                return Ok(*ni);
            }
        }

        if let Some(ni) = db
            .server()
            .c_children_network_interface(server)
            .iter()
            .next()
        {
            return Ok(*ni);
        }

        panic!("We assume all servers have interfaces?")
    })
}

pub fn find_internet_network_interfaces(
    db: &Database,
) -> HashMap<TableRowPointerServer, TableRowPointerNetworkInterface>
{
    let mut res = HashMap::new();

    for server in db.server().rows_iter() {
        for ni in db.server().c_children_network_interface(server) {
            if db
                .network()
                .c_network_name(db.network_interface().c_if_network(*ni))
                .as_str()
                == "internet"
            {
                assert!(res.insert(server, *ni).is_none());
            }
        }
    }

    res
}

pub fn find_vpn_network_interfaces(
    db: &Database,
) -> HashMap<TableRowPointerServer, TableRowPointerNetworkInterface>
{
    let mut res = HashMap::new();

    for server in db.server().rows_iter() {
        for ni in db.server().c_children_network_interface(server) {
            if db
                .network()
                .c_network_name(db.network_interface().c_if_network(*ni))
                .as_str()
                == "vpn"
            {
                // we check because coprocessor dc servers
                // have multiple wireguard interfaces
                if !res.contains_key(&server) {
                    assert!(res.insert(server, *ni).is_none());
                }
            }
        }
    }

    res
}

struct RegionPicks<T> {
    default: Option<T>,
    named: HashMap<String, T>,
}

pub struct ClusterPicker<T> {
    region_picks: HashMap<TableRowPointerRegion, RegionPicks<T>>,
}

impl<T: Copy + Clone> ClusterPicker<T> {
    fn new(region_picks: HashMap<TableRowPointerRegion, RegionPicks<T>>) -> Self {
        Self {
            region_picks,
        }
    }

    pub fn pick(&self, region: TableRowPointerRegion, choice: &str) -> Option<T> {
        if let Some(rp) = self.region_picks.get(&region) {
            if choice == "region_default" {
                return rp.default.clone();
            } else {
                return rp.named.get(choice).map(|i| i.clone());
            }
        } else {
            None
        }
    }

    pub fn region_default(&self, region: TableRowPointerRegion) -> Option<T> {
        self.region_picks.get(&region).map(|i| i.default).flatten()
    }
}

fn is_empty_region(db: &Database, region: TableRowPointerRegion) -> bool {
    for dc in db.region().c_referrers_datacenter__region(region) {
        for _ in db.datacenter().c_referrers_server__dc(*dc) {
            return false;
        }
    }

    true
}

pub fn compute_loki_clusters(db: &Database) -> Result<ClusterPicker<TableRowPointerLokiCluster>, PlatformValidationError> {
    let mut region_picks: HashMap<TableRowPointerRegion, RegionPicks<TableRowPointerLokiCluster>> = HashMap::new();

    let settings = super::get_global_settings(db);
    let enable_errors = !settings.disable_region_logging_tests;

    for region in db.region().rows_iter() {
        if !is_empty_region(db, region) {
            let reg_size = db.region().c_referrers_loki_cluster__region(region).len();
            match reg_size {
                0 if enable_errors => {
                    return Err(PlatformValidationError::NoLoggingClusterInsideRegion {
                        region_name: db.region().c_region_name(region).clone(),
                    });
                }
                1 => {
                    // if one set it as default
                    let cluster = db.region().c_referrers_loki_cluster__region(region).iter().next().unwrap();
                    let cluster_name = db.loki_cluster().c_cluster_name(*cluster).clone();
                    let mut named: HashMap<String, TableRowPointerLokiCluster> = HashMap::new();
                    assert!(named.insert(cluster_name, *cluster).is_none());
                    assert!(region_picks.insert(region, RegionPicks {
                        default: Some(*cluster), named
                    }).is_none());
                }
                _ => {
                    // if more than one then only one must be labeled `is_region_default`
                    let mut region_clusters = Vec::new();
                    let mut region_default_clusters = Vec::new();
                    let mut region_default = None;
                    let mut named: HashMap<String, TableRowPointerLokiCluster> = HashMap::new();

                    for cluster in db.region().c_referrers_loki_cluster__region(region) {
                        if db.loki_cluster().c_is_region_default(*cluster) {
                            region_default_clusters.push(db.loki_cluster().c_cluster_name(*cluster).clone());
                            region_default = Some(*cluster);
                        }
                        region_clusters.push(db.loki_cluster().c_cluster_name(*cluster).clone());
                        named.insert(db.loki_cluster().c_cluster_name(*cluster).clone(), *cluster);
                    }

                    if enable_errors {
                        if region_default_clusters.is_empty() {
                            return Err(PlatformValidationError::NoRegionDefaultLoggingClusterSpecified {
                                region_name: db.region().c_region_name(region).clone(),
                                region_clusters,
                                region_default_clusters,
                            });
                        }

                        if region_default_clusters.len() > 1 {
                            return Err(PlatformValidationError::MoreThanOneDefaultLoggingClusterFoundInRegion {
                                region_name: db.region().c_region_name(region).clone(),
                                region_default_clusters,
                            });
                        }
                    }

                    assert!(region_picks.insert(region, RegionPicks {
                        default: region_default, named
                    }).is_none());
                }
            }
        }
    }

    Ok(ClusterPicker { region_picks })
}

pub fn compute_monitoring_clusters(db: &Database) -> Result<ClusterPicker<TableRowPointerMonitoringCluster>, PlatformValidationError> {
    let mut region_picks: HashMap<TableRowPointerRegion, RegionPicks<TableRowPointerMonitoringCluster>> = HashMap::new();

    let settings = super::get_global_settings(db);
    let enable_errors = !settings.disable_region_monitoring_tests;

    for region in db.region().rows_iter() {
        if !is_empty_region(db, region) {
            let reg_size = db.region().c_referrers_monitoring_cluster__region(region).len();
            match reg_size {
                0 if enable_errors => {
                    return Err(PlatformValidationError::NoMonitoringClusterInsideRegion {
                        region_name: db.region().c_region_name(region).clone(),
                    });
                }
                1 => {
                    // if one set it as default
                    let cluster = db.region().c_referrers_monitoring_cluster__region(region).iter().next().unwrap();
                    let cluster_name = db.monitoring_cluster().c_cluster_name(*cluster).clone();
                    let mut named: HashMap<String, TableRowPointerMonitoringCluster> = HashMap::new();
                    assert!(named.insert(cluster_name, *cluster).is_none());
                    assert!(region_picks.insert(region, RegionPicks {
                        default: Some(*cluster), named
                    }).is_none());
                }
                _ => {
                    // if more than one then only one must be labeled `is_region_default`
                    let mut region_clusters = Vec::new();
                    let mut region_default_clusters = Vec::new();
                    let mut region_default = None;
                    let mut named: HashMap<String, TableRowPointerMonitoringCluster> = HashMap::new();

                    for cluster in db.region().c_referrers_monitoring_cluster__region(region) {
                        if db.monitoring_cluster().c_is_region_default(*cluster) {
                            region_default_clusters.push(db.monitoring_cluster().c_cluster_name(*cluster).clone());
                            region_default = Some(*cluster);
                        }
                        region_clusters.push(db.monitoring_cluster().c_cluster_name(*cluster).clone());
                        named.insert(db.monitoring_cluster().c_cluster_name(*cluster).clone(), *cluster);
                    }

                    if enable_errors {
                        if region_default_clusters.is_empty() {
                            return Err(PlatformValidationError::NoRegionDefaultMonitoringClusterSpecified {
                                region_name: db.region().c_region_name(region).clone(),
                                region_clusters,
                                region_default_clusters,
                            });
                        }

                        if region_default_clusters.len() > 1 {
                            return Err(PlatformValidationError::MoreThanOneDefaultMonitoringClusterFoundInRegion {
                                region_name: db.region().c_region_name(region).clone(),
                                region_default_clusters,
                            });
                        }
                    }

                    assert!(region_picks.insert(region, RegionPicks {
                        default: region_default, named
                    }).is_none());
                }
            }
        }
    }

    Ok(ClusterPicker::new(region_picks))
}

pub fn compute_tempo_clusters(db: &Database) -> Result<ClusterPicker<TableRowPointerTempoCluster>, PlatformValidationError> {
    let mut region_picks: HashMap<TableRowPointerRegion, RegionPicks<TableRowPointerTempoCluster>> = HashMap::new();

    let settings = super::get_global_settings(db);
    let enable_errors = !settings.disable_region_tracing_tests;

    for region in db.region().rows_iter() {
        if !is_empty_region(db, region) {
            let reg_size = db.region().c_referrers_tempo_cluster__region(region).len();
            match reg_size {
                0 if enable_errors => {
                    return Err(PlatformValidationError::NoTempoClusterInsideRegion {
                        region_name: db.region().c_region_name(region).clone(),
                    });
                }
                1 => {
                    // if one set it as default
                    let cluster = db.region().c_referrers_tempo_cluster__region(region).iter().next().unwrap();
                    let cluster_name = db.tempo_cluster().c_cluster_name(*cluster).clone();
                    let mut named: HashMap<String, TableRowPointerTempoCluster> = HashMap::new();
                    assert!(named.insert(cluster_name, *cluster).is_none());
                    assert!(region_picks.insert(region, RegionPicks {
                        default: Some(*cluster), named
                    }).is_none());
                }
                _ => {
                    // if more than one then only one must be labeled `is_region_default`
                    let mut region_clusters = Vec::new();
                    let mut region_default_clusters = Vec::new();
                    let mut region_default = None;
                    let mut named: HashMap<String, TableRowPointerTempoCluster> = HashMap::new();

                    for cluster in db.region().c_referrers_tempo_cluster__region(region) {
                        if db.tempo_cluster().c_is_region_default(*cluster) {
                            region_default_clusters.push(db.tempo_cluster().c_cluster_name(*cluster).clone());
                            region_default = Some(*cluster);
                        }
                        region_clusters.push(db.tempo_cluster().c_cluster_name(*cluster).clone());
                        named.insert(db.tempo_cluster().c_cluster_name(*cluster).clone(), *cluster);
                    }

                    if enable_errors {
                        if region_default_clusters.is_empty() {
                            return Err(PlatformValidationError::NoRegionDefaultTempoClusterSpecified {
                                region_name: db.region().c_region_name(region).clone(),
                                region_clusters,
                                region_default_clusters,
                            });
                        }

                        if region_default_clusters.len() > 1 {
                            return Err(PlatformValidationError::MoreThanOneDefaultTempoClusterFoundInRegion {
                                region_name: db.region().c_region_name(region).clone(),
                                region_default_clusters,
                            });
                        }
                    }

                    assert!(region_picks.insert(region, RegionPicks {
                        default: region_default, named
                    }).is_none());
                }
            }
        }
    }

    Ok(ClusterPicker::new(region_picks))
}

pub fn region_monitoring_clusters(db: &Database, region: TableRowPointerRegion) -> Vec<String> {
    let refs = db.region().c_referrers_monitoring_cluster__region(region);
    let mut res = Vec::with_capacity(refs.len());
    for cl in refs {
        res.push(db.monitoring_cluster().c_cluster_name(*cl).clone());
    }
    res
}

pub fn region_loki_clusters(db: &Database, region: TableRowPointerRegion) -> Vec<String> {
    let refs = db.region().c_referrers_loki_cluster__region(region);
    let mut res = Vec::with_capacity(refs.len());
    for cl in refs {
        res.push(db.loki_cluster().c_cluster_name(*cl).clone());
    }
    res
}

pub fn region_tempo_clusters(db: &Database, region: TableRowPointerRegion) -> Vec<String> {
    let refs = db.region().c_referrers_tempo_cluster__region(region);
    let mut res = Vec::with_capacity(refs.len());
    for cl in refs {
        res.push(db.tempo_cluster().c_cluster_name(*cl).clone());
    }
    res
}

pub fn server_region(db: &Database, server: TableRowPointerServer) -> TableRowPointerRegion {
    let dc = db.server().c_dc(server);
    db.datacenter().c_region(dc)
}

pub fn first_region_server(db: &Database, region: TableRowPointerRegion) -> Option<TableRowPointerServer> {
    let dcs = db.region().c_referrers_datacenter__region(region);
    if dcs.len() > 0 {
        let servers = db.datacenter().c_referrers_server__dc(dcs[0]);
        if servers.len() > 0 {
            return Some(servers[0]);
        }
    }

    return None;
}

pub fn first_first_region_vault_server(db: &Database, region: TableRowPointerRegion) -> Option<TableRowPointerServer> {
    let dcs = db.region().c_referrers_datacenter__region(region);
    if dcs.len() > 0 {
        let servers = db.datacenter().c_referrers_server__dc(dcs[0]);
        for server in servers {
            if db.server().c_is_vault_instance(*server) {
                return Some(*server);
            }
        }
    }

    return None;
}

pub fn find_zfs_dataset_disk_medium(db: &Database, dataset: TableRowPointerServerZfsDataset) -> &str {
    let zpool = db.server_zfs_dataset().c_parent(dataset);
    // we assume zpools have same mediums,
    // this should have been checked earlier
    for vdev in db.server_zpool().c_children_server_zpool_vdev(zpool) {
        for vdev_disk in db.server_zpool_vdev().c_children_server_zpool_vdev_disk(*vdev) {
            let disk = db.server_zpool_vdev_disk().c_disk_id(*vdev_disk);
            let kind = db.server_disk().c_disk_kind(disk);
            return db.disk_kind().c_medium(kind).as_str();
        }
    }

    panic!("Can't find any disk in zfs dataset, this should have been checked earlier")
}

pub fn find_server_root_disk_medium(db: &Database, server: TableRowPointerServer) -> &str {
    let root_disk = db.server().c_root_disk(server);
    let disk_kind = db.server_disk().c_disk_kind(root_disk);
    db.disk_kind().c_medium(disk_kind).as_str()
}

pub fn find_xfs_volume_root_disk_medium(db: &Database, server: TableRowPointerServerXfsVolume) -> &str {
    let disk = db.server_xfs_volume().c_xfs_disk(server);
    assert!(db.server_disk().c_xfs_format(disk), "Should have been checked earlier");
    let disk_kind = db.server_disk().c_disk_kind(disk);
    db.disk_kind().c_medium(disk_kind).as_str()
}

pub fn admin_service_responds_test(
    db: &Database,
    l1proj: &L1Projections,
    admin_service: String,
    path: &str,
    expected_string: &str,
) -> IntegrationTest {
    let mut tld_str = String::new();
    let mut ingress_ips = Vec::new();
    for region in db.region().rows_iter() {
        let tld = db.region().c_tld(region);
        if db.tld().c_expose_admin(tld) {
            tld_str = db.tld().c_domain(tld).to_string();
            if let Some(v) = l1proj.region_ingresses.get(&region) {
                ingress_ips = v.ipv4.iter().map(|i| i.to_string()).collect();
            }
        }
    }

    // grafana has its own auth mechanism
    let use_admin_panel_credentials =
        if !admin_service.starts_with("adm-grafana-") {
            Some(super::server_runtime::IntegrationTestCredentials::AdminPanel)
        } else {
            Some(super::server_runtime::IntegrationTestCredentials::GrafanaCluster(admin_service.replace("adm-grafana-", "")))
        };
    IntegrationTest::HttpGetRespondsString {
        hostname: Some(format!("{admin_service}.{tld_str}")),
        server_ips: ingress_ips,
        http_server_port: 443,
        path: path.to_string(),
        is_https: true,
        expected_string: expected_string.to_string(),
        use_admin_panel_credentials,
    }
}

pub fn prometheus_metric_exists_test(
    db: &Database,
    l1proj: &L1Projections,
    mon_cluster: TableRowPointerMonitoringCluster,
    metric: &str,
) -> IntegrationTest {
    let instances = db.monitoring_cluster().c_children_monitoring_instance(mon_cluster);
    let port = db.monitoring_cluster().c_prometheus_port(mon_cluster);
    let i = instances[0];
    let srv_vol = db.monitoring_instance().c_monitoring_server(i);
    let srv = db.server_volume().c_parent(srv_vol);
    let iface = l1proj.consul_network_iface.value(srv);
    let ip = db.network_interface().c_if_ip(*iface);
    return IntegrationTest::PrometheusMetricExists {
        prometheus_server_ip: ip.clone(),
        prometheus_server_port: port,
        metric: metric.to_string(),
        should_exist: true,
    };
}

pub fn prometheus_metric_doesnt_exist_test(
    db: &Database,
    l1proj: &L1Projections,
    mon_cluster: TableRowPointerMonitoringCluster,
    metric: &str,
) -> IntegrationTest {
    let instances = db.monitoring_cluster().c_children_monitoring_instance(mon_cluster);
    let port = db.monitoring_cluster().c_prometheus_port(mon_cluster);
    let i = instances[0];
    let srv_vol = db.monitoring_instance().c_monitoring_server(i);
    let srv = db.server_volume().c_parent(srv_vol);
    let iface = l1proj.consul_network_iface.value(srv);
    let ip = db.network_interface().c_if_ip(*iface);
    return IntegrationTest::PrometheusMetricExists {
        prometheus_server_ip: ip.clone(),
        prometheus_server_port: port,
        metric: metric.to_string(),
        should_exist: false,
    };
}

pub fn consul_services_exists_integration_test(
    db: &Database,
    l1proj: &L1Projections,
    region: TableRowPointerRegion,
    service_name: String,
    target_servers: &[TableRowPointerServer],
) -> IntegrationTest {
    assert!(service_name.ends_with(".service.consul"), "Service {service_name} doesn't end in .service.consul");
    let reg_server = first_region_server(db, region);
    let mut dns_server_ip: Vec<String> = Vec::new();
    if let Some(reg_server) = reg_server {
        let iface = l1proj.consul_network_iface.value(reg_server);
        let ip = db.network_interface().c_if_ip(*iface).clone();
        dns_server_ip.push(format!("{ip}:53"));
    }

    let mut expected_ips: Vec<String> = Vec::new();
    for server in target_servers {
        let iface = l1proj.consul_network_iface.value(*server);
        expected_ips.push(db.network_interface().c_if_ip(*iface).clone());
    }

    return IntegrationTest::DnsResolutionWorksARecords {
        target_servers: dns_server_ip,
        queries: vec![(service_name.clone(), expected_ips)],
    };
}

pub fn networking_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    networking_tests_external(db, l1proj, runtime);
    networking_tests_internal(db, l1proj, runtime);
}

// tests made from provisioner node
fn networking_tests_external(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut private_ips = Vec::new();
    let mut public_ips = Vec::new();

    for server in db.server().rows_iter() {
        let priv_iface = l1proj.consul_network_iface.value(server);
        let ip = db.network_interface().c_if_ip(*priv_iface);
        private_ips.push(ip.clone());

        if let Some(pub_iface) = l1proj.internet_network_iface.get(&server) {
            let ip = db.network_interface().c_if_ip(*pub_iface);
            public_ips.push(ip.clone());
        }
    }

    runtime.add_integration_test(
        "private_ips_ping".to_string(),
        IntegrationTest::PingWorks { server_ips: private_ips }
    );
    runtime.add_integration_test(
        "public_ips_ping".to_string(),
        IntegrationTest::PingWorks { server_ips: public_ips }
    );
}

// tests made from inside remote node
fn networking_tests_internal(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    networking_all_dc_internet_and_dns_tests(db, l1proj, runtime);
    networking_in_dc_subnet_ping_tests(db, l1proj, runtime);
}

fn networking_in_dc_subnet_ping_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    struct SubnetInfo<'a> {
        ping_hosts: Vec<TableRowPointerNetworkInterface>,
        ipnet: &'a Ipv4Net,
        is_vpn_subnet: bool,
    }

    let mut dc_subnets: BTreeMap<TableRowPointerDatacenter, Vec<SubnetInfo>> = BTreeMap::new();
    let lan_network = db.network().rows_iter().filter(|n| db.network().c_network_name(*n) == "lan").next();
    for (ipnet, hosts) in l1proj.networking.subnets_to_interfaces_map.iter().filter(|(_, hosts)| {
        Some(hosts.network) == lan_network
    }) {
        let dc = hosts.dc;
        let e = dc_subnets.entry(dc).or_default();
        let mut router_host = false;
        let mut vpn_host = false;
        let mut private_host = false;
        let mut ping_hosts = Vec::new();

        for iface in &hosts.interfaces {
            let srv = db.network_interface().c_parent(*iface);
            if db.server().c_is_vpn_gateway(srv) {
                if !vpn_host {
                    vpn_host = true;
                    ping_hosts.push(*iface);
                }
            } else if db.server().c_is_router(srv) {
                if !router_host {
                    router_host = true;
                    ping_hosts.push(*iface);
                }
            } else {
                if !private_host {
                    private_host = true;
                    ping_hosts.push(*iface);
                }
            }
        }

        assert!(
            ping_hosts.len() <= 2,
            "maximum of two hosts per subnetcan be cross pinged, one vpn/router and one private server"
        );

        e.push(SubnetInfo {
            ipnet,
            ping_hosts,
            is_vpn_subnet: vpn_host,
        });
    }

    for (dc, subnets) in &dc_subnets {
        let dc_name = db.datacenter().c_dc_name(*dc).to_case(convert_case::Case::Snake);
        for subnet_a in subnets {
            for subnet_b in subnets {
                if subnet_a.ipnet != subnet_b.ipnet {
                    let subnet_a_str = subnet_a.ipnet.to_string().replace(".", "_").replace("/", "p");
                    let subnet_b_str = subnet_b.ipnet.to_string().replace(".", "_").replace("/", "p");
                    let test_name = format!("intra_dc_{dc_name}_subnet_ping_from_{subnet_a_str}_to_{subnet_b_str}_works");
                    let mut server_ips: BTreeMap<String, Vec<String>> = BTreeMap::new();

                    for source_ping_host in &subnet_a.ping_hosts {
                        let source_private_ip = db.network_interface().c_if_ip(*source_ping_host);
                        let from_srv = db.network_interface().c_parent(*source_ping_host);
                        assert!(source_private_ip.starts_with("10."), "We assume only private ips here, got {source_private_ip}");
                        let ssh_ip =
                            if let Some(internet_iface) = l1proj.internet_network_iface.get(&from_srv) {
                                db.network_interface().c_if_ip(*internet_iface)
                            } else { source_private_ip };

                        let to_ping = server_ips.entry(ssh_ip.clone()).or_default();
                        for target_ping_host in &subnet_b.ping_hosts {
                            let target_private_ip = db.network_interface().c_if_ip(*target_ping_host);
                            assert!(target_private_ip.starts_with("10."), "We assume only private ips here, got {target_private_ip}");
                            to_ping.push(target_private_ip.clone());
                        }
                    }

                    runtime.add_integration_test(
                        test_name,
                        IntegrationTest::InsideNodePingWorks { server_ips }
                    );
                } else if subnet_a.ping_hosts.len() > 1 {
                    // inside subnet ping
                    let subnet = subnet_a;
                    let subnet_str = subnet.ipnet.to_string().replace(".", "_").replace("/", "p");
                    let test_name = format!("intra_dc_{dc_name}_inside_subnet_ping_{subnet_str}_works");
                    let mut server_ips: BTreeMap<String, Vec<String>> = BTreeMap::new();
                    for ping_host_a in &subnet.ping_hosts {
                        for ping_host_b in &subnet.ping_hosts {
                            if ping_host_a != ping_host_b {
                                let source_private_ip = db.network_interface().c_if_ip(*ping_host_a);
                                let target_private_ip = db.network_interface().c_if_ip(*ping_host_b);
                                let from_srv = db.network_interface().c_parent(*ping_host_a);
                                assert!(source_private_ip.starts_with("10."), "We assume only private ips here, got {source_private_ip}");
                                assert!(target_private_ip.starts_with("10."), "We assume only private ips here, got {target_private_ip}");
                                let ssh_ip =
                                    if let Some(internet_iface) = l1proj.internet_network_iface.get(&from_srv) {
                                        db.network_interface().c_if_ip(*internet_iface)
                                    } else { source_private_ip };
                                let to_ping = server_ips.entry(ssh_ip.clone()).or_default();
                                to_ping.push(target_private_ip.clone());
                            }
                        }
                    }

                    runtime.add_integration_test(
                        test_name,
                        IntegrationTest::InsideNodePingWorks { server_ips }
                    );
                }
            }
        }
    }

    // cross DC pings for:
    // 1. public <-> public subnets
    // 2. public <-> private subnets
    // 2. private <-> public subnets
    // 2. private <-> private subnets
    for (dc_a, subnets_a) in &dc_subnets {
        for (dc_b, subnets_b) in &dc_subnets {
            if dc_a == dc_b { continue; }

            let mut public_to_private_done = false;
            let mut public_to_public_done = false;
            let mut private_to_private_done = false;
            let mut private_to_public_done = false;

            let mut add_ping_test = |subnet_a: &SubnetInfo, subnet_b: &SubnetInfo| {
                let dc_a_name = db.datacenter().c_dc_name(*dc_a).to_case(convert_case::Case::Snake);
                let dc_b_name = db.datacenter().c_dc_name(*dc_b).to_case(convert_case::Case::Snake);
                let subnet_a_str = subnet_a.ipnet.to_string().replace(".", "_").replace("/", "p");
                let subnet_b_str = subnet_b.ipnet.to_string().replace(".", "_").replace("/", "p");
                let test_name = format!("inter_dc_{dc_a_name}_to_{dc_b_name}_subnet_ping_from_{subnet_a_str}_to_{subnet_b_str}_works");
                let mut server_ips: BTreeMap<String, Vec<String>> = BTreeMap::new();

                for source_ping_host in &subnet_a.ping_hosts {
                    let source_private_ip = db.network_interface().c_if_ip(*source_ping_host);
                    let from_srv = db.network_interface().c_parent(*source_ping_host);
                    assert!(source_private_ip.starts_with("10."), "We assume only private ips here, got {source_private_ip}");
                    let ssh_ip =
                        if let Some(internet_iface) = l1proj.internet_network_iface.get(&from_srv) {
                            db.network_interface().c_if_ip(*internet_iface)
                        } else { source_private_ip };

                    let to_ping = server_ips.entry(ssh_ip.clone()).or_default();
                    for target_ping_host in &subnet_b.ping_hosts {
                        let target_private_ip = db.network_interface().c_if_ip(*target_ping_host);
                        assert!(target_private_ip.starts_with("10."), "We assume only private ips here, got {target_private_ip}");
                        to_ping.push(target_private_ip.clone());
                    }
                }

                runtime.add_integration_test(
                    test_name,
                    IntegrationTest::InsideNodePingWorks { server_ips }
                );
            };

            for subnet_a in subnets_a {
                for subnet_b in subnets_b {
                    let a_public = subnet_a.is_vpn_subnet;
                    let b_public = subnet_b.is_vpn_subnet;
                    match (a_public, b_public) {
                        (true, true) if !public_to_public_done => {
                            public_to_public_done = true;
                            add_ping_test(subnet_a, subnet_b);
                        }
                        (true, false) if !public_to_private_done => {
                            public_to_private_done = true;
                            add_ping_test(subnet_a, subnet_b);
                        }
                        (false, true) if !private_to_public_done => {
                            private_to_public_done = true;
                            add_ping_test(subnet_a, subnet_b);
                        }
                        (false, false) if !private_to_private_done => {
                            private_to_private_done = true;
                            add_ping_test(subnet_a, subnet_b);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let mut port_counter: usize = 42777;
    // iperf tests
    for (iperf_server_dc, iperf_server_subnets) in &dc_subnets {
        let iperf_server_dc_name = db.datacenter().c_dc_name(*iperf_server_dc).replace("-", "_");

        let find_most_private_subnet = |subnets: &[SubnetInfo]| -> Option<usize> {
            for (idx, subnet) in subnets.iter().enumerate() {
                if !subnet.is_vpn_subnet {
                    return Some(idx);
                }
            }

            for (idx, _) in subnets.iter().enumerate() {
                return Some(idx);
            }

            return None;
        };

        let find_most_private_subnet_server = |subnet: &SubnetInfo| -> Option<TableRowPointerNetworkInterface> {
            for host in &subnet.ping_hosts {
                let srv = db.network_interface().c_parent(*host);
                if !l1proj.internet_network_iface.contains_key(&srv) && !db.server().c_is_router(srv) {
                    return Some(*host);
                }
            }

            for host in &subnet.ping_hosts {
                let srv = db.network_interface().c_parent(*host);
                if db.server().c_is_router(srv) {
                    return Some(*host);
                }
            }

            for host in &subnet.ping_hosts {
                return Some(*host);
            }

            return None;
        };

        // first try to find private nodes on both ends
        // then if fails go for any node
        // we want to do that to test as many hops across subnets as possible
        let most_private_server_subnet = find_most_private_subnet(&iperf_server_subnets);
        let most_private_server_host = most_private_server_subnet.map(|idx| find_most_private_subnet_server(&iperf_server_subnets[idx])).flatten();
        let mut iperf_clients: Vec<(String, String)> = Vec::new();
        for (iperf_client_dc, iperf_client_subnets) in &dc_subnets {
            if iperf_client_dc != iperf_server_dc {
                let most_private_client_subnet = find_most_private_subnet(&iperf_client_subnets);
                let most_private_client_host = most_private_client_subnet.map(|idx| find_most_private_subnet_server(&iperf_client_subnets[idx])).flatten();

                if let Some(most_private_client_host) = most_private_client_host {
                    let client_private_ip = db.network_interface().c_if_ip(most_private_client_host);
                    let client_srv = db.network_interface().c_parent(most_private_client_host);
                    let ssh_ip =
                        if let Some(internet_iface) = l1proj.internet_network_iface.get(&client_srv) {
                            db.network_interface().c_if_ip(*internet_iface)
                        } else { client_private_ip };
                    iperf_clients.push((ssh_ip.clone(), client_private_ip.clone()))
                }
            }
        }

        if let Some(most_private_server_host) = most_private_server_host {
            if !iperf_clients.is_empty() {
                let server_private_ip = db.network_interface().c_if_ip(most_private_server_host);
                let server_srv = db.network_interface().c_parent(most_private_server_host);
                let ssh_ip =
                    if let Some(internet_iface) = l1proj.internet_network_iface.get(&server_srv) {
                        db.network_interface().c_if_ip(*internet_iface)
                    } else { server_private_ip };
                let test_name = format!("iperf_to_dc_{iperf_server_dc_name}_from_all_dcs_preserves_source_ip");
                let port_range_start = port_counter;
                port_counter += 1 + iperf_clients.len();
                runtime.add_integration_test(
                    test_name,
                    IntegrationTest::CrossDcSourceIpCheck {
                        port_range_start,
                        server_to_run_iperf_server_from_with_private_ip: (
                            ssh_ip.clone(), server_private_ip.clone()
                        ),
                        servers_to_run_iperf_client_from_with_expected_ips: iperf_clients,
                    }
                );
            }
        }
    }
}

fn networking_all_dc_internet_and_dns_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    for dc in db.datacenter().rows_iter() {
        let dc_name = db.datacenter().c_dc_name(dc).to_case(convert_case::Case::Snake);
        let region_name = db.region().c_region_name(db.datacenter().c_region(dc)).to_case(convert_case::Case::Snake);
        let mut public_dc_nodes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut internal_dc_routers: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut private_dc_nodes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut public_dc_nodes_dns: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut internal_dc_routers_dns: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut private_dc_nodes_dns: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for server in db.datacenter().c_referrers_server__dc(dc) {
            let internet_iface = l1proj.internet_network_iface.get(server);
            let lan_iface = l1proj.consul_network_iface.value(*server);
            let is_dcrouter = db.server().c_is_router(*server);
            let private_ip = db.network_interface().c_if_ip(*lan_iface);
            if let Some(internet_iface) = internet_iface {
                let internet_ip = db.network_interface().c_if_ip(*internet_iface);
                assert!(public_dc_nodes.insert(internet_ip.clone(), vec!["1.1.1.1".to_string()]).is_none());
                assert!(public_dc_nodes_dns.insert(internet_ip.clone(), vec!["example.com".to_string()]).is_none());
            } else if is_dcrouter {
                assert!(internal_dc_routers.insert(private_ip.clone(), vec!["1.1.1.1".to_string()]).is_none());
                assert!(internal_dc_routers_dns.insert(private_ip.clone(), vec!["example.com".to_string()]).is_none());
            } else {
                assert!(private_dc_nodes.insert(private_ip.clone(), vec!["1.1.1.1".to_string()]).is_none());
                assert!(private_dc_nodes_dns.insert(private_ip.clone(), vec!["example.com".to_string()]).is_none());
            }
        }

        if !public_dc_nodes.is_empty() {
            let public_node_internet_test = format!(
                "public_nodes_have_internet_region_{region_name}_dc_{dc_name}"
            );
            runtime.add_integration_test(
                public_node_internet_test,
                IntegrationTest::InsideNodePingWorks { server_ips: public_dc_nodes }
            );

            let public_node_dns_test = format!(
                "public_nodes_resolve_public_dns_region_{region_name}_dc_{dc_name}"
            );
            runtime.add_integration_test(
                public_node_dns_test,
                IntegrationTest::InsideNodeDnsAResolutionWorks { server_ips: public_dc_nodes_dns }
            );
        }

        if !internal_dc_routers.is_empty() {
            let public_node_internet_test = format!(
                "dcrouter_nodes_have_internet_region_{region_name}_dc_{dc_name}"
            );
            runtime.add_integration_test(
                public_node_internet_test,
                IntegrationTest::InsideNodePingWorks { server_ips: internal_dc_routers }
            );

            let public_node_dns_test = format!(
                "dcrouter_nodes_resolve_public_dns_region_{region_name}_dc_{dc_name}"
            );
            runtime.add_integration_test(
                public_node_dns_test,
                IntegrationTest::InsideNodeDnsAResolutionWorks { server_ips: internal_dc_routers_dns }
            );
        }

        if !private_dc_nodes.is_empty() {
            let public_node_internet_test = format!(
                "internal_nodes_have_internet_region_{region_name}_dc_{dc_name}"
            );
            runtime.add_integration_test(
                public_node_internet_test,
                IntegrationTest::InsideNodePingWorks { server_ips: private_dc_nodes }
            );

            let public_node_dns_test = format!(
                "internal_nodes_resolve_public_dns_region_{region_name}_dc_{dc_name}"
            );
            runtime.add_integration_test(
                public_node_dns_test,
                IntegrationTest::InsideNodeDnsAResolutionWorks { server_ips: private_dc_nodes_dns }
            );
        }
    }
}

pub fn ensure_no_double_use_minio_buckets(
    db: &Database,
    wirings: &HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationS3Bucket, TableRowPointerMinioBucket>>
) -> Result<(), PlatformValidationError> {
    let mut bucket_users: HashMap<TableRowPointerMinioBucket, String> = HashMap::new();

    for dri in db.docker_registry_instance().rows_iter() {
        let region = db.docker_registry_instance().c_region(dri);
        let region_name = db.region().c_region_name(region);
        let this_usage = format!("docker_registry_instance, region: {region_name}");
        let bucket = db.docker_registry_instance().c_minio_bucket(dri);
        let minio_cluster = db.minio_bucket().c_parent(bucket);
        let minio_cluster_name = db.minio_cluster().c_cluster_name(minio_cluster);
        if let Some(previous_usage) = bucket_users.get(&bucket) {
            return Err(PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
                minio_cluster: minio_cluster_name.clone(),
                minio_bucket: db.minio_bucket().c_bucket_name(bucket).clone(),
                previous_usage: previous_usage.clone(),
                clashing_usage: this_usage,
            });
        }

        assert!(bucket_users.insert(bucket, this_usage).is_none());
    }

    for loki_cluster in db.loki_cluster().rows_iter() {
        let cluster_name = db.loki_cluster().c_cluster_name(loki_cluster);
        let this_usage = format!("loki_cluster, cluster name: {cluster_name}");
        let bucket = db.loki_cluster().c_storage_bucket(loki_cluster);
        let minio_cluster = db.minio_bucket().c_parent(bucket);
        let minio_cluster_name = db.minio_cluster().c_cluster_name(minio_cluster);

        if let Some(previous_usage) = bucket_users.get(&bucket) {
            return Err(PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
                minio_cluster: minio_cluster_name.clone(),
                minio_bucket: db.minio_bucket().c_bucket_name(bucket).clone(),
                previous_usage: previous_usage.clone(),
                clashing_usage: this_usage,
            });
        }

        assert!(bucket_users.insert(bucket, this_usage).is_none());
    }

    for tempo_cluster in db.tempo_cluster().rows_iter() {
        let cluster_name = db.tempo_cluster().c_cluster_name(tempo_cluster);
        let this_usage = format!("tempo_cluster, cluster name: {cluster_name}");
        let bucket = db.tempo_cluster().c_storage_bucket(tempo_cluster);
        let minio_cluster = db.minio_bucket().c_parent(bucket);
        let minio_cluster_name = db.minio_cluster().c_cluster_name(minio_cluster);

        if let Some(previous_usage) = bucket_users.get(&bucket) {
            return Err(PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
                minio_cluster: minio_cluster_name.clone(),
                minio_bucket: db.minio_bucket().c_bucket_name(bucket).clone(),
                previous_usage: previous_usage.clone(),
                clashing_usage: this_usage,
            });
        }

        assert!(bucket_users.insert(bucket, this_usage).is_none());
    }

    for depl in db.backend_application_deployment().rows_iter() {
        let wirings = wirings.get(&depl).unwrap();
        for (k, bucket) in wirings {
            let app_bucket_name = db.backend_application_s3_bucket().c_bucket_name(*k);
            let this_usage = format!("backend_application_deployment, app s3 bucket: {app_bucket_name}");
            let minio_cluster = db.minio_bucket().c_parent(*bucket);
            let minio_cluster_name = db.minio_cluster().c_cluster_name(minio_cluster);

            if let Some(previous_usage) = bucket_users.get(bucket) {
                return Err(PlatformValidationError::MinIODoubleUseOfExclusiveBucketDetected {
                    minio_cluster: minio_cluster_name.clone(),
                    minio_bucket: db.minio_bucket().c_bucket_name(*bucket).clone(),
                    previous_usage: previous_usage.clone(),
                    clashing_usage: this_usage,
                });
            }
        }
    }
    // we want to be mindful that applications are last here
    // we don't insert applications not to clash with themselves,
    // because we assume that users who use buckets from multiple
    // deployments know what they're doing
    drop(bucket_users);

    Ok(())
}

pub fn libvirt_network_topology(
    db: &Database,
    subnets_to_interfaces_map: &BTreeMap<Ipv4Net, SubnetWithHosts>,
) -> LibvirtNetworkTopology {
    let mut res = LibvirtNetworkTopology { networks: Default::default() };
    let mut virbr_counter = 6;

    let mut hashes: HashSet<[u8; 6]> = HashSet::new();

    let mut needs_libvirt_networks = false;
    for dc in db.datacenter().rows_iter() {
        if db.datacenter().c_implementation(dc) == "testvms" {
            needs_libvirt_networks = true;
        }
    }

    for (k, v) in subnets_to_interfaces_map {
        let is_coproc = db.datacenter().c_implementation(v.dc) == "coprocessor";

        let net_name = db.network().c_network_name(v.network);
        // Don't include VPN network interface
        // which is automatically configured in wireguard
        if net_name == "vpn" {
            continue;
        }

        // virtual ips for coprocessor network
        // put on single interface
        if is_coproc && net_name == "lan" {
            continue;
        }

        if let Some(_first_iface) = &v.interfaces.iter().next() {
            if !needs_libvirt_networks {
                // only relevant for testvms dcs
                continue;
            }
        }

        // dcrouter network is same in all dcs by cidr
        // but must be split in libvirt
        virbr_counter += 1;
        let virbr_name = format!("virbr{}", virbr_counter);
        let gw_ip = k.hosts().take(1).next().unwrap();

        let mut network = LibvirtNetwork {
            libvirt_name: virbr_name.clone(),
            servers: Default::default(),
            gw_ip,
            ip_net: k.clone(),
            network: v.network,
            dhcp_enable: true,
        };

        for net_int in &v.interfaces {
            let server = db.network_interface().c_parent(*net_int);
            let hostname = db.server().c_hostname(server);
            let if_name = db.network_interface().c_if_name(*net_int);
            let mut hash = hmac_sha256::Hash::new();
            hash.update(hostname.as_bytes());
            hash.update(if_name);
            let hash = hash.finalize();
            let mut mac_bytes: [u8; 6] = [0; 6];
            mac_bytes.copy_from_slice(&hash[0..6]);
            mac_bytes[0] &= 0xfe; // set unicast bit to 0
            mac_bytes[0] |= 0x02; // set locally administered bit to 1
            let mac = eui48::MacAddress::from_bytes(&mac_bytes).unwrap();
            // I'm paranoid for 48 bit number collisions and I'm not ashamed of that at all
            assert!(hashes.insert(mac_bytes), "well lucky you boi... 48 bit collision, gz");
            let ip = std::net::Ipv4Addr::from_str(db.network_interface().c_if_ip(*net_int)).unwrap();

            assert!(network.servers.insert(server, LibvirtServer {
                mac, ip
            }).is_none());
        }

        if !network.servers.is_empty() {
            assert!(res.networks.insert(virbr_name, network).is_none());
        } else {
            // undo increment
            virbr_counter -= 1;
        }
    }

    res
}

pub fn vpn_gateways(
    db: &crate::database::Database,
    internet_interfaces: &HashMap<TableRowPointerServer, TableRowPointerNetworkInterface>,
    vpn_interfaces: &HashMap<TableRowPointerServer, TableRowPointerNetworkInterface>,
) -> Result<BTreeMap<TableRowPointerDatacenter, DcVpnGateways>, super::PlatformValidationError> {
    let settings = super::get_global_settings(db);
    let enable_errors = !settings.disable_vpn_gateway_tests;
    let mut final_res: BTreeMap<TableRowPointerDatacenter, DcVpnGateways> = BTreeMap::new();

    for dc in db.datacenter().rows_iter() {
        let mut res = DcVpnGateways { servers: Default::default() };

        for server in db.datacenter().c_referrers_server__dc(dc) {
            if db.server().c_is_vpn_gateway(*server) {
                let mut internet_interface = None;
                let mut vpn_interface = None;
                if let Some(iface) = internet_interfaces.get(server) {
                    internet_interface = Some(*iface);
                }

                if let Some(iface) = vpn_interfaces.get(server) {
                    if db.network_interface().c_if_name(*iface) != "wg0" {
                        return Err(super::PlatformValidationError::VpnGatewayServerVpnInterfaceNameMustBeWg0 {
                            server: db.server().c_hostname(*server).clone(),
                            network: "vpn".to_string(),
                            actual_interface_name: db.network_interface().c_if_name(*iface).clone(),
                            expected_interface_name: "wg0".to_string(),
                        });
                    }
                    vpn_interface = Some(*iface);
                } else if enable_errors {
                    return Err(super::PlatformValidationError::VpnGatewayServerMustHaveVpnInterface {
                        server: db.server().c_hostname(*server).clone(),
                        server_dc: db.datacenter().c_dc_name(dc).clone(),
                        network_interfaces: db.server().c_children_network_interface(*server)
                                                       .iter().map(|i| db.network().c_network_name(db.network_interface().c_if_network(*i)).clone()).collect(),
                    });
                }

                if let (iiface, Some(vpn_interface)) = (internet_interface, vpn_interface) {
                    assert!(res.servers.insert(*server, VpnGateway {
                        internet_interface: iiface,
                        vpn_interface,
                    }).is_none());
                }
            }
        }

        assert!(final_res.insert(dc, res).is_none());
    }

    Ok(final_res)
}

pub fn vpn_p2p_links(
    db: &crate::database::Database,
    dc_topology: &BTreeMap<TableRowPointerDatacenter, DcVpnGateways>,
) -> Result<BTreeMap<TableRowPointerServer, Vec<VpnGateway>>, PlatformValidationError> {
    let mut res: BTreeMap<TableRowPointerServer, Vec<VpnGateway>> = BTreeMap::new();
    let settings = super::get_global_settings(db);
    let enable_errors = !settings.disable_vpn_gateway_tests;

    let no_coproc_dcs = || {
        db.datacenter().rows_iter().filter(|i| {
            db.datacenter().c_implementation(*i) != "coprocessor"
        })
    };

    for left_dc in no_coproc_dcs() {
        for right_dc in no_coproc_dcs() {
            if left_dc != right_dc {
                let left_vpn_gws = dc_topology.get(&left_dc).unwrap();
                let right_vpn_gws = dc_topology.get(&right_dc).unwrap();

                let mut left_iter = left_vpn_gws.servers.values();
                let mut right_iter = right_vpn_gws.servers.values();

                while let (Some(left), Some(right)) = (left_iter.next(), right_iter.next()) {
                    if enable_errors && left.internet_interface.is_none() && right.internet_interface.is_none() {
                        let left_srv = db.network_interface().c_parent(left.vpn_interface);
                        let right_srv = db.network_interface().c_parent(right.vpn_interface);
                        return Err(super::PlatformValidationError::VpnGatewayServerPairMustHaveAtLeastOneInternetInterface {
                            server_a: db.server().c_hostname(left_srv).clone(),
                            server_b: db.server().c_hostname(right_srv).clone(),
                            server_a_has_internet_interface: false,
                            server_b_has_internet_interface: false,
                        });
                    }

                    let srv = db.network_interface().c_parent(left.vpn_interface);
                    let e = res.entry(srv).or_default();
                    e.push(right.clone());
                }
            }
        }
    }

    Ok(res)
}

pub fn first_two_octets(ip_addr: &str) -> String {
    ip_addr.split(".").take(2).collect::<Vec<_>>().join(".")
}

#[test]
fn test_first_two_octet_ip_extraction() {
    assert_eq!(first_two_octets("10.18.12.13"), "10.18");
}

pub fn first_three_octets(ip_addr: &str) -> String {
    ip_addr.split(".").take(3).collect::<Vec<_>>().join(".")
}

#[test]
fn test_first_three_octet_ip_extraction() {
    assert_eq!(first_three_octets("10.18.12.13"), "10.18.12");
}

pub fn get_dc_parameters(input: &str) -> DcParameters {
    let res = match input {
        "aws" => DcParameters {
            is_epl_responsible_for_inter_subnet_routing: false,
            is_epl_responsible_for_internal_node_internet: true,
            is_same_dcimpl_connection_managed_by_provider: true,
            are_floating_ips_available_in_subnets: false,
            are_public_ips_hidden: true,
            provides_admin_vpn_access: true,
            use_l3_hop_for_vpn_gateways: false,
            can_have_more_than_one_subnet: true,
            interfaces_need_vlan: false,
            disk_id_transform: None,
        },
        "gcloud" => DcParameters {
            is_epl_responsible_for_inter_subnet_routing: false,
            is_epl_responsible_for_internal_node_internet: false,
            is_same_dcimpl_connection_managed_by_provider: true,
            are_floating_ips_available_in_subnets: false,
            are_public_ips_hidden: true,
            provides_admin_vpn_access: true,
            can_have_more_than_one_subnet: true,
            // I wish google cloud garbage didn't exist and supported l2 traffic
            use_l3_hop_for_vpn_gateways: true,
            interfaces_need_vlan: false,
            // google cloud faggots
            disk_id_transform: Some("/dev/disk/by-id/google-DISK_ID".to_string()),
        },
        // qemu testing environment, should be same are bare metals
        "testvms" => DcParameters {
            is_epl_responsible_for_inter_subnet_routing: true,
            is_epl_responsible_for_internal_node_internet: true,
            is_same_dcimpl_connection_managed_by_provider: false,
            are_floating_ips_available_in_subnets: true,
            are_public_ips_hidden: false,
            provides_admin_vpn_access: false,
            use_l3_hop_for_vpn_gateways: false,
            can_have_more_than_one_subnet: true,
            interfaces_need_vlan: false,
            disk_id_transform: None,
        },
        // we assume here we are using bare metals
        "manual" => DcParameters {
            is_epl_responsible_for_inter_subnet_routing: true,
            is_epl_responsible_for_internal_node_internet: true,
            is_same_dcimpl_connection_managed_by_provider: false,
            are_floating_ips_available_in_subnets: true,
            are_public_ips_hidden: false,
            provides_admin_vpn_access: true,
            use_l3_hop_for_vpn_gateways: false,
            can_have_more_than_one_subnet: true,
            interfaces_need_vlan: false,
            disk_id_transform: None,
        },
        "coprocessor" => DcParameters {
            is_epl_responsible_for_inter_subnet_routing: false,
            is_epl_responsible_for_internal_node_internet: false,
            is_same_dcimpl_connection_managed_by_provider: false,
            are_floating_ips_available_in_subnets: false,
            are_public_ips_hidden: false,
            provides_admin_vpn_access: false,
            use_l3_hop_for_vpn_gateways: false,
            can_have_more_than_one_subnet: true,
            interfaces_need_vlan: false,
            disk_id_transform: None,
        },
        "bm_simple" => DcParameters {
            is_epl_responsible_for_inter_subnet_routing: false,
            is_epl_responsible_for_internal_node_internet: false,
            is_same_dcimpl_connection_managed_by_provider: false,
            are_floating_ips_available_in_subnets: true,
            are_public_ips_hidden: true,
            provides_admin_vpn_access: false,
            use_l3_hop_for_vpn_gateways: false,
            can_have_more_than_one_subnet: false,
            interfaces_need_vlan: false,
            disk_id_transform: None,
        },
        "hetzner" => DcParameters {
            is_epl_responsible_for_inter_subnet_routing: true,
            is_epl_responsible_for_internal_node_internet: false,
            // it is kind of by hetzner provider vlan, but we do the wiring
            // so its kind of by us
            is_same_dcimpl_connection_managed_by_provider: false,
            are_floating_ips_available_in_subnets: true,
            are_public_ips_hidden: false,
            provides_admin_vpn_access: true,
            use_l3_hop_for_vpn_gateways: false,
            can_have_more_than_one_subnet: true,
            interfaces_need_vlan: true,
            disk_id_transform: None,
        },
        _ => panic!("Unknown dc {input}")
    };
    if let Some(did) = &res.disk_id_transform {
        assert!(did.contains("DISK_ID"));
    }
    res
}

struct FloatingIp {
    uses: usize,
    ip_address: std::net::Ipv4Addr,
}

fn validate_router_floating_ips(db: &Database) -> Result<BTreeMap<ipnet::Ipv4Net, FloatingIp>, PlatformValidationError> {
    let lan_net: ipnet::Ipv4Net = "10.0.0.0/8".parse().unwrap();
    let mut floating_ips_usage: BTreeMap<ipnet::Ipv4Net, FloatingIp> = BTreeMap::new();
    for ip in db.subnet_router_floating_ip().rows_iter() {
        let str_addr = db.subnet_router_floating_ip().c_ip_address(ip);
        let addr: ipnet::Ipv4Net = str_addr.parse().map_err(|e: ipnet::AddrParseError| {
            PlatformValidationError::InvalidSubnetRouterFloatingIp {
                value: str_addr.clone(),
                parsing_error: e.to_string(),
                valid_example: "10.17.49.2/24".to_string(),
            }
        })?;

        if !lan_net.contains(&addr) {
            return Err(PlatformValidationError::SubnetRouterFloatingIpDoesntBelongToLanNetwork {
                value: addr.to_string(),
                expected_to_be_in_network: lan_net.to_string(),
            });
        }

        if addr.prefix_len() != 24 {
            return Err(PlatformValidationError::SubnetRouterFloatingIpInvalidPrefixLength {
                value: addr.to_string(),
                expected_prefix_length: "/24".to_string(),
            });
        }

        let network = addr.trunc();

        if network == addr {
            return Err(PlatformValidationError::SubnetRouterFloatingIpCannotBeNetworkAddress {
                value: addr.to_string(),
                network_address: network.to_string(),
            });
        }

        if addr.addr() == addr.broadcast() {
            return Err(PlatformValidationError::SubnetRouterFloatingIpCannotBeBroadcastAddress {
                value: addr.to_string(),
                broadcast_address: addr.to_string(),
            });
        }

        // ip math is difficult and this is simple
        if str_addr.ends_with(".1/24") {
            return Err(PlatformValidationError::SubnetRouterFloatingIpCannotBeFirstAddressInNetwork {
                value: addr.to_string(),
            });
        }


        if let Some(existing) = floating_ips_usage.get(&network) {
            return Err(PlatformValidationError::SubnetRouterTwoFloatingIpsFoundForNetwork {
                network: network.to_string(),
                floating_ip_a: existing.ip_address.to_string(),
                floating_ip_b: addr.addr().to_string(),
            });
        }

        assert!(floating_ips_usage.insert(network, FloatingIp { uses: 0, ip_address: addr.addr(), }).is_none());
    }

    Ok(floating_ips_usage)
}

fn is_subnet_routing_needed(
    subnets_len: usize,
    params: &DcParameters,
    wireguard_across_dc_needed: bool,
) -> bool {
    subnets_len > 1 &&
        ( params.is_epl_responsible_for_inter_subnet_routing
          || params.is_epl_responsible_for_internal_node_internet
          || wireguard_across_dc_needed )
}

fn networking_answers(
    db: &Database,
    subnets_with_hosts: &BTreeMap<Ipv4Net, SubnetWithHosts>,
    wireguard_across_dc_needed: bool,
    datacenters_with_native_routing: &BTreeSet<TableRowPointerDatacenter>,
) -> Result<NetworkingAnswers, PlatformValidationError> {
    let mut dcs = BTreeMap::new();
    let mut network_floating_ips = validate_router_floating_ips(db)?;
    let router_network_exists =
        db.network().rows_iter().filter(|n| db.network().c_network_name(*n) == "dcrouter").next().is_some();

    let settings = super::get_global_settings(db);
    let enable_vpn_gw_errors = !settings.disable_vpn_gateway_tests;

    for dc in db.datacenter().rows_iter() {
        let is_dc_empty = db.datacenter().c_referrers_server__dc(dc).is_empty();
        let is_coprocessor_dc = db.datacenter().c_implementation(dc) == "coprocessor";
        let params = get_dc_parameters(&db.datacenter().c_implementation(dc));
        let dc_network = ipnet::Ipv4Net::from_str(db.datacenter().c_network_cidr(dc)).unwrap();
        let dc_subnets =
            subnets_with_hosts
                .iter()
                .filter(|(subnet, sv)| {
                    db.network().c_network_name(sv.network) == "lan" && dc_network.contains(*subnet)
                })
                .collect::<Vec<_>>();

        // what if DC native?
        let is_subnet_routing_needed = is_subnet_routing_needed(
            dc_subnets.len(), &params, wireguard_across_dc_needed
        ) && !is_coprocessor_dc;
        // VPN node is always internet gateway and the subnet gateway
        let is_vpn_node_same_as_routing_node = true;
        let is_private_node_manual_internet_routing_needed =
            params.is_epl_responsible_for_internal_node_internet;
        // we need to route traffic manually to gateways only if:
        // 1. there's other wireguard DCs
        // 2. we need internet
        // 3. we have multiple subnets to route to
        let is_private_node_to_gw_routing_needed =
            is_private_node_manual_internet_routing_needed ||
            wireguard_across_dc_needed ||
            is_subnet_routing_needed;
        let is_consul_vrrp =
            is_private_node_to_gw_routing_needed &&
            !params.are_floating_ips_available_in_subnets;
        let is_hardware_vrrp =
            is_private_node_to_gw_routing_needed &&
            params.are_floating_ips_available_in_subnets;
        let is_floating_subnet_ip_needed = is_hardware_vrrp;
        // we need routing if:
        // 1. epl is reponsible for routing between subnets
        // 2. we have any wireguard between datacenter
        // keepalived is separate from routing, just two gateways that refresh static routes in consul
        // if is_private_node_manual_internet_routing_needed then every subnet must have nodes with public ip
        let is_ospf_routing_needed =
            is_subnet_routing_needed || wireguard_across_dc_needed;
        let has_managed_routing_to_other_dcs = datacenters_with_native_routing.contains(&dc);

        if is_subnet_routing_needed && !router_network_exists && !is_dc_empty {
            return Err(PlatformValidationError::IntraDcRoutingNeededButNoDcrouterNetworkExists {
                dc: db.datacenter().c_dc_name(dc).clone(),
                subnet_count: dc_subnets.len(),
                missing_network: "dcrouter".to_string(),
            });
        }

        // move this earlier to fix the tests,
        // VPN error is more specific than error about
        // one subnet router later, especially because
        // vpn being subnet router is implicit
        let vpn_servers_in_dc =
            db.datacenter().c_referrers_server__dc(dc)
                           .iter().filter(|srv| db.server().c_is_vpn_gateway(**srv))
                                  .collect::<Vec<_>>();
        // VPN servers must always exist, at minimum to
        // use wireguard for provisioner admin instance
        // behind NAT
        if !is_coprocessor_dc && vpn_servers_in_dc.len() != 2 && enable_vpn_gw_errors && !is_dc_empty {
            return Err(super::PlatformValidationError::DcMustHaveExactlyTwoVpnGateways {
                dc: db.datacenter().c_dc_name(dc).clone(),
                actual_count: vpn_servers_in_dc.len(),
                expected_count: 2,
                vpn_gateway_servers: vpn_servers_in_dc
                    .iter()
                    .map(|i| db.server().c_hostname(**i).clone())
                    .collect(),
            });
        }

        // cloud manages its own routing, so we should set static route to the
        // default gateway
        let should_overshadow_ospf_dc_blackhole_route = !is_subnet_routing_needed;
        // in cloud they should?
        // but on prem? Maybe let routers figure it out via OSPF
        // and it is meaningless?
        // If we're responsible for private node internet there
        // must be at least two servers in the datacenter with
        // public ip that participate in routing.
        // Subnet must have either 0 or 2 VPN servers with public ip.
        // VPN server is also AlwAYS a subnet router

        let mut subnets = BTreeMap::new();
        let mut routers_with_internet_interfaces = Vec::new();
        for (subnet, subnet_values) in dc_subnets {
            let mut declared_vpn_gateways = Vec::new();
            let mut declared_routers = Vec::new();
            let mut vpn_interfaces = Vec::new();
            let mut routing_interfaces = Vec::new();
            let floating_ip = if is_floating_subnet_ip_needed {
                if let Some(floating_ip) = network_floating_ips.get_mut(subnet) {
                    floating_ip.uses += 1;
                    Some(floating_ip.ip_address)
                } else {
                    return Err(PlatformValidationError::SubnetRouterFloatingIpForSubnetNotFound {
                        subnet: subnet.to_string(),
                    });
                }
            } else { None };

            let floating_ip_str = floating_ip.as_ref().map(|i| i.to_string());
            for subnet_interface in &subnet_values.interfaces {
                let server = db.network_interface().c_parent(*subnet_interface);
                let srv_private_ip = db.network_interface().c_if_ip(*subnet_interface);
                if let Some(floating_ip_str) = &floating_ip_str {
                    if floating_ip_str == srv_private_ip {
                        return Err(PlatformValidationError::SubnetRouterFloatingIpClashWithServerIp {
                            server: db.server().c_hostname(server).clone(),
                            server_ip: srv_private_ip.clone(),
                            subnet_router_floating_ip: format!("{floating_ip_str}/24"),
                        });
                    }
                }

                let vpn_iface = db.server().c_children_network_interface(server).iter().filter(|i| {
                    db.network().c_network_name(db.network_interface().c_if_network(**i)) == "vpn"
                }).next().cloned();
                let dcrouting_iface = db.server().c_children_network_interface(server).iter().filter(|i| {
                    db.network().c_network_name(db.network_interface().c_if_network(**i)) == "dcrouter"
                }).next().cloned();
                let internet_iface = db.server().c_children_network_interface(server).iter().filter(|i| {
                    db.network().c_network_name(db.network_interface().c_if_network(**i)) == "internet"
                }).next().cloned();
                let interfaces = ServerInterfaces {
                    lan_iface: *subnet_interface,
                    vpn_iface,
                    dcrouting_iface,
                    internet_iface,
                    server,
                };
                match &(interfaces.dcrouting_iface, interfaces.internet_iface) {
                    &(Some(router_iface), Some(_)) => {
                        routers_with_internet_interfaces.push(router_iface);
                    }
                    _ => {}
                }
                let is_declared_vpn_gateway = db.server().c_is_vpn_gateway(server);
                let is_declared_router = db.server().c_is_router(server);
                let is_vpn_gateway = is_declared_vpn_gateway;
                let is_router = is_declared_router ||
                    (is_vpn_gateway && is_vpn_node_same_as_routing_node);
                if is_router {
                    routing_interfaces.push(interfaces.clone());
                }
                if is_vpn_gateway {
                    vpn_interfaces.push(interfaces);
                }

                if is_declared_vpn_gateway {
                    declared_vpn_gateways.push(server);
                }
                if is_declared_router {
                    declared_routers.push(server);
                }
            }

            if !is_coprocessor_dc && routing_interfaces.len() != 2 {
                let router_servers = routing_interfaces.iter().map(|i| {
                    db.server().c_hostname(db.network_interface().c_parent(i.lan_iface)).clone()
                }).collect::<Vec<_>>();
                return Err(PlatformValidationError::DcRoutingSubnetMustHaveExactlyTwoRouters {
                    dc: db.datacenter().c_dc_name(dc).clone(),
                    subnet: subnet.to_string(),
                    router_servers,
                    expected_router_servers: 2,
                });
            }

            if !declared_vpn_gateways.is_empty() && !declared_routers.is_empty() {
                return Err(PlatformValidationError::DcRoutingSubnetCannotMixDeclaredVpnGatewaysAndDeclaredRouters {
                    dc: db.datacenter().c_dc_name(dc).clone(),
                    subnet: subnet.to_string(),
                    declared_vpn_gateways_found: declared_vpn_gateways.into_iter().map(|i| db.server().c_hostname(i)).cloned().collect::<Vec<_>>(),
                    declared_router_server_found: declared_routers.into_iter().map(|i| db.server().c_hostname(i)).cloned().collect::<Vec<_>>(),
                });
            }

            let subnet_answers = SubnetNetworkingAnswers {
                vpn_interfaces,
                routing_interfaces,
                floating_ip,
            };

            assert!(subnets.insert(subnet.clone(), subnet_answers).is_none());
        }

        let dc_answers = DcNetworkingAnswers {
            params,
            is_subnet_routing_needed,
            is_private_node_manual_internet_routing_needed,
            is_consul_vrrp,
            is_hardware_vrrp,
            is_floating_subnet_ip_needed,
            is_ospf_routing_needed,
            is_vpn_node_same_as_routing_node,
            is_private_node_to_gw_routing_needed,
            should_overshadow_ospf_dc_blackhole_route,
            subnets,
            has_managed_routing_to_other_dcs,
            routers_with_internet_interfaces,
        };

        assert!(dcs.insert(dc, dc_answers).is_none());
    }

    for nfi in network_floating_ips.values() {
        if nfi.uses == 0 {
            return Err(PlatformValidationError::SubnetRouterFloatingIpDefinedButNeverUsed {
                subnet_router_floating_ip: format!("{}/24", nfi.ip_address.to_string()),
            });
        }
    }

    Ok(NetworkingAnswers { dcs })
}

pub fn check_servers_regional_distribution(
    db: &Database, region: TableRowPointerRegion,
    servers: impl Iterator<Item = TableRowPointerServer>,
    context: String,
) -> Result<(), PlatformValidationError> {
    if db.region().c_availability_mode(region) != "multi_dc" {
        return Ok(());
    }

    let mut dc_distribution: BTreeMap<TableRowPointerDatacenter, Vec<TableRowPointerServer>> = BTreeMap::new();

    let mut total_servers: usize = 0;
    for server in servers {
        let dc = db.server().c_dc(server);
        let v = dc_distribution.entry(dc).or_default();
        v.push(server);
        total_servers += 1;
    }

    assert!(total_servers >= 2);

    let mk_buckets = || {
        let mut res: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for (k, v) in &dc_distribution {
            let dc_name = db.datacenter().c_dc_name(*k).clone();
            let other_v = res.entry(dc_name).or_default();
            for iv in v {
                other_v.push(db.server().c_hostname(*iv).clone());
            }
        }

        res
    };

    let min_dcs =
        if total_servers <= 2 {
            2
        } else {
            3
        };

    if dc_distribution.len() < min_dcs {
        return Err(PlatformValidationError::ApplicationInsideMultiDcRegionIsNotDistributedAcrossEnoughDatacenters {
            region: db.region().c_region_name(region).clone(),
            region_availability_mode: db.region().c_availability_mode(region).clone(),
            found_dcs: dc_distribution.len(),
            min_dcs,
            context,
            application_servers_buckets: mk_buckets(),
        });
    }

    let mut min_cnt: usize = 0;
    let mut min_cnt_dc: Option<TableRowPointerDatacenter> = None;
    let mut max_cnt: usize = 0;
    let mut max_cnt_dc: Option<TableRowPointerDatacenter> = None;

    for (dc, vec) in &dc_distribution {
        if min_cnt_dc.is_none() || vec.len() < min_cnt {
            min_cnt = vec.len();
            min_cnt_dc = Some(*dc);
        }
        if max_cnt_dc.is_none() || vec.len() > max_cnt {
            max_cnt = vec.len();
            max_cnt_dc = Some(*dc);
        }
    }

    match (min_cnt_dc, max_cnt_dc) {
        (Some(min_dc), Some(max_dc)) => {
            let difference = max_cnt - min_cnt;
            let maximum_allowed_difference = 1;
            if difference > maximum_allowed_difference {
                return Err(PlatformValidationError::ApplicationInsideMultiDcRegionIsDistributedDistributedNonEqually {
                    region: db.region().c_region_name(region).clone(),
                    region_availability_mode: db.region().c_availability_mode(region).clone(),
                    found_dcs: dc_distribution.len(),
                    dc_with_lowest_nodes: db.datacenter().c_dc_name(min_dc).clone(),
                    dc_with_lowest_nodes_count: min_cnt,
                    dc_with_most_nodes: db.datacenter().c_dc_name(max_dc).clone(),
                    dc_with_most_nodes_count: max_cnt,
                    difference,
                    maximum_allowed_difference,
                    context,
                    application_servers_buckets: mk_buckets(),
                });
            }
        }
        other => {
            panic!("Should never happen? {:?}", other);
        }
    }

    Ok(())
}

impl DcNetworkingAnswers {
    pub fn all_routers_set(&self) -> BTreeSet<TableRowPointerServer> {
        let mut res = BTreeSet::new();
        for subnet in self.subnets.values() {
            for ri in &subnet.routing_interfaces {
                assert!(res.insert(ri.server));
            }
        }
        res
    }
}
