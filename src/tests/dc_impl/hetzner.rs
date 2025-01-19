#[cfg(test)]
use crate::static_analysis::PlatformValidationError;
#[cfg(test)]
use super::super::common;
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_hetzner_vlan_interface_id_not_set() {
    assert_eq!(
        PlatformValidationError::VlanIdForInterfaceIsNotSet {
            server_name: "server-a".to_string(),
            interface_network: "lan".to_string(),
            interface_name: "eth0.4001".to_string(),
            interface_ip: "10.18.0.10".to_string(),
            interface_vlan: -1,
            datacenter_implementation: "hetzner".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4001,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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

#[test]
fn test_hetzner_vlan_id_out_of_range() {
    assert_eq!(
        PlatformValidationError::VlanIdForInterfaceIsInInvalidRange {
            server_name: "server-a".to_string(),
            interface_network: "lan".to_string(),
            interface_name: "eth0.4001".to_string(),
            interface_ip: "10.18.0.10".to_string(),
            interface_vlan: 3000,
            datacenter_implementation: "hetzner".to_string(),
            min_vlan: 4000,
            max_vlan: 4091,
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4001,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 3000,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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

#[test]
fn test_hetzner_vlan_id_mismatch_in_subnet() {
    assert_eq!(
        PlatformValidationError::VlanSubnetHasDifferentIds {
            server_name: "server-b".to_string(),
            interface_network: "lan".to_string(),
            interface_name: "eth0.4002".to_string(),
            interface_ip: "10.18.0.11".to_string(),
            interface_vlan: 4002,
            prev_vlan_id: 4001,
            current_vlan_id: 4002,
            datacenter_implementation: "hetzner".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4001,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4001,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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

#[test]
fn test_hetzner_invalid_interface_name() {
    assert_eq!(
        PlatformValidationError::VlanInvalidInterfaceName {
            server_name: "server-a".to_string(),
            interface_network: "lan".to_string(),
            interface_name: "zookie".to_string(),
            interface_ip: "10.18.0.10".to_string(),
            vlan_id: 4002,
            datacenter_implementation: "hetzner".to_string(),
            example_valid_interface_name: "eth0.4002".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: zookie,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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

#[test]
fn test_hetzner_interface_name_not_a_number() {
    assert_eq!(
        PlatformValidationError::VlanInterfaceNameIdIsNotANumber {
            server_name: "server-a".to_string(),
            interface_network: "lan".to_string(),
            interface_name: "eth0.woot".to_string(),
            interface_ip: "10.18.0.10".to_string(),
            interface_vlan: 4002,
            datacenter_implementation: "hetzner".to_string(),
            example_valid_interface_name: "eth0.4002".to_string(),
            parsing_error: "invalid digit found in string".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.woot,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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

#[test]
fn test_hetzner_interface_name_with_vlan_mismatch() {
    assert_eq!(
        PlatformValidationError::VlanInterfaceNameIdMismatchToVlanId {
            server_name: "server-a".to_string(),
            interface_network: "lan".to_string(),
            interface_name: "eth0.4001".to_string(),
            interface_ip: "10.18.0.10".to_string(),
            interface_vlan: 4002,
            datacenter_implementation: "hetzner".to_string(),
            example_valid_interface_name: "eth0.4002".to_string(),
            interface_name_vlan_id: 4001,
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4001,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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

#[test]
fn test_hetzner_srouce_interface_name_with_vlan_not_found() {
    assert_eq!(
        PlatformValidationError::VlanCantFindVlanParentNetworkInterface {
            server_name: "server-a".to_string(),
            interface_network: "lan".to_string(),
            interface_name: "eth1.4002".to_string(),
            interface_ip: "10.18.0.10".to_string(),
            interface_vlan: 4002,
            datacenter_implementation: "hetzner".to_string(),
            example_valid_interface_name: "eth0.4002".to_string(),
            interfaces_on_server: vec![
                "eth0".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth1.4002,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
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
    dc: hz-1,
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
      disk_kind: bm-ssd,
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

#[test]
fn test_vlan_id_specified_on_non_vlan_dc() {
    assert_eq!(
        PlatformValidationError::VlanIdForRouterSubnetIsInInvalidRange {
            min_vlan: 4000,
            max_vlan: 4091,
            router_subnet_vlan_id: 3000,
            datacenter_implementation: "hetzner".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    router_subnet_vlan_id: 3000,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.12,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.12,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.13,
            if_prefix: 32,
        },
        {
            if_name: eth0.4002,
            if_network: lan,
            if_ip: 10.18.0.13,
            if_prefix: 24,
            if_vlan: 4002,
        },
    ]
  },
]

"#,
        ),
    );
}

#[test]
fn test_vlan_id_clash_with_inter_dc_vlan() {
    assert_eq!(
        PlatformValidationError::HetznerInterDcVlanIdClashesWithSubnetVlanIds {
            interface_vlan: 4077,
            hetzner_inter_dc_vlan_id: 4077,
            datacenter_implementation: "hetzner".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    router_subnet_vlan_id: 4007,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4077,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4077,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4077,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4077,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.12,
            if_prefix: 32,
        },
        {
            if_name: eth0.4077,
            if_network: lan,
            if_ip: 10.18.0.12,
            if_prefix: 24,
            if_vlan: 4077,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.13,
            if_prefix: 32,
        },
        {
            if_name: eth0.4077,
            if_network: lan,
            if_ip: 10.18.0.13,
            if_prefix: 24,
            if_vlan: 4077,
        },
    ]
  },
]

"#,
        ),
    );
}

#[test]
fn test_vlan_id_clash_with_subnet_router_vlan_id() {
    assert_eq!(
        PlatformValidationError::VlanSubnetRouterVlanIdClashesWithSubnetVlanId {
            subnet_count: 1,
            router_subnet_vlan_id: 4007,
            subnet_vlan_id: 4007,
            subnet: "10.18.0.0/16".to_string(),
            datacenter_implementation: "hetzner".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    router_subnet_vlan_id: 4007,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.12,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.12,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.13,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.13,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
]

"#,
        ),
    );
}

#[test]
fn test_vlan_id_subnet_router_id_unspecified() {
    assert_eq!(
        PlatformValidationError::VlanSubnetRouterVlanIdUnspecified {
            subnet_count: 1,
            vlan_id: -1,
            datacenter_implementation: "hetzner".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: hetzner,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.12,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.12,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.13,
            if_prefix: 32,
        },
        {
            if_name: eth0.4007,
            if_network: lan,
            if_ip: 10.18.0.13,
            if_prefix: 24,
            if_vlan: 4007,
        },
    ]
  },
]

"#,
        ),
    );
}

#[test]
fn test_subnet_router_id_specified_but_unneeded() {
    assert_eq!(
        PlatformValidationError::VlanSubnetRouterVlanIdSpecifiedButNotUsed {
            subnet_count: 1,
            vlan_id: 4012,
            datacenter_implementation: "bm_simple".to_string(),
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
  }
]

DATA STRUCT region [
  {
    region_name: us-west-2
  }
]

DATA STRUCT server_kind [
  {
    memory_bytes: 17179869184,
    cores: 8,
    kind: beefy-bm1,
    architecture: x86_64,
    bare_metal: true,
  }
]

DATA STRUCT datacenter [
  {
    dc_name: hz-1,
    region: us-west-2,
    network_cidr: '10.18.0.0/16',
    default_server_kind: beefy-bm1,
    implementation: bm_simple,
    router_subnet_vlan_id: 4012,
  }
]

DATA STRUCT disk_kind [
  {
     kind: bm-ssd,
     medium: ssd,
     is_elastic: false,
     max_capacity_bytes: 1099511627776,
  },
]

DATA STRUCT server [
  {
    dc: hz-1,
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-0,
    WITH server_disk {
      disk_id: some-disk-serial-0,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.10,
            if_prefix: 32,
        },
        {
            if_name: eth1,
            if_network: lan,
            if_ip: 10.18.0.10,
            if_prefix: 24,
        },
    ]
  },
  {
    dc: hz-1,
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    root_disk: some-disk-serial-1,
    WITH server_disk {
      disk_id: some-disk-serial-1,
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.11,
            if_prefix: 32,
        },
        {
            if_name: eth1,
            if_network: lan,
            if_ip: 10.18.0.11,
            if_prefix: 24,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.12,
            if_prefix: 32,
        },
        {
            if_name: eth1,
            if_network: lan,
            if_ip: 10.18.0.12,
            if_prefix: 24,
        },
    ]
  },
  {
    dc: hz-1,
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
      disk_kind: bm-ssd,
      capacity_bytes: 21474836480,
    }
    WITH network_interface [
        {
            if_name: eth0,
            if_network: internet,
            if_ip: 77.77.77.13,
            if_prefix: 32,
        },
        {
            if_name: eth1,
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
