
#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_global_settings_defined() {
    assert_eq!(
        PlatformValidationError::EnvironmentMustHaveExactlyOneGlobalSettingsRow {
            table_name: "global_settings".to_string(),
            expected_row_count: 1,
            actual_row_count: 0,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
            r#"
DEFAULTS {
    server.dc dc1,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
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
"#,
        )
    );
}

#[test]
fn test_multiple_global_settings_defined() {
    assert_eq!(
        PlatformValidationError::EnvironmentMustHaveExactlyOneGlobalSettingsRow {
            table_name: "global_settings".to_string(),
            expected_row_count: 1,
            actual_row_count: 2,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
            r#"
DEFAULTS {
    server.dc dc1,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
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


DATA STRUCT tld {
    domain: epl-infra.net,
}

DATA STRUCT global_settings [
    {
      project_name: some,
      admin_email: admin@epl-infra.net,
      admin_tld: epl-infra.net,
    },
    {
      project_name: project,
      admin_email: admin@epl-infra.net,
      admin_tld: epl-infra.net,
    },
]
"#,
        )
    );
}

#[test]
fn test_global_settings_project_name_empty() {
    assert_eq!(
        PlatformValidationError::EnvironmentProjectNameCannotBeEmpty {
            table_name: "global_settings".to_string(),
            column_name: "project_name".to_string(),
            value: "".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
            r#"
DEFAULTS {
    server.dc dc1,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
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

DATA STRUCT tld {
    domain: epl-infra.net,
}

DATA STRUCT global_settings [
    {
      project_name: '',
      admin_email: admin@epl-infra.net,
      admin_tld: epl-infra.net,
    },
]
"#,
        )
    );
}

#[test]
fn test_global_settings_project_name_too_long() {
    assert_eq!(
        PlatformValidationError::EnvironmentProjectNameMustBeNotTooLong {
            table_name: "global_settings".to_string(),
            column_name: "project_name".to_string(),
            value: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            length: 33,
            max_length: 32,
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
            r#"
DEFAULTS {
    server.dc dc1,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
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

DATA STRUCT tld {
    domain: epl-infra.net,
}

DATA STRUCT global_settings [
    {
      project_name: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
      admin_email: admin@epl-infra.net,
      admin_tld: epl-infra.net,
    },
]
"#,
        )
    );
}

#[test]
fn test_global_settings_project_name_non_kebab_case() {
    assert_eq!(
        PlatformValidationError::EnvironmentProjectNameMustBeKebabCase {
            table_name: "global_settings".to_string(),
            column_name: "project_name".to_string(),
            value: "Ayo".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
            r#"
DEFAULTS {
    server.dc dc1,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
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

DATA STRUCT tld {
    domain: epl-infra.net,
}

DATA STRUCT global_settings [
    {
      project_name: Ayo,
      admin_email: admin@epl-infra.net,
      admin_tld: epl-infra.net,
    },
]
"#,
        )
    );
}

#[test]
fn test_server_kind_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ServerKindSpecifiedOnServerDoesntExist {
            server_hostname: "server-a".to_string(),
            non_existing_server_kind: "non_existing_sk".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, non_existing_sk, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  };
  server-b, non_existing_sk, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  };
}
"#,
        )
    );
}

#[test]
fn test_too_small_servers_not_supported() {
    assert_eq!(
        PlatformValidationError::NodeKindCannotBeUsedInEdenPlatform {
            server_hostname: "server-a".to_string(),
            uneligible_server_kind: "gcloud.e2-micro".to_string(),
            reason: "Too little memory to run Eden platform node, node size is 1.0GB, minimum required is 1.5GB (1.0GB system reserved + 0.5GB for workloads)".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, gcloud.e2-micro, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  };
  server-b, non_existing_sk, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  };
}
"#,
        )
    );
}

#[test]
fn test_arm64_servers_not_supported_yet() {
    assert_eq!(
        PlatformValidationError::Arm64ServerKindsAreNotSupportedYet {
            server_hostname: "server-a".to_string(),
            unsupported_server_kind: "gcloud.t2a-standard-2".to_string(),
            unsupported_server_cpu_architecture: "arm64".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, gcloud.t2a-standard-2, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  };
  server-b, non_existing_sk, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  };
}
"#,
        )
    );
}

#[test]
fn test_unsupported_architecture() {
    assert_eq!(
        PlatformValidationError::UnsupportedEdenPlatformServerArchitecture {
            server_hostname: "server-a".to_string(),
            unsupported_server_kind: "some_mac".to_string(),
            unsupported_server_cpu_architecture: "x86_64_mac".to_string(),
            supported_architectures: vec![
                "x86_64".to_string(),
                "arm64".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}


DATA STRUCT server_kind {
  kind: some_mac,
  cores: 12,
  memory_bytes: 34359738368,
  architecture: x86_64_mac,
  non_eligible_reason: "",
  bare_metal: true,
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, some_mac, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  };
  server-b, non_existing_sk, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  };
}
"#,
        )
    );
}

#[test]
fn test_arm64_servers_enabled_with_global_flag() {
    let _ = common::assert_platform_validation_success_wargs(
            common::TestArgs {
                add_default_data: true,
                add_default_global_flags: false,
            },
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
    disable_region_tracing_tests: true,
    disable_region_docker_registry_tests: true,
    experimental_enable_arm64_support: true,
}

DATA STRUCT server_kind {
    kind: testvm.arm-cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: arm64,
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, kind, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, testvm.arm-cpu4ram8192, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  };
  server-b, testvm.arm-cpu4ram8192, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  };
}
"#,
        );
}
