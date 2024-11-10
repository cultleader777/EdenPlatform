#[cfg(test)]
use crate::tests::common;
#[cfg(test)]
use crate::tests::common::{ServerDescription, Config};
#[cfg(test)]
use crate::codegen::secrets::SecretsStorage;
#[cfg(test)]
use crate::codegen::generate_outputs;

#[test]
fn test_network_simulation_config_testvms_single_dc() {
    // single DC, what we expect:
    // VPN gateways provide internet?
    // If one subnet we expect VPN gateways to be subnet routers?
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_single_dc_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);
    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", None),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
              export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

              while :
              do
                  consul members | grep alive &>/dev/null && break
                  sleep 1
              done
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-a";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = true;
              networking.firewall.trustedInterfaces = [
                "eth0"
              ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", None),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
              systemd.services.wireguard-wg0 = {
                description = "WireGuard Tunnel - wg0";
                after = [ "network-pre.target" ];
                wants = [ "network.target" ];
                before = [ "network.target" ];
                wantedBy = [ "multi-user.target" ];
                environment.DEVICE = "wg0";
                path = with pkgs; [ kmod iproute2 wireguard-tools ];

                serviceConfig = {
                  Type = "oneshot";
                  RemainAfterExit = true;
                  Restart = "on-failure";
                  RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.10/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                '';

                postStop = ''
                  ip link del dev "wg0"
                '';
              };
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-c";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = false;
              networking.firewall.trustedInterfaces = [
                "eth0"
                "wg0"
              ];
              networking.firewall.interfaces."eth1".allowedTCPPorts = [ 22 80 443 53 ];
              networking.firewall.interfaces."eth1".allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
              systemd.services.wireguard-wg0 = {
                description = "WireGuard Tunnel - wg0";
                after = [ "network-pre.target" ];
                wants = [ "network.target" ];
                before = [ "network.target" ];
                wantedBy = [ "multi-user.target" ];
                environment.DEVICE = "wg0";
                path = with pkgs; [ kmod iproute2 wireguard-tools ];

                serviceConfig = {
                  Type = "oneshot";
                  RemainAfterExit = true;
                  Restart = "on-failure";
                  RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.11/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                '';

                postStop = ''
                  ip link del dev "wg0"
                '';
              };
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-d";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = false;
              networking.firewall.trustedInterfaces = [
                "eth0"
                "wg0"
              ];
              networking.firewall.interfaces."eth1".allowedTCPPorts = [ 22 80 443 53 ];
              networking.firewall.interfaces."eth1".allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_testvms_multi_dc() {
    // single DC, what we expect:
    // VPN gateways provide internet?
    // If one subnet we expect VPN gateways to be subnet routers?
    //
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_multi_dc_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);

    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                        consul members | grep alive &>/dev/null && break
                        sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                        "enp1s0"
                        "enp3s0"
                        "wg0"
                ];
                networking.firewall.interfaces."enp2s0".allowedTCPPorts = [ 22 53 ];
                networking.firewall.interfaces."enp2s0".allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                        description = "WireGuard Tunnel - wg0";
                        after = [ "network-pre.target" ];
                        wants = [ "network.target" ];
                        before = [ "network.target" ];
                        wantedBy = [ "multi-user.target" ];
                        environment.DEVICE = "wg0";
                        path = with pkgs; [ kmod iproute2 wireguard-tools ];

                        serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                };

                script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard

                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.10/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"

                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                        wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-c_public_key" allowed-ips "172.21.7.12/32,10.18.0.0/16" endpoint "77.77.77.12:51820"
                        wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-e_public_key" allowed-ips "172.21.7.14/32,10.19.0.0/16" endpoint "77.77.77.14:51820"
                '';

                postStop = ''
                ip link del dev "wg0"
                '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.17.0.10
                        redistribute bgp
                        network 10.17.0.0/16 area 10.17.0.0
                        area 10.17.0.0 range 10.17.0.0/16 advertise
                        area 10.17.0.0 range 0.0.0.0/0 not-advertise
                        area 10.17.0.0 authentication message-digest
                        default-information originate always
                        neighbor 10.17.252.11
                        neighbor 10.17.252.12
                        neighbor 10.17.252.13
                      !
                      interface enp3s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.17.0.10
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp3s0
                        ip address 10.17.252.10/22
                      !
                      interface enp1s0
                        ip address 10.17.0.10/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.10
                    unicast_peer {
                        10.17.0.13
                    }
                    virtual_ipaddress {
                        10.17.0.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "enp3s0"
                    "wg0"
                ];
                networking.firewall.interfaces."enp2s0".allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.interfaces."enp2s0".allowedUDPPorts = [ 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];

                    serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.11/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-d_public_key" allowed-ips "172.21.7.13/32,10.18.0.0/16" endpoint "77.77.77.13:51820"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-f_public_key" allowed-ips "172.21.7.15/32,10.19.0.0/16" endpoint "77.77.77.15:51820"
                '';

                postStop = ''
                ip link del dev "wg0"
                '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.17.0.13
                        redistribute bgp
                        network 10.17.0.0/16 area 10.17.0.0
                        area 10.17.0.0 range 10.17.0.0/16 advertise
                        area 10.17.0.0 range 0.0.0.0/0 not-advertise
                        area 10.17.0.0 authentication message-digest
                        default-information originate always
                        neighbor 10.17.252.10
                        neighbor 10.17.252.12
                        neighbor 10.17.252.13
                      !
                      interface enp3s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.17.0.13
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp3s0
                        ip address 10.17.252.11/22
                      !
                      interface enp1s0
                        ip address 10.17.0.13/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.13
                    unicast_peer {
                        10.17.0.10
                    }
                    virtual_ipaddress {
                        10.17.0.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "wg0"
                ];
                networking.firewall.interfaces."enp2s0".allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.interfaces."enp2s0".allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];

                    serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.12/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc1_server-a_public_key" allowed-ips "172.21.7.10/32,10.17.0.0/16" endpoint "77.77.77.10:51820"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-e_public_key" allowed-ips "172.21.7.14/32,10.19.0.0/16" endpoint "77.77.77.14:51820"
                '';

                postStop = ''
                ip link del dev "wg0"
                '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.18.0.10
                        redistribute bgp
                        network 10.18.0.0/16 area 10.18.0.0
                        area 10.18.0.0 range 10.18.0.0/16 advertise
                        area 10.18.0.0 range 0.0.0.0/0 not-advertise
                        area 10.18.0.0 authentication message-digest
                        default-information originate always
                        neighbor 10.18.0.11
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc2_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.18.0.10
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp1s0
                        ip address 10.18.0.10/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.18.0.10
                    unicast_peer {
                        10.18.0.11
                    }
                    virtual_ipaddress {
                        10.18.0.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "wg0"
                ];
                networking.firewall.interfaces."enp2s0".allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.interfaces."enp2s0".allowedUDPPorts = [ 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];

                    serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.13/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc1_server-b_public_key" allowed-ips "172.21.7.11/32,10.17.0.0/16" endpoint "77.77.77.11:51820"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-f_public_key" allowed-ips "172.21.7.15/32,10.19.0.0/16" endpoint "77.77.77.15:51820"
                '';

                postStop = ''
                ip link del dev "wg0"
                '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.18.0.11
                        redistribute bgp
                        network 10.18.0.0/16 area 10.18.0.0
                        area 10.18.0.0 range 10.18.0.0/16 advertise
                        area 10.18.0.0 range 0.0.0.0/0 not-advertise
                        area 10.18.0.0 authentication message-digest
                        default-information originate always
                        neighbor 10.18.0.10
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc2_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.18.0.11
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp1s0
                        ip address 10.18.0.11/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.18.0.11
                    unicast_peer {
                        10.18.0.10
                    }
                    virtual_ipaddress {
                        10.18.0.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-e", vec![
            Config::new("firewall", Some(r#"
                networking.hostName = "server-e";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "enp3s0"
                    "wg0"
                ];
                networking.firewall.interfaces."enp2s0".allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.interfaces."enp2s0".allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];

                    serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.14/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc1_server-a_public_key" allowed-ips "172.21.7.10/32,10.17.0.0/16" endpoint "77.77.77.10:51820"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-c_public_key" allowed-ips "172.21.7.12/32,10.18.0.0/16" endpoint "77.77.77.12:51820"
                '';

                postStop = ''
                ip link del dev "wg0"
                '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.19.0.10
                        redistribute bgp
                        network 10.19.0.0/16 area 10.19.0.0
                        area 10.19.0.0 range 10.19.0.0/16 advertise
                        area 10.19.0.0 range 0.0.0.0/0 not-advertise
                        area 10.19.0.0 authentication message-digest
                        default-information originate always
                        neighbor 10.19.252.11
                        neighbor 10.19.252.12
                        neighbor 10.19.252.13
                      !
                      interface enp3s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.19.0.10
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp3s0
                        ip address 10.19.252.10/22
                      !
                      interface enp1s0
                        ip address 10.19.0.10/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.19.0.10
                    unicast_peer {
                        10.19.0.11
                    }
                    virtual_ipaddress {
                        10.19.0.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-f", vec![
            Config::new("firewall", Some(r#"
                networking.hostName = "server-f";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "enp3s0"
                    "wg0"
                ];
                networking.firewall.interfaces."enp2s0".allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.interfaces."enp2s0".allowedUDPPorts = [ 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];

                    serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.15/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc1_server-b_public_key" allowed-ips "172.21.7.11/32,10.17.0.0/16" endpoint "77.77.77.11:51820"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-d_public_key" allowed-ips "172.21.7.13/32,10.18.0.0/16" endpoint "77.77.77.13:51820"
                '';

                postStop = ''
                ip link del dev "wg0"
                '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.19.0.11
                        redistribute bgp
                        network 10.19.0.0/16 area 10.19.0.0
                        area 10.19.0.0 range 10.19.0.0/16 advertise
                        area 10.19.0.0 range 0.0.0.0/0 not-advertise
                        area 10.19.0.0 authentication message-digest
                        default-information originate always
                        neighbor 10.19.252.10
                        neighbor 10.19.252.12
                        neighbor 10.19.252.13
                      !
                      interface enp3s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.19.0.11
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp3s0
                        ip address 10.19.252.11/22
                      !
                      interface enp1s0
                        ip address 10.19.0.11/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.19.0.11
                    unicast_peer {
                        10.19.0.10
                    }
                    virtual_ipaddress {
                        10.19.0.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-h", vec![
            Config::new("wireguard_configs", None),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-h";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "enp2s0"
                ];
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.19.1.10
                        redistribute bgp
                        network 10.19.0.0/16 area 10.19.0.0
                        area 10.19.0.0 authentication message-digest
                        neighbor 10.19.252.10
                        neighbor 10.19.252.11
                        neighbor 10.19.252.13
                      !
                      interface enp2s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.19.1.10
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp2s0
                        ip address 10.19.252.12/22
                      !
                      interface enp1s0
                        ip address 10.19.1.10/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.19.1.10
                    unicast_peer {
                        10.19.1.11
                    }
                    virtual_ipaddress {
                        10.19.1.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-i", vec![
            Config::new("wireguard_configs", None),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-i";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "enp2s0"
                ];
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.19.1.11
                        redistribute bgp
                        network 10.19.0.0/16 area 10.19.0.0
                        area 10.19.0.0 authentication message-digest
                        neighbor 10.19.252.10
                        neighbor 10.19.252.11
                        neighbor 10.19.252.12
                      !
                      interface enp2s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.19.1.11
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp2s0
                        ip address 10.19.252.13/22
                      !
                      interface enp1s0
                        ip address 10.19.1.11/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.19.1.11
                    unicast_peer {
                        10.19.1.10
                    }
                    virtual_ipaddress {
                        10.19.1.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-j", vec![
            Config::new("wireguard_configs", None),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-j";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "enp2s0"
                ];
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.17.1.10
                        redistribute bgp
                        network 10.17.0.0/16 area 10.17.0.0
                        area 10.17.0.0 authentication message-digest
                        neighbor 10.17.252.10
                        neighbor 10.17.252.11
                        neighbor 10.17.252.13
                      !
                      interface enp2s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.17.1.10
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp2s0
                        ip address 10.17.252.12/22
                      !
                      interface enp1s0
                        ip address 10.17.1.10/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.1.10
                    unicast_peer {
                        10.17.1.11
                    }
                    virtual_ipaddress {
                        10.17.1.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-k", vec![
            Config::new("wireguard_configs", None),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-k";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                    "enp2s0"
                ];
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                    enable = true;
                    config = ''
                      !
                      router ospf
                        ospf router-id 10.17.1.11
                        redistribute bgp
                        network 10.17.0.0/16 area 10.17.0.0
                        area 10.17.0.0 authentication message-digest
                        neighbor 10.17.252.10
                        neighbor 10.17.252.11
                        neighbor 10.17.252.12
                      !
                      interface enp2s0
                        ip ospf cost 100
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                      !
                      interface enp1s0
                        ip ospf cost 500
                        ip ospf hello-interval 1
                        ip ospf dead-interval 3
                        ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                        ip ospf authentication message-digest
                        ip ospf network non-broadcast
                    '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                      !
                      ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                      !
                      ip prefix-list ANY seq 100 permit 0.0.0.0/0
                      !
                      route-map LANRM permit 100
                        match ip address prefix-list LAN
                        set src 10.17.1.11
                      !
                      route-map LANRM permit 110
                        match ip address prefix-list ANY
                      !
                      ip protocol ospf route-map LANRM
                      !
                      ip protocol bgp route-map LANRM
                      !
                      interface enp2s0
                        ip address 10.17.252.13/22
                      !
                      interface enp1s0
                        ip address 10.17.1.11/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface enp1s0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.1.11
                    unicast_peer {
                        10.17.1.10
                    }
                    virtual_ipaddress {
                        10.17.1.2
                    }
                }
            "#.to_string())),
        ]),
        ServerDescription::new("server-g", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-g";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                ];
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."enp1s0".ipv4.routes = [
                  { address = "10.17.0.1"; prefixLength = 32; via = "10.19.0.1"; }
                  { address = "10.0.0.0"; prefixLength = 8; via = "10.19.0.2"; }
                  { address = "0.0.0.0"; prefixLength = 0; via = "10.19.0.2"; }
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-l", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-l";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                ];
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."enp1s0".ipv4.routes = [
                  { address = "10.17.0.1"; prefixLength = 32; via = "10.17.1.1"; }
                  { address = "10.0.0.0"; prefixLength = 8; via = "10.17.1.2"; }
                  { address = "0.0.0.0"; prefixLength = 0; via = "10.17.1.2"; }
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-m", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-m";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "enp1s0"
                ];
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."enp1s0".ipv4.routes = [
                  { address = "10.17.0.1"; prefixLength = 32; via = "10.19.1.1"; }
                  { address = "10.0.0.0"; prefixLength = 8; via = "10.19.1.2"; }
                  { address = "0.0.0.0"; prefixLength = 0; via = "10.19.1.2"; }
                ];
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_aws_single_dc() {
    // single DC, what we expect:
    // VPN gateways provide internet?
    // If one subnet we expect VPN gateways to be subnet routers?
    //
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_aws_single_dc_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);

    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                    { address = "10.17.0.0"; prefixLength = 16; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.12

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                    { address = "10.17.0.0"; prefixLength = 16; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.12

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                            # ROUTES CREATE
                            ip route add 0.0.0.0/0 via 10.17.0.12

                            # ROUTES DELETE
                            ip route del 0.0.0.0/0

                            # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];

                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };

                  script = ''
                      modprobe wireguard || true
                      ip link add dev "wg0" type wireguard

                      # this might fail as kernel seems to remember ip address from previously
                      ip address add "172.21.7.10/16" dev "wg0" || true
                      wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                      ip link set up dev "wg0"

                      # peers
                      wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                  '';

                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                    family = "ip";
                    content = ''
                      chain EPL_POSTROUTING {
                        type nat hook postrouting priority 0;
                        ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                        ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                        ip saddr 10.17.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
                      }
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.12
                    unicast_peer {
                        10.17.0.13
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("consul_vrrp_switch_script", Some(r#"
               /run/current-system/sw/bin/echo '
                 # ROUTES CREATE
                 ip route add 0.0.0.0/0 via 10.17.0.12
                 # ROUTES DELETE
                 ip route del 0.0.0.0/0
                 # FINISH
                ' | \
                CONSUL_HTTP_TOKEN=$( ${pkgs.coreutils}/bin/cat /run/keys/consul-vrrp-token.txt ) \
                ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                            # ROUTES CREATE
                            ip route add 0.0.0.0/0 via 10.17.0.12

                            # ROUTES DELETE
                            ip route del 0.0.0.0/0

                            # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];

                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };

                  script = ''
                      modprobe wireguard || true
                      ip link add dev "wg0" type wireguard

                      # this might fail as kernel seems to remember ip address from previously
                      ip address add "172.21.7.11/16" dev "wg0" || true
                      wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                      ip link set up dev "wg0"

                      # peers
                      wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                  '';

                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                    family = "ip";
                    content = ''
                      chain EPL_POSTROUTING {
                        type nat hook postrouting priority 0;
                        ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                        ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                        ip saddr 10.17.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
                      }
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.13
                    unicast_peer {
                        10.17.0.12
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("consul_vrrp_switch_script", Some(r#"
                /run/current-system/sw/bin/echo '
                  # ROUTES CREATE
                  ip route add 0.0.0.0/0 via 10.17.0.13
                  # ROUTES DELETE
                  ip route del 0.0.0.0/0
                  # FINISH
                  ' | \
                CONSUL_HTTP_TOKEN=$( ${pkgs.coreutils}/bin/cat /run/keys/consul-vrrp-token.txt ) \
                ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_aws_single_dc_multisubnet() {
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_aws_single_dc_multisubnet_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);

    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                   enable = true;
                   config = ''
                       !
                       router ospf
                           ospf router-id 10.17.0.10
                           redistribute bgp
                           network 10.17.0.0/16 area 10.17.0.0
                           area 10.17.0.0 authentication message-digest
                           neighbor 10.17.252.11
                           neighbor 10.17.252.12
                           neighbor 10.17.252.13
                       !
                       interface eth1
                           ip ospf cost 100
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                       !
                       interface eth0
                           ip ospf cost 500
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                   '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                        !
                        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                        !
                        ip prefix-list ANY seq 100 permit 0.0.0.0/0
                        !
                        route-map LANRM permit 100
                            match ip address prefix-list LAN
                            set src 10.17.0.10
                        !
                        route-map LANRM permit 110
                            match ip address prefix-list ANY
                        !
                        ip protocol ospf route-map LANRM
                        !
                        ip protocol bgp route-map LANRM
                        !
                        ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
                        !
                        route-map LANRM deny 90
                        match ip address prefix-list INTERSUBNET
                        !
                        interface eth1
                            ip address 10.17.252.10/22
                        !
                        interface eth0
                            ip address 10.17.0.10/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.10
                    unicast_peer {
                        10.17.0.11
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.1.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.1.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "eth1"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                   enable = true;
                   config = ''
                       !
                       router ospf
                           ospf router-id 10.17.0.11
                           redistribute bgp
                           network 10.17.0.0/16 area 10.17.0.0
                           area 10.17.0.0 authentication message-digest
                           neighbor 10.17.252.10
                           neighbor 10.17.252.12
                           neighbor 10.17.252.13
                       !
                       interface eth1
                           ip ospf cost 100
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                       !
                       interface eth0
                           ip ospf cost 500
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                   '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                        !
                        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                        !
                        ip prefix-list ANY seq 100 permit 0.0.0.0/0
                        !
                        route-map LANRM permit 100
                            match ip address prefix-list LAN
                            set src 10.17.0.11
                        !
                        route-map LANRM permit 110
                            match ip address prefix-list ANY
                        !
                        ip protocol ospf route-map LANRM
                        !
                        ip protocol bgp route-map LANRM
                        !
                        ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
                        !
                        route-map LANRM deny 90
                        match ip address prefix-list INTERSUBNET
                        !
                        interface eth1
                            ip address 10.17.252.11/22
                        !
                        interface eth0
                            ip address 10.17.0.11/24
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.11
                    unicast_peer {
                        10.17.0.10
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.1.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.1.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "eth1"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.10/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                   enable = true;
                   config = ''
                       !
                       router ospf
                           ospf router-id 10.17.1.10
                           redistribute bgp
                           network 10.17.0.0/16 area 10.17.0.0
                           area 10.17.0.0 range 10.17.0.0/16 advertise
                           area 10.17.0.0 range 0.0.0.0/0 not-advertise
                           area 10.17.0.0 authentication message-digest
                           default-information originate always
                           neighbor 10.17.252.10
                           neighbor 10.17.252.11
                           neighbor 10.17.252.13
                       !
                       interface eth1
                           ip ospf cost 100
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                       !
                       interface eth0
                           ip ospf cost 500
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                   '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                        !
                        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                        !
                        ip prefix-list ANY seq 100 permit 0.0.0.0/0
                        !
                        route-map LANRM permit 100
                            match ip address prefix-list LAN
                            set src 10.17.1.10
                        !
                        route-map LANRM permit 110
                            match ip address prefix-list ANY
                        !
                        ip protocol ospf route-map LANRM
                        !
                        ip protocol bgp route-map LANRM
                        !
                        ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
                        !
                        route-map LANRM deny 90
                        match ip address prefix-list INTERSUBNET
                        !
                        interface eth1
                            ip address 10.17.252.12/22
                        !
                        interface eth0
                            ip address 10.17.1.10/24
                    '';
                };
            "#.to_string())),
            Config::new("frr_static_routes", Some(r#"
                # You gotta be kidding me... https://github.com/NixOS/nixpkgs/issues/274286
                services.frr.mgmt.enable = true;
                environment.etc."frr/staticd.conf".text = ''
                  !
                  ip route 10.17.0.0/16 10.17.1.1
                  !
                  ip route 0.0.0.0/0 10.17.1.1
                '';
                systemd.services.staticd.serviceConfig.ExecStart = lib.mkForce "${pkgs.frr}/libexec/frr/staticd -A localhost";
                services.frr.static.enable = true;
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.1.10
                    unicast_peer {
                        10.17.1.11
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.1.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.1.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.1.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "eth1"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.11/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                   enable = true;
                   config = ''
                       !
                       router ospf
                           ospf router-id 10.17.1.11
                           redistribute bgp
                           network 10.17.0.0/16 area 10.17.0.0
                           area 10.17.0.0 range 10.17.0.0/16 advertise
                           area 10.17.0.0 range 0.0.0.0/0 not-advertise
                           area 10.17.0.0 authentication message-digest
                           default-information originate always
                           neighbor 10.17.252.10
                           neighbor 10.17.252.11
                           neighbor 10.17.252.12
                       !
                       interface eth1
                           ip ospf cost 100
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                       !
                       interface eth0
                           ip ospf cost 500
                           ip ospf hello-interval 1
                           ip ospf dead-interval 3
                           ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                           ip ospf authentication message-digest
                           ip ospf network non-broadcast
                   '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                    enable = true;
                    config = ''
                        !
                        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                        !
                        ip prefix-list ANY seq 100 permit 0.0.0.0/0
                        !
                        route-map LANRM permit 100
                            match ip address prefix-list LAN
                            set src 10.17.1.11
                        !
                        route-map LANRM permit 110
                            match ip address prefix-list ANY
                        !
                        ip protocol ospf route-map LANRM
                        !
                        ip protocol bgp route-map LANRM
                        !
                        ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
                        !
                        route-map LANRM deny 90
                        match ip address prefix-list INTERSUBNET
                        !
                        interface eth1
                            ip address 10.17.252.13/22
                        !
                        interface eth0
                            ip address 10.17.1.11/24
                    '';
                };
            "#.to_string())),
            Config::new("frr_static_routes", Some(r#"
                # You gotta be kidding me... https://github.com/NixOS/nixpkgs/issues/274286
                services.frr.mgmt.enable = true;
                environment.etc."frr/staticd.conf".text = ''
                  !
                  ip route 10.17.0.0/16 10.17.1.1
                  !
                  ip route 0.0.0.0/0 10.17.1.1
                '';
                systemd.services.staticd.serviceConfig.ExecStart = lib.mkForce "${pkgs.frr}/libexec/frr/staticd -A localhost";
                services.frr.static.enable = true;
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.1.11
                    unicast_peer {
                        10.17.1.10
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.1.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.1.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.1.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "eth1"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-e", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("frr_static_routes", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                    { address = "10.17.0.0"; prefixLength = 16; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.1.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.1.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-e";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-f", vec![
            Config::new("wireguard_configs", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("frr_static_routes", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.1.1"; }
                    { address = "10.17.0.0"; prefixLength = 16; via = "10.17.1.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi


                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.1.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.1.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-f";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                ];
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_aws_multi_dc() {
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_aws_multi_dc_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);

    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.10/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.10
                    unicast_peer {
                        10.17.0.11
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc2" \
                        -description "VRRP policy for datacenter dc2" \
                        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc2" \
                        -policy-name "vrrp-policy-dc2" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/10.18.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.18.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/10.18.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc3" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc3" \
                        -description "VRRP policy for datacenter dc3" \
                        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc3" \
                        -policy-name "vrrp-policy-dc3" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.19.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.11/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.17.0.11
                    unicast_peer {
                        10.17.0.10
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc2" \
                        -description "VRRP policy for datacenter dc2" \
                        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc2" \
                        -policy-name "vrrp-policy-dc2" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/10.18.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.18.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/10.18.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc3" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc3" \
                        -description "VRRP policy for datacenter dc3" \
                        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc3" \
                        -policy-name "vrrp-policy-dc3" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.19.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.12/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.18.0.10
                    unicast_peer {
                        10.18.0.11
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.18.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc2" \
                        -description "VRRP policy for datacenter dc2" \
                        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc2" \
                        -policy-name "vrrp-policy-dc2" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/10.18.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.18.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/10.18.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc3" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc3" \
                        -description "VRRP policy for datacenter dc3" \
                        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc3" \
                        -policy-name "vrrp-policy-dc3" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.19.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.13/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.18.0.11
                    unicast_peer {
                        10.18.0.10
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.18.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc2" \
                        -description "VRRP policy for datacenter dc2" \
                        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc2" \
                        -policy-name "vrrp-policy-dc2" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/10.18.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.18.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/10.18.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc3" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc3" \
                        -description "VRRP policy for datacenter dc3" \
                        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc3" \
                        -policy-name "vrrp-policy-dc3" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.19.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-e", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.14/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.19.0.10
                    unicast_peer {
                        10.19.0.11
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.19.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.19.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc2" \
                        -description "VRRP policy for datacenter dc2" \
                        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc2" \
                        -policy-name "vrrp-policy-dc2" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/10.18.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.18.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/10.18.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc3" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc3" \
                        -description "VRRP policy for datacenter dc3" \
                        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc3" \
                        -policy-name "vrrp-policy-dc3" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.19.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-e";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-f", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.15/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                    description = "Keepalive Daemon (LVS and VRRP)";
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" "network-online.target" "syslog.target" ];
                    wants = [ "network-online.target" ];
                    serviceConfig = {
                        Type = "forking";
                        PIDFile = "/run/keepalived.pid";
                        KillMode = "process";
                        RuntimeDirectory = "keepalived";
                        ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                        Restart = "always";
                        RestartSec = "1s";
                    };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                    enable_script_security
                    script_user consul
                }

                vrrp_instance vpnRouter {
                    interface eth0
                    state MASTER
                    virtual_router_id 1
                    priority 50
                    unicast_src_ip 10.19.0.11
                    unicast_peer {
                        10.19.0.10
                    }
                    virtual_ipaddress {
                    }
                    notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.19.0.1"; }
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.19.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done


                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc1" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc1" \
                        -description "VRRP policy for datacenter dc1" \
                        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc1" \
                        -policy-name "vrrp-policy-dc1" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.17.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc2" \
                        -description "VRRP policy for datacenter dc2" \
                        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc2" \
                        -policy-name "vrrp-policy-dc2" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/10.18.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.18.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/10.18.0.0p24 -

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc3" {
                        policy = "write"
                    }
                EOL

                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc3" \
                        -description "VRRP policy for datacenter dc3" \
                        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc3" \
                        -policy-name "vrrp-policy-dc3" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi

                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                    # ROUTES CREATE
                    ip route add 0.0.0.0/0 via 10.19.0.10

                    # ROUTES DELETE
                    ip route del 0.0.0.0/0

                    # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-f";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_gcloud_single_dc() {
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_gcloud_single_dc_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);

    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.10/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.11/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_gcloud_multi_dc() {
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_gcloud_multi_dc_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);

    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.10/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.11/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.12/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.13/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-e", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.14/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.19.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-e";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-f", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.15/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.19.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-f";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
    ]);
}


#[test]
fn test_network_simulation_config_gcloud_single_dc_multisubnet() {
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_gcloud_single_dc_multisub_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);

    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.10/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.1.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                    description = "WireGuard Tunnel - wg0";
                    after = [ "network-pre.target" ];
                    wants = [ "network.target" ];
                    before = [ "network.target" ];
                    wantedBy = [ "multi-user.target" ];
                    environment.DEVICE = "wg0";
                    path = with pkgs; [ kmod iproute2 wireguard-tools ];
                    serviceConfig = {
                        Type = "oneshot";
                        RemainAfterExit = true;
                        Restart = "on-failure";
                        RestartSec = "10s";
                    };
                    script = ''
                        modprobe wireguard || true
                        ip link add dev "wg0" type wireguard
                        # this might fail as kernel seems to remember ip address from previously
                        ip address add "172.21.7.11/16" dev "wg0" || true
                        wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                        ip link set up dev "wg0"
                        # peers
                        wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    '';
                    postStop = ''
                        ip link del dev "wg0"
                    '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.1.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-e", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-e";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-f", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.1.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-f";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_gcloud_aws_multi_dc() {
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_gcloud_aws_multi_dc_env());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);


    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                  family = "ip";
                  content = ''
                    chain EPL_POSTROUTING {
                      type nat hook postrouting priority 0;
                      ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                      ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                    }
                  '';
                };
            "#.to_string())),
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", Some(r#"
                networking.nftables.tables.l3-vpn-hop-address-translation = {
                  family = "ip";
                  content = ''
                    chain PREROUTING {
                      type filter hook prerouting priority -300; policy accept;
                      ip daddr 10.17.0.12 ip saddr 10.0.0.0/8 ip daddr set 10.17.128.12;
                      ip saddr 10.17.128.12 ip saddr set 10.17.0.12;
                    }
                  '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.17.0.10
                      redistribute bgp
                      network 10.17.0.0/16 area 10.17.0.0
                      area 10.17.0.0 range 10.17.0.0/16 advertise
                      area 10.17.0.0 range 0.0.0.0/0 not-advertise
                      area 10.17.0.0 authentication message-digest
                      neighbor 10.17.0.11
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.17.0.10
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
                    !
                    ip prefix-list INTERSUBNET seq 101 permit 10.17.0.0/16 le 24
                    !
                    route-map LANRM deny 90
                      match ip address prefix-list INTERSUBNET
                    !
                    interface eth0
                      ip address 10.17.0.10/24
                  '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                  enable_script_security
                  script_user consul
                }

                vrrp_instance vpnRouter {
                  interface eth0
                  state MASTER
                  virtual_router_id 1
                  priority 50
                  unicast_src_ip 10.17.0.10
                  unicast_peer {
                    10.17.0.11
                  }
                  virtual_ipaddress {
                  }
                  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];
                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };
                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard
                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.10/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"
                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-e_public_key" allowed-ips "172.21.7.14/32,10.19.0.0/16" endpoint "77.77.77.14:51820"
                  '';
                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-a";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                  family = "ip";
                  content = ''
                    chain EPL_POSTROUTING {
                      type nat hook postrouting priority 0;
                      ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                      ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                    }
                  '';
                };
            "#.to_string())),
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", Some(r#"
                networking.nftables.tables.l3-vpn-hop-address-translation = {
                  family = "ip";
                  content = ''
                    chain PREROUTING {
                      type filter hook prerouting priority -300; policy accept;
                      ip daddr 10.17.0.12 ip saddr 10.0.0.0/8 ip daddr set 10.17.128.12;
                      ip saddr 10.17.128.12 ip saddr set 10.17.0.12;
                    }
                  '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.17.0.11
                      redistribute bgp
                      network 10.17.0.0/16 area 10.17.0.0
                      area 10.17.0.0 range 10.17.0.0/16 advertise
                      area 10.17.0.0 range 0.0.0.0/0 not-advertise
                      area 10.17.0.0 authentication message-digest
                      neighbor 10.17.0.10
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.17.0.11
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
                    !
                    ip prefix-list INTERSUBNET seq 101 permit 10.17.0.0/16 le 24
                    !
                    route-map LANRM deny 90
                      match ip address prefix-list INTERSUBNET
                    !
                    interface eth0
                      ip address 10.17.0.11/24
                  '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                  enable_script_security
                  script_user consul
                }

                vrrp_instance vpnRouter {
                  interface eth0
                  state MASTER
                  virtual_router_id 1
                  priority 50
                  unicast_src_ip 10.17.0.11
                  unicast_peer {
                    10.17.0.10
                  }
                  virtual_ipaddress {
                  }
                  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];
                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };
                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard
                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.11/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"
                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-f_public_key" allowed-ips "172.21.7.15/32,10.19.0.0/16" endpoint "77.77.77.15:51820"
                  '';
                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-b";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                  family = "ip";
                  content = ''
                    chain EPL_POSTROUTING {
                      type nat hook postrouting priority 0;
                      ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                      ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                    }
                  '';
                };
            "#.to_string())),
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", Some(r#"
                networking.nftables.tables.l3-vpn-hop-address-translation = {
                  family = "ip";
                  content = ''
                    chain PREROUTING {
                      type filter hook prerouting priority -300; policy accept;
                      ip daddr 10.18.0.12 ip saddr 10.0.0.0/8 ip daddr set 10.18.128.12;
                      ip saddr 10.18.128.12 ip saddr set 10.18.0.12;
                    }
                  '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.18.0.10
                      redistribute bgp
                      network 10.18.0.0/16 area 10.18.0.0
                      area 10.18.0.0 range 10.18.0.0/16 advertise
                      area 10.18.0.0 range 0.0.0.0/0 not-advertise
                      area 10.18.0.0 authentication message-digest
                      neighbor 10.18.0.11
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc2_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.18.0.10
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    ip prefix-list INTERSUBNET seq 100 permit 10.18.0.0/16 le 24
                    !
                    ip prefix-list INTERSUBNET seq 101 permit 10.18.0.0/16 le 24
                    !
                    route-map LANRM deny 90
                      match ip address prefix-list INTERSUBNET
                    !
                    interface eth0
                      ip address 10.18.0.10/24
                  '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                  enable_script_security
                  script_user consul
                }

                vrrp_instance vpnRouter {
                  interface eth0
                  state MASTER
                  virtual_router_id 1
                  priority 50
                  unicast_src_ip 10.18.0.10
                  unicast_peer {
                    10.18.0.11
                  }
                  virtual_ipaddress {
                  }
                  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];
                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };
                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard
                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.12/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"
                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-e_public_key" allowed-ips "172.21.7.14/32,10.19.0.0/16" endpoint "77.77.77.14:51820"
                  '';
                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-c";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                  family = "ip";
                  content = ''
                    chain EPL_POSTROUTING {
                      type nat hook postrouting priority 0;
                      ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                      ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                    }
                  '';
                };
            "#.to_string())),
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", Some(r#"
                networking.nftables.tables.l3-vpn-hop-address-translation = {
                  family = "ip";
                  content = ''
                    chain PREROUTING {
                      type filter hook prerouting priority -300; policy accept;
                      ip daddr 10.18.0.12 ip saddr 10.0.0.0/8 ip daddr set 10.18.128.12;
                      ip saddr 10.18.128.12 ip saddr set 10.18.0.12;
                    }
                  '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.18.0.11
                      redistribute bgp
                      network 10.18.0.0/16 area 10.18.0.0
                      area 10.18.0.0 range 10.18.0.0/16 advertise
                      area 10.18.0.0 range 0.0.0.0/0 not-advertise
                      area 10.18.0.0 authentication message-digest
                      neighbor 10.18.0.10
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc2_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.18.0.11
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    ip prefix-list INTERSUBNET seq 100 permit 10.18.0.0/16 le 24
                    !
                    ip prefix-list INTERSUBNET seq 101 permit 10.18.0.0/16 le 24
                    !
                    route-map LANRM deny 90
                      match ip address prefix-list INTERSUBNET
                    !
                    interface eth0
                      ip address 10.18.0.11/24
                  '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                  enable_script_security
                  script_user consul
                }

                vrrp_instance vpnRouter {
                  interface eth0
                  state MASTER
                  virtual_router_id 1
                  priority 50
                  unicast_src_ip 10.18.0.11
                  unicast_peer {
                    10.18.0.10
                  }
                  virtual_ipaddress {
                  }
                  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];
                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };
                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard
                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.13/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"
                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc3_server-f_public_key" allowed-ips "172.21.7.15/32,10.19.0.0/16" endpoint "77.77.77.15:51820"
                  '';
                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-d";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                    "vpnGre"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-e", vec![
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", None),
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                  family = "ip";
                  content = ''
                    chain EPL_POSTROUTING {
                      type nat hook postrouting priority 0;
                      ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                      ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                      ip saddr 10.19.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
                    }
                  '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.19.0.10
                      redistribute bgp
                      network 10.19.0.0/16 area 10.19.0.0
                      area 10.19.0.0 range 10.19.0.0/16 advertise
                      area 10.19.0.0 range 0.0.0.0/0 not-advertise
                      area 10.19.0.0 authentication message-digest
                      default-information originate always
                      neighbor 10.19.0.11
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.19.0.10
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    ip prefix-list INTERSUBNET seq 100 permit 10.19.0.0/16 le 24
                    !
                    route-map LANRM deny 90
                      match ip address prefix-list INTERSUBNET
                    !
                    interface eth0
                      ip address 10.19.0.10/24
                  '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                  enable_script_security
                  script_user consul
                }

                vrrp_instance vpnRouter {
                  interface eth0
                  state MASTER
                  virtual_router_id 1
                  priority 50
                  unicast_src_ip 10.19.0.10
                  unicast_peer {
                    10.19.0.11
                  }
                  virtual_ipaddress {
                  }
                  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];
                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };
                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard
                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.14/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"
                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc1_server-a_public_key" allowed-ips "172.21.7.10/32,10.17.0.0/16" endpoint "77.77.77.10:51820"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-c_public_key" allowed-ips "172.21.7.12/32,10.18.0.0/16" endpoint "77.77.77.12:51820"
                  '';
                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
               networking.interfaces."eth0".ipv4.routes = [
                   { address = "169.254.169.254"; prefixLength = 32; via = "10.19.0.1"; }
               ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-e";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];
                networking.firewall.allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-f", vec![
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", None),
            Config::new("epl_nft_rules_epl-nat", Some(r#"
                networking.nftables.tables.epl-nat = {
                  family = "ip";
                  content = ''
                    chain EPL_POSTROUTING {
                      type nat hook postrouting priority 0;
                      ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
                      ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
                      ip saddr 10.19.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
                    }
                  '';
                };
            "#.to_string())),
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.19.0.11
                      redistribute bgp
                      network 10.19.0.0/16 area 10.19.0.0
                      area 10.19.0.0 range 10.19.0.0/16 advertise
                      area 10.19.0.0 range 0.0.0.0/0 not-advertise
                      area 10.19.0.0 authentication message-digest
                      default-information originate always
                      neighbor 10.19.0.10
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc3_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.19.0.11
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    ip prefix-list INTERSUBNET seq 100 permit 10.19.0.0/16 le 24
                    !
                    route-map LANRM deny 90
                      match ip address prefix-list INTERSUBNET
                    !
                    interface eth0
                      ip address 10.19.0.11/24
                  '';
                };
            "#.to_string())),
            Config::new("keepalived", Some(r#"
                systemd.services.keepalived = {
                  description = "Keepalive Daemon (LVS and VRRP)";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" "network-online.target" "syslog.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = {
                    Type = "forking";
                    PIDFile = "/run/keepalived.pid";
                    KillMode = "process";
                    RuntimeDirectory = "keepalived";
                    ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
                    ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                    Restart = "always";
                    RestartSec = "1s";
                  };
                };
            "#.to_string())),
            Config::new("secret_value_keepalived.conf", Some(r#"
                global_defs {
                  enable_script_security
                  script_user consul
                }

                vrrp_instance vpnRouter {
                  interface eth0
                  state MASTER
                  virtual_router_id 1
                  priority 50
                  unicast_src_ip 10.19.0.11
                  unicast_peer {
                    10.19.0.10
                  }
                  virtual_ipaddress {
                  }
                  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch
                }
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];
                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };
                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard
                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.15/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"
                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc1_server-b_public_key" allowed-ips "172.21.7.11/32,10.17.0.0/16" endpoint "77.77.77.11:51820"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-d_public_key" allowed-ips "172.21.7.13/32,10.18.0.0/16" endpoint "77.77.77.13:51820"
                  '';
                  postStop = ''
                    ip link del dev "wg0"
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.19.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-f";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = false;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "wg0"
                ];
                networking.firewall.allowedTCPPorts = [ 22 ];
                networking.firewall.allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-g", vec![
            Config::new("epl_nft_rules_epl-nat", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", None),
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", Some(r#"
                networking.nftables.tables.l3-vpn-hop-address-translation = {
                  family = "ip";
                  content = ''
                    chain PREROUTING {
                      type filter hook prerouting priority -300; policy accept;
                      ip daddr 10.17.128.12 ip daddr set 10.17.0.12
                    }
                    chain SNAT_POSTROUTING {
                      type nat hook postrouting priority srcnat; policy accept;
                      ip daddr 10.0.0.0/8 snat to 10.17.0.12
                    }
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-g";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-h", vec![
            Config::new("epl_nft_rules_epl-nat", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", None),
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", Some(r#"
                networking.nftables.tables.l3-vpn-hop-address-translation = {
                  family = "ip";
                  content = ''
                    chain PREROUTING {
                      type filter hook prerouting priority -300; policy accept;
                      ip daddr 10.18.128.12 ip daddr set 10.18.0.12
                    }
                    chain SNAT_POSTROUTING {
                      type nat hook postrouting priority srcnat; policy accept;
                      ip daddr 10.0.0.0/8 snat to 10.18.0.12
                    }
                  '';
                };
            "#.to_string())),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-h";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                    "vpnGre"
                ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-i", vec![
            Config::new("epl_nft_rules_epl-nat", None),
            Config::new("frr_ospf_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("keepalived", None),
            Config::new("secret_value_keepalived.conf", None),
            Config::new("wireguard_configs", None),
            Config::new("epl_nft_rules_l3-vpn-hop-address-translation", None),
            Config::new("static_node_routes", Some(r#"
                networking.interfaces."eth0".ipv4.routes = [
                    { address = "169.254.169.254"; prefixLength = 32; via = "10.19.0.1"; }
                    { address = "10.19.0.0"; prefixLength = 16; via = "10.19.0.1"; }
                ];
            "#.to_string())),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
                  key_prefix "epl-interdc-routes/dc1" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc1" \
                    -description "VRRP policy for datacenter dc1" \
                    -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc1" \
                    -policy-name "vrrp-policy-dc1" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.17.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc1.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                  cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                    key_prefix "epl-interdc-routes/dc2" {
                    policy = "write"
                  }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc2" \
                    -description "VRRP policy for datacenter dc2" \
                    -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc2" \
                    -policy-name "vrrp-policy-dc2" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
                  # ROUTES CREATE
                  ip route add 10.19.0.0/16 via 10.18.128.10
                  # ROUTES DELETE
                  ip route del 10.19.0.0/16
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
                      key_prefix "epl-interdc-routes/dc3" {
                      policy = "write"
                    }
                EOL
                  ${pkgs.consul}/bin/consul acl policy create \
                    -name "vrrp-policy-dc3" \
                    -description "VRRP policy for datacenter dc3" \
                    -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl
                  ${pkgs.consul}/bin/consul acl token create \
                    -description "VRRP Token for datacenter dc3" \
                    -policy-name "vrrp-policy-dc3" \
                    -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
                fi
                ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
                  # ROUTES CREATE
                  ip route add 10.0.0.0/8 via 10.19.0.10
                  ip route add 0.0.0.0/0 via 10.19.0.10
                  # ROUTES DELETE
                  ip route del 10.0.0.0/8
                  ip route del 0.0.0.0/0
                  # FINISH
                ' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -
                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc3.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
                networking.hostName = "server-i";
                networking.firewall.allowPing = true;
                networking.firewall.enable = true;
                networking.firewall.checkReversePath = true;
                networking.firewall.trustedInterfaces = [
                    "eth0"
                ];
            "#.to_string())),
        ]),
    ]);
}

#[test]
fn test_network_simulation_config_testvms_single_dc_coprocessor() {
    // single DC, what we expect:
    // VPN gateways provide internet?
    // If one subnet we expect VPN gateways to be subnet routers?
    let db = common::assert_platform_validation_success_plain(super::scenarios::scenario_single_dc_coprocessor());
    let mut secrets = SecretsStorage::new_testing();
    let plan = generate_outputs(&db, &mut secrets);
    common::ensure_config_plans(&plan, vec![
        ServerDescription::new("server-a", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_bgp_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", None),
            Config::new("consul_vrrp_bootstrap_script", Some(r#"
                export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )
                while :
                do
                    consul members | grep alive &>/dev/null && break
                    sleep 1
                done

                if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
                then
                    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
                        key_prefix "epl-interdc-routes/dc2" {
                            policy = "write"
                        }
                EOL
                    ${pkgs.consul}/bin/consul acl policy create \
                        -name "vrrp-policy-dc2" \
                        -description "VRRP policy for datacenter dc2" \
                        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl
                    ${pkgs.consul}/bin/consul acl token create \
                        -description "VRRP Token for datacenter dc2" \
                        -policy-name "vrrp-policy-dc2" \
                        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
                fi

                # after policy provisioning key is no longer needed
                rm -f /run/keys/consul-vrrp-token-dc2.txt
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-a";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = true;
              networking.firewall.trustedInterfaces = [
                "eth0"
              ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-b", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_bgp_config", None),
            Config::new("frr_zebra_config", None),
            Config::new("wireguard_configs", None),
        ]),
        ServerDescription::new("server-c", vec![
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.17.0.12
                      redistribute bgp
                      network 10.17.0.0/16 area 10.17.0.0
                      area 10.17.0.0 range 10.17.0.0/16 advertise
                      area 10.17.0.0 range 0.0.0.0/0 not-advertise
                      area 10.17.0.0 authentication message-digest
                      default-information originate always
                      neighbor 10.17.0.13
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_bgp_config", Some(r#"
                services.frr.bgp = {
                  enable = true;
                  config = ''
                    !
                    router bgp 64529
                      bgp router-id 10.17.0.12
                      address-family ipv4 unicast
                      network 10.17.0.0/16
                      exit-address-family
                      neighbor 10.17.0.13 remote-as 64529
                      neighbor 10.17.0.13 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-c.server-d
                      neighbor 10.17.0.13 bfd
                      neighbor 172.21.8.10 remote-as 4201179658
                      neighbor 172.21.8.10 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-c.server-e
                      neighbor 172.21.8.10 bfd
                      neighbor 172.21.8.12 remote-as 4201179659
                      neighbor 172.21.8.12 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-c.server-f
                      neighbor 172.21.8.12 bfd
                      address-family ipv4 unicast
                        network 10.17.0.0/16
                        aggregate-address 10.18.0.0/16
                      exit-address-family
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.17.0.12
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    interface eth0
                      ip address 10.17.0.12/24
                  '';
                };
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
              systemd.services.wireguard-wg0 = {
                description = "WireGuard Tunnel - wg0";
                after = [ "network-pre.target" ];
                wants = [ "network.target" ];
                before = [ "network.target" ];
                wantedBy = [ "multi-user.target" ];
                environment.DEVICE = "wg0";
                path = with pkgs; [ kmod iproute2 wireguard-tools ];

                serviceConfig = {
                  Type = "oneshot";
                  RemainAfterExit = true;
                  Restart = "on-failure";
                  RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.10/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-e_wg0_public_key" allowed-ips "172.21.8.10/32,10.18.0.10/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-f_wg0_public_key" allowed-ips "172.21.8.12/32,10.18.0.11/32"
                '';

                postStop = ''
                  ip link del dev "wg0"
                '';
              };
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-c";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = false;
              networking.firewall.trustedInterfaces = [
                "eth0"
                "wg0"
              ];
              networking.firewall.interfaces."eth1".allowedTCPPorts = [ 22 80 443 53 ];
              networking.firewall.interfaces."eth1".allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-d", vec![
            Config::new("frr_ospf_config", Some(r#"
                services.frr.ospf = {
                  enable = true;
                  config = ''
                    !
                    router ospf
                      ospf router-id 10.17.0.13
                      redistribute bgp
                      network 10.17.0.0/16 area 10.17.0.0
                      area 10.17.0.0 range 10.17.0.0/16 advertise
                      area 10.17.0.0 range 0.0.0.0/0 not-advertise
                      area 10.17.0.0 authentication message-digest
                      default-information originate always
                      neighbor 10.17.0.12
                    !
                    interface eth0
                      ip ospf cost 500
                      ip ospf hello-interval 1
                      ip ospf dead-interval 3
                      ip ospf message-digest-key 12 md5 SECRET_VALUE_ospf_dc_dc1_key
                      ip ospf authentication message-digest
                      ip ospf network non-broadcast
                  '';
                };
            "#.to_string())),
            Config::new("frr_bgp_config", Some(r#"
                services.frr.bgp = {
                  enable = true;
                  config = ''
                    !
                    router bgp 64529
                      bgp router-id 10.17.0.13
                      address-family ipv4 unicast
                      network 10.17.0.0/16
                      exit-address-family
                      neighbor 10.17.0.12 remote-as 64529
                      neighbor 10.17.0.12 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-c.server-d
                      neighbor 10.17.0.12 bfd
                      neighbor 172.21.8.11 remote-as 4201179658
                      neighbor 172.21.8.11 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-d.server-e
                      neighbor 172.21.8.11 bfd
                      neighbor 172.21.8.13 remote-as 4201179659
                      neighbor 172.21.8.13 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-d.server-f
                      neighbor 172.21.8.13 bfd
                      address-family ipv4 unicast
                        network 10.17.0.0/16
                        aggregate-address 10.18.0.0/16
                      exit-address-family
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.17.0.13
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol ospf route-map LANRM
                    !
                    ip protocol bgp route-map LANRM
                    !
                    interface eth0
                      ip address 10.17.0.13/24
                  '';
                };
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
              systemd.services.wireguard-wg0 = {
                description = "WireGuard Tunnel - wg0";
                after = [ "network-pre.target" ];
                wants = [ "network.target" ];
                before = [ "network.target" ];
                wantedBy = [ "multi-user.target" ];
                environment.DEVICE = "wg0";
                path = with pkgs; [ kmod iproute2 wireguard-tools ];

                serviceConfig = {
                  Type = "oneshot";
                  RemainAfterExit = true;
                  Restart = "on-failure";
                  RestartSec = "10s";
                };

                script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.7.11/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set wg0 peer "SECRET_VALUE_wireguard_admin_vpn_1_public_key" allowed-ips "172.21.7.254/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-e_wg1_public_key" allowed-ips "172.21.8.11/32,10.18.0.10/32"
                    wg set wg0 peer "SECRET_VALUE_wireguard_dc2_server-f_wg1_public_key" allowed-ips "172.21.8.13/32,10.18.0.11/32"
                '';

                postStop = ''
                  ip link del dev "wg0"
                '';
              };
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-d";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = false;
              networking.firewall.trustedInterfaces = [
                "eth0"
                "wg0"
              ];
              networking.firewall.interfaces."eth1".allowedTCPPorts = [ 22 80 443 53 ];
              networking.firewall.interfaces."eth1".allowedUDPPorts = [ 53 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-e", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_bgp_config", Some(r#"
                services.frr.bgp = {
                  enable = true;
                  config = ''
                    !
                    router bgp 4201179658
                    bgp router-id 10.18.0.10
                    address-family ipv4 unicast
                      redistribute connected
                      redistribute static
                      network 10.18.0.10/32
                    exit-address-family
                    neighbor 172.21.7.10 remote-as 64529
                    neighbor 172.21.7.10 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-c.server-e
                    neighbor 172.21.7.10 bfd
                    neighbor 172.21.7.10 disable-connected-check
                    neighbor 172.21.7.11 remote-as 64529
                    neighbor 172.21.7.11 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-d.server-e
                    neighbor 172.21.7.11 bfd
                    neighbor 172.21.7.11 disable-connected-check
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.18.0.10
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol bgp route-map LANRM
                    !
                    interface eth0
                      ip address 10.18.0.10/32
                  '';
                };
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];

                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };

                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.8.10/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key-wg0" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set "wg0" peer "SECRET_VALUE_wireguard_dc1_server-c_public_key" allowed-ips "172.21.7.10/32,10.0.0.0/8" endpoint "77.77.77.12:51820" persistent-keepalive 10
                    wg set "wg0" peer "SECRET_VALUE_wireguard_dc2_server-f_wg0_public_key" allowed-ips "172.21.8.12/32,10.18.0.11/32" endpoint "77.77.77.15:51820" persistent-keepalive 10

                    # routes
                    ip route del '172.21.0.0/16' dev wg0 || true
                    ip route add '172.21.7.10/32' dev wg0 || true
                    ip route add '172.21.8.12/32' dev wg0 || true
                    ip route add '10.18.0.11/32' via 172.21.8.12 src 10.18.0.10 || true
                  '';

                  postStop = ''

                    ip route del '172.21.8.12/32' || true
                    ip route del '10.18.0.11/32' || true
                    ip route del '172.21.7.10/32' dev wg0 || true

                    ip link del dev "wg0"
                  '';
                };

                systemd.services.wireguard-wg1 = {
                  description = "WireGuard Tunnel - wg1";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg1";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];

                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };

                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg1" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.8.11/16" dev "wg1" || true
                    wg set "wg1" private-key "/run/keys/epl-wireguard-key-wg1" listen-port "51821"
                    ip link set up dev "wg1"

                    # peers
                    wg set "wg1" peer "SECRET_VALUE_wireguard_dc1_server-d_public_key" allowed-ips "172.21.7.11/32,10.0.0.0/8" endpoint "77.77.77.13:51820" persistent-keepalive 10

                    # routes
                    ip route del '172.21.0.0/16' dev wg1 || true
                    ip route add '172.21.7.11/32' dev wg1 || true
                  '';

                  postStop = ''

                    ip route del '172.21.7.11/32' dev wg1 || true

                    ip link del dev "wg1"
                  '';
                };
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-e";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = true;
              networking.firewall.trustedInterfaces = [
                "eth0"
                "wg0"
                "wg1"
              ];
              networking.firewall.interfaces."eth0".allowedTCPPorts = [ 22 ];
              networking.firewall.interfaces."eth0".allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
        ServerDescription::new("server-f", vec![
            Config::new("frr_ospf_config", None),
            Config::new("frr_bgp_config", Some(r#"
                services.frr.bgp = {
                  enable = true;
                  config = ''
                    !
                    router bgp 4201179659
                    bgp router-id 10.18.0.11
                    address-family ipv4 unicast
                      redistribute connected
                      redistribute static
                      network 10.18.0.11/32
                    exit-address-family
                    neighbor 172.21.7.10 remote-as 64529
                    neighbor 172.21.7.10 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-c.server-f
                    neighbor 172.21.7.10 bfd
                    neighbor 172.21.7.10 disable-connected-check
                    neighbor 172.21.7.11 remote-as 64529
                    neighbor 172.21.7.11 password DETERMINISTIC_PW_SECRET_VALUE_bgp_peering_seed.server-d.server-f
                    neighbor 172.21.7.11 bfd
                    neighbor 172.21.7.11 disable-connected-check
                  '';
                };
            "#.to_string())),
            Config::new("frr_zebra_config", Some(r#"
                services.frr.zebra = {
                  enable = true;
                  config = ''
                    !
                    ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
                    !
                    ip prefix-list ANY seq 100 permit 0.0.0.0/0
                    !
                    route-map LANRM permit 100
                      match ip address prefix-list LAN
                      set src 10.18.0.11
                    !
                    route-map LANRM permit 110
                      match ip address prefix-list ANY
                    !
                    ip protocol bgp route-map LANRM
                    !
                    interface eth0
                      ip address 10.18.0.11/32
                  '';
                };
            "#.to_string())),
            Config::new("wireguard_configs", Some(r#"
                systemd.services.wireguard-wg0 = {
                  description = "WireGuard Tunnel - wg0";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg0";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];

                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };

                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg0" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.8.12/16" dev "wg0" || true
                    wg set "wg0" private-key "/run/keys/epl-wireguard-key-wg0" listen-port "51820"
                    ip link set up dev "wg0"

                    # peers
                    wg set "wg0" peer "SECRET_VALUE_wireguard_dc1_server-c_public_key" allowed-ips "172.21.7.10/32,10.0.0.0/8" endpoint "77.77.77.12:51820" persistent-keepalive 10
                    wg set "wg0" peer "SECRET_VALUE_wireguard_dc2_server-e_wg0_public_key" allowed-ips "172.21.8.10/32,10.18.0.10/32" endpoint "77.77.77.14:51820" persistent-keepalive 10

                    # routes
                    ip route del '172.21.0.0/16' dev wg0 || true
                    ip route add '172.21.7.10/32' dev wg0 || true
                    ip route add '172.21.8.10/32' dev wg0 || true
                    ip route add '10.18.0.10/32' via 172.21.8.10 src 10.18.0.11 || true
                  '';

                  postStop = ''

                    ip route del '172.21.8.10/32' || true
                    ip route del '10.18.0.10/32' || true
                    ip route del '172.21.7.10/32' dev wg0 || true

                    ip link del dev "wg0"
                  '';
                };

                systemd.services.wireguard-wg1 = {
                  description = "WireGuard Tunnel - wg1";
                  after = [ "network-pre.target" ];
                  wants = [ "network.target" ];
                  before = [ "network.target" ];
                  wantedBy = [ "multi-user.target" ];
                  environment.DEVICE = "wg1";
                  path = with pkgs; [ kmod iproute2 wireguard-tools ];

                  serviceConfig = {
                    Type = "oneshot";
                    RemainAfterExit = true;
                    Restart = "on-failure";
                    RestartSec = "10s";
                  };

                  script = ''
                    modprobe wireguard || true
                    ip link add dev "wg1" type wireguard

                    # this might fail as kernel seems to remember ip address from previously
                    ip address add "172.21.8.13/16" dev "wg1" || true
                    wg set "wg1" private-key "/run/keys/epl-wireguard-key-wg1" listen-port "51821"
                    ip link set up dev "wg1"

                    # peers
                    wg set "wg1" peer "SECRET_VALUE_wireguard_dc1_server-d_public_key" allowed-ips "172.21.7.11/32,10.0.0.0/8" endpoint "77.77.77.13:51820" persistent-keepalive 10

                    # routes
                    ip route del '172.21.0.0/16' dev wg1 || true
                    ip route add '172.21.7.11/32' dev wg1 || true
                  '';

                  postStop = ''

                    ip route del '172.21.7.11/32' dev wg1 || true

                    ip link del dev "wg1"
                  '';
                };
            "#.to_string())),
            Config::new("firewall", Some(r#"
              networking.hostName = "server-f";
              networking.firewall.allowPing = true;
              networking.firewall.enable = true;
              networking.firewall.checkReversePath = true;
              networking.firewall.trustedInterfaces = [
                "eth0"
                "wg0"
                "wg1"
              ];
              networking.firewall.interfaces."eth0".allowedTCPPorts = [ 22 ];
              networking.firewall.interfaces."eth0".allowedUDPPorts = [ 51820 ];
            "#.to_string())),
        ]),
    ]);
}
