#[cfg(test)]
use crate::static_analysis::PlatformValidationError;
#[cfg(test)]
use crate::tests::common::assert_eden_db_error_wcustom_data;
#[cfg(test)]
use crate::tests::common::assert_platform_validation_error_wcustom_data;
#[cfg(test)]
use crate::tests::common;
#[cfg(test)]
use crate::tests::common::assert_platform_validation_error_wcustom_data_wargs;
#[cfg(test)]
use crate::tests::common::assert_platform_validation_success_wargs;

#[test]
fn test_detect_invalid_ips() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA server {
    server1, eth0 WITH network_interface {
        eth0, internet, a.b.c.d;
    };
}

DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InvalidIpV4Address {
            server_name: "server1".to_string(),
            interface_name: "eth0".to_string(),
            value: "a.b.c.d".to_string(),
            parsing_error: "invalid IPv4 address syntax".to_string(),
        }
    )
}

#[test]
fn test_detect_invalid_subnets() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA server {
    server1, eth0 WITH network_interface {
        eth0, internet, a.b.c.d;
    };
}

DATA EXCLUSIVE network {
    internet, "300.0.0.0/0";
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InvalidNetworkIpV4Subnet {
            network_name: "internet".to_string(),
            subnet_value: "300.0.0.0/0".to_string(),
            parsing_error: "invalid IP address syntax".to_string(),
        }
    )
}

#[test]
fn test_mismatching_subnets() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA server {
    server1, eth0 WITH network_interface {
        eth0, internet, 100.100.101.100;
    };
}

DATA EXCLUSIVE network {
    internet, "100.100.100.0/24";
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InterfaceIpIsNotInsideSubnet {
            server_name: "server1".to_string(),
            interface_name: "eth0".to_string(),
            interface_ip: "100.100.101.100/24".to_string(),
            network_name: "internet".to_string(),
            subnet_range: "100.100.100.0/24".to_string(),
        }
    )
}

#[test]
fn test_server_cannot_have_network_address() {
    assert_eq!(
        PlatformValidationError::ServerIpCannotBeNetworkAddress {
            server_name: "server1".to_string(),
            interface_name: "eth0".to_string(),
            interface_ip: "10.17.1.0".to_string(),
            network_name: "lan".to_string(),
            subnet_range: "10.17.1.0/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA server(hostname, dc, ssh_interface) {
    server1, dc1, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.0, 24;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_server_cannot_have_broadcast_address() {
    assert_eq!(
        PlatformValidationError::ServerIpCannotBeBroadcastAddress {
            server_name: "server1".to_string(),
            interface_name: "eth0".to_string(),
            interface_ip: "10.17.1.255".to_string(),
            network_name: "lan".to_string(),
            subnet_range: "10.17.1.0/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA server(hostname, dc, ssh_interface) {
    server1, dc1, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.255, 24;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_server_cannot_have_first_reserved_ip() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA server {
    server1, eth0 WITH network_interface {
        eth0, internet, 100.100.100.1;
    };
}

DATA EXCLUSIVE network {
    internet, "100.100.100.0/24";
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FirstIpInSubnetIsReservedToGateway {
            server_name: "server1".to_string(),
            interface_name: "eth0".to_string(),
            interface_ip: "100.100.100.1/24".to_string(),
            network_name: "internet".to_string(),
            subnet_range: "100.100.100.0/24".to_string(),
        }
    )
}

#[test]
fn test_port_range_in_same_subnet_clash() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA server {
    server1, eth0 WITH network_interface {
        eth0, lan, 10.100.100.10;
    };
    server2, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.100.100.11, 23;
    };
}

DATA EXCLUSIVE network {
    lan, "10.100.100.0/22";
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
  standard_24_disk_setup_test('server2')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::SubnetOverlapAcrossInterfaces {
            server_a_name: "server2".to_string(),
            interface_a_name: "eth0".to_string(),
            interface_a_ip: "10.100.100.11/23".to_string(),
            server_b_name: "server1".to_string(),
            interface_b_name: "eth0".to_string(),
            interface_b_ip: "10.100.100.10/24".to_string(),
            subnet_name: "lan".to_string(),
            subnet_range: "10.100.100.0/22".to_string(),
        }
    )
}

#[test]
fn test_ip_clash_across_network() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA server {
    server1, eth0 WITH network_interface {
        eth0, lan, 10.100.100.10;
    };
    server2, eth0 WITH network_interface {
        eth0, lan, 10.100.100.10;
    };
}

DATA EXCLUSIVE network {
    lan, "10.100.100.0/22";
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
  standard_24_disk_setup_test('server2')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::DuplicateIpFoundOnTheNetwork {
            server_a_name: "server1".to_string(),
            interface_a_name: "eth0".to_string(),
            interface_a_ip: "10.100.100.10/24".to_string(),
            server_b_name: "server2".to_string(),
            interface_b_name: "eth0".to_string(),
            interface_b_ip: "10.100.100.10/24".to_string(),
            subnet_name: "lan".to_string(),
            subnet_range: "10.100.100.0/22".to_string(),
        }
    )
}

#[test]
fn test_untruncated_network() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.100.100.1/24";
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::NetworkSubnetIsNotTruncated {
            network_name: "lan".to_string(),
            subnet_value: "10.100.100.1/24".to_string(),
            expected_value: "10.100.100.0/24".to_string(),
        }
    );
}

