use std::{fmt::Write, collections::{HashSet, HashMap, BTreeMap}, str::FromStr};
use ipnet::Ipv4Net;

use crate::{static_analysis::{CheckedDB, networking::{CrossDcConnectivity, first_three_octets}}, codegen::{nixplan::{NixAllServerPlans, custom_user_secret_key, root_secret_config, mk_nix_region, NixServerPlan}, secrets::{SecretsStorage, SecretValue}}, database::{TableRowPointerDatacenter, TableRowPointerServer}};

use super::{consul::derive_consul_secrets, utils::{pad_string, gen_reload_service_on_path_change}, disks::generate_deterministic_password_with_seed};

pub fn provision_routing(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &mut SecretsStorage) {
    let bootstrap_routes = provision_routers(db, plans, secrets);
    provision_servers(db, plans, &bootstrap_routes);
    provision_coprocessor_routing(db, plans, secrets);
}

fn bgp_peering_seed(secrets: &mut SecretsStorage) -> SecretValue {
    secrets.fetch_secret(
        "bgp_peering_seed".to_string(),
        crate::codegen::secrets::SecretKind::StrongPassword42Symbols
    )
}

fn derive_bgp_peering_password(db: &CheckedDB, seed: &SecretValue, srv1: TableRowPointerServer, srv2: TableRowPointerServer) -> String {
    let mut hostname_vec = vec![
        db.db.server().c_hostname(srv1),
        db.db.server().c_hostname(srv2),
    ];
    hostname_vec.sort();

    let pw_name = format!("{}.{}", hostname_vec[0], hostname_vec[1]);
    generate_deterministic_password_with_seed(&seed.value(), &pw_name, 42)
}

