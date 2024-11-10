#[cfg(test)]
use pretty_assertions::assert_eq;

#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_gcloud_bad_yaml() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
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
            dc: "gcloud-1".to_string(),
            dc_implementation: "gcloud".to_string(),
            current_settings: "
          availability_zone: abc
          extra_field: huh
        ".to_string(),
            parsing_error: "unknown field `extra_field`, expected `availability_zone` at line 3 column 11".to_string(),
            example_settings: "availability_zone: us-east1-b\n".to_string(),
        }
    );
}

#[test]
fn test_gcloud_non_existing_az() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: abc
        ',
    }
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::GcloudDatacenterUnknownAvailabilityZone {
            dc: "gcloud-1".to_string(),
            dc_implementation: "gcloud".to_string(),
            unknown_availability_zone: "abc".to_string(),
            current_settings: "
          availability_zone: abc
        ".to_string(),
        }
    );
}

#[test]
fn test_gcloud_same_az_multiple_dc() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
    {
        dc_name: gcloud-2,
        region: us-west-3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::GcloudSameAvailabilityZoneUsedForMultipleDatacenters {
            duplicate_az: "us-west1-b".to_string(),
            previous_dc: "gcloud-1".to_string(),
            current_dc: "gcloud-2".to_string(),
        }
    );
}

#[test]
fn test_gcloud_multiple_regions_in_same_epl_region() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
    {
        dc_name: gcloud-2,
        region: us-west-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-east1-b
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
        PlatformValidationError::GcloudMoreThanGcloudOneRegionInsideEdenPlatformRegion {
            eden_platform_region: "us-west-2".to_string(),
            found_gcloud_regions: vec!["us-east1".to_string(), "us-west1".to_string()],
        }
    );
}

#[test]
fn test_gcloud_region_across_multiple_epl_regions() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
    {
        dc_name: gcloud-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-c
        ',
    },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::GcloudRegionIsUsedInMoreThanOneEdenPlatformRegion {
            overused_gcloud_region: "us-west1".to_string(),
            epl_regions_using_gcloud_region: vec!["us-west-2".to_string(), "us-east-2".to_string()],
        }
    );
}

#[test]
fn test_gcloud_server_public_ip_interface_must_have_one_name() {
    assert_eq!(
        PlatformValidationError::GcloudDcrouterNetworkInterfaceNameMustBeEth1 {
            gcloud_dc: "gcloud-1".to_string(),
            gcloud_server: "server-a".to_string(),
            gcloud_dcrouter_network_interface: "eth2".to_string(),
            gcloud_dcrouter_network_interface_only_allowed_name: "eth1".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_artefacts_bucket_name: muh-bucket,
    aws_artefacts_s3_bucket_name: muh-bucket,
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
  {
    network_name: dcrouter,
    cidr: '10.0.0.0/8',
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
        dc_name: gcloud-1,
        region: us-west-2,
        allow_small_subnets: true,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
    {
        dc_name: aws-1,
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
    dc: gcloud-1,
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
            if_name: eth2,
            if_network: dcrouter,
            if_ip: 10.18.252.10,
            if_prefix: 22,
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
    dc: gcloud-1,
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
            if_name: eth1,
            if_network: dcrouter,
            if_ip: 10.18.252.11,
            if_prefix: 22,
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
    dc: gcloud-1,
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: true,
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
        {
            if_name: eth1,
            if_network: dcrouter,
            if_ip: 10.18.252.12,
            if_prefix: 22,
        },
    ]
  },
  {
    dc: gcloud-1,
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_router: true,
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
        {
            if_name: eth1,
            if_network: dcrouter,
            if_ip: 10.18.252.13,
            if_prefix: 22,
        },
    ]
  },
  {
    dc: aws-1,
    hostname: server-e,
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
            if_ip: 10.19.0.10,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.12,
            if_prefix: 32,
        },
    ]
  },
  {
    dc: aws-1,
    hostname: server-f,
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
            if_ip: 10.19.0.11,
            if_prefix: 24,
        },
        {
            if_name: void,
            if_network: internet,
            if_ip: 77.77.77.13,
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
fn test_gcloud_server_lan_ip_interface_must_have_one_name() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
    {
        dc_name: gcloud-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-c
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
    dc: gcloud-1,
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
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::GcloudLanNetworkInterfaceNameMustBeEth0 {
            gcloud_dc: "gcloud-1".to_string(),
            gcloud_server: "server-a".to_string(),
            gcloud_public_network_interface: "enX0".to_string(),
            gcloud_public_network_interface_only_allowed_name: "eth0".to_string(),
        }
    );
}

