#[cfg(test)]
use crate::static_analysis::PlatformValidationError;
#[cfg(test)]
use crate::tests::common::assert_platform_validation_error_wcustom_data;

#[test]
fn test_coproc_region_must_have_two_gateways() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorDcMustHaveTwoGateways {
            region: "us-east".to_string(),
            region_dc: "dc2".to_string(),
            region_availability_mode: "single_dc".to_string(),
            region_has_coprocessor_dc: true,
            region_expected_coprocessor_gateways: 2,
            region_found_coprocessor_gateways: 0,
            region_found_coprocessor_gateways_servers: Vec::new(),
        }
    )
}

#[test]
fn test_coproc_region_dc_has_mpre_than_one_gw() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: multi_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    network_cidr: '10.19.0.0/16',
  },
  {
    dc_name: dc4,
    region: us-east,
    network_cidr: '10.20.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-b,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorCannotHaveMoreThanOneGatewayInDatacenter {
            region: "us-east".to_string(),
            region_dc: "dc2".to_string(),
            region_availability_mode: "multi_dc".to_string(),
            region_has_coprocessor_dc: true,
            region_expected_maximum_coprocessor_gateways_per_dc: 1,
            region_found_coprocessor_gateways: 2,
            region_found_coprocessor_gateways_servers: vec![
                "server-a".to_string(),
                "server-b".to_string(),
            ],
        }
    )
}

#[test]
fn test_non_coproc_region_dc_coproc_gws() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: multi_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: false,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    network_cidr: '10.19.0.0/16',
  },
  {
    dc_name: dc4,
    region: us-east,
    network_cidr: '10.20.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-b,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorDcIsNotEnabledInRegionButCoprocessorServersExist {
            region: "us-east".to_string(),
            region_dc: "dc2".to_string(),
            region_availability_mode: "multi_dc".to_string(),
            region_has_coprocessor_dc: false,
            region_expected_coprocessor_gateways: 0,
            region_found_coprocessor_gateways: 2,
            region_found_coprocessor_gateways_servers: vec![
                "server-a".to_string(),
                "server-b".to_string(),
            ],
        }
    )
}

#[test]
fn test_non_coproc_region_has_coproc_dc() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: multi_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: false,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    network_cidr: '10.19.0.0/16',
  },
  {
    dc_name: dc4,
    region: us-east,
    network_cidr: '10.20.0.0/16',
  },
  {
    dc_name: dc5,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.21.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: false,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-b,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: false,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::RegionNotMarkedAsCoprocessorHasCoprocessorDatacenters {
            region: "us-east".to_string(),
            region_has_coprocessor_dc: false,
            coprocessor_dcs: vec![
                "dc5".to_string(),
            ],
            coprocessor_dcs_expected: 0,
            coprocessor_dcs_found: 1,
        }
    )
}

#[test]
fn test_coproc_region_dc_has_too_few_gateways() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: multi_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    network_cidr: '10.19.0.0/16',
  },
  {
    dc_name: dc4,
    region: us-east,
    network_cidr: '10.20.0.0/16',
  },
  {
    dc_name: dc5,
    implementation: coprocessor,
    region: us-east,
    network_cidr: '10.21.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-b,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: false,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorRegionMustHaveTwoCoprocessorGateways {
            region: "us-east".to_string(),
            region_availability_mode: "multi_dc".to_string(),
            region_has_coprocessor_dc: true,
            region_expected_coprocessor_gateways: 2,
            region_found_coprocessor_gateways: 1,
            region_found_coprocessor_gateways_servers: vec![
                "server-a".to_string(),
            ],
        }
    )
}

#[test]
fn test_coproc_server_is_not_vpn() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.19.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-b,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: false,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-c,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: false,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.12,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorServerMustBeVPNGateway {
            region: "us-east".to_string(),
            server: "server-c".to_string(),
            datacenter: "dc2".to_string(),
            is_vpn_gateway: false,
            is_coprocessor_gateway: true,
        }
    )
}

