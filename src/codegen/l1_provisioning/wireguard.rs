use std::collections::HashMap;
use std::fmt::Write;

use crate::codegen::nixplan::mk_nix_region;
use crate::database::TableRowPointerNetworkInterface;
use crate::static_analysis::networking::CrossDcConnectivity;
use crate::{static_analysis::CheckedDB, codegen::{nixplan::{NixAllServerPlans, self}, secrets::{SecretValue, SecretsStorage, SecretFile, SecretKind}}, database::TableRowPointerServer};

pub struct WireGuardServerSecrets {
    pub public_key: SecretValue,
    pub private_key: SecretValue,
}

pub fn generate_admin_vpn_config(db: &CheckedDB, secrets: &WgSecrets) -> String {
    let mut res = String::with_capacity(512);

    let admin_private_vpn_key = secrets.admin_vpn.private_key.value();
    write!(&mut res, r#"
[Interface]
#Address = 172.21.7.254/16 # Only for wg-quick, left for reference
PrivateKey = {admin_private_vpn_key}
ListenPort = 51877
"#).unwrap();
    for dc in db.db.datacenter().rows_iter() {
        let region = db.db.datacenter().c_region(dc);
        let dc_impl = db.db.datacenter().c_implementation(dc);
        if dc_impl == "coprocessor" {
            continue;
        }

        let gws = db.projections.vpn_gateways.get(&dc).unwrap();
        // we rely on deterministic sorting of
        // btree so configuration VPN pairs are always generated the same
        assert_eq!(gws.servers.len(), 2);
        for (gw_srv, gw_iface) in &gws.servers {
            let gw_vpn_ip = db.db.network_interface().c_if_ip(gw_iface.vpn_interface);
            let gw_secrets = secrets.servers_secrets.get(gw_srv).unwrap();
            let gw_public_key = gw_secrets.public_key.value();
            let gw_subnet = db.db.datacenter().c_network_cidr(dc);
            let mut gw_subnets = String::new();
            gw_subnets += ",";
            gw_subnets += gw_subnet;

            if db.db.server().c_is_coprocessor_gateway(*gw_srv) {
                let coproc_dc = db.sync_res.network.region_coprocessor_gws.get(&region).unwrap();
                let coproc_cidr = db.db.datacenter().c_network_cidr(coproc_dc.coprocessor_dc);
                gw_subnets += ",";
                gw_subnets += coproc_cidr;
            }

            // go for first gateway with internet ip
            if let Some(internet_iface) = gw_iface.internet_interface {
                let internet_ip = db.db.network_interface().c_if_ip(internet_iface);
                write!(&mut res, r#"
[Peer]
PublicKey = {gw_public_key}
AllowedIPs = {gw_vpn_ip}/32{gw_subnets}
Endpoint = {internet_ip}:51820
PersistentKeepalive = 25
"#).unwrap();
                break;
            }
        }
    }

    res
}

pub fn provision_wireguard(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &mut SecretsStorage) {
    let secrets = generate_wg_secrets(db, secrets);
    provision_wireguard_gateways(db, plans, &secrets);
    provision_coprocessor_wg_connections(db, plans, &secrets);
}

fn provision_wireguard_gateways(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &WgSecrets) {
    // use nftables everywhere
    for server in db.db.server().rows_iter() {
        let plan = plans.fetch_plan(server);
        plan.add_server_feature(nixplan::NixServerFeatures::Nftables);
        plan.add_nix_package("nftables");
    }

    for dc in db.db.datacenter().rows_iter() {
        // we rely on deterministic sorting of
        // btree so configuration VPN pairs are always generated the same
        let dc_impl = db.db.datacenter().c_implementation(dc);
        if dc_impl == "coprocessor" {
            continue;
        }
        let gws = db.projections.vpn_gateways.get(&dc).unwrap();
        assert_eq!(gws.servers.len(), 2);
        let mut gw_id = 0;
        for (gw_srv, gw_iface) in &gws.servers {
            gw_id += 1;
            let vpn_ip = db.db.network_interface().c_if_ip(gw_iface.vpn_interface);
            let is_coprocessor_gw = db.db.server().c_is_coprocessor_gateway(*gw_srv);
            let gw_secrets = secrets.servers_secrets.get(gw_srv).unwrap();
            let plan = plans.fetch_plan(*gw_srv);
            plan.add_nix_package("wireguard-tools");
            let pkey_file = plan.add_secret(nixplan::root_secret_key(
                "epl-wireguard-key".to_string(),
                gw_secrets.private_key.clone(),
            ));

            let pkey_path = pkey_file.absolute_path();

            // we use low level systemd unit creation instead of nixos
            // here because 'ip address add "{vpn_ip}/16" dev "wg0"'
            // doesn't work in aws and we add || true to that, it is simple
            // to implement so lets keep it that way
            let mut wg_setup_script = format!(r#"
modprobe wireguard || true
ip link add dev "wg0" type wireguard

# this might fail as kernel seems to remember ip address from previously
ip address add "{vpn_ip}/16" dev "wg0" || true
wg set "wg0" private-key "{pkey_path}" listen-port "51820"
ip link set up dev "wg0"

# peers"#);

            // add admin vpn
            let admin_public_vpn_key = &secrets.admin_vpn.public_key;
            write!(&mut wg_setup_script, r#"
wg set wg0 peer "{admin_public_vpn_key}" allowed-ips "172.21.7.254/32"
"#).unwrap();

            for peer_dc in db.db.datacenter().rows_iter() {
                let dc_impl = db.db.datacenter().c_implementation(peer_dc);
                if dc_impl == "coprocessor" {
                    if peer_dc != dc && is_coprocessor_gw {
                        for server in db.db.datacenter().c_referrers_server__dc(peer_dc) {
                            let peer_consul_iface = db.projections.consul_network_iface.value(*server);
                            let peer_lan_ip = db.db.network_interface().c_if_ip(*peer_consul_iface);
                            let mut peer_gw_id = 0;
                            for peer_ni in db.db.server().c_children_network_interface(*server) {
                                let peer_ni_network = db.db.network_interface().c_if_network(*peer_ni);
                                let peer_net_name = db.db.network().c_network_name(peer_ni_network);
                                if peer_net_name == "vpn" {
                                    peer_gw_id += 1;
                                    let peer_vpn_ip = db.db.network_interface().c_if_ip(*peer_ni);
                                    let peer_secrets = secrets.coprocessor_server_secrets.get(peer_ni).unwrap();
                                    let peer_public_key = peer_secrets.public_key.value();
                                    if gw_id == peer_gw_id {
                                        write!(&mut wg_setup_script, r#"
wg set wg0 peer "{peer_public_key}" allowed-ips "{peer_vpn_ip}/32,{peer_lan_ip}/32"
"#).unwrap();
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let peer_dc_subnet = db.db.datacenter().c_network_cidr(peer_dc);
                    let peer_gws = db.projections.vpn_gateways.get(&peer_dc).unwrap();
                    let mut peer_gw_id = 0;
                    for (peer_gw_srv, peer_gw_iface) in &peer_gws.servers {
                        peer_gw_id += 1;
                        let peer_secrets = secrets.servers_secrets.get(peer_gw_srv).unwrap();
                        let peer_public_key = peer_secrets.public_key.value();
                        if dc != peer_dc && gw_id == peer_gw_id {
                            let maybe_internet_ip =
                                if let Some(internet_iface) = peer_gw_iface.internet_interface {
                                    let peer_internet_ip = db.db.network_interface().c_if_ip(internet_iface);
                                    format!(" endpoint \"{peer_internet_ip}:51820\"")
                                } else { String::new() };
                            let peer_vpn_ip = db.db.network_interface().c_if_ip(peer_gw_iface.vpn_interface);
                            let peer_server = db.db.network_interface().c_parent(peer_gw_iface.vpn_interface);
                            let conn = db.sync_res.network.cross_dc_connectivity.get(&(dc, peer_dc)).unwrap();
                            if *conn == CrossDcConnectivity::Wireguard && peer_server != *gw_srv {
                                // configure point to point links full mesh between DCs
                                // only one peer per datacenter.
                                write!(&mut wg_setup_script, r#"
wg set wg0 peer "{peer_public_key}" allowed-ips "{peer_vpn_ip}/32,{peer_dc_subnet}"{maybe_internet_ip}
"#).unwrap();
                            }
                        }
                    }
                }
            }

        let wg_plan = format!(r#"
  systemd.services.wireguard-wg0 = {{
    description = "WireGuard Tunnel - wg0";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    environment.DEVICE = "wg0";
    path = with pkgs; [ kmod iproute2 wireguard-tools ];

    serviceConfig = {{
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    }};

    script = ''
{wg_setup_script}
    '';

    postStop = ''
      ip link del dev "wg0"
    '';
  }};
"#);

            plan.add_custom_nix_block(mk_nix_region("wireguard_configs", wg_plan));
        }
    }
}

fn provision_coprocessor_wg_connections(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &WgSecrets) {
    for dc in db.db.datacenter().rows_iter() {
        let dc_impl = db.db.datacenter().c_implementation(dc);
        if dc_impl != "coprocessor" {
            continue;
        }

        let region = db.db.datacenter().c_region(dc);
        let gws = db.sync_res.network.region_coprocessor_gws.get(&region).unwrap();

        for server in db.db.datacenter().c_referrers_server__dc(dc) {
            let plan = plans.fetch_plan(*server);
            plan.add_nix_package("wireguard-tools");
            let server_lan_iface = db.projections.consul_network_iface.value(*server);
            let server_lan_ip = db.db.network_interface().c_if_ip(*server_lan_iface);

            let mut wg_plan = String::new();

            let mut ni_idx = 0;
            for ni in db.db.server().c_children_network_interface(*server) {
                let wg_port = 51820 + ni_idx;
                let if_name = db.db.network_interface().c_if_name(*ni);
                let if_network = db.db.network_interface().c_if_network(*ni);
                let network_name = db.db.network().c_network_name(if_network);
                if network_name != "vpn" {
                    continue;
                }
                ni_idx += 1;
                let vpn_ip = db.db.network_interface().c_if_ip(*ni);

                let wg_secrets = secrets.coprocessor_server_secrets.get(ni).unwrap();
                let pkey_file = plan.add_secret(nixplan::root_secret_key(
                    format!("epl-wireguard-key-{if_name}"),
                    wg_secrets.private_key.clone(),
                ));

                let pkey_path = pkey_file.absolute_path();

                let mut wg_setup_script = format!(r#"
modprobe wireguard || true
ip link add dev "{if_name}" type wireguard

# this might fail as kernel seems to remember ip address from previously
ip address add "{vpn_ip}/16" dev "{if_name}" || true
wg set "{if_name}" private-key "{pkey_path}" listen-port "{wg_port}"
ip link set up dev "{if_name}"

# peers"#);
                let mut wg_routes_script = String::new();

                let mut gw_idx = 0;
                let mut routes_del = String::new();
                for gw in &gws.gateways {
                    gw_idx += 1;
                    let peer_internet_ip_iface = db.projections.internet_network_iface.get(gw).unwrap();
                    let peer_internet_ip = db.db.network_interface().c_if_ip(*peer_internet_ip_iface);
                    let peer_vpn_iface = db.projections.vpn_network_iface.get(gw).unwrap();
                    let peer_vpn_ip = db.db.network_interface().c_if_ip(*peer_vpn_iface);
                    let gw_secrets = secrets.servers_secrets.get(gw).unwrap();
                    let peer_public_key = gw_secrets.public_key.value();
                    if ni_idx == gw_idx {
                        write!(&mut wg_setup_script, r#"
wg set "{if_name}" peer "{peer_public_key}" allowed-ips "{peer_vpn_ip}/32,10.0.0.0/8" endpoint "{peer_internet_ip}:51820" persistent-keepalive 10"#).unwrap();
                        write!(&mut wg_routes_script, r#"
ip route del '172.21.0.0/16' dev {if_name} || true
ip route add '{peer_vpn_ip}/32' dev {if_name} || true"#).unwrap();

                        // on first interface establish p2p connections
                        // to all coprocessor peers if they have public ip available
                        if ni_idx == 1 {
                            for peer_server in db.db.datacenter().c_referrers_server__dc(dc) {
                                if peer_server == server {
                                    continue;
                                }

                                if let Some(internet_iface) = db.projections.internet_network_iface.get(peer_server) {
                                    let vpn_wg0_iface =
                                        db.db
                                          .server()
                                          .c_children_network_interface(*peer_server)
                                          .iter()
                                          .find(|iface| {
                                              "wg0" == db.db.network_interface().c_if_name(**iface)
                                          })
                                          .unwrap();
                                    let peer_internet_ip = db.db.network_interface().c_if_ip(*internet_iface);
                                    let lan_iface = db.projections.consul_network_iface.value(*peer_server);
                                    let peer_lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
                                    let peer_vpn_ip = db.db.network_interface().c_if_ip(*vpn_wg0_iface);

                                    let peer_wg_secrets =
                                        secrets.coprocessor_server_secrets.get(vpn_wg0_iface).unwrap();
                                    let peer_public_key = peer_wg_secrets.public_key.value();
                                    write!(&mut wg_setup_script, r#"
wg set "{if_name}" peer "{peer_public_key}" allowed-ips "{peer_vpn_ip}/32,{peer_lan_ip}/32" endpoint "{peer_internet_ip}:51820" persistent-keepalive 10"#).unwrap();
                                    write!(&mut wg_routes_script, r#"
ip route add '{peer_vpn_ip}/32' dev {if_name} || true
ip route add '{peer_lan_ip}/32' via {peer_vpn_ip} src {server_lan_ip} || true"#).unwrap();
                                    write!(&mut routes_del, r#"
      ip route del '{peer_vpn_ip}/32' || true
      ip route del '{peer_lan_ip}/32' || true"#).unwrap();
                                }
                            }
                        }

                        write!(&mut routes_del, r#"
      ip route del '{peer_vpn_ip}/32' dev {if_name} || true
"#).unwrap();
                    }
                }

                write!(&mut wg_plan, r#"
  systemd.services.wireguard-{if_name} = {{
    description = "WireGuard Tunnel - {if_name}";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    environment.DEVICE = "{if_name}";
    path = with pkgs; [ kmod iproute2 wireguard-tools ];

    serviceConfig = {{
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    }};

    script = ''
{wg_setup_script}

# routes{wg_routes_script}
    '';

    postStop = ''
{routes_del}
      ip link del dev "{if_name}"
    '';
  }};
"#).unwrap();
            }

            plan.add_custom_nix_block(mk_nix_region("wireguard_configs", wg_plan));
        }
    }
}

pub struct WgSecrets {
    pub servers_secrets: HashMap<TableRowPointerServer, WireGuardServerSecrets>,
    pub coprocessor_server_secrets: HashMap<TableRowPointerNetworkInterface, WireGuardServerSecrets>,
    pub admin_vpn: WireGuardServerSecrets,
}

pub fn generate_wg_secrets(db: &CheckedDB, secrets: &mut SecretsStorage) -> WgSecrets {
    let mut servers_secrets: HashMap<TableRowPointerServer, WireGuardServerSecrets> = HashMap::new();
    let mut coprocessor_server_secrets: HashMap<TableRowPointerNetworkInterface, WireGuardServerSecrets> = HashMap::new();
    for dc in db.db.datacenter().rows_iter() {
        let dc_name = db.db.datacenter().c_dc_name(dc);
        let dc_impl = db.db.datacenter().c_implementation(dc);
        if dc_impl == "coprocessor" {
            for coproc_srv in db.db.datacenter().c_referrers_server__dc(dc) {
                for coproc_ni in db.db.server().c_children_network_interface(*coproc_srv) {
                    let ni_net = db.db.network_interface().c_if_network(*coproc_ni);
                    let ni_name = db.db.network_interface().c_if_name(*coproc_ni);
                    let ni_net_name = db.db.network().c_network_name(ni_net);
                    if ni_net_name == "vpn" {
                        let hostname = db.db.server().c_hostname(*coproc_srv);
                        let mut output_sec_files = Vec::with_capacity(2);
                        output_sec_files.push(SecretFile {
                            kind: SecretKind::WireguardPrivateKey,
                            key: format!("wireguard_{dc_name}_{hostname}_{ni_name}_private_key"),
                            file_name: "privkey".to_string(),
                        });
                        output_sec_files.push(SecretFile {
                            kind: SecretKind::WireguardPublicKey,
                            key: format!("wireguard_{dc_name}_{hostname}_{ni_name}_public_key"),
                            file_name: "pubkey".to_string(),
                        });

                        let mut secrets = secrets.multi_secret_derive(
                            &[],
                            Vec::new(),
                            output_sec_files,
                            format!(r#"
                    wg genkey | tr -d '\n' > privkey
                    cat privkey | wg pubkey | tr -d '\n' > pubkey
                "#).as_str(),
                        );

                        let public_key = secrets.pop().unwrap();
                        let private_key = secrets.pop().unwrap();
                        assert!(secrets.is_empty());
                        assert!(coprocessor_server_secrets.insert(*coproc_ni, WireGuardServerSecrets { public_key, private_key }).is_none());
                    }
                }
            }
        } else {
            let gws = db.projections.vpn_gateways.get(&dc).unwrap();
            for gw_srv in gws.servers.keys() {
                let hostname = db.db.server().c_hostname(*gw_srv);
                let mut output_sec_files = Vec::with_capacity(2);
                output_sec_files.push(SecretFile {
                    kind: SecretKind::WireguardPrivateKey,
                    key: format!("wireguard_{dc_name}_{hostname}_private_key"),
                    file_name: "privkey".to_string(),
                });
                output_sec_files.push(SecretFile {
                    kind: SecretKind::WireguardPublicKey,
                    key: format!("wireguard_{dc_name}_{hostname}_public_key"),
                    file_name: "pubkey".to_string(),
                });

                let mut secrets = secrets.multi_secret_derive(
                    &[],
                    Vec::new(),
                    output_sec_files,
                    format!(r#"
                    wg genkey | tr -d '\n' > privkey
                    cat privkey | wg pubkey | tr -d '\n' > pubkey
                "#).as_str(),
                );

                let public_key = secrets.pop().unwrap();
                let private_key = secrets.pop().unwrap();
                assert!(secrets.is_empty());
                assert!(servers_secrets.insert(*gw_srv, WireGuardServerSecrets { public_key, private_key }).is_none());
            }
        }
    }

    let admin_vpn: WireGuardServerSecrets = {
        let mut output_sec_files = Vec::with_capacity(2);
        output_sec_files.push(SecretFile {
            kind: SecretKind::WireguardPrivateKey,
            key: format!("wireguard_admin_vpn_1_private_key"),
            file_name: "privkey".to_string(),
        });
        output_sec_files.push(SecretFile {
            kind: SecretKind::WireguardPublicKey,
            key: format!("wireguard_admin_vpn_1_public_key"),
            file_name: "pubkey".to_string(),
        });

        let mut secrets = secrets.multi_secret_derive(
            &[],
            Vec::new(),
            output_sec_files,
            format!(r#"
                wg genkey | tr -d '\n' > privkey
                cat privkey | wg pubkey | tr -d '\n' > pubkey
            "#).as_str(),
        );

        let public_key = secrets.pop().unwrap();
        let private_key = secrets.pop().unwrap();
        assert!(secrets.is_empty());

        WireGuardServerSecrets { public_key, private_key }
    };

    WgSecrets {
        servers_secrets,
        coprocessor_server_secrets,
        admin_vpn,
    }
}
