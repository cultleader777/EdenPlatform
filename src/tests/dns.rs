#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[test]
fn test_dns_more_than_one_master() {
    assert_eq!(
        PlatformValidationError::DnsHasMoreThanOneMaster {
            first_master_server: "server1.us-west.epl-infra.net".to_string(),
            second_master_server: "server2.us-west.epl-infra.net".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                ..Default::default()
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DATA STRUCT server [
    {
        hostname: server1,
        is_dns_master: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}
"#,
    ));
}

#[test]
fn test_dns_more_than_two_slaves() {
    assert_eq!(
        PlatformValidationError::DnsHasMoreThanTwoSlaves {
            slave_servers: vec![
                "server1.us-west.epl-infra.net".to_string(),
                "server2.us-west.epl-infra.net".to_string(),
                "server3.us-west.epl-infra.net".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                ..Default::default()
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DATA STRUCT server [
    {
        hostname: server1,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_slave: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.125,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}
"#,
    ));
}

#[test]
fn test_dns_no_master_specified() {
    assert_eq!(
        PlatformValidationError::DnsNoMasterServerSpecifiedInRegion {
            explanation: "There must be exactly one server specified with `is_dns_master` column set to true inside single region".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                ..Default::default()
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DATA STRUCT server [
    {
        hostname: server1,
        is_dns_slave: true,
        ssh_interface: eth0,
        is_vpn_gateway: true,
        WITH server_disk {
            disk_id: vda,
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
        is_dns_slave: true,
        ssh_interface: eth0,
        is_vpn_gateway: true,
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}
"#,
    ));
}

#[test]
fn test_dns_no_slaves_specified() {
    assert_eq!(
        PlatformValidationError::DnsNoSlaveServersSpecifiedInRegion {
            explanation: "There must be from one to two slave dns servers specified with `is_dns_slave` column set to true inside single region".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                ..Default::default()
            },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DATA STRUCT server [
    {
        hostname: server1,
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
            if_prefix: 24,
        }
    },
]

DATA EXCLUSIVE network {
    internet, "0.0.0.0/0";
}
"#,
    ));
}

#[test]
fn test_dns_more_than_one_master_region() {
    assert_eq!(
        PlatformValidationError::DnsMoreThanOneMasterRegion {
            first_master_region: "us-west".to_string(),
            first_master_server: "server1.us-west.epl-infra.net".to_string(),
            second_master_region: "us-east".to_string(),
            second_master_server: "server3.us-east.epl-infra.net".to_string(),
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
    disable_vault_quorum_tests: true,
}

DEFAULTS {
    region.tld 'epl-infra.net',
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc dc1,
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

DATA STRUCT region [
    {
        region_name: us-west,
        is_dns_master: true,
    },
    {
        region_name: us-east,
        is_dns_master: true,
    },
]

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        region: us-west,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        region: us-east,
        network_cidr: '10.18.0.0/16',
    }
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        dc: dc2,
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
fn test_dns_more_than_two_slave_dc() {
    assert_eq!(
        PlatformValidationError::DnsMoreThanTwoSlaveRegions {
            slave_regions: vec![
                "us-east".to_string(),
                "eu-east".to_string(),
                "eu-west".to_string(),
            ],
            slave_servers: vec![
                "server3.us-east.epl-infra.net".to_string(),
                "server5.eu-east.epl-infra.net".to_string(),
                "server7.eu-west.epl-infra.net".to_string(),
            ],
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
    disable_vault_quorum_tests: true,
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

DATA STRUCT region [
    {
        region_name: us-west,
        is_dns_master: true,
    },
    {
        region_name: us-east,
        is_dns_slave: true,
    },
    {
        region_name: eu-east,
        is_dns_slave: true,
    },
    {
        region_name: eu-west,
        is_dns_slave: true,
    },
]

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        region: us-west,
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        region: us-east,
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
        region: eu-east,
    },
    {
        dc_name: dc4,
        network_cidr: '10.20.0.0/16',
        region: eu-west,
    }
]

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: false,
}

DATA STRUCT server [
    {
        hostname: server1,
        dc: dc1,
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        dc: dc2,
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_vpn_gateway: true,
        is_dns_slave: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        dc: dc3,
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.127,
            if_prefix: 24,
        }
    },
    {
        hostname: server6,
        dc: dc3,
        is_dns_master: false,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.128,
            if_prefix: 24,
        }
    },
    {
        hostname: server7,
        dc: dc4,
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.129,
            if_prefix: 24,
        }
    },
    {
        hostname: server8,
        dc: dc4,
        is_dns_master: false,
        is_vpn_gateway: true,
        is_dns_slave: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.130,
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
fn test_dns_no_master_region_specified() {
    assert_eq!(
        PlatformValidationError::DnsNoMasterRegionSpecified {
            explanation: "There must be exactly one region specified with `is_dns_master` column set to true".to_string(),
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
    disable_vault_quorum_tests: true,
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

DATA STRUCT region [
    {
        region_name: us-west,
        is_dns_master: false,
        is_dns_slave: false,
    },
    {
        region_name: us-east,
        is_dns_master: false,
        is_dns_slave: false,
    },
]

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        region: us-west,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        region: us-east,
        network_cidr: '10.18.0.0/16',
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
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        dc: dc2,
        is_dns_master: true,
        is_vpn_gateway: true,
        is_dns_slave: false,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_vpn_gateway: true,
        is_dns_slave: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
fn test_dns_no_slave_region_specified() {
    assert_eq!(
        PlatformValidationError::DnsNoSlaveRegionSpecified {
            explanation: "There must be from one to two regions specified with `is_dns_slave` column set to true".to_string(),
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
    disable_vault_quorum_tests: true,
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

DATA STRUCT region [
    {
        region_name: us-west,
        is_dns_master: true,
    },
    {
        region_name: us-east,
    },
]

DATA STRUCT datacenter [
    {
        dc_name: dc1,
        region: us-west,
        network_cidr: '10.17.0.0/16',
    },
    {
        dc_name: dc2,
        region: us-east,
        network_cidr: '10.18.0.0/16',
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
        is_dns_master: true,
        is_vpn_gateway: true,
        is_dns_slave: false,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_vpn_gateway: true,
        is_dns_slave: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        dc: dc2,
        is_dns_master: true,
        is_vpn_gateway: true,
        is_dns_slave: false,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_vpn_gateway: true,
        is_dns_slave: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
fn test_dns_single_dc_no_master() {
    assert_eq!(
        PlatformValidationError::DnsNoMasterRegionSpecified {
            explanation: "There must be exactly one region specified with `is_dns_master` column set to true".to_string(),
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
    disable_vault_quorum_tests: true,
}

DEFAULTS {
    region.tld epl-infra.net,
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
    is_dns_master: false,
    is_dns_slave: false,
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
        is_dns_master: true,
        is_vpn_gateway: true,
        is_dns_slave: false,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
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
        is_dns_master: false,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: internet,
            if_ip: 123.123.123.124,
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
fn test_dns_lan_interface_master() {
    assert_eq!(
        PlatformValidationError::DnsMasterMustHaveInternetNetworkInterface {
            found_interfaces: vec!["lan".to_string()],
            server: "server1.us-west.epl-infra.net".to_string(),
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
    disable_vault_quorum_tests: true,
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
        is_dns_master: true,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.17.0.10,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_dns_master: false,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.17.0.11,
            if_prefix: 24,
        }
    },
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
}
"#,
    ));
}

#[test]
fn test_dns_lan_interface_slave() {
    assert_eq!(
        PlatformValidationError::DnsSlaveMustHaveInternetNetworkInterface {
            found_interfaces: vec!["lan".to_string()],
            server: "server2.us-west.epl-infra.net".to_string(),
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
    disable_vault_quorum_tests: true,
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
        is_dns_master: false,
        is_dns_slave: false,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.17.0.10,
            if_prefix: 24,
        }
    },
    {
        hostname: server2,
        dc: dc1,
        is_dns_master: false,
        is_dns_slave: true,
        is_vpn_gateway: true,
        ssh_interface: eth0
        WITH server_disk {
            disk_id: vda,
        }
        WITH network_interface {
            if_name: eth0,
            if_network: lan,
            if_ip: 10.17.0.11,
            if_prefix: 24,
        }
    },
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA EXCLUSIVE network {
    internet, '0.0.0.0/0';
    lan, '10.0.0.0/8';
}
"#,
    ));
}

#[test]
fn test_dc_name_and_frontend_app_ingress_clash() {
    assert_eq!(
        PlatformValidationError::DnsRegionNameAndIngressSubdomainFqdnClash {
            fqdn: "us-west.epl-infra.net.".to_string(),
            clash_source_table: "frontend_application_deployment_ingress",
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
    disable_vault_quorum_tests: true,
}

DEFAULTS {
    region.tld epl-infra.net,
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

DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: '/',
    tld: epl-infra.net,
    subdomain: us-west,
  },
]
"#,
    ));
}

#[test]
fn test_dc_name_and_backend_app_ingress_clash() {
    assert_eq!(
        PlatformValidationError::DnsRegionNameAndIngressSubdomainFqdnClash {
            fqdn: "us-west.epl-infra.net.".to_string(),
            clash_source_table: "backend_application_deployment_ingress",
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
    disable_vault_quorum_tests: true,
}

DEFAULTS {
    region.tld epl-infra.net,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    server.dc dc1,
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

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/',
    subdomain: 'us-west',
    tld: epl-infra.net,
    endpoint_list: '
      primary
    ',
}

"#,
    ));
}