#[test]
fn test_coproc_dc_has_coproc_gws() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.19.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-d,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-e,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc3,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 32,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-b,
    ssh_interface: eth0,
    dc: dc3,
    is_coprocessor_gateway: false,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 32,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-c,
    ssh_interface: eth0,
    dc: dc3,
    is_coprocessor_gateway: true,
    is_vpn_gateway: false,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.12,
        if_prefix: 32,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorDatacenterMustNotHaveCoprocessorGateways {
            region: "us-east".to_string(),
            dc_implementation: "coprocessor".to_string(),
            dc_expected_coprocessor_gateways: 0,
            dc_found_coprocessor_gateways: 2,
            dc_found_coprocessor_gateways_servers: vec![
                "server-a".to_string(),
                "server-c".to_string(),
            ],
            datacenter: "dc3".to_string(),
        }
    )
}

#[test]
fn test_coproc_region_no_coproc_dc() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-d,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-e,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorRegionDoesntHaveCoprocessorDc {
            region: "us-east".to_string(),
            coprocessor_dcs: Vec::new(),
            coprocessor_dcs_expected: 1,
            coprocessor_dcs_found: 0,
        }
    )
}

#[test]
fn test_coproc_region_more_than_one_coproc_dc() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.19.0.0/16',
  },
  {
    dc_name: dc4,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.20.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-d,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-e,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorRegionHasMoreThanOneCoprocessorDc {
            region: "us-east".to_string(),
            coprocessor_dcs: vec![
                "dc3".to_string(),
                "dc4".to_string(),
            ],
            coprocessor_dcs_expected: 1,
            coprocessor_dcs_found: 2,
        }
    )
}

#[test]
fn test_coproc_forbidden_roles() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.19.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-d,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-e,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc3,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 32,
    } WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorDatacenterServerHasForbiddenRole {
            region: "us-east".to_string(),
            datacenter: "dc3".to_string(),
            forbidden_role: "is_vpn_gateway".to_string(),
            server_hostname: "server-a".to_string(),
        }
    )
}

#[test]
fn test_coproc_forbidden_vpn_iface_name() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.19.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-d,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-e,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc3,
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 32,
      },
      {
        if_name: vpn0,
        if_network: vpn,
        if_ip: 172.21.0.77,
        if_prefix: 16,
      },
    ]
    WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorServerVpnInterfaceNamesMustBeWg0AndWg1 {
            region: "us-east".to_string(),
            datacenter: "dc3".to_string(),
            vpn_interface_name: "vpn0".to_string(),
            only_allowed_names: vec![
                "wg0".to_string(),
                "wg1".to_string(),
            ],
            server_hostname: "server-a".to_string(),
        }
    )
}

#[test]
fn test_coproc_too_few_vpn_interfaces() {
    let err = assert_platform_validation_error_wcustom_data(
        r#"

DATA STRUCT region {
  region_name: us-east,
  availability_mode: single_dc,
  tld: epl-infra.net,
  has_coprocessor_dc: true,
}

DATA EXCLUSIVE network {
    lan, '10.0.0.0/8';
    vpn, '172.21.0.0/16';
}

DATA STRUCT datacenter [
  {
    dc_name: dc2,
    region: us-east,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    region: us-east,
    implementation: coprocessor,
    network_cidr: '10.19.0.0/16',
  },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server {
    hostname: server-d,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-e,
    ssh_interface: eth0,
    dc: dc2,
    is_coprocessor_gateway: true,
    is_vpn_gateway: true,
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
    } WITH server_disk {
        disk_id: vda,
    },
}

DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    dc: dc3,
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 32,
      },
    ]
    WITH server_disk {
        disk_id: vda,
    },
}

"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::CoprocessorServerMustHaveTwoVpnInterfaces {
            region: "us-east".to_string(),
            datacenter: "dc3".to_string(),
            vpn_interfaces_expected: 2,
            vpn_interfaces_found: 0,
            vpn_interfaces_found_names: vec![],
            server_hostname: "server-a".to_string(),
        }
    )
}
