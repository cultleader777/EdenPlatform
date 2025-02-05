#[cfg(test)]
use crate::static_analysis::PlatformValidationError;
#[cfg(test)]
use super::super::common;
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_aws_bad_yaml() {
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
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: abc
          extra_field: huh
        ',
    }
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::DatacenterImplementationInvalidSettings {
            dc: "aws-1".to_string(),
            dc_implementation: "aws".to_string(),
            current_settings: "
          availability_zone: abc
          extra_field: huh
        ".to_string(),
            parsing_error: "unknown field `extra_field`, expected `availability_zone` at line 3 column 11".to_string(),
            example_settings: "availability_zone: us-west-1b\n".to_string(),
        }
    );
}

#[test]
fn test_aws_non_existing_az() {
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
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: abc
        ',
    }
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsDatacenterUnknownAvailabilityZone {
            dc: "aws-1".to_string(),
            dc_implementation: "aws".to_string(),
            unknown_availability_zone: "abc".to_string(),
            current_settings: "
          availability_zone: abc
        ".to_string(),
        }
    );
}

#[test]
fn test_aws_same_az_multiple_dc() {
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
  },
  {
    region_name: us-west-3
  }
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-west-3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsSameAvailabilityZoneUsedForMultipleDatacenters {
            duplicate_az: "us-west-1b".to_string(),
            previous_dc: "aws-1".to_string(),
            current_dc: "aws-2".to_string(),
        }
    );
}

#[test]
fn test_aws_multiple_regions_in_same_epl_region() {
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
    region_name: us-west-2,
    availability_mode: multi_dc,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-west-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-east-1b
        ',
    },
    {
        dc_name: dc-3,
        region: us-west-2,
        network_cidr: '10.20.0.0/16',
    },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsMoreThanAwsOneRegionInsideEdenPlatformRegion {
            eden_platform_region: "us-west-2".to_string(),
            found_aws_regions: vec!["us-east-1".to_string(), "us-west-1".to_string()],
        }
    );
}

#[test]
fn test_aws_region_across_multiple_epl_regions() {
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
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsRegionIsUsedInMoreThanOneEdenPlatformRegion {
            overused_aws_region: "us-west-1".to_string(),
            epl_regions_using_aws_region: vec!["us-west-2".to_string(), "us-east-2".to_string()],
        }
    );
}

#[test]
fn test_aws_server_public_ip_interface_must_have_one_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
            if_name: eth2,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::ForThisDatacenterImplementationAllInternetInterfaceNamesMustBeVoid {
            dc: "aws-1".to_string(),
            dc_implementation: "aws".to_string(),
            server: "server-a".to_string(),
            network: "internet".to_string(),
            network_interface_name: "eth2".to_string(),
            network_interface_only_allowed_name: "void".to_string(),
        }
    );
}

#[test]
fn test_aws_server_dcrouter_ip_interface_must_have_one_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT network [
  {
    network_name: dcrouter,
    cidr: '10.0.0.0/8',
  },
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        allow_small_subnets: true,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
        {
            if_name: eth2,
            if_network: dcrouter,
            if_ip: 10.18.252.10,
            if_prefix: 22,
        },
    ]
  },
  {
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
        {
            if_name: eth2,
            if_network: dcrouter,
            if_ip: 10.18.252.11,
            if_prefix: 22,
        },
    ]
  },
  {
    dc: aws-1,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: false,
    is_router: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.1.10,
            if_prefix: 24,
        },
        {
            if_name: eth2,
            if_network: dcrouter,
            if_ip: 10.18.252.12,
            if_prefix: 22,
        },
    ]
  },
  {
    dc: aws-1,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: false,
    is_router: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.1.11,
            if_prefix: 24,
        },
        {
            if_name: eth2,
            if_network: dcrouter,
            if_ip: 10.18.252.13,
            if_prefix: 22,
        },
    ]
  },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsDcrouterNetworkInterfaceNameMustBeEth1 {
            aws_dc: "aws-1".to_string(),
            aws_server: "server-a".to_string(),
            aws_network_interface: "eth2".to_string(),
            aws_network_interface_only_allowed_name: "eth1".to_string(),
        }
    );
}

#[test]
fn test_aws_server_public_ip_interface_must_be_ssh_target() {
    let err = common::assert_platform_validation_error_wcustom_data(
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsSshInterfaceForPublicServerMustBePublic {
            aws_dc: "aws-1".to_string(),
            aws_server: "server-a".to_string(),
            aws_server_expected_ssh_interface: "void".to_string(),
            aws_server_ssh_interface: "eth0".to_string(),
        }
    );
}

#[test]
fn test_aws_server_lan_ip_interface_must_have_one_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: enX0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: enX0,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsLanNetworkInterfaceNameMustBeEth0 {
            aws_dc: "aws-1".to_string(),
            aws_server: "server-a".to_string(),
            aws_lan_network_interface: "enX0".to_string(),
            aws_lan_network_interface_only_allowed_name: "eth0".to_string(),
        }
    );
}