#[test]
fn test_root_network_clash() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.100.100.0/22";
    vpn, "10.100.0.0/16";
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::SubnetOverlapAcrossNetworks {
            network_a_name: "lan".to_string(),
            network_a_cidr: "10.100.100.0/22".to_string(),
            network_b_name: "vpn".to_string(),
            network_b_cidr: "10.100.0.0/16".to_string(),
        }
    )
}

#[test]
fn test_internet_but_private_ip() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}

DATA server {
    server1, eth0 WITH network_interface {
        eth0, internet, 10.100.100.10;
    };
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InternetNetworkCannotHavePrivateIpAddresses {
            server_name: "server1".to_string(),
            interface_name: "eth0".to_string(),
            network_name: "internet".to_string(),
            ip_address: "10.100.100.10".to_string(),
            forbidden_ranges: vec![
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
                "192.168.0.0/16".to_string(),
            ],
        }
    )
}

#[test]
fn test_ipv6_bad_address() {
    assert_eq!(
        PlatformValidationError::InvalidPublicIpV6AddressForNode {
            server_name: "server1".to_string(),
            ipv6_address: "127.0.0.1".to_string(),
            parsing_error: "invalid IPv6 address syntax".to_string(),
        },
        assert_platform_validation_error_wcustom_data(
            r#"
DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}

DATA server(hostname, ssh_interface, public_ipv6_address) {
    server1, eth0, '127.0.0.1' WITH network_interface {
        eth0, internet, 77.100.100.10;
    };
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
        ),
    );
}

#[test]
fn test_ipv6_loopback_address() {
    assert_eq!(
        PlatformValidationError::PublicIpV6AddressForNodeIsLoopback {
            server_name: "server1".to_string(),
            ipv6_address: "::1".to_string(),
        },
        assert_platform_validation_error_wcustom_data(
            r#"
DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}

DATA server(hostname, ssh_interface, public_ipv6_address) {
    server1, eth0, '::1' WITH network_interface {
        eth0, internet, 77.100.100.10;
    };
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
        ),
    );
}

#[test]
fn test_ipv6_multicast_address() {
    assert_eq!(
        PlatformValidationError::PublicIpV6AddressForNodeIsMulticast {
            server_name: "server1".to_string(),
            ipv6_address: "ff02::1".to_string(),
        },
        assert_platform_validation_error_wcustom_data(
            r#"
DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}

DATA server(hostname, ssh_interface, public_ipv6_address) {
    server1, eth0, 'ff02::1' WITH network_interface {
        eth0, internet, 77.100.100.10;
    };
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
        ),
    );
}

#[test]
fn test_ipv6_private_address() {
    assert_eq!(
        PlatformValidationError::PublicIpV6AddressIsPrivate {
            server_name: "server1".to_string(),
            ipv6_address: "fc00::a2:103".to_string(),
        },
        assert_platform_validation_error_wcustom_data(
            r#"
DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}

DATA server(hostname, ssh_interface, public_ipv6_address) {
    server1, eth0, 'fc00::a2:103' WITH network_interface {
        eth0, internet, 77.100.100.10;
    };
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
        ),
    );
}

#[test]
fn test_ipv6_public_address_but_no_internet_ip() {
    assert_eq!(
        PlatformValidationError::ServerHasPublicIpV6AddressButDoesntHaveIpV4PublicAddress {
            server_name: "server1".to_string(),
            ipv6_address: "2a03:2880:f32e:3:face:b00c:0:25de".to_string(),
            ipv4_network_interfaces: vec!["lan".to_string()],
        },
        assert_platform_validation_error_wcustom_data(
            r#"
DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
    lan, "10.0.0.0/8";
}

DATA server(hostname, ssh_interface, public_ipv6_address) {
    server1, eth0, '2a03:2880:f32e:3:face:b00c:0:25de' WITH network_interface {
        eth0, lan, 10.17.10.10;
    };
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
        ),
    );
}

#[test]
fn test_lan_but_public_ip() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "0.0.0.0/0";
}

DATA server {
    server1, eth0 WITH network_interface {
        eth0, lan, 123.123.123.123;
    };
}

INCLUDE LUA {
  function standard_24_disk_setup_test(hostname)
    data('server_disk', { hostname = hostname, disk_id = "vda"})
    for i = string.byte('b'), string.byte('z') do
      data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
    end
  end

  standard_24_disk_setup_test('server1')
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::NonInternetNetworkCannotHavePublicIpAddresses {
            server_name: "server1".to_string(),
            interface_name: "eth0".to_string(),
            network_name: "lan".to_string(),
            ip_address: "123.123.123.123".to_string(),
            allowed_ranges: vec![
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
                "192.168.0.0/16".to_string(),
            ],
        }
    )
}

