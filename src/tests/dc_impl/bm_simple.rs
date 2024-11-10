#[cfg(test)]
use pretty_assertions::assert_eq;

#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_bm_simple_bad_yaml() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT datacenter [
    {
        dc_name: bms-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        implementation: bm_simple,
        implementation_settings: '
          gateway_ip: 10.18.21.1
          extra_field: huh
        ',
    }
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::DatacenterImplementationInvalidSettings {
            dc: "bms-1".to_string(),
            dc_implementation: "bm_simple".to_string(),
            current_settings: "
          gateway_ip: 10.18.21.1
          extra_field: huh
        ".to_string(),
            parsing_error: "unknown field `extra_field`, expected `gateway_ip` at line 3 column 11".to_string(),
            example_settings: "gateway_ip: 10.12.17.1\n".to_string(),
        }
    );
}

#[test]
fn test_bm_simple_bad_gw_ip() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT datacenter [
    {
        dc_name: bms-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        implementation: bm_simple,
        implementation_settings: '
          gateway_ip: abc
        ',
    }
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::DatacenterImplementationInvalidSettings {
            dc: "bms-1".to_string(),
            dc_implementation: "bm_simple".to_string(),
            current_settings: "
          gateway_ip: abc
        ".to_string(),
            parsing_error: "gateway_ip: invalid IPv4 address syntax at line 2 column 23".to_string(),
            example_settings: "gateway_ip: 10.12.17.1\n".to_string(),
        }
    );
}

#[test]
fn test_bm_simple_gw_ip_outside_dc() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT datacenter [
    {
        dc_name: bms-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        implementation: bm_simple,
        implementation_settings: '
          gateway_ip: 10.19.0.1
        ',
    }
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::BmSimpleGatewayIpIsOutsideDatacenterNetwork {
            bm_simple_datacenter: "bms-1".to_string(),
            gateway_ip: "10.19.0.1".to_string(),
            dc_network: "10.18.0.0/16".to_string(),
        }
    );
}

#[test]
fn test_bm_simple_doesnt_allow_more_than_one_subnet() {
    // TODO: error ordering fix
    assert_eq!(
        PlatformValidationError::DatacenterImplementationDoesntAllowMoreThanOneSubnet {
            dc: "bms-1".to_string(),
            subnet_count: 2,
            max_subnets: 1,
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT datacenter [
    {
        dc_name: bms-1,
        region: us-west-2,
        allow_small_subnets: true,
        network_cidr: '10.18.0.0/16',
        implementation: bm_simple,
        implementation_settings: '
          gateway_ip: 10.18.10.1
        ',
    }
]

DATA STRUCT server [
  {
    dc: bms-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: bms-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: bms-1,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: false,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.1.10,
            if_prefix: 24,
        },
    ]
  },
  {
    dc: bms-1,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: false,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.1.11,
            if_prefix: 24,
        },
    ]
  },
]

"#,
        ),
    );
}

#[test]
fn test_bm_simple_gw_is_outside_single_subnet() {
    assert_eq!(
        PlatformValidationError::BmSimpleOnlySubnetDoesntContainGatewayIp {
            bm_simple_datacenter: "bms-1".to_string(),
            gateway_ip: "10.18.10.1".to_string(),
            only_dc_subnet: "10.18.0.0/24".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT datacenter [
    {
        dc_name: bms-1,
        region: us-west-2,
        allow_small_subnets: true,
        network_cidr: '10.18.0.0/16',
        implementation: bm_simple,
        implementation_settings: '
          gateway_ip: 10.18.10.1
        ',
    }
]

DATA subnet_router_floating_ip {
  '10.18.0.7/24';
}

DATA STRUCT server [
  {
    dc: bms-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: bms-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: bms-1,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: false,
    root_disk: some-disk-serial-2,
    WITH server_disk {
      disk_id: some-disk-serial-2,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.0.12,
            if_prefix: 24,
        },
    ]
  },
  {
    dc: bms-1,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: false,
    root_disk: some-disk-serial-3,
    WITH server_disk {
      disk_id: some-disk-serial-3,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.0.13,
            if_prefix: 24,
        },
    ]
  },
]

"#,
        ),
    );
}