#[test]
fn test_aws_interface_only_allowed_mask() {
    let err = common::assert_platform_validation_error_wcustom_data(
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-east-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
            if_prefix: 24,
        },
    ]
  },
  {
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
            if_prefix: 24,
        },
    ]
  },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::AwsInternetInterfaceMustHave32Prefix {
            aws_dc: "aws-1".to_string(),
            aws_server: "server-a".to_string(),
            aws_server_interface: "void".to_string(),
            aws_server_mask: 24,
            aws_server_only_allowed_mask: 32,
        }
    );
}

#[test]
fn test_aws_ipv6_only_allowed_prefix() {
    assert_eq!(
        PlatformValidationError::AwsPublicIpv6AddressMustHave128AsPrefix {
            aws_dc: "aws-1".to_string(),
            aws_server: "server-a".to_string(),
            aws_server_ipv6_prefix: 120,
            aws_server_only_allowed_prefix: 128,
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-east-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    public_ipv6_address_prefix: 120,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    ),
    );
}

#[test]
fn test_aws_subnet_only_one_vpn_gateway_and_one_router() {
    assert_eq!(
        PlatformValidationError::DcRoutingSubnetCannotMixDeclaredVpnGatewaysAndDeclaredRouters {
            dc: "aws-1".to_string(),
            subnet: "10.18.0.0/24".to_string(),
            declared_vpn_gateways_found: vec!["server-a".to_string()],
            declared_router_server_found: vec!["server-b".to_string()],
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_undefined_bucket_name() {
    assert_eq!(
        PlatformValidationError::AwsArtefactsBucketIsUndefined {
            table_name: "global_settings".to_string(),
            table_column: "aws_artefacts_s3_bucket_name".to_string(),
            current_value: "".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

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

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-east-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    ),
    );
}

#[test]
fn test_aws_too_long_bucket_name() {
    assert_eq!(
        PlatformValidationError::AwsArtefactsBucketIsTooLong {
            table_name: "global_settings".to_string(),
            table_column: "aws_artefacts_s3_bucket_name".to_string(),
            current_value: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            current_length: 35,
            max_length: 32,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    aws_artefacts_s3_bucket_name: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

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

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-east-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    ),
    );
}

#[test]
fn test_aws_non_kebab_case_bucket_name() {
    assert_eq!(
        PlatformValidationError::AwsArtefactsBucketHasNonKebabCaseName {
            table_name: "global_settings".to_string(),
            table_column: "aws_artefacts_s3_bucket_name".to_string(),
            current_value: " hou bois ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    aws_artefacts_s3_bucket_name: ' hou bois ',
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

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

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-east-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    ),
    );
}

#[test]
fn test_aws_unused_bucket_name() {
    assert_eq!(
        PlatformValidationError::AwsNotUsedButArtefactsBucketIsDefined {
            table_name: "global_settings".to_string(),
            table_column: "aws_artefacts_s3_bucket_name".to_string(),
            current_value: "abc".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    aws_artefacts_s3_bucket_name: abc,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
}

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

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: man-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        implementation: manual,
    },
    {
        dc_name: man-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        implementation: manual,
    },
]

DATA subnet_router_floating_ip {
  '10.18.0.2/24';
}

DATA STRUCT server [
  {
    dc: man-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: man-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
    ),
    );
}

#[test]
fn test_aws_non_aws_server_kind() {
    assert_eq!(
        PlatformValidationError::DcRoutingSubnetCannotMixDeclaredVpnGatewaysAndDeclaredRouters {
            dc: "aws-1".to_string(),
            subnet: "10.18.0.0/24".to_string(),
            declared_vpn_gateways_found: vec!["server-a".to_string()],
            declared_router_server_found: vec!["server-b".to_string()],
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    kind: testvm.cpu4ram8192,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_non_aws_dc_default_server_kind() {
    assert_eq!(
        PlatformValidationError::AwsDatacenterDefaultServerKindMustStartWithAws {
            aws_datacenter: "aws-1".to_string(),
            invalid_server_kind: "testvm.cpu4ram8192".to_string(),
            expected_server_kind_prefix: "aws.".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: testvm.cpu4ram8192,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_non_aws_instance_types() {
    assert_eq!(
        PlatformValidationError::AwsEveryServerKindInAwsMustStartWithAws {
            aws_server: "server-a".to_string(),
            server_kind: "testvm.cpu4ram8192".to_string(),
            expected_server_kind_prefix: "aws.".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    kind: testvm.cpu4ram8192,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_prohibit_adding_custom_instance_types() {
    assert_eq!(
        PlatformValidationError::AwsAddingCustomInstanceTypesIsNotAllowed {
            alien_aws_server_kind: "aws.mcwoozie".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT server_kind {
    kind: aws.mcwoozie,
    cores: 777,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_prohibit_adding_custom_disk_types() {
    assert_eq!(
        PlatformValidationError::AwsAddingCustomDiskKindsIsNotAllowed {
            alien_aws_disk_kind: "aws.mcwoozie".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT disk_kind {
    kind: aws.mcwoozie,
    is_elastic: false,
    capacity_bytes: 21474836480,
    medium: nvme,
}

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_prohibit_non_aws_disk_types_in_aws_dc() {
    assert_eq!(
        PlatformValidationError::AwsEveryDiskKindInAwsMustStartWithAws {
            aws_server: "server-a".to_string(),
            disk_id: "xvda".to_string(),
            disk_kind: "default-ssd".to_string(),
            expected_disk_kind_prefix: "aws.".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: default-ssd,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_bad_disk_size_remainder() {
    assert_eq!(
        PlatformValidationError::AwsDiskSizeMustBeMultipleOfGigabyte {
            aws_server: "server-a".to_string(),
            disk_id: "xvda".to_string(),
            disk_size: 21474836481,
            remainder_of_current_gigabyte: 1,
            bytes_until_next_gigabyte: 1073741823,
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836481,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_bad_disk_for_root_partition() {
    assert_eq!(
        PlatformValidationError::AwsDiskKindIsNotAllowedToBeRoot {
            aws_server: "server-a".to_string(),
            disk_id: "xvda".to_string(),
            disk_kind: "aws.st1".to_string(),
            forbidden_root_disk_kinds: vec![
                "aws.st1".to_string(),
                "aws.sc1".to_string(),
            ]
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.st1,
      capacity_bytes: 137438953472,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_bad_root_disk_name_nitro() {
    assert_eq!(
        PlatformValidationError::AwsRootDiskOnHypervisorMustBeNamedThis {
            aws_server: "server-a".to_string(),
            hypervisor: "nitro".to_string(),
            root_disk_id: "vda".to_string(),
            root_disk_only_expected_id: "nvme0n1".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    kind: aws.c5.2xlarge,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
      capacity_bytes: 137438953472,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_bad_root_disk_name_metal() {
    assert_eq!(
        PlatformValidationError::AwsRootDiskOnHypervisorMustBeNamedThis {
            aws_server: "server-a".to_string(),
            hypervisor: "metal".to_string(),
            root_disk_id: "vda".to_string(),
            root_disk_only_expected_id: "nvme0n1".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    kind: aws.m5.metal,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
      capacity_bytes: 137438953472,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_bad_root_disk_name_xen() {
    assert_eq!(
        PlatformValidationError::AwsRootDiskOnHypervisorMustBeNamedThis {
            aws_server: "server-a".to_string(),
            hypervisor: "xen".to_string(),
            root_disk_id: "vda".to_string(),
            root_disk_only_expected_id: "xvda".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    kind: aws.c4.2xlarge,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
      capacity_bytes: 137438953472,
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_extra_disk_doesnt_follow_convention() {
    assert_eq!(
        PlatformValidationError::AwsNonRootDiskMustFollowRecommendedConvention {
            aws_server: "server-a".to_string(),
            non_root_disk_id: "sde".to_string(),
            must_match_regex: "^sd[f-p]$".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk [
      {
        disk_id: 'xvda',
        disk_kind: aws.gp2,
        capacity_bytes: 137438953472,
      },
      {
        disk_id: 'sde',
        disk_kind: aws.gp2,
        capacity_bytes: 137438953472,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_cannot_use_io1_with_non_nitro_or_metal_hypervisor() {
    assert_eq!(
        PlatformValidationError::AwsCannotUseDiskKindOnHypervisorInstance {
            aws_server: "server-a".to_string(),
            aws_disk_id: "xvda".to_string(),
            aws_disk_kind: "aws.io1".to_string(),
            server_hypervisor: "xen".to_string(),
            only_allowed_hypervisors_for_disk_kind: vec![
                "nitro".to_string(),
                "metal".to_string(),
            ]
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk [
      {
        disk_id: 'xvda',
        disk_kind: aws.io1,
        capacity_bytes: 137438953472,
      },
      {
        disk_id: 'sde',
        disk_kind: aws.gp2,
        capacity_bytes: 137438953472,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_cannot_use_io2_with_non_nitro_or_metal_hypervisor() {
    assert_eq!(
        PlatformValidationError::AwsCannotUseDiskKindOnHypervisorInstance {
            aws_server: "server-a".to_string(),
            aws_disk_id: "xvda".to_string(),
            aws_disk_kind: "aws.io2".to_string(),
            server_hypervisor: "xen".to_string(),
            only_allowed_hypervisors_for_disk_kind: vec![
                "nitro".to_string(),
                "metal".to_string(),
            ]
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: xvda,
    WITH server_disk [
      {
        disk_id: 'xvda',
        disk_kind: aws.io2,
        capacity_bytes: 137438953472,
      },
      {
        disk_id: 'sde',
        disk_kind: aws.gp2,
        capacity_bytes: 137438953472,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_extra_config_bad_data() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "gp3".to_string(),
            config_provided: "wookie: 123".to_string(),
            error: "unknown field `wookie`, expected `provisioned_iops` or `provisioned_throughput_mb`".to_string(),
            example_valid_config: "provisioned_iops: 3000\nprovisioned_throughput_mb: 125\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.gp3,
        extra_config: 'wookie: 123',
        capacity_bytes: 137438953472,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_extra_config_too_little_iops() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "gp3".to_string(),
            config_provided: "provisioned_iops: 100".to_string(),
            error: "iops provided must be at least 3000 and no more than 16000, got 100".to_string(),
            example_valid_config: "provisioned_iops: 3000\nprovisioned_throughput_mb: 125\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.gp3,
        extra_config: 'provisioned_iops: 100',
        capacity_bytes: 137438953472,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_extra_config_too_big_iops() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "gp3".to_string(),
            config_provided: "provisioned_iops: 16001".to_string(),
            error: "iops provided must be at least 3000 and no more than 16000, got 16001".to_string(),
            example_valid_config: "provisioned_iops: 3000\nprovisioned_throughput_mb: 125\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.gp3,
        extra_config: 'provisioned_iops: 16001',
        capacity_bytes: 137438953472,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_extra_config_too_much_iops_for_disk_space() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "io1".to_string(),
            config_provided: "provisioned_iops: 1001".to_string(),
            error: "iops provided is more than disk size/iops ratio (50 IOPS/GB), maximum possible with 20GB disk is 1000".to_string(),
            example_valid_config: "provisioned_iops: 100\nprovisioned_throughput_mb: null\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.io1,
        extra_config: 'provisioned_iops: 1001',
        capacity_bytes: 21474836480,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_extra_config_too_much_iops_for_ec2_instance() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "io2".to_string(),
            config_provided: "provisioned_iops: 20001".to_string(),
            error: "iops 20001 provisioned for disk is more than maximum available for EC2 instance kind of c5n.large which has maximum of 20000".to_string(),
            example_valid_config: "provisioned_iops: 100\nprovisioned_throughput_mb: null\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.io2,
        extra_config: 'provisioned_iops: 20001',
        capacity_bytes: 32212254720,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_extra_config_too_little_throughput() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "gp3".to_string(),
            config_provided: "provisioned_throughput_mb: 100".to_string(),
            error: "throughput provided must be at least 125MB/s and no more than 1000MB/s, got 100MB/s".to_string(),
            example_valid_config: "provisioned_iops: 3000\nprovisioned_throughput_mb: 125\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.gp3,
        extra_config: 'provisioned_throughput_mb: 100',
        capacity_bytes: 32212254720,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_extra_config_too_much_throughput() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "gp3".to_string(),
            config_provided: "provisioned_throughput_mb: 1001".to_string(),
            error: "throughput provided must be at least 125MB/s and no more than 1000MB/s, got 1001MB/s".to_string(),
            example_valid_config: "provisioned_iops: 3000\nprovisioned_throughput_mb: 125\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.gp3,
        extra_config: 'provisioned_throughput_mb: 1001',
        capacity_bytes: 32212254720,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}

#[test]
fn test_aws_disk_throughput_not_allowed_for_disk_type() {
    assert_eq!(
        PlatformValidationError::AwsInvalidDiskExtraConfig {
            aws_server: "server-a".to_string(),
            aws_disk_id: "nvme0n1".to_string(),
            aws_disk_kind: "io2".to_string(),
            config_provided: "provisioned_throughput_mb: 512".to_string(),
            error: "provisioned_throughput_mb should not be set for disk type io2".to_string(),
            example_valid_config: "provisioned_iops: 100\nprovisioned_throughput_mb: null\n".to_string(),
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
]

DATA STRUCT region [
  {
    region_name: us-west-2,
  },
  {
    region_name: us-east-2,
  },
]

DATA STRUCT datacenter [
    {
        dc_name: aws-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.c5n.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1b
        ',
    },
    {
        dc_name: aws-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
          availability_zone: us-west-1c
        ',
    },
]

DATA STRUCT server [
  {
    dc: aws-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: nvme0n1,
    WITH server_disk [
      {
        disk_id: 'nvme0n1',
        disk_kind: aws.io2,
        extra_config: 'provisioned_throughput_mb: 512',
        capacity_bytes: 32212254720,
      },
    ]
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
    dc: aws-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda',
      disk_kind: aws.gp2,
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
]

"#,
        )
    );
}