fn provision_routers(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &mut SecretsStorage) -> HashMap<TableRowPointerDatacenter, String> {
    // deterministic, can be called in other contexts
    let consul_secrets = derive_consul_secrets(db, secrets);

    let bgp_peering_seed = bgp_peering_seed(secrets);
    let bgp_peering_pwd = |srv_a: TableRowPointerServer, srv_b: TableRowPointerServer| -> String {
        derive_bgp_peering_password(db, &bgp_peering_seed, srv_a, srv_b)
    };

    let mut inter_dc_bootstrap_routes: HashMap<TableRowPointerDatacenter, String> = HashMap::new();

    let mut dcs_by_impl: HashMap<&str, Vec<TableRowPointerDatacenter>> = HashMap::new();

    for dc in db.db.datacenter().rows_iter() {
        let dc_impl = db.db.datacenter().c_implementation(dc);
        let dcs = dcs_by_impl.entry(dc_impl.as_str()).or_default();
        dcs.push(dc);
    }

    for dc in db.db.datacenter().rows_iter() {
        let dc_net = db.sync_res.network.networking_answers.dcs.get(&dc).unwrap();
        let inter_dc_routes = inter_dc_routes(db, dc);
        let region = db.db.datacenter().c_region(dc);
        let region_has_coprocessor_dc = db.db.region().c_has_coprocessor_dc(region);
        //let ospf_pwd = secrets.fetch_secret("ospf_vpn_key".to_string(), crate::codegen::secrets::SecretKind::OspfPassword);
        //let vpn_key = ospf_pwd.value();
        let dc_cidr = db.db.datacenter().c_network_cidr(dc);
        let bgp_asn = dc_bgp_as_number(dc_cidr);
        let dc_ip = db.db.datacenter().c_network_cidr(dc).replace("/16", "");
        let dc_name = db.db.datacenter().c_dc_name(dc);
        let dc_routing_pwd = secrets.fetch_secret(format!("ospf_dc_{dc_name}_key"), crate::codegen::secrets::SecretKind::OspfPassword);
        let dc_routing_key = dc_routing_pwd.value();
        let gre_neighbors: Option<GreNeighbors> =
            if dc_net.params.use_l3_hop_for_vpn_gateways {
                Some(generate_datacenter_l3_vpn_gre_neighbors(db, dc))
            } else { None };

        for (subnet_key, rs) in &dc_net.subnets {
            let vrrp_priority = 50;
            for ri in &rs.routing_interfaces {
                let srv = ri.server;
                let mut neighbors: HashSet<String> = HashSet::new();
                let plan = plans.fetch_plan(srv);

                let lan_iface = ri.lan_iface;
                let lan_iface_name = db.db.network_interface().c_if_name(lan_iface);
                let lan_iface_ip = db.db.network_interface().c_if_ip(lan_iface);
                let has_internet = db.projections.internet_network_iface.get(&srv).is_some();
                let mut bfd_cfg = String::new();
                write!(&mut bfd_cfg, r#"
        !
        bfd
          peer {lan_iface_ip}
            no shutdown"#).unwrap();
                let mut ospf_cfg = String::new();
                let mut bgp_cfg = String::new();
                let mut bgp_addr_fam = String::new();
                let mut zebra_cfg = String::new();
                write!(&mut zebra_cfg, r#"
        !
        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
        !
        ip prefix-list ANY seq 100 permit 0.0.0.0/0
        !
        route-map LANRM permit 100
          match ip address prefix-list LAN
          set src {lan_iface_ip}
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol ospf route-map LANRM
        !
        ip protocol bgp route-map LANRM"#).unwrap();

                if !dc_net.params.is_epl_responsible_for_inter_subnet_routing {
                    let mut prefix_cnt = 100;
                    write!(&mut zebra_cfg, r#"
        !
        ip prefix-list INTERSUBNET seq {prefix_cnt} permit {dc_cidr} le 24"#).unwrap();
                    if dc_net.params.is_same_dcimpl_connection_managed_by_provider {
                        if let Some(dcs) = dcs_by_impl.get(db.db.datacenter().c_implementation(dc).as_str()) {
                            for peer_dc in dcs {
                                if dc != *peer_dc {
                                    prefix_cnt += 1;
                                    let peer_dc_cidr = db.db.datacenter().c_network_cidr(dc);
                                    write!(&mut zebra_cfg, r#"
        !
        ip prefix-list INTERSUBNET seq {prefix_cnt} permit {peer_dc_cidr} le 24"#).unwrap();
                                }
                            }
                        }
                    }
                    write!(&mut zebra_cfg, r#"
        !
        route-map LANRM deny 90
          match ip address prefix-list INTERSUBNET"#).unwrap();
                }

                write!(&mut ospf_cfg, r#"
        !
        router ospf
          ospf router-id {lan_iface_ip}
          redistribute bgp
          network {dc_cidr} area {dc_ip}"#).unwrap();
                // this is border router, summarize entire dc
                if db.db.server().c_is_vpn_gateway(srv) {
                    // Add vpn interface if exists
                    if let Some(vpn_iface) = &ri.vpn_iface {
                        let vpn_ip = db.db.network_interface().c_if_ip(*vpn_iface);
                        write!(&mut bfd_cfg, r#"
          peer {vpn_ip}
            no shutdown"#).unwrap();
                    }

                    write!(&mut bgp_addr_fam, r#"
            network {dc_cidr}"#).unwrap();
                    write!(&mut bgp_cfg, r#"
        !
        router bgp {bgp_asn}
          bgp router-id {lan_iface_ip}
          address-family ipv4 unicast
            network {dc_cidr}
          exit-address-family"#).unwrap();
                    // summarise at the border
                    write!(&mut ospf_cfg, r#"
          area {dc_ip} range {dc_cidr} advertise
          area {dc_ip} range 0.0.0.0/0 not-advertise"#).unwrap();
                }

                write!(&mut ospf_cfg, r#"
          area {dc_ip} authentication message-digest"#).unwrap();

                let mut epl_postrouting_rules = String::new();
                if db.db.server().c_is_vpn_gateway(srv) {
                    // don't masquerade vpn traffic
                    write!(&mut epl_postrouting_rules, "
               ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment \"Admin VPN\";
               ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment \"Inter DC EPL traffic\";").unwrap();
                }

                if dc_net.params.is_epl_responsible_for_internal_node_internet && has_internet {
                    // we always need this for admin VPN for google cloud, rewrite
                    write!(&mut ospf_cfg, r#"
          default-information originate always"#).unwrap();
                    write!(&mut epl_postrouting_rules, "
               ip saddr {dc_cidr} ip daddr != {{ 10.0.0.0/8 }} masquerade comment \"Internet for private EPL nodes\";").unwrap();
                }

                if !epl_postrouting_rules.is_empty() {
                    let table_content = format!(r#"
       chain EPL_POSTROUTING {{
           type nat hook postrouting priority 0;
{epl_postrouting_rules}
       }}
"#);
                    create_nftables_table_service(
                        plan,
                        "ip",
                        "epl-nat",
                        &table_content
                    );
                }

                // Add subnet routing neighbors
                for (rs_k, rs_n) in &dc_net.subnets {
                    for r_i in &rs_n.routing_interfaces {
                        let r_srv = r_i.server;
                        if r_srv != srv {
                            // if in same 10.x.x.0/24 subnet use that subnet address
                            let bgp_pass = bgp_peering_pwd(r_srv, srv);
                            if subnet_key == rs_k {
                                // always use dcrouter network if exists
                                if let Some(dcrouting_iface) = &r_i.dcrouting_iface {
                                    let neighbor_ip = db.db.network_interface().c_if_ip(*dcrouting_iface);
                                    if neighbors.insert(neighbor_ip.clone()) {
                                        write!(&mut ospf_cfg, r#"
          neighbor {neighbor_ip}"#).unwrap();
                                        write!(&mut bgp_cfg, r#"
          neighbor {neighbor_ip} remote-as {bgp_asn}
          neighbor {neighbor_ip} password {bgp_pass}
          neighbor {neighbor_ip} bfd"#).unwrap();
                                    }
                                } else {
                                    let neighbor_lan_ip = db.db.network_interface().c_if_ip(r_i.lan_iface);
                                    if neighbors.insert(neighbor_lan_ip.clone()) {
                                        write!(&mut ospf_cfg, r#"
          neighbor {neighbor_lan_ip}"#).unwrap();
                                        write!(&mut bgp_cfg, r#"
          neighbor {neighbor_lan_ip} remote-as {bgp_asn}
          neighbor {neighbor_lan_ip} password {bgp_pass}
          neighbor {neighbor_lan_ip} bfd"#).unwrap();
                                    }
                                }
                            } else {
                                if let Some(dcrouting_iface) = &r_i.dcrouting_iface {
                                    let neighbor_ip = db.db.network_interface().c_if_ip(*dcrouting_iface);
                                    if neighbors.insert(neighbor_ip.clone()) {
                                        write!(&mut ospf_cfg, r#"
          neighbor {neighbor_ip}"#).unwrap();
                                        write!(&mut bgp_cfg, r#"
          neighbor {neighbor_ip} remote-as {bgp_asn}
          neighbor {neighbor_ip} password {bgp_pass}
          neighbor {neighbor_ip} bfd"#).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }

                // Add cross DC wireguard neighbors
                if let Some(vpn_gws) = db.projections.vpn_p2p_links.get(&srv) {
                    for vpn_gw in vpn_gws {
                        let vpn_srv = db.db.network_interface().c_parent(vpn_gw.vpn_interface);
                        let neighbor_lan_ip =
                            db.db.network_interface().c_if_ip(
                                *db.projections.consul_network_iface.value(vpn_srv)
                            );
                        let neighbor_ip = db.db.network_interface().c_if_ip(vpn_gw.vpn_interface);
                        let neighbor_bgp_asn = dc_bgp_as_number(&neighbor_lan_ip);
                        let bgp_pass = bgp_peering_pwd(vpn_srv, srv);
                        if neighbors.insert(neighbor_ip.clone()) {
                            write!(&mut bgp_cfg, r#"
          neighbor {neighbor_ip} remote-as {neighbor_bgp_asn}
          neighbor {neighbor_ip} password {bgp_pass}
          neighbor {neighbor_ip} bfd"#).unwrap();
                        }
                    }
                }

                if region_has_coprocessor_dc {
                    let coproc_region_data = db.sync_res.network.region_coprocessor_gws.get(&region).unwrap();
                    let coproc_gw = coproc_region_data
                        .gateways
                        .iter()
                        .enumerate()
                        .find(|(_idx, this_srv)| **this_srv == srv);
                    let coproc_dc_cidr = db.db.datacenter().c_network_cidr(coproc_region_data.coprocessor_dc);
                    write!(&mut bgp_addr_fam, r#"
            aggregate-address {coproc_dc_cidr}"#).unwrap();
                    if let Some((coproc_idx, _coproc_gw)) = coproc_gw {
                        let coproc_idx = coproc_idx + 1;
                        for neighbor_server in db.db.datacenter().c_referrers_server__dc(coproc_region_data.coprocessor_dc) {
                            let mut ni_idx = 0;
                            let bgp_pass = bgp_peering_pwd(*neighbor_server, srv);
                            for ni in db.db.server().c_children_network_interface(*neighbor_server) {
                                let network = db.db.network_interface().c_if_network(*ni);
                                let network_name = db.db.network().c_network_name(network);
                                if network_name != "vpn" {
                                    continue;
                                }

                                ni_idx += 1;

                                if coproc_idx == ni_idx {
                                    let neighbor_lan_iface = db.projections.consul_network_iface.value(*neighbor_server);
                                    let neighbor_lan_ip = db.db.network_interface().c_if_ip(*neighbor_lan_iface);
                                    let neighbor_bgp_asn = coproc_server_asn_number(&neighbor_lan_ip);
                                    let neighbor_ip = db.db.network_interface().c_if_ip(*ni);
                                    write!(&mut bgp_cfg, r#"
          neighbor {neighbor_ip} remote-as {neighbor_bgp_asn}
          neighbor {neighbor_ip} password {bgp_pass}
          neighbor {neighbor_ip} bfd"#).unwrap();
                                }
                            }
                        }
                    }
                }

                write!(&mut bgp_cfg, r#"
          address-family ipv4 unicast{bgp_addr_fam}
          exit-address-family"#).unwrap();

                // Add subnet routing interfaces if exist
                if let Some(routing_iface) = &ri.dcrouting_iface {
                    let routing_ip = db.db.network_interface().c_if_ip(*routing_iface);
                    let iface_name = db.db.network_interface().c_if_name(*routing_iface);
                    // about costs, we prefer subnet router
                    // network always as lowest cost
                    // then LAN network
                    // then across DC network
                    write!(&mut ospf_cfg, r#"
        !
        interface {iface_name}
          ip ospf cost 100
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 {dc_routing_key}
          ip ospf authentication message-digest
          ip ospf network non-broadcast"#).unwrap();
                    write!(&mut zebra_cfg, r#"
        !
        interface {iface_name}
          ip address {routing_ip}/22"#).unwrap();
                }

                write!(&mut ospf_cfg, r#"
        !
        interface {lan_iface_name}
          ip ospf cost 500
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 {dc_routing_key}
          ip ospf authentication message-digest
          ip ospf network non-broadcast"#).unwrap();

                write!(&mut zebra_cfg, r#"
        !
        interface {lan_iface_name}
          ip address {lan_iface_ip}/24"#).unwrap();

                if dc_net.params.use_l3_hop_for_vpn_gateways && db.db.server().c_is_vpn_gateway(srv) {
                    plan.add_custom_nix_block(r#"
    boot.kernelModules = [ "gre" ];
"#.to_string());

                    let neighbors = gre_neighbors.as_ref().unwrap();
                    plan.add_custom_nix_block(
                        mk_nix_region(
                            "l3_vpn_hop_interface",
                            generate_datacenter_l3_vpn_gre_tunnels(db, srv, &neighbors.non_vpns)
                        )
                    );
                    let ip_translation = &neighbors.inter_dc_hops_for_non_vpns;

                    // google cloud can kiss my ...
                    // but that's what we have to do to avoid faggity
                    // BGP peerings with faggity google cloud
                    create_nftables_table_service(
                        plan,
                        "ip",
                        "l3-vpn-hop-address-translation",
                        &ip_translation
                    );
                }

                if dc_net.is_ospf_routing_needed {
                    plan.add_custom_nix_block(mk_nix_region("frr_ospf_config", format!(r#"
  services.frr.ospf = {{
      enable = true;
      config = ''{ospf_cfg}
      '';
  }};"#)));
                    plan.add_custom_nix_block(mk_nix_region("frr_bfd_config", format!(r#"
  services.frr.bfd = {{
      enable = true;
      config = ''{bfd_cfg}
      '';
  }};"#)));
                    if db.db.server().c_is_vpn_gateway(srv) {
                        plan.add_custom_nix_block(mk_nix_region("frr_bgp_config", format!(r#"
  services.frr.bgp = {{
      enable = true;
      config = ''{bgp_cfg}
      '';
  }};"#)));
                    }
                    plan.add_custom_nix_block(mk_nix_region("frr_zebra_config", format!(r#"
  services.frr.zebra = {{
      enable = true;
      config = ''{zebra_cfg}
      '';
  }};"#)));

                    let mut static_routes = String::new();
                    // this must be first host via route and not through
                    // interface otherwise it will not work with google cloud/aws
                    let first_host = format!(
                        "{}.1",
                        first_three_octets(db.db.network_interface().c_if_ip(lan_iface))
                    );
                    if dc_net.should_overshadow_ospf_dc_blackhole_route
                        || !dc_net.params.is_epl_responsible_for_inter_subnet_routing
                    {

                        // add static route to local datacenter via default interface to
                        // overshadow ospf blackhole route
                        write!(&mut static_routes, r#"
        !
        ip route {dc_cidr} {first_host}
"#).unwrap();
                        if dc_net.params.is_same_dcimpl_connection_managed_by_provider {
                            if let Some(dcs) = dcs_by_impl.get(db.db.datacenter().c_implementation(dc).as_str()) {
                                for peer_dc in dcs {
                                    if dc != *peer_dc {
                                        let peer_dc_cidr = db.db.datacenter().c_network_cidr(*peer_dc);
                                        write!(&mut static_routes, r#"
        !
        ip route {peer_dc_cidr} {first_host}
"#).unwrap();
                                    }
                                }
                            }
                        }
                    }

                    if dc_net.params.are_public_ips_hidden && has_internet {
                        // static default route to first ip
                        write!(&mut static_routes, r#"
        !
        ip route 0.0.0.0/0 {first_host}
"#).unwrap();
                    }

                    if "testvms" == db.db.datacenter().c_implementation(dc) {
                        if let Some(dr_ip) = &db.sync_res.network.test_docker_registry_gw_address {
                            write!(&mut static_routes, r#"
        !
        ip route {dr_ip}/32 {first_host}
"#).unwrap();
                        }
                    }

                    if !static_routes.is_empty() {
                        plan.add_custom_nix_block(mk_nix_region("frr_static_routes", format!(r#"
  # You gotta be kidding me... https://github.com/NixOS/nixpkgs/issues/274286
  services.frr.mgmt.enable = true;
  environment.etc."frr/staticd.conf".text = ''{static_routes}
  '';
  systemd.services.staticd.serviceConfig.ExecStart = lib.mkForce "${{pkgs.frr}}/libexec/frr/staticd -A localhost";
  services.frr.static.enable = true;"#)));
                    }
                }

                let unicast_peers: Vec<String> =
                    rs
                    .routing_interfaces
                    .iter()
                    .filter_map(|unicast_iface| {
                        if unicast_iface.server != srv {
                            Some(db.db.network_interface().c_if_ip(unicast_iface.lan_iface).clone())
                        } else { None }
                    })
                    .collect();
                let unicast_peers = unicast_peers.join("\n");
                let routes = inter_dc_routes.get(subnet_key).unwrap();

                // Will be useful to know state of both nodes if we want to
                // implement equal cost load balancing between them
                // https://serverfault.com/questions/560024/view-current-state-of-keepalived
                let maybe_vrrp_notify =
                    if dc_net.is_consul_vrrp {
                        let routes = routes.iter().find(|i| i.gateway_server == Some(srv)).unwrap();
                        let routes_file = routes.route_script();
                        let subnet_section = subnet_key.to_string().replace("/", "p");
                        plan.add_shell_package("epl-consul-vrrp-switch", &mk_nix_region("consul_vrrp_switch_script", format!(r#"
/run/current-system/sw/bin/echo '{routes_file}' | \
  CONSUL_HTTP_TOKEN=$( ${{pkgs.coreutils}}/bin/cat /run/keys/consul-vrrp-token.txt ) \
  ${{pkgs.consul}}/bin/consul kv put epl-interdc-routes/{dc_name}/{subnet_section} -
"#)));
                        if !inter_dc_bootstrap_routes.contains_key(&dc) {
                            assert!(inter_dc_bootstrap_routes.insert(dc, routes_file).is_none());
                        }
                    r#"
  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
"#
                } else { "" };

                let manual_virtual_ip =
                    if dc_net.is_floating_subnet_ip_needed {
                        let floating_ip = rs.floating_ip.as_ref().unwrap().to_string();
                        format!("    {floating_ip}")
                    } else { "".to_string() };

                if dc_net.is_private_node_to_gw_routing_needed {
                    let region_secrets = consul_secrets.region_secrets.get(&region).unwrap();
                    let acl_token = region_secrets.consul_vrrp_acl_tokens.get(&dc).unwrap();
                    if dc_net.is_consul_vrrp {
                        plan.add_secret(
                            custom_user_secret_key(
                                "consul".to_string(),
                                // this token might already exist
                                // added by consul during bootstrap, but
                                // this token is actually used by this router
                                "consul-vrrp-token.txt".to_string(),
                                acl_token.clone()
                            )
                        );
                    }
                    let keepalived_conf = plan.add_secret_config(
                        root_secret_config(
                            "keepalived.conf".to_string(),
                            format!(r#"
global_defs {{
  enable_script_security
  script_user consul
}}

vrrp_instance vpnRouter {{
  interface {lan_iface_name}
  state MASTER
  virtual_router_id 1
  priority {vrrp_priority}
  unicast_src_ip {lan_iface_ip}
  unicast_peer {{
    {unicast_peers}
  }}
  virtual_ipaddress {{
{manual_virtual_ip}
  }}
{maybe_vrrp_notify}
}}
"#)
                        )
                    );
                    let abs_path = keepalived_conf.absolute_path();
                    let mut restart_block = String::new();
                    gen_reload_service_on_path_change(
                        &mut restart_block,
                        &abs_path,
                        "keepalived-restart",
                        "keepalived.service",
                        false,
                    );
                    plan.add_custom_nix_block(mk_nix_region("keepalived", format!(r#"
  systemd.services.keepalived = {{
    description = "Keepalive Daemon (LVS and VRRP)";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" "network-online.target" "syslog.target" ];
    wants = [ "network-online.target" ];
    serviceConfig = {{
      Type = "forking";
      PIDFile = "/run/keepalived.pid";
      KillMode = "process";
      RuntimeDirectory = "keepalived";
      ExecStart = "${{pkgs.keepalived}}/sbin/keepalived -f {abs_path} -p /run/keepalived.pid";
      ExecReload = "${{pkgs.coreutils}}/bin/kill -HUP $MAINPID";
      Restart = "always";
      RestartSec = "1s";
    }};
  }};
"#)));
                    plan.add_custom_nix_block(restart_block);

                }
            }
        }
    }

    inter_dc_bootstrap_routes
}

fn provision_coprocessor_routing(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &mut SecretsStorage) {
    let bgp_peering_seed = bgp_peering_seed(secrets);
    let bgp_peering_pwd = |srv_a: TableRowPointerServer, srv_b: TableRowPointerServer| -> String {
        derive_bgp_peering_password(db, &bgp_peering_seed, srv_a, srv_b)
    };

    for dc in db.db.datacenter().rows_iter() {
        let is_coproc = db.db.datacenter().c_implementation(dc) == "coprocessor";
        if !is_coproc {
            continue;
        }

        let region = db.db.datacenter().c_region(dc);
        let coproc_data = db.sync_res.network.region_coprocessor_gws.get(&region).unwrap();
        for coproc_srv in db.db.datacenter().c_referrers_server__dc(dc) {
            let plan = plans.fetch_plan(*coproc_srv);
            let mut ni_idx = 0;
            let mut bfd_cfg = String::new();
            let mut bgp_cfg = String::new();
            let mut bgp_neighbors = String::new();
            let mut zebra_cfg = String::new();
            let lan_iface = db.projections.consul_network_iface.value(*coproc_srv);
            let lan_iface_ip = db.db.network_interface().c_if_ip(*lan_iface);
            let coproc_bgp_asn = coproc_server_asn_number(&lan_iface_ip);
            write!(&mut zebra_cfg, r#"
        !
        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
        !
        ip prefix-list ANY seq 100 permit 0.0.0.0/0
        !
        route-map LANRM permit 100
          match ip address prefix-list LAN
          set src {lan_iface_ip}
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol bgp route-map LANRM"#).unwrap();

            write!(&mut bgp_cfg, r#"
        !
        router bgp {coproc_bgp_asn}
          bgp router-id {lan_iface_ip}
          address-family ipv4 unicast
            redistribute connected
            redistribute static
            network {lan_iface_ip}/32
          exit-address-family"#).unwrap();

            let mut epl_postrouting_rules = String::new();
            // don't masquerade vpn traffic
            write!(&mut epl_postrouting_rules, "
           ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment \"Admin VPN\";
           ip saddr {lan_iface_ip}/32 ip daddr 10.0.0.0/8 return comment \"Inter DC EPL traffic\";").unwrap();
            let table_content = format!(r#"
       chain EPL_POSTROUTING {{
           type nat hook postrouting priority 0;
{epl_postrouting_rules}
       }}
"#);
            create_nftables_table_service(
                plan,
                "ip",
                "epl-nat",
                &table_content
            );

            write!(&mut bfd_cfg, r#"
        !"#).unwrap();
            for ni in db.db.server().c_children_network_interface(*coproc_srv) {
                let ni_net = db.db.network_interface().c_if_network(*ni);
                let ni_net_name = db.db.network().c_network_name(ni_net);
                if ni_net_name == "vpn" {
                    let matching_gw = coproc_data.gateways.iter().skip(ni_idx).next().unwrap();
                    let bgp_pass = bgp_peering_pwd(*matching_gw, *coproc_srv);
                    let neighbor_dc = db.db.server().c_dc(*matching_gw);
                    let neighbor_dc_net = db.db.datacenter().c_network_cidr(neighbor_dc);
                    let vpn_iface = db.projections.vpn_network_iface.get(matching_gw).unwrap();
                    let neighbor_ip = db.db.network_interface().c_if_ip(*vpn_iface);
                    let neighbor_bgp_asn = dc_bgp_as_number(&neighbor_dc_net);
                    let this_vpn_ip = db.db.network_interface().c_if_ip(*ni);
            write!(&mut bfd_cfg, r#"
        bfd peer {this_vpn_ip}
          no shutdown"#).unwrap();

                    write!(&mut bgp_neighbors, r#"
          neighbor {neighbor_ip} remote-as {neighbor_bgp_asn}
          neighbor {neighbor_ip} password {bgp_pass}
          neighbor {neighbor_ip} bfd
          neighbor {neighbor_ip} disable-connected-check"#).unwrap();
                    ni_idx += 1;
                } else if ni_net_name == "lan" {
                    let lan_iface_name = db.db.network_interface().c_if_name(*ni);
                    let lan_iface_name = if lan_iface_name.contains(":") {
                        lan_iface_name.split(":").next().unwrap()
                    } else { lan_iface_name.as_str() };
                    write!(&mut zebra_cfg, r#"
        !
        interface {lan_iface_name}
          ip address {lan_iface_ip}/32"#).unwrap();
                }
            }

            write!(&mut bgp_cfg, r#"{bgp_neighbors}"#).unwrap();

            plan.add_custom_nix_block(mk_nix_region("frr_bfd_config", format!(r#"
  services.frr.bfd = {{
      enable = true;
      config = ''{bfd_cfg}
      '';
  }};"#)));
            plan.add_custom_nix_block(mk_nix_region("frr_bgp_config", format!(r#"
  services.frr.bgp = {{
      enable = true;
      config = ''{bgp_cfg}
      '';
  }};"#)));
            plan.add_custom_nix_block(mk_nix_region("frr_zebra_config", format!(r#"
  services.frr.zebra = {{
      enable = true;
      config = ''{zebra_cfg}
      '';
  }};"#)));
        }
    }
}

fn dc_bgp_as_number(dc_cidr: &str) -> u16 {
    64512 + dc_cidr
        .split(".")
        .skip(1)
        .next()
        .unwrap()
        .parse::<u16>().expect("Can't parse asn number")
}

fn coproc_server_asn_number(dc_cidr: &str) -> u32 {
    let parsed = std::net::Ipv4Addr::from_str(dc_cidr).unwrap();
    let o1 = parsed.octets()[1] as u32 * 256 * 256;
    let o2 = parsed.octets()[2] as u32 * 256;
    let o3 = parsed.octets()[3] as u32;
    4200000000 + o1 + o2 + o3
}

#[test]
fn test_coproc_srv_bgp_asn() {
    assert_eq!(coproc_server_asn_number("10.17.7.3"), 4201115907)
}

#[test]
fn test_dc_cidr_bgp_asn() {
    assert_eq!(dc_bgp_as_number("10.17.0.0/16"), 64529)
}

fn make_ip_into_l3_gre_ip(input: &str) -> String {
    input.split(".").enumerate().map(|(octet, part)| {
        if octet == 2 {
            (part.parse::<u32>().unwrap() + 128).to_string()
        } else {
            part.to_string()
        }
    }).collect::<Vec<_>>().join(".")
}

#[test]
fn test_l3_gre_ip_remap() {
    assert_eq!(make_ip_into_l3_gre_ip("10.17.3.10"), "10.17.131.10");
}

struct GreNeighbors {
    vpns: String,
    non_vpns: String,
    inter_dc_hops_for_non_vpns: String,
}

fn generate_datacenter_l3_vpn_gre_neighbors(
    db: &CheckedDB, dc: TableRowPointerDatacenter
) -> GreNeighbors {
    let mut vpns = String::new();
    let mut non_vpns = String::new();
    let mut inter_dc_hops_for_non_vpns = String::new();

    let mut srvs = db.db.datacenter().c_referrers_server__dc(dc).to_vec();
    srvs.sort_by_key(|i| {
        let iface = db.projections.consul_network_iface.value(*i);
        db.db.network_interface().c_if_ip(*iface);
    });

    write!(&mut inter_dc_hops_for_non_vpns, r#"
        chain PREROUTING {{
            type filter hook prerouting priority -300; policy accept;
"#).unwrap();

    for srv in &srvs {
        let lan_iface = db.projections.consul_network_iface.value(*srv);
        let lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
        let gre_ip = make_ip_into_l3_gre_ip(&lan_ip);
        if db.db.server().c_is_vpn_gateway(*srv) {
            write!(&mut vpns, r#"
        ip neighbor add {gre_ip} lladdr {lan_ip} dev vpnGre"#).unwrap();
        } else {
            write!(&mut inter_dc_hops_for_non_vpns, r#"
            ip daddr {lan_ip} ip saddr 10.0.0.0/8 ip daddr set {gre_ip};
            ip saddr {gre_ip} ip saddr set {lan_ip};"#).unwrap();
            write!(&mut non_vpns, r#"
        ip neighbor add {gre_ip} lladdr {lan_ip} dev vpnGre"#).unwrap();
        }
    }

    write!(&mut inter_dc_hops_for_non_vpns, r#"
        }}
"#).unwrap();

    GreNeighbors { vpns, non_vpns, inter_dc_hops_for_non_vpns }
}

pub fn generate_datacenter_l3_vpn_gre_tunnels(
    db: &CheckedDB,
    srv: TableRowPointerServer,
    gre_neighbors: &str
) -> String {
    let lan_iface = db.projections.consul_network_iface.value(srv);
    let lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
    let tunnel_lan_ip = make_ip_into_l3_gre_ip(&lan_ip);
    let tunnel_iface_name = "vpnGre";

    format!(r#"
  systemd.services.vpn-gre-tunnel = {{
    description = "VPN GRE Tunnel - {tunnel_iface_name}";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    path = with pkgs; [ kmod iproute2 ];

    serviceConfig = {{
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    }};

    script = ''
        ip tunnel add {tunnel_iface_name} mode gre local {lan_ip} key 17
        ip addr add {tunnel_lan_ip}/17 dev {tunnel_iface_name}

{gre_neighbors}

        ip link set dev {tunnel_iface_name} up
    '';

    postStop = ''
      ip link del dev {tunnel_iface_name}
    '';
  }};
"#)
}

pub struct InterDcRoutes {
    routes_add: String,
    routes_del: String,
    gateway_server: Option<TableRowPointerServer>,
}

impl InterDcRoutes {
    pub fn route_script(&self) -> String {
        let routes_add = &self.routes_add;
        let routes_del = &self.routes_del;
        format!(r#"
# ROUTES CREATE
{routes_add}
# ROUTES DELETE
{routes_del}
# FINISH
"#)
    }
}

pub fn inter_dc_routes(db: &CheckedDB, dc: TableRowPointerDatacenter)
                       -> BTreeMap<ipnet::Ipv4Net, Vec<InterDcRoutes>>
{
    let dc_net = db.sync_res.network.networking_answers.dcs.get(&dc).unwrap();
    let mut res = BTreeMap::new();

    for (subnet, rs) in &dc_net.subnets {
        let mut subnet_gateways = Vec::new();
        if dc_net.is_consul_vrrp {
            let interfaces =
                if dc_net.params.use_l3_hop_for_vpn_gateways {
                    &rs.vpn_interfaces
                } else { &rs.routing_interfaces };
            for ri in interfaces {
                let mut routes_add = String::new();
                let mut routes_del = String::new();

                let server = ri.server;
                let lan_iface = db.projections.consul_network_iface.value(server);
                let lan_iface_ip = db.db.network_interface().c_if_ip(*lan_iface);
                // TODO: we need to change routing of this algorithm in L3 to set only one key
                let routing_ip =
                    if dc_net.params.use_l3_hop_for_vpn_gateways {
                        make_ip_into_l3_gre_ip(&lan_iface_ip)
                    } else {
                        lan_iface_ip.to_string()
                    };
                let gateway_server = Some(server);

                if dc_net.has_managed_routing_to_other_dcs {
                    // worst case, when we need to split subnets one by one
                    for peer_dc in db.db.datacenter().rows_iter() {
                        if dc != peer_dc {
                            let link = db.sync_res.network.cross_dc_connectivity.get(&(dc, peer_dc)).unwrap();
                            if matches!(link, CrossDcConnectivity::Wireguard) {
                                let peer_dc_cidr = db.db.datacenter().c_network_cidr(peer_dc);
                                writeln!(&mut routes_add, "ip route add {peer_dc_cidr} via {routing_ip}").unwrap();
                                writeln!(&mut routes_del, "ip route del {peer_dc_cidr}").unwrap();
                            }
                        }
                    }
                } else if db.sync_res.network.wireguard_across_dc_needed {
                    // good case, we can just have one route to 10.0.0.0/8 and add gateway
                    writeln!(&mut routes_add, "ip route add 10.0.0.0/8 via {routing_ip}").unwrap();
                    writeln!(&mut routes_del, "ip route del 10.0.0.0/8").unwrap();
                }

                if dc_net.params.is_epl_responsible_for_internal_node_internet {
                    assert!(!dc_net.params.use_l3_hop_for_vpn_gateways, "What to do in this case, epl responsible for node internet but we have l3 routing?");

                    writeln!(&mut routes_add, "ip route add 0.0.0.0/0 via {lan_iface_ip}").unwrap();
                    writeln!(&mut routes_del, "ip route del 0.0.0.0/0").unwrap();
                }

                // could screw over our shell scripts if have quotes
                assert!(!routes_add.contains(['"', '\'']));
                assert!(!routes_del.contains(['"', '\'']));

                subnet_gateways.push(InterDcRoutes { routes_add, routes_del, gateway_server });
            }
        } else if dc_net.is_hardware_vrrp {
            let floating_ip = rs.floating_ip.as_ref().unwrap();
            let gateway_server = None;
            let mut routes_add = String::new();
            let mut routes_del = String::new();
            if dc_net.has_managed_routing_to_other_dcs {
                for peer_dc in db.db.datacenter().rows_iter() {
                    if dc != peer_dc {
                        let link = db.sync_res.network.cross_dc_connectivity.get(&(dc, peer_dc)).unwrap();
                        if matches!(link, CrossDcConnectivity::Wireguard) {
                            let peer_dc_cidr = db.db.datacenter().c_network_cidr(peer_dc);
                            writeln!(&mut routes_add, "ip route add {peer_dc_cidr} via {floating_ip}").unwrap();
                            writeln!(&mut routes_del, "ip route del {peer_dc_cidr}").unwrap();
                        }
                    }
                }

                // could screw over our shell scripts if have quotes
                assert!(!routes_add.contains(['"', '\'']));
                assert!(!routes_del.contains(['"', '\'']));
            } else if db.sync_res.network.wireguard_across_dc_needed {
                // good case, we can just have one route to 10.0.0.0/8 and add gateway
                writeln!(&mut routes_add, "ip route add 10.0.0.0/8 via {floating_ip}").unwrap();
                writeln!(&mut routes_del, "ip route del 10.0.0.0/8").unwrap();
            }

            if dc_net.params.is_epl_responsible_for_internal_node_internet {
                writeln!(&mut routes_add, "ip route add 0.0.0.0/0 via {floating_ip}").unwrap();
                writeln!(&mut routes_del, "ip route del 0.0.0.0/0").unwrap();
            }

            subnet_gateways.push(InterDcRoutes { routes_add, routes_del, gateway_server });
        }

        assert!(res.insert(subnet.clone(), subnet_gateways).is_none());
    }

    res
}

fn provision_servers(db: &CheckedDB, plans: &mut NixAllServerPlans, inter_dc_routes: &HashMap<TableRowPointerDatacenter, String>) {
    for dc in db.db.datacenter().rows_iter() {
        let dc_name = db.db.datacenter().c_dc_name(dc);
        let dc_impl = db.db.datacenter().c_implementation(dc);
        if dc_impl == "coprocessor" {
            // different configs for coprocessor nodes
            continue;
        }

        let dc_net = db.sync_res.network.networking_answers.dcs.get(&dc).unwrap();
        let dc_routers = dc_net.all_routers_set();
        let gre_neighbors: Option<GreNeighbors> =
            if dc_net.params.use_l3_hop_for_vpn_gateways {
                Some(generate_datacenter_l3_vpn_gre_neighbors(db, dc))
            } else { None };

        if dc_net.is_consul_vrrp {
            let routes = inter_dc_routes.get(&dc).unwrap();
            let routes = pad_string(routes, "            ");
            // for such datacenters make sure every server is listening to the routes
            for server in db.db.datacenter().c_referrers_server__dc(dc) {
                if !dc_routers.contains(server) {
                    let plan = plans.fetch_plan(*server);
                    let lan_iface = db.projections.consul_network_iface.value(*server);
                    let subnet = format!("{}.0p24", first_three_octets(db.db.network_interface().c_if_ip(*lan_iface)));
                    plan.add_shell_package("epl-process-route-data", route_processing_script());
                    plan.add_shell_package("epl-watch-route-data", &route_watching_script(&dc_name, &subnet));
                    plan.add_post_second_round_secrets_shell_hook(format!(r#"
# bootstrap inter dc routes
function epl_bootstrap_routes() {{
    ORIG_ROUTES='{routes}'
    echo -n '{{"Value":"' > /run/epl-routes-tmp2
    echo -n "$ORIG_ROUTES" | base64 -w0 >> /run/epl-routes-tmp2
    echo -n '"}}' >> /run/epl-routes-tmp2
    mv -f /run/epl-routes-tmp2 /run/epl-routes
    echo "$ORIG_ROUTES" | sed -n '/ROUTES CREATE/,/ROUTES DELETE/p' | sh || true
}}

consul kv get epl-interdc-routes/{dc_name} || epl_bootstrap_routes
"#));
                    plan.add_custom_nix_block(format!(r#"
    systemd.services.epl-route-watcher = {{
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];

      serviceConfig = {{
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStart = "/run/current-system/sw/bin/epl-watch-route-data";
        Restart = "always";
        RestartSec = "10";
      }};

      enable = true;
    }};
"#));
                }

                if dc_net.params.use_l3_hop_for_vpn_gateways && !db.db.server().c_is_vpn_gateway(*server) {
                    let plan = plans.fetch_plan(*server);

                    let lan_if = db.projections.consul_network_iface.value(*server);
                    let lan_ip = db.db.network_interface().c_if_ip(*lan_if);
                    let gre_ip = make_ip_into_l3_gre_ip(&lan_ip);

                    plan.add_custom_nix_block(r#"
    boot.kernelModules = [ "gre" ];
"#.to_string());

                    let neighbors = gre_neighbors.as_ref().unwrap();
                    plan.add_custom_nix_block(
                        mk_nix_region(
                            "l3_vpn_hop_interface",
                            generate_datacenter_l3_vpn_gre_tunnels(db, *server, &neighbors.vpns)
                        )
                    );
                    let table_content =
                        format!(r#"
        chain PREROUTING {{
            type filter hook prerouting priority -300; policy accept;
            ip daddr {gre_ip} ip daddr set {lan_ip}
        }}

        chain SNAT_POSTROUTING {{
                type nat hook postrouting priority srcnat; policy accept;
                ip daddr 10.0.0.0/8 snat to {lan_ip}
        }}
"#);
                    create_nftables_table_service(
                        plan,
                        "ip",
                        "l3-vpn-hop-address-translation",
                        &table_content
                    );
                }
            }
        } else if dc_net.is_hardware_vrrp {
            assert!(!dc_net.params.use_l3_hop_for_vpn_gateways);
            for server in db.db.datacenter().c_referrers_server__dc(dc) {
                if !dc_routers.contains(server) {
                    let lan_if = db.projections.consul_network_iface.value(*server);
                    let if_ip = db.db.network_interface().c_if_ip(*lan_if);
                    let if_name = db.db.network_interface().c_if_name(*lan_if);
                    let subnet = format!("{}.0/24", first_three_octets(&if_ip));
                    let subnet_net = Ipv4Net::from_str(&subnet).unwrap();
                    let subnet_info = dc_net.subnets.get(&subnet_net).unwrap();
                    let gw = subnet_info.floating_ip.as_ref().unwrap().to_string();
                    let plan = plans.fetch_plan(*server);
                    plan.add_interface_route(
                        if_name,
                        format!(r#"{{ address = "10.0.0.0"; prefixLength = 8; via = "{gw}"; }}"#)
                    );
                    if dc_net.params.is_epl_responsible_for_internal_node_internet {
                        plan.add_interface_route(
                            if_name,
                            format!(r#"{{ address = "0.0.0.0"; prefixLength = 0; via = "{gw}"; }}"#)
                        );
                    }
                }
            }
        }
    }
}

fn route_watching_script(dc_name: &str, subnet: &str) -> String {
    format!(r#"
# wait for consul to become available
while ! ${{pkgs.consul}}/bin/consul kv get epl-interdc-routes/{dc_name}/{subnet}
do
  sleep 7
done

exec ${{pkgs.consul}}/bin/consul watch -type=key -key=epl-interdc-routes/{dc_name}/{subnet} /run/current-system/sw/bin/epl-process-route-data
"#)
}

fn route_processing_script() -> &'static str {
    r#"
PREFIX=/run/current-system/sw/bin
$PREFIX/cat /dev/stdin > /run/epl-routes-tmp

# delete old routes if exist
if [ -f /run/epl-routes ];
then
    OLD_SCRIPT=$( $PREFIX/cat /run/epl-routes | $PREFIX/jq -r '.Value' | $PREFIX/base64 -d )
    # delete old routes
    DELETE_BLOCK=$( $PREFIX/echo "$OLD_SCRIPT" | $PREFIX/sed -n '/ROUTES DELETE/,/FINISH/p' )
    $PREFIX/echo "export PATH=/run/current-system/sw/bin/:\$PATH; $DELETE_BLOCK" | /bin/sh
    $PREFIX/echo old routes deleted
    $PREFIX/echo "$DELETE_BLOCK"
fi

NEW_SCRIPT=$( $PREFIX/cat /run/epl-routes-tmp | $PREFIX/jq -r '.Value' | $PREFIX/base64 -d )
NEW_ADD_BLOCK=$( $PREFIX/echo "$NEW_SCRIPT" | $PREFIX/sed -n '/ROUTES CREATE/,/ROUTES DELETE/p' )
# add new routes
$PREFIX/echo "export PATH=/run/current-system/sw/bin/:\$PATH; $NEW_ADD_BLOCK" | /bin/sh

# set new file in place for old route deletion
$PREFIX/mv -f /run/epl-routes-tmp /run/epl-routes

$PREFIX/echo routes were changed
$PREFIX/echo "$NEW_ADD_BLOCK"

"#
}

pub fn create_nftables_table_service(
    plan: &mut NixServerPlan,
    table_family: &str,
    table_name: &str,
    table_content: &str
) {
    let region_name = format!("epl_nft_rules_{table_name}");
    let block = mk_nix_region(
        &region_name,
        format!(r#"
            networking.nftables.tables.{table_name} = {{
              family = "{table_family}";
              content = ''
{table_content}
              '';
            }};
"#)
    );
    plan.add_custom_nix_block(block);
}
