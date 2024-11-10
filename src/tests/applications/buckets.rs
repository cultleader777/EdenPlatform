#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::super::common;

#[test]
fn test_application_bucket_wiring_redefined_multiple_times() {
    assert_eq!(
        PlatformValidationError::ApplicationBucketWiringApplicationBucketDefinedMultipleTimes {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_bucket: global=>app".to_string(),
            explanation: "Specified application S3 bucket redefined multiple times",
            redefined_app_s3_bucket_name: "test_bucket".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
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
  s3_bucket_wiring: '
    test_bucket: global=>app
    test_bucket: global=>app
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_s3_bucket [
      {
        bucket_name: test_bucket,
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
fn test_application_bucket_wiring_target_bucket_doesnt_exist() {
    assert_eq!(
        PlatformValidationError::ApplicationBucketWiringTargetBucketDoesntExist {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_bucket: global=>epp".to_string(),
            explanation: "Specified MinIO cluster with bucket doesn't exist",
            missing_minio_bucket: "global=>epp".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
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
  s3_bucket_wiring: '
    test_bucket: global=>epp
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_s3_bucket [
      {
        bucket_name: test_bucket,
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
fn test_application_bucket_wiring_has_no_bucket_specified() {
    assert_eq!(
        PlatformValidationError::ApplicationBucketsWiringApplicationHasNoBucketSpecified {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_backet: global=>app".to_string(),
            explanation: "Specified application bucket is missing",
            missing_application_bucket: "test_backet".to_string(),
            valid_app_buckets: vec!["test_bucket".to_string()],
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
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
  s3_bucket_wiring: '
    test_backet: global=>app
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world
    WITH backend_application_s3_bucket [
      {
        bucket_name: test_bucket,
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
fn test_application_bucket_wiring_has_no_buckets() {
    assert_eq!(
        PlatformValidationError::ApplicationBucketWiringApplicationHasNoBuckets {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "test_backet: global=>app".to_string(),
            explanation: "This application has no S3 buckets that need to be wired",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
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
  s3_bucket_wiring: '
    test_backet: global=>app
  '
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
fn test_application_bucket_wiring_invalid_format() {
    assert_eq!(
        PlatformValidationError::ApplicationBucketWiringInvalidFormat {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            bad_line: "haiku".to_string(),
            explanation: "Valid example of bucket wiring \"bucket_a: minio_cluster_b=>bucket_c\"",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
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
  s3_bucket_wiring: '
    haiku
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_s3_bucket {
      bucket_name: test_bucket
    }
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
fn test_application_bucket_wiring_undefined_app_bucket() {
    assert_eq!(
        PlatformValidationError::ApplicationBucketWiringUndefinedAppBucket {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            undefined_application_bucket: "test_bucket".to_string(),
            explanation: "Specified application bucket was not defined in wiring",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
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
  s3_bucket_wiring: '
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_s3_bucket {
      bucket_name: test_bucket
    }
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
fn test_application_bucket_wiring_points_to_same_bucket() {
    assert_eq!(
        PlatformValidationError::ApplicationBucketWiringDifferentAppBucketsPointToSameMinioBucket {
            application_deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            app_bucket_b_name: "test_bucket".to_string(),
            app_bucket_a_name: "other_bucket".to_string(),
            target_physical_bucket_a: "global=>app".to_string(),
            target_physical_bucket_b: "global=>app".to_string(),
            explanation: "Two different MinIO buckets in application point to the same physical MinIO bucket",
        },
        common::assert_platform_validation_error_wcustom_data_wargs(
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
  s3_bucket_wiring: '
    test_bucket: global=>app
    other_bucket: global=>app
  '
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_s3_bucket [
      { bucket_name: test_bucket },
      { bucket_name: other_bucket },
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