#[test]
fn test_disallowed_network_name() {
    let err = assert_eden_db_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    unknown, "0.0.0.0/0";
}
"#,
    );
    match err {
        edendb::checker::errors::DatabaseValidationError::LuaCheckEvaluationFailed { .. } => {}
        other => {
            panic!("Unexpected error: {}", other.to_string());
        }
    }
}

#[test]
fn test_bad_lan_network() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "127.0.0.1/32";
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::NetworkLanDisallowedSubnetValue {
            network_name: "lan".to_string(),
            subnet_value: "127.0.0.1/32".to_string(),
            only_allowed_value: "10.0.0.0/8".to_string(),
        },
    );
}

#[test]
fn test_bad_vpn_network() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    vpn, "127.0.0.1/32";
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::NetworkVpnDisallowedSubnetValue {
            network_name: "vpn".to_string(),
            subnet_value: "127.0.0.1/32".to_string(),
            only_allowed_value: "172.21.0.0/16".to_string(),
        },
    );
}

#[test]
fn test_bad_dcrouter_network() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    dcrouter, "127.0.0.1/32";
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::NetworkDcrouterDisallowedSubnetValue {
            network_name: "dcrouter".to_string(),
            subnet_value: "127.0.0.1/32".to_string(),
            only_allowed_value: "10.0.0.0/8".to_string(),
        },
    );
}

#[test]
fn test_bad_dc_network_cidr() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter {
    dc_name: dc-bad,
    region: bad,
    network_cidr: lolwat,
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InvalidDcIpV4Subnet {
            datacenter_name: "dc-bad".to_string(),
            subnet_value: "lolwat".to_string(),
            parsing_error: "invalid IP address syntax".to_string(),
        },
    );
}

#[test]
fn test_bad_dc_network_cidr_prefix() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter {
    dc_name: dc-bad,
    region: bad,
    network_cidr: '10.27.0.0/17',
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::DatacenterLanNetworkMustHaveSlash16Prefix {
            datacenter_name: "dc-bad".to_string(),
            network_cidr: "10.27.0.0/17".to_string(),
            expected_prefix: "/16".to_string(),
            actual_prefix: "/17".to_string(),
        },
    );
}

#[test]
fn test_bad_dc_lan_address() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter {
    dc_name: dc-bad,
    region: bad,
    network_cidr: '11.27.0.0/16',
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::DatacenterNetworkDoesntBelongToGlobalLan {
            network_name: "lan".to_string(),
            network_cidr: "10.0.0.0/8".to_string(),
            datacenter_name: "dc-bad".to_string(),
            datacenter_cidr: "11.27.0.0/16".to_string(),
        },
    );
}

#[test]
fn test_dc_lan_address_clash() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region [
    {
        region_name: bad-a,
    },
    {
        region_name: bad-b,
    },
]

DATA STRUCT datacenter [
  {
      dc_name: dc-clash-1,
      region: bad-a,
      network_cidr: '10.27.0.0/16',
  },
  {
      dc_name: dc-clash-2,
      region: bad-b,
      network_cidr: '10.27.0.0/16',
  },
]"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::DatacenterNetworkClash {
            datacenter_a_name: "dc-clash-1".to_string(),
            datacenter_a_network_cidr: "10.27.0.0/16".to_string(),
            datacenter_b_name: "dc-clash-2".to_string(),
            datacenter_b_network_cidr: "10.27.0.0/16".to_string(),
        },
    );
}

#[test]
fn test_server_ip_not_inside_dc() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.28.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InterfaceIpIsNotInsideDatacenterSubnet {
            server_name: "server1".to_string(),
            interface_ip: "10.28.0.10".to_string(),
            interface_name: "eth0".to_string(),
            interface_network: "lan".to_string(),
            datacenter_name: "some-dc".to_string(),
            datacenter_subnet: "10.27.0.0/16".to_string(),
        },
    );
}

#[test]
fn test_server_ip_lan_cidr_is_not_24() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.27.0.10,
        if_prefix: 25,
    } WITH server_disk {
        disk_id: vda,
    },
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::LanInterfaceCidrIsNot24 {
            server_name: "server1".to_string(),
            interface_network: "lan".to_string(),
            interface_ip: "10.27.0.10".to_string(),
            interface_name: "eth0".to_string(),
            interface_cidr: 25,
            expected_cidr: 24,
        },
    );
}

#[test]
fn test_server_lan_ip_is_in_dcrouter_range() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.27.252.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::LanInterfaceIsInsideForbiddenDcrouterRange {
            server_name: "server1".to_string(),
            interface_network: "lan".to_string(),
            interface_ip: "10.27.252.10".to_string(),
            interface_name: "eth0".to_string(),
            interface_cidr: 24,
            dcrouter_range: "10.27.252.0/22".to_string()
        },
    );
}