#[test]
fn test_gcloud_server_public_ip_interface_must_be_ssh_target() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
    {
        dc_name: gcloud-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-c
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
    hostname: server-a,
    ssh_interface: eth0,
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
    dc: gcloud-1,
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
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::GcloudSshInterfaceForPublicServerMustBePublic {
            gcloud_dc: "gcloud-1".to_string(),
            gcloud_server: "server-a".to_string(),
            gcloud_server_expected_ssh_interface: "void".to_string(),
            gcloud_server_ssh_interface: "eth0".to_string(),
        }
    );
}

#[test]
fn test_gcloud_internet_address_only_32_prefix() {
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
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
    {
        dc_name: gcloud-2,
        region: us-east-2,
        network_cidr: '10.19.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-c
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_prefix: 24,
        },
    ]
  },
  {
    dc: gcloud-1,
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
            if_prefix: 24,
        },
    ]
  },
]

"#,
    );

    assert_eq!(
        err,
        PlatformValidationError::GcloudInternetInterfaceMustHave32Prefix {
            gcloud_dc: "gcloud-1".to_string(),
            gcloud_server: "server-a".to_string(),
            gcloud_server_interface: "void".to_string(),
            gcloud_server_mask: 24,
            gcloud_server_only_allowed_mask: 32,
        }
    );
}

#[test]
fn test_gcloud_third_octet_too_large() {
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.129.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.129.11,
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
        PlatformValidationError::InterfaceIpThirdOctetIsTooLarge {
            server_name: "server-a".to_string(),
            interface_ip: "10.18.129.10".to_string(),
            interface_name: "eth0".to_string(),
            interface_network: "lan".to_string(),
            datacenter_name: "gcloud-1".to_string(),
            datacenter_subnet: "10.18.0.0/16".to_string(),
            explanation: "10.18.128.0 and above ips are used for direct L3 to VPN routing hop GRE tunnel".to_string(),
        },
    );
}

