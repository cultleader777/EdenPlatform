#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_redundant_tld() {
    assert_eq!(
        PlatformValidationError::DnsMiscSubdomainFullTldIsRedundant {
            record_type: "TXT".to_string(),
            tld: "epl-infra.net".to_string(),
            subdomain: "_acme-challenge.epl-infra.net.".to_string(),
            explanation: "Your subdomain should never include the full tld domain, it is already assumed".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_txt_record {
        subdomain: '_acme-challenge.epl-infra.net.',
        WITH tld_txt_record_value {
          value: 'abcdefg',
        }
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_forbidden_misc_txt_record() {
    assert_eq!(
        PlatformValidationError::DnsForbiddenMiscTxtRecord {
            record_type: "TXT".to_string(),
            tld: "epl-infra.net".to_string(),
            subdomain: "_acme-challenge".to_string(),
            reserved_values: vec![
              "_acme-challenge".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_txt_record {
        subdomain: '_acme-challenge',
        WITH tld_txt_record_value {
          value: 'abcdefg',
        }
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_invalid_txt_record_subdomain() {
    assert_eq!(
        PlatformValidationError::DnsInvalidMiscTxtRecordSubdomain {
            failed_regex_check: "^[a-zA-Z0-9_]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9_])?$".to_string(),
            tld: "epl-infra.net".to_string(),
            subdomain: "*#@124".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_txt_record {
        subdomain: '*#@124',
        WITH tld_txt_record_value {
          value: 'abcdefg',
        }
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_txt_record_no_values() {
    assert_eq!(
        PlatformValidationError::DnsMiscTxtRecordNoValues {
            tld: "epl-infra.net".to_string(),
            subdomain: "@".to_string(),
            values_found: 0,
            values_min: 1,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_txt_record {
        subdomain: '@',
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_txt_record_too_many_values() {
    assert_eq!(
        PlatformValidationError::DnsMiscTxtRecordTooManyValues {
            tld: "epl-infra.net".to_string(),
            subdomain: "@".to_string(),
            values_found: 31,
            values_max: 30,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_txt_record {
        subdomain: '@',
        WITH tld_txt_record_value [
          { value: 'abcdefg' },
          { value: 'bbcdefg' },
          { value: 'cbcdefg' },
          { value: 'dbcdefg' },
          { value: 'ebcdefg' },
          { value: 'fbcdefg' },
          { value: 'gbcdefg' },
          { value: 'hbcdefg' },
          { value: 'ibcdefg' },
          { value: 'jbcdefg' },
          { value: 'kbcdefg' },
          { value: 'lbcdefg' },
          { value: 'mbcdefg' },
          { value: 'nbcdefg' },
          { value: 'obcdefg' },
          { value: 'pbcdefg' },
          { value: 'rbcdefg' },
          { value: 'sbcdefg' },
          { value: 'tbcdefg' },
          { value: 'ubcdefg' },
          { value: 'vbcdefg' },
          { value: 'wbcdefg' },
          { value: 'xbcdefg' },
          { value: 'ybcdefg' },
          { value: 'zbcdefg' },
          { value: 'aacdefg' },
          { value: 'accdefg' },
          { value: 'adcdefg' },
          { value: 'aecdefg' },
          { value: 'afcdefg' },
          { value: 'agcdefg' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_txt_record_invalid_value() {
    assert_eq!(
        PlatformValidationError::DnsMiscTxtRecordInvalidValue {
            tld: "epl-infra.net".to_string(),
            subdomain: "@".to_string(),
            value: "\tabc".to_string(),
            failed_regex_check: "^([ -~]{1,253})$".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_txt_record {
        subdomain: '@',
        WITH tld_txt_record_value [
          { value: '	abc' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_tld_in_domain_is_redundant() {
    assert_eq!(
        PlatformValidationError::DnsMiscSubdomainFullTldIsRedundant {
            tld: "epl-infra.net".to_string(),
            subdomain: "honk.epl-infra.net.".to_string(),
            record_type: "CNAME".to_string(),
            explanation: "Your subdomain should never include the full tld domain, it is already assumed".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: 'honk.epl-infra.net.',
        WITH tld_cname_record_value [
          { value: 'abc' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_record_forbidden_subdomain() {
    assert_eq!(
        PlatformValidationError::DnsForbiddenMiscCnameRecordSubdomain {
            tld: "epl-infra.net".to_string(),
            subdomain: "@".to_string(),
            forbidden_value: "@".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: '@',
        WITH tld_cname_record_value [
          { value: 'abc' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_record_invalid_subdomain() {
    assert_eq!(
        PlatformValidationError::DnsInvalidMiscCnameRecordSubdomain {
            tld: "epl-infra.net".to_string(),
            subdomain: "&#12".to_string(),
            failed_regex_check: "^[a-zA-Z0-9]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?(\\.[a-zA-Z0-9_]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?)*$".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: '&#12',
        WITH tld_cname_record_value [
          { value: 'abc' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_record_reserved_subdomain_prefix() {
    assert_eq!(
        PlatformValidationError::DnsMiscCnameRecordReservedPrefix {
            tld: "epl-infra.net".to_string(),
            subdomain: "adm-ponkey".to_string(),
            forbidden_prefix: "adm-".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: 'adm-ponkey',
        WITH tld_cname_record_value [
          { value: 'abc' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_record_ingress_clash() {
    assert_eq!(
        PlatformValidationError::DnsMiscCnameRecordClashSubdomain {
            tld: "epl-infra.net".to_string(),
            subdomain: "ponkey".to_string(),
            previous_source: "backend_application_deployment_ingress".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: 'ponkey',
        WITH tld_cname_record_value [
          { value: 'abc' },
        ]
    }
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
    subdomain: 'ponkey',
    tld: epl-infra.net,
    endpoint_list: '
      primary
    ',
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_record_no_values() {
    assert_eq!(
        PlatformValidationError::DnsMiscCnameRecordNoValues {
            tld: "epl-infra.net".to_string(),
            subdomain: "ponkey".to_string(),
            values_found: 0,
            values_min: 1,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: 'ponkey',
        // WITH tld_cname_record_value [
        //   { value: 'abc' },
        // ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_record_too_many_values() {
    assert_eq!(
        PlatformValidationError::DnsMiscCnameRecordTooManyValues {
            tld: "epl-infra.net".to_string(),
            subdomain: "ponkey".to_string(),
            values_found: 31,
            values_max: 30,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: 'ponkey',
        WITH tld_cname_record_value [
          { value: 'abc' },
          { value: 'bbc' },
          { value: 'cbc' },
          { value: 'dbc' },
          { value: 'ebc' },
          { value: 'fbc' },
          { value: 'gbc' },
          { value: 'hbc' },
          { value: 'ibc' },
          { value: 'kbc' },
          { value: 'lbc' },
          { value: 'mbc' },
          { value: 'nbc' },
          { value: 'obc' },
          { value: 'pbc' },
          { value: 'rbc' },
          { value: 'sbc' },
          { value: 'tbc' },
          { value: 'ubc' },
          { value: 'vbc' },
          { value: 'xbc' },
          { value: 'ybc' },
          { value: 'zbc' },
          { value: 'acc' },
          { value: 'adc' },
          { value: 'aec' },
          { value: 'afc' },
          { value: 'ahc' },
          { value: 'aic' },
          { value: 'ajc' },
          { value: 'akc' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_record_invalid_value() {
    assert_eq!(
        PlatformValidationError::DnsMiscCnameRecordInvalidValue {
            tld: "epl-infra.net".to_string(),
            subdomain: "ponkey".to_string(),
            value: "@#1".to_string(),
            failed_regex_check: "^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*\\.?$".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: 'ponkey',
        WITH tld_cname_record_value [
          { value: 'abc' },
          { value: '@#1' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_cname_fqdn_domain_has_no_period_at_end() {
    assert_eq!(
        PlatformValidationError::DnsMiscCnameRecordWithAtLeastTwoPeriodsMustBeFqdn {
            tld: "epl-infra.net".to_string(),
            subdomain: "ponkey".to_string(),
            value: "henlo.bois.net".to_string(),
            value_expected_at_the_end: ".".to_string(),
            periods_found: 2,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_cname_record {
        subdomain: 'ponkey',
        WITH tld_cname_record_value [
          { value: 'henlo.bois.net' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_mx_invalid_subdomain() {
    assert_eq!(
        PlatformValidationError::DnsInvalidMiscTxtRecordSubdomain {
            tld: "epl-infra.net".to_string(),
            subdomain: "#".to_string(),
            failed_regex_check: "^[a-zA-Z0-9]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?(\\.[a-zA-Z0-9_]([a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?)*$".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_mx_record {
        subdomain: '#',
        WITH tld_mx_record_value [
          { priority: 1, value: 'henlo.bois.net' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_mx_record_no_values() {
    assert_eq!(
        PlatformValidationError::DnsMiscMxRecordNoValues {
            tld: "epl-infra.net".to_string(),
            subdomain: "mail".to_string(),
            values_found: 0,
            values_min: 1,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_mx_record {
        subdomain: 'mail',
        // WITH tld_mx_record_value [
        //   { priority: 1, value: 'henlo.bois.net' },
        // ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_mx_record_too_many_values() {
    assert_eq!(
        PlatformValidationError::DnsMiscMxRecordTooManyValues {
            tld: "epl-infra.net".to_string(),
            subdomain: "mail".to_string(),
            values_found: 31,
            values_max: 30,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_mx_record {
        subdomain: 'mail',
        WITH tld_mx_record_value [
          { priority: 1, value: 'henlo.bois.net' },
          { priority: 1, value: 'aenlo.bois.net' },
          { priority: 1, value: 'benlo.bois.net' },
          { priority: 1, value: 'cenlo.bois.net' },
          { priority: 1, value: 'denlo.bois.net' },
          { priority: 1, value: 'eenlo.bois.net' },
          { priority: 1, value: 'fenlo.bois.net' },
          { priority: 1, value: 'genlo.bois.net' },
          { priority: 1, value: 'ienlo.bois.net' },
          { priority: 1, value: 'jenlo.bois.net' },
          { priority: 1, value: 'lenlo.bois.net' },
          { priority: 1, value: 'menlo.bois.net' },
          { priority: 1, value: 'nenlo.bois.net' },
          { priority: 1, value: 'oenlo.bois.net' },
          { priority: 1, value: 'penlo.bois.net' },
          { priority: 1, value: 'renlo.bois.net' },
          { priority: 1, value: 'senlo.bois.net' },
          { priority: 1, value: 'tenlo.bois.net' },
          { priority: 1, value: 'uenlo.bois.net' },
          { priority: 1, value: 'venlo.bois.net' },
          { priority: 1, value: 'xenlo.bois.net' },
          { priority: 1, value: 'yenlo.bois.net' },
          { priority: 1, value: 'zenlo.bois.net' },
          { priority: 1, value: 'hanlo.bois.net' },
          { priority: 1, value: 'hbnlo.bois.net' },
          { priority: 1, value: 'hcnlo.bois.net' },
          { priority: 1, value: 'hdnlo.bois.net' },
          { priority: 1, value: 'hfnlo.bois.net' },
          { priority: 1, value: 'hgnlo.bois.net' },
          { priority: 1, value: 'hhnlo.bois.net' },
          { priority: 1, value: 'hinlo.bois.net' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_mx_record_invalid_value() {
    assert_eq!(
        PlatformValidationError::DnsMiscMxRecordInvalidValue {
            tld: "epl-infra.net".to_string(),
            subdomain: "mail".to_string(),
            priority: 1,
            value: "##$!".to_string(),
            failed_regex_check: "^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*\\.?$".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_mx_record {
        subdomain: 'mail',
        WITH tld_mx_record_value [
          { priority: 1, value: '##$!' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_mx_record_full_domain_value_does_not_end_with_period() {
    assert_eq!(
        PlatformValidationError::DnsMiscMxRecordWithAtLeastTwoPeriodsMustBeFqdn {
            tld: "epl-infra.net".to_string(),
            subdomain: "mail".to_string(),
            value: "henlo.bois.net".to_string(),
            periods_found: 2,
            value_expected_at_the_end: ".".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_mx_record {
        subdomain: 'mail',
        WITH tld_mx_record_value [
          { priority: 1, value: 'henlo.bois.net' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
fn test_mx_tld_in_domain_is_redundant() {
    assert_eq!(
        PlatformValidationError::DnsMiscSubdomainFullTldIsRedundant {
            tld: "epl-infra.net".to_string(),
            subdomain: "honk.epl-infra.net.".to_string(),
            record_type: "MX".to_string(),
            explanation: "Your subdomain should never include the full tld domain, it is already assumed".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
        r#"
DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    WITH tld_mx_record {
        subdomain: 'honk.epl-infra.net.',
        WITH tld_mx_record_value [
          { priority: 1, value: 'abc' },
        ]
    }
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
}

DEFAULTS {
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