#[test]
fn test_server_dcrouter_ip_is_outside_dc_range() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
    dcrouter, "10.0.0.0/8";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    WITH server_disk {
        disk_id: 'vda',
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.27.1.10,
            if_prefix: 24,
        },
        {
            if_name: eth1,
            if_network: dcrouter,
            if_ip: 10.26.252.10,
            if_prefix: 22,
        },
    ],
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InterfaceIpIsNotInsideDatacenterSubnet {
            server_name: "server1".to_string(),
            interface_network: "dcrouter".to_string(),
            interface_ip: "10.26.252.10".to_string(),
            interface_name: "eth1".to_string(),
            datacenter_name: "some-dc".to_string(),
            datacenter_subnet: "10.27.0.0/16".to_string(),
        },
    );
}


#[test]
fn test_server_dcrouter_ip_is_outside_dcrouter_allowed_range() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
    dcrouter, "10.0.0.0/8";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    is_router: true,
    dc: some-dc,
    WITH server_disk {
        disk_id: 'vda',
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.27.1.10,
            if_prefix: 24,
        },
        {
            if_name: eth1,
            if_network: dcrouter,
            if_ip: 10.27.2.10,
            if_prefix: 22,
        },
    ],
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::DcrouterInterfaceIsOutsideAllowedRange {
            server_name: "server1".to_string(),
            interface_network: "dcrouter".to_string(),
            interface_ip: "10.27.2.10".to_string(),
            interface_cidr: 22,
            interface_name: "eth1".to_string(),
            dcrouter_range: "10.27.252.0/22".to_string(),
        },
    );
}

#[test]
fn test_router_ip_cidr_is_not_22() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    WITH server_disk {
        disk_id: vda,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.27.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: dcrouter,
        if_ip: 10.27.252.10,
        if_prefix: 24,
      },
    ]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::DcrouterInterfaceCidrIsNot22 {
            server_name: "server1".to_string(),
            interface_network: "dcrouter".to_string(),
            interface_ip: "10.27.252.10".to_string(),
            interface_name: "eth1".to_string(),
            interface_cidr: 24,
            expected_cidr: 22,
        },
    );
}

#[test]
fn test_interface_ip_is_not_inside_vpn_subnet() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    is_vpn_gateway: true,
    WITH server_disk {
        disk_id: vda,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.27.0.10,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.22.8.1,
        if_prefix: 16,
      },
    ],
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::InterfaceIpIsNotInsideSubnet {
            server_name: "server1".to_string(),
            interface_ip: "172.22.8.1/16".to_string(),
            interface_name: "wg0".to_string(),
            network_name: "vpn".to_string(),
            subnet_range: "172.21.0.0/16".to_string(),
        },
    );
}

#[test]
fn test_server_ip_vpn_cidr_is_not_24() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.27.0.10,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 25,
      },
    ],
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::VpnInterfaceCidrIsNot16 {
            server_name: "server1".to_string(),
            interface_network: "vpn".to_string(),
            interface_ip: "172.21.7.10".to_string(),
            interface_name: "wg0".to_string(),
            interface_cidr: 25,
            expected_cidr: 16,
        },
    );
}

#[test]
fn test_server_ip_vpn_cidr_is_forbidden_addr() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"
DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT region {
    region_name: bad,
}

DATA STRUCT datacenter [
  {
      dc_name: some-dc,
      region: bad,
      network_cidr: '10.27.0.0/16',
  },
]

DATA STRUCT server {
    hostname: server1,
    ssh_interface: eth0,
    dc: some-dc,
    is_vpn_gateway: true,
    WITH server_disk {
        disk_id: vda,
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.27.0.10,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.254,
        if_prefix: 16,
      },
    ],
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::VpnAddressReservedForAdmin {
            server_name: "server1".to_string(),
            forbidden_vpn_ip: "172.21.7.254".to_string(),
            vpn_interface_ip: "172.21.7.254".to_string(),
        },
    );
}

#[test]
fn test_vault_quorum_count() {
    assert_eq!(
        PlatformValidationError::VaultServersQuorumMustBeThreeOrFiveInRegion {
            region: "us-west".to_string(),
            found_vault_instances: vec!["server1".to_string(), "server2".to_string()]
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA region {
    us-west;
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
}
"#,
    ));
}

