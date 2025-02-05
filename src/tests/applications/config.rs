#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_application_config_invalid_format() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidFormat {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "bad format".to_string(),
            explanation: "Valid example of application config \"conf_a: abc\"",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    bad format
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_defined_multiple_times() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigDefinedMultipleTimesForDeployment {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: false".to_string(),
            redefined_app_config: "some_conf".to_string(),
            explanation: "Specified application config redefined multiple times",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: true
    some_conf: false
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: bool,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_undefined() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigUndefinedAppConfig {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            undefined_application_config: "some_conf".to_string(),
            explanation: "Specified application config was not defined and doesn't have default value",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: bool,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_defined_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigDoesntExist {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "unknown_conf: 123".to_string(),
            missing_application_config: "unknown_conf".to_string(),
            explanation: "Specified application config is missing",
            valid_app_configs: vec![
                "some_conf".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    unknown_conf: 123
    some_conf: true
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: bool,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_doesnt_have_any_configs() {
    assert_eq!(
        PlatformValidationError::ApplicationDoesntHaveAnyConfigs {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "unknown_conf: 123".to_string(),
            explanation: "This application has no configurations that need to be specified",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    unknown_conf: 123
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_cant_specify_regex_for_bool() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeCannotHaveRegexCheck {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "bool".to_string(),
            application_config_regex_check: "^[a-z]+$".to_string(),
            application_config_regex_check_only_allowed_value: "".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: bool,
        regex_check: '^[a-z]+$',
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_invalid_regex_check() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidRegexCheck {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "string".to_string(),
            application_config_regex_check: "^[a-".to_string(),
            regex_compilation_error: "regex parse error:\n    ^[a-\n     ^\nerror: unclosed character class".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: string,
        regex_check: '^[a-',
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_cant_have_min_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeCannotHaveMinCheck {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "bool".to_string(),
            application_config_min_value: "0".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: bool,
        min_value: 0,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_cant_parse_min_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeCannotParseMinValue {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "int".to_string(),
            application_config_min_value: "abc".to_string(),
            application_config_min_value_parsing_error: "Failed to parse int type: invalid digit found in string".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: int,
        min_value: abc,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_cant_have_max_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeCannotHaveMaxCheck {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "bool".to_string(),
            application_config_max_value: "123".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: bool,
        max_value: 123,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_cant_parse_max_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeCannotParseMaxValue {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "float".to_string(),
            application_config_max_value: "z".to_string(),
            application_config_max_value_parsing_error: "Failed to parse float type: invalid float literal".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: float,
        max_value: z,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_min_int_bigger_than_max() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeMinValueMustBeLessThanMaxValue {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "int".to_string(),
            application_config_min_value: "10".to_string(),
            application_config_max_value: "9".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: int,
        min_value: 10,
        max_value: 9,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_min_float_bigger_than_max() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeMinValueMustBeLessThanMaxValue {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "float".to_string(),
            application_config_min_value: "1.5".to_string(),
            application_config_max_value: "1.4".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: float,
        min_value: 1.5,
        max_value: 1.4,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_min_string_bigger_than_max() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigTypeMinValueMustBeLessThanMaxValue {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "string".to_string(),
            application_config_min_value: "abc".to_string(),
            application_config_max_value: "abb".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: string,
        min_value: abc,
        max_value: abb,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_invalid_default_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidDefaultValue {
            application_name: "hello-world".to_string(),
            application_config: "some_conf".to_string(),
            application_config_type: "bool".to_string(),
            application_config_default_value: "folse".to_string(),
            application_config_default_value_error: "For bool type expected 'true' or 'false', got 'folse'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: bool,
        default_value: folse,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_cant_parse() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: 1.23".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "int".to_string(),
            error: "Failed to parse int type: invalid digit found in string".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: 1.23
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: int,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_float_more_than_min_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: 1.23".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "float".to_string(),
            error: "Value '1.23' is less than minimum config value of '1.5'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: 1.23
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: float,
        min_value: 1.5,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_float_less_than_max_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: 1.9".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "float".to_string(),
            error: "Value '1.9' is more than maximum config value of '1.7'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: 1.9
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: float,
        min_value: 1.5,
        max_value: 1.7,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_float_precision_loss() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: 1.333333333333333333333333".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "float".to_string(),
            error: "Float value loses precision when parsed, initial: '1.333333333333333333333333', after parsing: 1.3333333333333333".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: 1.333333333333333333333333
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: float,
        min_value: 1,
        max_value: 1.7,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_regex_fail() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: vd1".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "string".to_string(),
            error: "Value 'vd1' doesn't match regex check of '^vd[a-z]$'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: vd1
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: string,
        regex_check: '^vd[a-z]$',
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_int_more_than_min_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: 12".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "int".to_string(),
            error: "Value '12' is less than minimum config value of '15'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: 12
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: int,
        min_value: 15,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_int_less_than_max_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: 19".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "int".to_string(),
            error: "Value '19' is more than maximum config value of '17'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: 19
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: int,
        min_value: 15,
        max_value: 17,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_string_less_than_min_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: aaaaaaa".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "string".to_string(),
            error: "Value 'aaaaaaa' is lexicographically less than minimum config value of 'aaabaaaa'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: aaaaaaa
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: string,
        min_value: aaabaaaa,
        max_value: aaazzzzz,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_config_string_more_than_max_value() {
    assert_eq!(
        PlatformValidationError::ApplicationConfigInvalidValue {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "some_conf: aaazaa".to_string(),
            bad_app_config: "some_conf".to_string(),
            bad_app_config_type: "string".to_string(),
            error: "Value 'aaazaa' is lexicographically more than maximum config value of 'aaabbb'".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs { add_default_global_flags: false, add_default_data: true },
            r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    admin_tld: epl-infra.net,
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
  config: '
    some_conf: aaazaa
  ',
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_config [
      {
        config_name: some_conf,
        config_type: string,
        min_value: aaaaaa,
        max_value: aaabbb,
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
    minio;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
    minio;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: app, },
  ]
}
"#,
        )
    );
}
