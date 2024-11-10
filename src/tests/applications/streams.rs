#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_application_stream_wiring_bad_syntax() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    bad corrupt syntax
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
      }
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamsWiringInvalidFormat {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "bad corrupt syntax".to_string(),
            explanation:
                "Valid example of stream wiring \"stream_a: nats_cluster_b=>stream_name_c\"",
        }
    );
}

#[test]
fn test_application_stream_wiring_app_no_streams() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    some_stream: default=>deployed_stream
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamsWiringApplicationHasNoStreams {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_stream: default=>deployed_stream".to_string(),
            explanation: "This application has no NATS streams that need to be wired",
        }
    );
}

#[test]
fn test_application_stream_wiring_app_non_existing_stream() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    non_existing_stream: default=>deployed_stream
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
      }
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamsWiringApplicationHasNoStreamSpecified {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "non_existing_stream: default=>deployed_stream".to_string(),
            explanation: "Specified application stream is missing",
            missing_application_stream: "non_existing_stream".to_string(),
            valid_app_streams: vec!["test_stream".to_string(),]
        }
    );
}

#[test]
fn test_application_stream_wiring_no_target_stream() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    test_stream: non_existing_cluster=>non_existing_stream
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
      }
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamWiringTargetStreamDoesntExist {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_stream: non_existing_cluster=>non_existing_stream".to_string(),
            explanation: "Specified nats cluster with stream doesn't exist",
            missing_nats_stream: "non_existing_cluster=>non_existing_stream".to_string(),
        }
    );
}

#[test]
fn test_application_stream_wiring_type_mismatch() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    test_stream: default=>deployed_stream
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
      }
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  },
  {
    type_name: other_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
          z @2 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: other_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamWiringTypeMismatch {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_stream: default=>deployed_stream".to_string(),
            explanation:
                "Application expected NATS stream type mismatches wired NATS cluster stream type",
            application_expected_stream_type: "test_vtype".to_string(),
            target_deployment_stream_type: "other_vtype".to_string(),
        }
    );
}

#[test]
fn test_application_stream_wiring_enable_subjects_mismatch() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    test_stream: default=>deployed_stream
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
        enable_subjects: true,
      }
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  },
  {
    type_name: other_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
          z @2 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamWiringSubjectsEnabledMismatch {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_stream: default=>deployed_stream".to_string(),
            explanation:
                "Application expected NATS stream enable_subjects value mismatches wired NATS cluster stream enable_subjects value",
            application_expected_enable_subjects: true,
            target_deployment_stream_enable_subjects: false,
        }
    );
}

#[test]
fn test_application_stream_wiring_redefined_multiple_times() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    test_stream: default=>deployed_stream
    test_stream: default=>deployed_stream
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
      }
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  },
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamWiringApplicationStreamDefinedMultipleTimes {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_stream: default=>deployed_stream".to_string(),
            explanation: "Specified application NATS stream redefined multiple times",
            redefined_app_stream_name: "test_stream".to_string(),
        }
    );
}

#[test]
fn test_application_stream_wiring_undefined() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
      }
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  },
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamsWiringUndefinedAppNatsStream {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            explanation: "Specified application stream was not defined in wiring",
            undefined_application_stream: "test_stream".to_string(),
        }
    );
}

#[test]
fn test_application_stream_wiring_double_point_to_same_stream() {
    let err = common::assert_platform_validation_error_wcustom_data_wargs(
        common::TestArgs { add_default_global_flags: false, add_default_data: true },
        r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_region_logging_tests: true,
    disable_region_monitoring_tests: true,
    disable_region_tracing_tests: true,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}

DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
  nats_stream_wiring: '
    test_stream: default=>deployed_stream
    other_stream: default=>deployed_stream
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_nats_stream [
      {
        stream_name: test_stream,
        stream_type: test_vtype,
        enable_consumer: true,
      },
      {
        stream_name: other_stream,
        stream_type: test_vtype,
        enable_producer: true,
      },
    ]
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          x @0 :I64,
          y @1 :I64,
        }"
      },
    ]
  },
]

DATA STRUCT nats_cluster {
  cluster_name: default
  WITH nats_jetstream_stream [
    {
      stream_name: deployed_stream,
      stream_type: test_vtype,
    }
  ]
  WITH nats_deployment_instance [
    {
      instance_id: 1,
      nats_server: server-a=>nats1,
    },
    {
      instance_id: 2,
      nats_server: server-b=>nats1,
    },
    {
      instance_id: 3,
      nats_server: server-c=>nats1,
    },
  ]
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

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::ApplicationStreamWiringDifferentAppStreamsPointToSameNatsStream {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            explanation:
                "Two different NATS streams in application point to the same physical NATS stream",
            app_stream_a_name: "other_stream".to_string(),
            app_stream_b_name: "test_stream".to_string(),
            target_physical_stream_a: "default=>deployed_stream".to_string(),
            target_physical_stream_b: "default=>deployed_stream".to_string(),
        }
    );
}