#[test]
fn test_vault_too_many_instances_in_single_dc_3() {
    assert_eq!(
        PlatformValidationError::VaultServersQuorumTooManyServersInSingleDc {
            region: "us-west".to_string(),
            datacenter: "dc1".to_string(),
            total_vault_servers: 3,
            max_allowed_per_dc: 1,
            found_in_dc: 2,
            vault_servers_in_dc: vec!["server1".to_string(), "server2".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
    region_name: us-west,
    availability_mode: multi_dc,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc2,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}


#[test]
fn test_vault_too_many_instances_in_single_dc_5() {
    assert_eq!(
        PlatformValidationError::VaultServersQuorumTooManyServersInSingleDc {
            region: "us-west".to_string(),
            datacenter: "dc1".to_string(),
            total_vault_servers: 5,
            max_allowed_per_dc: 2,
            found_in_dc: 3,
            vault_servers_in_dc: vec!["server1".to_string(), "server2".to_string(), "server3".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
    region_name: us-west,
    availability_mode: multi_dc,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0,
        is_vault_instance: true,
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc2,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
    {
        hostname: server5,
        dc: dc2,
        is_vault_instance: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.127,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_nomad_quorum_count() {
    assert_eq!(
        PlatformValidationError::NomadServersQuorumMustBeThreeOrFiveInRegion {
            region: "us-west".to_string(),
            found_nomad_servers: vec!["server1".to_string(), "server2".to_string()]
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA region {
    us-west;
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
}
"#,
    ));
}

#[test]
fn test_nomad_too_many_instances_in_single_dc_3() {
    assert_eq!(
        PlatformValidationError::NomadServersQuorumTooManyServersInSingleDc {
            region: "us-west".to_string(),
            datacenter: "dc1".to_string(),
            total_nomad_servers: 3,
            max_allowed_per_dc: 1,
            found_in_dc: 2,
            nomad_servers_in_dc: vec!["server1".to_string(), "server2".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
    region_name: us-west,
    availability_mode: multi_dc,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc2,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_nomad_too_many_instances_in_single_dc_5() {
    assert_eq!(
        PlatformValidationError::NomadServersQuorumTooManyServersInSingleDc {
            region: "us-west".to_string(),
            datacenter: "dc1".to_string(),
            total_nomad_servers: 5,
            max_allowed_per_dc: 2,
            found_in_dc: 3,
            nomad_servers_in_dc: vec!["server1".to_string(), "server2".to_string(), "server3".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
    region_name: us-west,
    availability_mode: multi_dc,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0,
        is_nomad_master: true,
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc2,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
    {
        hostname: server5,
        dc: dc2,
        is_nomad_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.127,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_consul_quorum_count() {
    assert_eq!(
        PlatformValidationError::ConsulServerQuorumMustBeThreeOrFiveInRegion {
            region: "us-west".to_string(),
            found_consul_servers: vec!["server1".to_string(), "server2".to_string()]
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA region {
    us-west;
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
}
"#,
    ));
}

#[test]
fn test_consul_too_many_instances_in_single_dc_3() {
    assert_eq!(
        PlatformValidationError::ConsulServersQuorumTooManyServersInSingleDc {
            region: "us-west".to_string(),
            datacenter: "dc1".to_string(),
            total_consul_servers: 3,
            max_allowed_per_dc: 1,
            found_in_dc: 2,
            consul_servers_in_dc: vec!["server1".to_string(), "server2".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
    region_name: us-west,
    availability_mode: multi_dc,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc2,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_consul_too_many_instances_in_single_dc_5() {
    assert_eq!(
        PlatformValidationError::ConsulServersQuorumTooManyServersInSingleDc {
            region: "us-west".to_string(),
            datacenter: "dc1".to_string(),
            total_consul_servers: 5,
            max_allowed_per_dc: 2,
            found_in_dc: 3,
            consul_servers_in_dc: vec!["server1".to_string(), "server2".to_string(), "server3".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
    region_name: us-west,
    availability_mode: multi_dc,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.123,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
    {
        hostname: server3,
        dc: dc1,
        ssh_interface: eth0,
        is_consul_master: true,
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
    {
        hostname: server4,
        dc: dc2,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.126,
            if_prefix: 24,
        }
    },
    {
        hostname: server5,
        dc: dc2,
        is_consul_master: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: 'vda',
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.127,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}
"#,
    ));
}

#[test]
fn test_single_dc_region_violation() {
    assert_eq!(
        PlatformValidationError::SingleDcRegionMustHaveOnlyOneDatacenter {
            region: "us-west".to_string(),
            expected_count: 1,
            found_datacenters: vec!["dc1".to_string(), "dc2".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA region {
    us-west;
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        region: us-west,
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        region: us-west,
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
}
"#,
    ));
}

#[test]
fn test_multi_dc_region_violation() {
    assert_eq!(
        PlatformValidationError::MultiDcRegionMustHaveAtLeastThreeDatacenters {
            region: "us-west".to_string(),
            expected_at_least: 3,
            found_datacenters: vec!["dc1".to_string(), "dc2".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
    region_name: us-west,
    availability_mode: multi_dc,
}

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        region: us-west,
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        region: us-west,
    },
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
}
"#,
    ));
}

#[test]
fn test_vpn_gateway_instance_pair_has_no_public_ip() {
    assert_eq!(
        PlatformValidationError::VpnGatewayServerPairMustHaveAtLeastOneInternetInterface {
            server_a: "server-b".to_string(),
            server_b: "server-f".to_string(),
            server_a_has_internet_interface: false,
            server_b_has_internet_interface: false,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_data: false,
                add_default_global_flags: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: us-west=>docker,
}

DATA STRUCT EXCLUSIVE tld {
    domain: epl-infra.net,
    expose_admin: false,
}

DATA STRUCT EXCLUSIVE datacenter [
  {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
  },
  {
    dc_name: dc2,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    network_cidr: '10.19.0.0/16',
  },
]

DATA STRUCT EXCLUSIVE region [
  { region_name: us-west, is_dns_master: true, availability_mode: multi_dc }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
]

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-c=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
  '10.18.0.2/24';
  '10.19.0.2/24';
}

DATA STRUCT server [
  {
    dc: dc1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_vpn_gateway: true,
    is_dns_slave: false
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.3,
        if_prefix: 16,
      },
    ]

    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.4,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.5,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.6,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-e,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.7,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.8,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  }
]
"#,
    ));
}

#[test]
fn test_vpn_gateway_instance_has_no_vpn_interface() {
    assert_eq!(
        PlatformValidationError::VpnGatewayServerMustHaveVpnInterface {
            server: "server-b".to_string(),
            server_dc: "dc1".to_string(),
            network_interfaces: vec!["lan".to_string(), "internet".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_data: false,
                add_default_global_flags: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: us-west=>docker,
}

DATA STRUCT EXCLUSIVE tld {
    domain: epl-infra.net,
    expose_admin: false,
}

DATA STRUCT EXCLUSIVE datacenter [
  {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
  },
  {
    dc_name: dc2,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    network_cidr: '10.19.0.0/16',
  },
]

DATA STRUCT EXCLUSIVE region [
  { region_name: us-west, is_dns_master: true, availability_mode: multi_dc }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: dcrouter,
    cidr: '10.0.0.0/8',
  },
]

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-c=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
  '10.18.0.2/24';
  '10.19.0.2/24';
}

DATA STRUCT server [
  {
    dc: dc1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_vpn_gateway: true,
    is_dns_slave: false
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]

    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-e,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  }
]
"#,
    ));
}

#[test]
fn test_vpn_gateway_instance_vpn_interface_is_not_named_wg0() {
    assert_eq!(
        PlatformValidationError::VpnGatewayServerVpnInterfaceNameMustBeWg0 {
            server: "server-b".to_string(),
            network: "vpn".to_string(),
            expected_interface_name: "wg0".to_string(),
            actual_interface_name: "eth2".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_data: false,
                add_default_global_flags: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: us-west=>docker,
}

DATA STRUCT EXCLUSIVE tld {
    domain: epl-infra.net,
    expose_admin: false,
}

DATA STRUCT EXCLUSIVE datacenter [
  {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
  },
  {
    dc_name: dc2,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    network_cidr: '10.19.0.0/16',
  },
]

DATA STRUCT EXCLUSIVE region [
  { region_name: us-west, is_dns_master: true, availability_mode: multi_dc }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: dcrouter,
    cidr: '10.0.0.0/8',
  },
]

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
  '10.18.0.2/24';
  '10.19.0.2/24';
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-c=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
}

DATA STRUCT server [
  {
    dc: dc1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_vpn_gateway: true,
    is_dns_master: true,
    is_dns_slave: false
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.3,
        if_prefix: 16,
      },
    ]

    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 24,
      },
      {
        if_name: eth2,
        if_network: vpn,
        if_ip: 172.21.7.7,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-e,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  }
]
"#,
    ));
}

#[test]
fn test_dc_has_not_enough_vpn_gateways() {
    assert_eq!(
        PlatformValidationError::DcMustHaveExactlyTwoVpnGateways {
            dc: "dc1".to_string(),
            actual_count: 1,
            expected_count: 2,
            vpn_gateway_servers: vec!["server-a".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_data: false,
                add_default_global_flags: false,
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc 'dc1',
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    tempo_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: us-west=>docker,
}

DATA STRUCT EXCLUSIVE tld {
    domain: epl-infra.net,
    expose_admin: false,
}

DATA STRUCT EXCLUSIVE datacenter [
  {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
  },
  {
    dc_name: dc2,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    network_cidr: '10.19.0.0/16',
  },
]

DATA STRUCT EXCLUSIVE region [
  { region_name: us-west, is_dns_master: true, availability_mode: multi_dc }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: dcrouter,
    cidr: '10.0.0.0/8',
  },
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}

DATA STRUCT loki_cluster {
  cluster_name: default-log,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default-mon,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-c=>mon },
    { instance_id: 3, monitoring_server: server-d=>mon },
  ]
}

DATA STRUCT server [
  {
    dc: dc1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.7,
        if_prefix: 16,
      },
    ]

    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.11,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-e,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio,
      },
      {
        volume_name: mon,
      },
    ]
  }
]
"#,
    ));
}

#[test]
fn test_dcrouter_network_ok() {
    let _ = assert_platform_validation_success_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA server(hostname, dc, ssh_interface, is_router, is_vpn_gateway) {
    server1, dc1, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.255.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.255.11, 22;
    } WITH server_disk {
        vda;
    };
    server3, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.10, 24;
        eth1, dcrouter, 10.17.255.12, 22;
    } WITH server_disk {
        vda;
    };
    server4, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.11, 24;
        eth1, dcrouter, 10.17.255.13, 22;
    } WITH server_disk {
        vda;
    };


    server5, dc2, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.49.0.10, 24;
        eth1, dcrouter, 10.49.255.10, 22;
    } WITH server_disk {
        vda;
    };
    server6, dc2, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.49.0.11, 24;
        eth1, dcrouter, 10.49.255.11, 22;
    } WITH server_disk {
        vda;
    };
    server7, dc2, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.49.1.10, 24;
        eth1, dcrouter, 10.49.255.12, 22;
    } WITH server_disk {
        vda;
    };
    server8, dc2, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.49.1.11, 24;
        eth1, dcrouter, 10.49.255.13, 22;
    } WITH server_disk {
        vda;
    };
}