#[test]
fn test_gcloud_project_name_undefined() {
    assert_eq!(
        PlatformValidationError::GcloudProjectIdIsUndefined {
            table_name: "global_settings".to_string(),
            table_column: "google_cloud_project_id".to_string(),
            current_value: "".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_artefacts_bucket_name_undefined() {
    assert_eq!(
        PlatformValidationError::GcloudArtefactsBucketIsUndefined {
            table_name: "global_settings".to_string(),
            table_column: "google_cloud_artefacts_bucket_name".to_string(),
            current_value: "".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_artefacts_bucket_name_too_long() {
    assert_eq!(
        PlatformValidationError::GcloudArtefactsBucketIsTooLong {
            table_name: "global_settings".to_string(),
            table_column: "google_cloud_artefacts_bucket_name".to_string(),
            current_value: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            current_length: 36,
            max_length: 32,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_artefacts_bucket_name_not_kebab_case() {
    assert_eq!(
        PlatformValidationError::GcloudArtefactsBucketHasNonKebabCaseName {
            table_name: "global_settings".to_string(),
            table_column: "google_cloud_artefacts_bucket_name".to_string(),
            current_value: "Aye".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_not_used_but_project_id_is_defined() {
    assert_eq!(
        PlatformValidationError::GcloudNotUsedButProjectIdIsDefined {
            table_name: "global_settings".to_string(),
            table_column: "google_cloud_project_id".to_string(),
            current_value: "12345-abc".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-abc,
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
]

DATA STRUCT datacenter [
    {
        dc_name: man-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        implementation: manual,
    },
]

"#,
        ),
    );
}

#[test]
fn test_gcloud_not_used_but_artefacts_bucket_is_defined() {
    assert_eq!(
        PlatformValidationError::GcloudNotUsedButArtefactsBucketIsDefined {
            table_name: "global_settings".to_string(),
            table_column: "google_cloud_artefacts_bucket_name".to_string(),
            current_value: "muh-bucket".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_artefacts_bucket_name: muh-bucket,
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
]

DATA STRUCT datacenter [
    {
        dc_name: man-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        implementation: manual,
    },
]

"#,
        ),
    );
}

#[test]
fn test_gcloud_default_datacenter_server_kind_not_of_gcloud() {
    assert_eq!(
        PlatformValidationError::GcloudDatacenterDefaultServerKindMustStartWithGcloud {
            gcloud_datacenter: "gcloud-1".to_string(),
            invalid_server_kind: "aws.t2.large".to_string(),
            expected_server_kind_prefix: "gcloud.".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_server_kind_not_of_gcloud() {
    assert_eq!(
        PlatformValidationError::GcloudEverServerKindInGoogleCloudMustStartWithGcloud {
            gcloud_server: "server-a".to_string(),
            server_kind: "testvm.cpu4ram8192".to_string(),
            expected_server_kind_prefix: "gcloud.".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
    hostname: server-a,
    kind: testvm.cpu4ram8192,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_prohibit_custom_instance_types() {
    assert_eq!(
        PlatformValidationError::GcloudAddingCustomInstanceTypesIsNotAllowed {
            alien_gcloud_server_kind: "gcloud.mcpoodle".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT server_kind {
  kind: gcloud.mcpoodle,
  cores: 1024,
  memory_bytes: 9999999999999,
  architecture: x86_64,
}


DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_prohibit_custom_disk_types() {
    assert_eq!(
        PlatformValidationError::GcloudAddingCustomDiskKindsIsNotAllowed {
            alien_gcloud_disk_kind: "gcloud.mcdisk".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT disk_kind {
  kind: gcloud.mcdisk,
  max_capacity_bytes: 9999999999999,
  is_elastic: true,
  medium: ssd,
}


DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_all_disks_must_start_with_gcloud() {
    assert_eq!(
        PlatformValidationError::GcloudEveryDiskKindInGcloudMustStartWithGcloud {
            gcloud_server: "server-a".to_string(),
            disk_id: "sda".to_string(),
            disk_kind: "default-ssd".to_string(),
            expected_disk_kind_prefix: "gcloud.".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
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
      disk_kind: default-ssd,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_root_disk_must_be_sda() {
    assert_eq!(
        PlatformValidationError::GcloudRootDiskMustBeNamedSda {
            gcloud_server: "server-a".to_string(),
            root_disk_id: "vda".to_string(),
            root_disk_only_expected_id: "sda".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: vda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_non_root_disk_doesnt_follow_convention() {
    assert_eq!(
        PlatformValidationError::GcloudNonRootDiskMustFollowThisConvention {
            gcloud_server: "server-a".to_string(),
            non_root_disk_id: "sdb".to_string(),
            must_match_regex: "^vd[a-z]$".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk [
      {
        disk_id: sda,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 21474836480,
      },
      {
        disk_id: sdb,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 21474836480,
      },
    ]
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: vda
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.11,
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
fn test_gcloud_disk_size_not_multiple_of_gigabyte() {
    assert_eq!(
        PlatformValidationError::GcloudDiskSizeMustBeMultipleOfGigabyte {
            gcloud_server: "server-a".to_string(),
            disk_id: "sda".to_string(),
            bytes_until_next_gigabyte: 1024 * 1024 * 1024 - 1,
            disk_size: 1024 * 1024 * 1024 * 20 + 1,
            remainder_of_current_gigabyte: 1,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk [
      {
        disk_id: sda,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 21474836481,
      },
    ]
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
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
            if_ip: 10.18.17.11,
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
fn test_gcloud_disk_sizes_exceed_256tb() {
    assert_eq!(
        PlatformValidationError::GcloudTotalDiskSizeOnServerExceedsLimit {
            gcloud_server: "server-a".to_string(),
            explanation: "Total disk sizes attached to google compute instance cannot be above 256TiB".to_string(),
            total_disk_size_limit: 256 * 1024 * 1024 * 1024 * 1024,
            total_disk_sizes_sum: 281496451547136,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true, },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: test-12345,
    google_cloud_artefacts_bucket_name: Aye,
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
]

DATA STRUCT datacenter [
    {
        dc_name: gcloud-1,
        region: us-west-2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
          availability_zone: us-west1-b
        ',
    },
]

DATA STRUCT server [
  {
    dc: gcloud-1,
    hostname: server-a,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: sda,
    WITH server_disk [
      {
        disk_id: sda,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 21474836480,
      },
      {
        disk_id: vda,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 140737488355328,
      },
      {
        disk_id: vdb,
        disk_kind: gcloud.pd-balanced,
        capacity_bytes: 140737488355328,
      },
    ]
    WITH network_interface [
        {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.18.17.10,
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
    dc: gcloud-1,
    hostname: server-b,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
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
            if_ip: 10.18.17.11,
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