DATA subnet_router_floating_ip {
    '10.17.0.2/24';
    '10.17.1.2/24';
    '10.49.0.2/24';
    '10.49.1.2/24';
}

DATA STRUCT datacenter {
    dc_name: dc2,
    network_cidr: '10.49.0.0/16',
    region: us-east,
    allow_small_subnets: true,
}

DATA region {
    us-east;
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_tracing_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    );
}


#[test]
fn test_ip_clash_across_router_network_in_same_dc() {
    assert_eq!(
        PlatformValidationError::DcrouterDuplicateRouterIpInsideDatacenterDetected {
            datacenter: "dc1".to_string(),
            previous_server_hostname: "server1".to_string(),
            previous_server_interface_ip: "10.17.252.10".to_string(),
            duplicate_server_hostname: "server2".to_string(),
            duplicate_server_interface_ip: "10.17.252.10".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA server(hostname, dc, ssh_interface, is_router) {
    server1, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
        )
    );
}

#[test]
fn test_vpn_interface_but_not_vpn_gateway_error() {
    assert_eq!(
        PlatformValidationError::VpnInterfaceExistsButServerIsNotMarkedVpnGateway {
            server_name: "server1".to_string(),
            vpn_interface_ip: "172.21.7.10".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA server(hostname, dc, ssh_interface) {
    server1, dc1, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
        eth2, vpn, 172.21.7.10, 16;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.11, 22;
        eth2, vpn, 172.21.7.11, 16;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
        )
    );
}

#[test]
fn test_dc_multiple_subnets_too_few_servers() {
    assert_eq!(
        PlatformValidationError::DcSubnetHasTooFewHosts {
            dc: "dc2".to_string(),
            subnet: "10.18.0.0/24".to_string(),
            servers_count: 1,
            minimum_count: 100,
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA server(hostname, dc, ssh_interface) {
    server1, dc2, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.18.0.10, 24;
    } WITH server_disk {
        vda;
    };
    server2, dc2, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.18.1.10, 24;
    } WITH server_disk {
        vda;
    };
}

DATA region {
    us-east;
}

DATA STRUCT datacenter {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    vpn, "172.21.0.0/16";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
        )
    );
}

#[test]
fn test_dcrouter_interface_but_not_marked_router_error() {
    assert_eq!(
        PlatformValidationError::DcrouterInterfaceExistsButServerIsNotMarkedAsRouter {
            server_name: "server1".to_string(),
            router_interface_ip: "10.17.252.10".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA server(hostname, dc, ssh_interface) {
    server1, dc1, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0 WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.11, 22;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
        )
    );
}

#[test]
fn test_router_server_but_has_no_dcrouter_interface() {
    assert_eq!(
        PlatformValidationError::DcrouterServerMustHaveDcrouterInterface {
            server: "server1".to_string(),
            server_dc: "dc1".to_string(),
            network_interfaces: vec!["lan".to_string()],
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA server(hostname, dc, ssh_interface, is_router) {
    server1, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.11, 22;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
        )
    );
}

#[test]
fn test_dc_subnet_has_too_few_routers() {
    assert_eq!(
        PlatformValidationError::DcRoutingSubnetMustHaveExactlyTwoRouters {
            dc: "dc1".to_string(),
            subnet: "10.17.0.0/24".to_string(),
            expected_router_servers: 2,
            router_servers: vec!["server1".to_string()],
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA server(hostname, dc, ssh_interface, is_router, is_vpn_gateway) {
    server1, dc1, eth0, true, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, true, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.10, 24;
        eth1, dcrouter, 10.17.252.11, 22;
    } WITH server_disk {
        vda;
    };
}

DATA subnet_router_floating_ip {
    '10.17.0.2/24';
    '10.17.1.2/24';
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_invalid_subnet_router_floating_ip() {
    assert_eq!(
        PlatformValidationError::InvalidSubnetRouterFloatingIp {
            value: "a.b.c.d".to_string(),
            parsing_error: "invalid IP address syntax".to_string(),
            valid_example: "10.17.49.2/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA subnet_router_floating_ip {
    a.b.c.d;
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_invalid_subnet_router_floating_ip_not_in_network() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpDoesntBelongToLanNetwork {
            value: "192.168.12.12/24".to_string(),
            expected_to_be_in_network: "10.0.0.0/8".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA subnet_router_floating_ip {
    '192.168.12.12/24';
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}


#[test]
fn test_invalid_subnet_router_floating_ip_invalid_prefix_length() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpInvalidPrefixLength {
            value: "10.17.12.12/22".to_string(),
            expected_prefix_length: "/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA subnet_router_floating_ip {
    '10.17.12.12/22';
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_invalid_subnet_router_floating_ip_cannot_be_network_address() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpCannotBeNetworkAddress {
            value: "10.17.12.0/24".to_string(),
            network_address: "10.17.12.0/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA subnet_router_floating_ip {
    '10.17.12.0/24';
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_invalid_subnet_router_floating_ip_cannot_be_broadcast_address() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpCannotBeBroadcastAddress {
            value: "10.17.12.255/24".to_string(),
            broadcast_address: "10.17.12.255/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA subnet_router_floating_ip {
    '10.17.12.255/24';
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}


#[test]
fn test_invalid_subnet_router_floating_ip_cannot_be_first_network_address() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpCannotBeFirstAddressInNetwork {
            value: "10.17.12.1/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA subnet_router_floating_ip {
    '10.17.12.1/24';
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_invalid_subnet_router_floating_ip_clashes_with_server_ip() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpClashWithServerIp {
            server: "server1".to_string(),
            server_ip: "10.17.0.10".to_string(),
            subnet_router_floating_ip: "10.17.0.10/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA server(hostname, dc, ssh_interface, is_router, is_vpn_gateway) {
    server1, dc1, eth0, true, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, true, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.11, 22;
    } WITH server_disk {
        vda;
    };
    server3, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.10, 24;
        eth1, dcrouter, 10.17.252.12, 22;
    } WITH server_disk {
        vda;
    };
    server4, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.11, 24;
        eth1, dcrouter, 10.17.252.13, 22;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA subnet_router_floating_ip {
    '10.17.0.10/24';
    '10.17.1.2/24';
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_invalid_subnet_router_floating_ip_not_found_for_subnet() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpForSubnetNotFound {
            subnet: "10.17.1.0/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA server(hostname, dc, ssh_interface, is_router, is_vpn_gateway) {
    server1, dc1, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.11, 22;
    } WITH server_disk {
        vda;
    };
    server3, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.10, 24;
        eth1, dcrouter, 10.17.252.12, 22;
    } WITH server_disk {
        vda;
    };
    server4, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.11, 24;
        eth1, dcrouter, 10.17.252.13, 22;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA subnet_router_floating_ip {
    '10.17.0.2/24';
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_invalid_subnet_router_floating_ip_never_used() {
    assert_eq!(
        PlatformValidationError::SubnetRouterFloatingIpDefinedButNeverUsed {
            subnet_router_floating_ip: "10.17.2.2/24".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA server(hostname, dc, ssh_interface, is_router, is_vpn_gateway) {
    server1, dc1, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, false, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.11, 22;
    } WITH server_disk {
        vda;
    };
    server3, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.10, 24;
        eth1, dcrouter, 10.17.252.12, 22;
    } WITH server_disk {
        vda;
    };
    server4, dc1, eth0, true, false WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.11, 24;
        eth1, dcrouter, 10.17.252.13, 22;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA subnet_router_floating_ip {
    '10.17.0.2/24';
    '10.17.1.2/24';
    '10.17.2.2/24';
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_router_floating_ip_defined_twice_per_subnet() {
    assert_eq!(
        PlatformValidationError::SubnetRouterTwoFloatingIpsFoundForNetwork {
            network: "10.17.1.0/24".to_string(),
            floating_ip_a: "10.17.1.2".to_string(),
            floating_ip_b: "10.17.1.3".to_string(),
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"

DATA server(hostname, dc, ssh_interface, is_router) {
    server1, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.10, 24;
        eth1, dcrouter, 10.17.252.10, 22;
    } WITH server_disk {
        vda;
    };
    server2, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.0.11, 24;
        eth1, dcrouter, 10.17.252.11, 22;
    } WITH server_disk {
        vda;
    };
    server3, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.10, 24;
        eth1, dcrouter, 10.17.252.12, 22;
    } WITH server_disk {
        vda;
    };
    server4, dc1, eth0, true WITH network_interface(if_name, if_network, if_ip, if_prefix) {
        eth0, lan, 10.17.1.11, 24;
        eth1, dcrouter, 10.17.252.13, 22;
    } WITH server_disk {
        vda;
    };
}

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA subnet_router_floating_ip {
    '10.17.0.2/24';
    '10.17.1.2/24';
    '10.17.1.3/24';
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}

#[test]
fn test_disallow_mixing_testvms_dc_implementations_with_production() {
    assert_eq!(
        PlatformValidationError::TestVmsDatacentersCannotBeMixedWithProductionDatacenters {
            test_vm_datacenters: vec!["test-dc2".to_string()],
            production_datacenters: vec!["test-dc1".to_string(), "dc1".to_string()],
        },
        assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT datacenter [
  {
    dc_name: test-dc1,
    region: us-west,
    implementation: manual,
    network_cidr: '10.17.0.0/24',
  },
  {
    dc_name: test-dc2,
    region: us-west,
    implementation: testvms,
    network_cidr: '10.18.0.0/24',
  },
]

DATA EXCLUSIVE network {
    lan, "10.0.0.0/8";
    dcrouter, "10.0.0.0/8";
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,

    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_docker_registry_tests: true,

    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#,
    ));
}
