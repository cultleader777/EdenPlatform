#[cfg(test)]
pub fn scenario_single_dc_env() -> &'static str {
    r#"
DATA STRUCT EXCLUSIVE datacenter {
  dc_name: dc1,
  network_cidr: '10.17.0.0/16',
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation testvms,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },
  {
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: eth0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.13,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: test-hello-world,
    application_name: hello-world,
    count: 2,
    pg_shard_wiring: '
      default: testdb=>testdb_a
    ',
    nats_stream_wiring: '
      some_test_stream_producer: main-nats=>some_output_stream
      some_test_stream_consumer: main-nats=>some_test_stream
    ',
    WITH backend_application_deployment_ingress {
      mountpoint: '/muh/app/',
      tld: epl-infra.net,
      subdomain: www,
      endpoint_list: '
        hello_world
        example
        mutate_test_1
        read_test_1
        dummy
      '
    }
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
      {
        shard_name: default,
        pg_schema: testdb,
        used_queries: '
          max_id_from_foo
          insert_id_returning
        ',
        used_mutators: '
          insert_id
        ',
        used_transactions: '
          all_at_once
        ',
      }
    ]
    WITH backend_application_nats_stream [
      {
        stream_name: some_test_stream_producer,
        stream_type: test_vtype,
        enable_producer: true,
      },
      {
        stream_name: some_test_stream_consumer,
        stream_type: test_vtype,
        enable_consumer: true,
      },
    ]
    WITH backend_http_endpoint [
      {
        http_endpoint_name: hello_world,
        data_type: html,
        path: "/hello_world/{arg:TEXT}/{more:BOOL}?{other:INT}&{floot:FLOAT[]}",
        http_method: GET,
        needs_headers: true,
      },
      {
        http_endpoint_name: example,
        data_type: json,
        path: "/example",
        http_method: POST,
        input_body_type: test_vtype,
        output_body_type: test_vtype,
      },
      {
        http_endpoint_name: mutate_test_1,
        data_type: json,
        path: "/mutate_test_1",
        http_method: POST,
        input_body_type: test_vtype,
        output_body_type: test_output_type,
      },
      {
        http_endpoint_name: read_test_1,
        data_type: json,
        path: "/rt_1",
        http_method: GET,
        output_body_type: test_output_type,
      },
      {
        http_endpoint_name: dummy,
        data_type: json,
        path: "/dummy?{int_arg:INT}&{floatv_arg:FLOAT[]}",
        http_method: POST,
        input_body_type: test_vtype,
        output_body_type: test_vtype,
      },
    ]
  }
]

DATA STRUCT frontend_application [
  {
    application_name: frontend-test,
    WITH frontend_page [
      {
        page_name: home,
        path: '/',
      },
      {
        page_name: single_arg,
        path: '/single/{id:INT}',
      },
      {
        page_name: next_page,
        path: '/next/{id:INT}/{farg:FLOAT}?{arg_a:BOOL}&{arg_b:BOOL[]}&{arg_c:INT}&{arg_d:INT[]}&{arg_e:FLOAT}&{arg_f:FLOAT[]}&{arg_g:TEXT}&{arg_h:TEXT[]}',
      },
      {
        page_name: text_arg,
        path: '/targ/{text:TEXT}',
      },
      {
        page_name: rest_test,
        path: '/rest_test/',
      },
      {
        page_name: links_test,
        path: '/links_test/',
      },
    ]
    WITH frontend_application_used_endpoint [
      {
        endpoint_name: first,
        backend_endpoint: hello-world=>dummy,
      },
    ]
    WITH frontend_application_external_link [
      {
        link_name: be_hello_world,
        backend_endpoint: hello-world=>hello_world,
      },
    ]
    WITH frontend_application_external_page [
      {
        link_name: fe_all_arg,
        frontend_page: frontend-other=>all_arg,
      },
    ]
  },
  {
    application_name: frontend-other,
    WITH frontend_page [
      {
        page_name: home,
        path: '/',
      },
      {
        page_name: single_arg,
        path: '/single/{id:INT}',
      },
      {
        page_name: other_page,
        path: '/other/',
      },
      {
        page_name: all_arg,
        path: '/all/{id:INT}/{name:TEXT}?{opt_i:INT}&{opt_m:FLOAT[]}',
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    deployment_name: frontend-test,
    application_name: frontend-test,
    link_wiring: '
      be_hello_world: test-hello-world
    ',
    page_wiring: '
      fe_all_arg: frontend-other
    ',
    http_port: 7437,
  },
  {
    deployment_name: frontend-other,
    application_name: frontend-other,
    http_port: 7438,
  },
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: frontend-test,
    mountpoint: '/',
    tld: epl-infra.net,
    subdomain: www,
  },
  {
    deployment: frontend-other,
    mountpoint: '/other/',
    tld: epl-infra.net,
    subdomain: www,
  },
]

DATA STRUCT nats_cluster [
  {
    cluster_name: main-nats,
    WITH nats_jetstream_stream [
      {
        stream_name: some_test_stream,
        stream_type: test_vtype,
      },
      {
        stream_name: some_output_stream,
        stream_type: test_vtype,
      },
    ]
    WITH nats_deployment_instance [
      {
        instance_id: 1,
        nats_server: server-b=>nats1,
      },
      {
        instance_id: 2,
        nats_server: server-c=>nats1,
      },
      {
        instance_id: 3,
        nats_server: server-d=>nats1,
      },
    ]
  }
]

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT monitoring_cluster [
  {
    cluster_name: secondary,
    prometheus_port: 9094,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-secondary, },
      { instance_id: 2, monitoring_server: server-c=>mon-secondary, },
      { instance_id: 3, monitoring_server: server-d=>mon-secondary, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-sec-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-sec-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-sec-am, },
    ]
  }
]

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: secondary,
    storage_bucket: global=>loki2,
    loki_writer_http_port: 3020,
    monitoring_cluster: secondary,
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          some_field @0 :I64,
        }"
      },
      {
        version: 5,
        snapshot_source: "{
          some_field @0 :I64,
          other_field @1 :F64 DEFAULT '1.23',
          coordinates @2 :{
            x @0 :F64 DEFAULT '0.0',
            y @1 :F64 DEFAULT '0.0',
          }?,
          is_nice @3 :Bool DEFAULT 'true',
        }"
      }
    ] WITH versioned_type_migration [
      {
        version: 2,
        migration_source: "
          ADD .other_field @1 F64 DEFAULT '1.23'
        "
      },
      {
        version: 3,
        migration_source: "
          ADD .coordinates?.x @0 F64 DEFAULT '0.0'
          ADD .coordinates.y @1 F64 DEFAULT '0.0'
        "
      },
      {
        version: 4,
        migration_source: "
          ADD .is_good @3 Bool DEFAULT 'true'
          ADD .nickname @4 String DEFAULT 'who knows'
        "
      },
      {
        version: 5,
        migration_source: "
          RENAME .is_good .is_nice
          DROP .nickname
        "
      }
    ]
  },
  {
    type_name: test_output_type,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          output_field @0 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT pg_schema [
  {
    schema_name: testdb,
    WITH pg_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE foo (
            id INT PRIMARY KEY
          );
        ",
        downgrade: "DROP TABLE foo;",
      }
    ]
    WITH pg_test_dataset [
      {
        dataset_name: default,
        dataset_contents: "
        foo:
        - id: 1
        - id: 2
        - id: 3
        "
      }
    ]
    WITH pg_query [
      {
        query_name: "max_id_from_foo",
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
        opt_fields: 'max_id',
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 3",
          outputs: "
          - max_id: 3
          "
        }
      },
      {
        query_name: "insert_id_returning",
        query_expression: "INSERT INTO foo(id) VALUES({test_arg:INT}) RETURNING id",
        is_mutating: true,
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 7",
          outputs: "
          - id: 7
          "
        }
      },
    ]
    WITH pg_mutator [
      {
        mutator_name: "insert_id",
        mutator_expression: "INSERT INTO foo(id) VALUES({test_arg:INT})",
        WITH pg_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: "test_arg: 4",
        }
      },
    ]
    WITH pg_transaction [
      {
        transaction_name: all_at_once,
        steps: "
          insert_id_returning
          max_id_from_foo[]
          insert_id[]
        "
      }
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    deployment_name: testdb,
    loki_cluster: secondary,
    docker_image_pg: pg_15.1,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
      }
    ] WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-docker-d,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#
}

#[cfg(test)]
pub fn scenario_multi_dc_env() -> &'static str {
    r#"
DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT pg_schema [
  {
    schema_name: testdb,
    WITH pg_migration [
      {
        time: 1,
        upgrade: "
          CREATE TABLE foo (
            id INT PRIMARY KEY
          );
        ",
        downgrade: "DROP TABLE foo;",
      }
    ]
    WITH pg_test_dataset [
      {
        dataset_name: default,
        dataset_contents: "
        foo:
        - id: 1
        - id: 2
        - id: 3
        "
      }
    ]
    WITH pg_query [
      {
        query_name: "max_id_from_foo",
        query_expression: "SELECT max(id) AS max_id FROM foo WHERE {test_arg:INT} > 0",
        opt_fields: 'max_id',
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 3",
          outputs: "
          - max_id: 3
          "
        }
      },
      {
        query_name: "insert_id_returning",
        query_expression: "INSERT INTO foo(id) VALUES({test_arg:INT}) RETURNING id",
        is_mutating: true,
        WITH pg_query_test {
          test_dataset: default,
          arguments: "test_arg: 7",
          outputs: "
          - id: 7
          "
        }
      },
    ]
    WITH pg_mutator [
      {
        mutator_name: "insert_id",
        mutator_expression: "INSERT INTO foo(id) VALUES({test_arg:INT})",
        WITH pg_mutator_test {
          test_dataset: default,
          resulting_data: '{}',
          arguments: "test_arg: 4",
        }
      },
    ]
    WITH pg_transaction [
      {
        transaction_name: all_at_once,
        steps: "
          insert_id_returning
          max_id_from_foo[]
          insert_id[]
        "
      }
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-g=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-j=>pgtest1,
      },
    ] WITH pg_deployment_schemas [
      {
        db_name: testdb_a,
        pg_schema: testdb,
      }
    ] WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: test-hello-world,
    application_name: hello-world,
    count: 2,
    pg_shard_wiring: '
      default: testdb=>testdb_a
    ',
    nats_stream_wiring: '
      some_test_stream_producer: main-nats=>some_output_stream
      some_test_stream_consumer: main-nats=>some_test_stream
    ',
    WITH backend_application_deployment_ingress {
      mountpoint: '/muh/app/',
      tld: epl-infra.net,
      subdomain: www,
      endpoint_list: '
        hello_world
        example
        mutate_test_1
        read_test_1
        dummy
      '
    }
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_application_pg_shard [
      {
        shard_name: default,
        pg_schema: testdb,
        used_queries: '
          max_id_from_foo
          insert_id_returning
        ',
        used_mutators: '
          insert_id
        ',
        used_transactions: '
          all_at_once
        ',
      }
    ]
    WITH backend_application_nats_stream [
      {
        stream_name: some_test_stream_producer,
        stream_type: test_vtype,
        enable_producer: true,
      },
      {
        stream_name: some_test_stream_consumer,
        stream_type: test_vtype,
        enable_consumer: true,
      },
    ]
    WITH backend_http_endpoint [
      {
        http_endpoint_name: hello_world,
        data_type: html,
        path: "/hello_world/{arg:TEXT}/{more:BOOL}?{other:INT}&{floot:FLOAT[]}",
        http_method: GET,
        needs_headers: true,
      },
      {
        http_endpoint_name: example,
        data_type: json,
        path: "/example",
        http_method: POST,
        input_body_type: test_vtype,
        output_body_type: test_vtype,
      },
      {
        http_endpoint_name: mutate_test_1,
        data_type: json,
        path: "/mutate_test_1",
        http_method: POST,
        input_body_type: test_vtype,
        output_body_type: test_output_type,
      },
      {
        http_endpoint_name: read_test_1,
        data_type: json,
        path: "/rt_1",
        http_method: GET,
        output_body_type: test_output_type,
      },
      {
        http_endpoint_name: dummy,
        data_type: json,
        path: "/dummy?{int_arg:INT}&{floatv_arg:FLOAT[]}",
        http_method: POST,
        input_body_type: test_vtype,
        output_body_type: test_vtype,
      },
    ]
  }
]

DATA STRUCT frontend_application [
  {
    application_name: frontend-test,
    WITH frontend_page [
      {
        page_name: home,
        path: '/',
      },
      {
        page_name: single_arg,
        path: '/single/{id:INT}',
      },
      {
        page_name: next_page,
        path: '/next/{id:INT}/{farg:FLOAT}?{arg_a:BOOL}&{arg_b:BOOL[]}&{arg_c:INT}&{arg_d:INT[]}&{arg_e:FLOAT}&{arg_f:FLOAT[]}&{arg_g:TEXT}&{arg_h:TEXT[]}',
      },
      {
        page_name: text_arg,
        path: '/targ/{text:TEXT}',
      },
      {
        page_name: rest_test,
        path: '/rest_test/',
      },
      {
        page_name: links_test,
        path: '/links_test/',
      },
    ]
    WITH frontend_application_used_endpoint [
      {
        endpoint_name: first,
        backend_endpoint: hello-world=>dummy,
      },
    ]
    WITH frontend_application_external_link [
      {
        link_name: be_hello_world,
        backend_endpoint: hello-world=>hello_world,
      },
    ]
    WITH frontend_application_external_page [
      {
        link_name: fe_all_arg,
        frontend_page: frontend-other=>all_arg,
      },
    ]
  },
  {
    application_name: frontend-other,
    WITH frontend_page [
      {
        page_name: home,
        path: '/',
      },
      {
        page_name: single_arg,
        path: '/single/{id:INT}',
      },
      {
        page_name: other_page,
        path: '/other/',
      },
      {
        page_name: all_arg,
        path: '/all/{id:INT}/{name:TEXT}?{opt_i:INT}&{opt_m:FLOAT[]}',
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    deployment_name: frontend-test,
    application_name: frontend-test,
    link_wiring: '
      be_hello_world: test-hello-world
    ',
    page_wiring: '
      fe_all_arg: frontend-other
    ',
    http_port: 7437,
  },
  {
    deployment_name: frontend-other,
    application_name: frontend-other,
    http_port: 7438,
  },
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: frontend-test,
    mountpoint: '/',
    tld: epl-infra.net,
    subdomain: www,
  },
  {
    deployment: frontend-other,
    mountpoint: '/other/',
    tld: epl-infra.net,
    subdomain: www,
  },
]

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
}

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  datacenter.region us-west,
  datacenter.implementation testvms,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-e=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-e=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}

DATA STRUCT EXCLUSIVE datacenter [
  {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
    allow_small_subnets: true,
  },
  {
    dc_name: dc2,
    network_cidr: '10.18.0.0/16',
  },
  {
    dc_name: dc3,
    network_cidr: '10.19.0.0/16',
    allow_small_subnets: true,
  },
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
  '10.17.1.2/24';
  '10.18.0.2/24';
  '10.19.0.2/24';
  '10.19.1.2/24';
}

DATA STRUCT EXCLUSIVE region [
  { region_name: us-west, is_dns_master: true, availability_mode: multi_dc }
]

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
  {
    network_name: dcrouter,
    cidr: '10.0.0.0/8',
  },
]


DATA STRUCT nats_cluster [
  {
    cluster_name: main-nats,
    WITH nats_jetstream_stream [
      {
        stream_name: some_test_stream,
        stream_type: test_vtype,
      },
      {
        stream_name: some_output_stream,
        stream_type: test_vtype,
      },
    ]
    WITH nats_deployment_instance [
      {
        instance_id: 1,
        nats_server: server-l=>nats1,
      },
      {
        instance_id: 2,
        nats_server: server-d=>nats1,
      },
      {
        instance_id: 3,
        nats_server: server-m=>nats1,
      },
    ]
  }
]

DATA STRUCT server [
  {
    dc: dc1,
    hostname: server-a,
    ssh_interface: enp1s0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 24,
      },
      {
        if_name: enp3s0,
        if_network: dcrouter,
        if_ip: 10.17.252.10,
        if_prefix: 22,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]

    WITH server_root_volume [
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-b,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.17.0.13,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 24,
      },
      {
        if_name: enp3s0,
        if_network: dcrouter,
        if_ip: 10.17.252.11,
        if_prefix: 22,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-c,
    ssh_interface: enp1s0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.12,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    dc: dc2,
    hostname: server-d,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.13,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-e,
    ssh_interface: enp1s0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 24,
      },
      {
        if_name: enp3s0,
        if_network: dcrouter,
        if_ip: 10.19.252.10,
        if_prefix: 22,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.14,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-e,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-f,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 24,
      },
      {
        if_name: enp3s0,
        if_network: dcrouter,
        if_ip: 10.19.252.11,
        if_prefix: 22,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.15,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-g,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.19.0.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-h,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: false,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.19.1.10,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: dcrouter,
        if_ip: 10.19.252.12,
        if_prefix: 22,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-i,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: false,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.19.1.11,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: dcrouter,
        if_ip: 10.19.252.13,
        if_prefix: 22,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-j,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: false,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.17.1.10,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: dcrouter,
        if_ip: 10.17.252.12,
        if_prefix: 22,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-k,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: false,
    is_router: true,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.17.1.11,
        if_prefix: 24,
      },
      {
        if_name: enp2s0,
        if_network: dcrouter,
        if_ip: 10.17.252.13,
        if_prefix: 22,
      },
    ]
  },
  {
    dc: dc1,
    hostname: server-l,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: false,
    is_router: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.17.1.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
  {
    dc: dc3,
    hostname: server-m,
    ssh_interface: enp1s0,
    is_consul_master: false,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
    is_vpn_gateway: false,
    is_router: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface [
      {
        if_name: enp1s0,
        if_network: lan,
        if_ip: 10.19.1.12,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
]

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-e=>minio-docker-e,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          some_field @0 :I64,
        }"
      },
      {
        version: 5,
        snapshot_source: "{
          some_field @0 :I64,
          other_field @1 :F64 DEFAULT '1.23',
          coordinates @2 :{
            x @0 :F64 DEFAULT '0.0',
            y @1 :F64 DEFAULT '0.0',
          }?,
          is_nice @3 :Bool DEFAULT 'true',
        }"
      }
    ] WITH versioned_type_migration [
      {
        version: 2,
        migration_source: "
          ADD .other_field @1 F64 DEFAULT '1.23'
        "
      },
      {
        version: 3,
        migration_source: "
          ADD .coordinates?.x @0 F64 DEFAULT '0.0'
          ADD .coordinates.y @1 F64 DEFAULT '0.0'
        "
      },
      {
        version: 4,
        migration_source: "
          ADD .is_good @3 Bool DEFAULT 'true'
          ADD .nickname @4 String DEFAULT 'who knows'
        "
      },
      {
        version: 5,
        migration_source: "
          RENAME .is_good .is_nice
          DROP .nickname
        "
      }
    ]
  },
  {
    type_name: test_output_type,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          output_field @0 :I64,
        }"
      },
    ]
  }
]
"#
}

#[cfg(test)]
pub fn scenario_aws_single_dc_env() -> &'static str {
  r#"
DATA STRUCT EXCLUSIVE datacenter {
  dc_name: dc1,
  network_cidr: '10.17.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: aws,
  implementation_settings: '
    availability_zone: us-west-1b
  ',
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    aws_artefacts_s3_bucket_name: henlo-bois,
}

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    root_disk: xvda,
    WITH server_disk {
      disk_id: 'xvda',
      disk_kind: aws.gp2,
      capacity_bytes: 21474836480,
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
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
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
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
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 52.53.193.207,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 10.17.0.13,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 54.215.170.43,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-docker-d,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#
}


#[cfg(test)]
pub fn scenario_aws_single_dc_multisubnet_env() -> &'static str {
  r#"
DATA STRUCT EXCLUSIVE datacenter {
  dc_name: dc1,
  allow_small_subnets: true,
  network_cidr: '10.17.0.0/16',
  default_server_kind: aws.t2.large,
  implementation: aws,
  implementation_settings: '
    availability_zone: us-west-1b
  ',
}

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    aws_artefacts_s3_bucket_name: henlo-bois,
}

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: dcrouter,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
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
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: dcrouter,
        if_ip: 10.17.252.10,
        if_prefix: 22,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
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
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: dcrouter,
        if_ip: 10.17.252.11,
        if_prefix: 22,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
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
        if_ip: 10.17.1.10,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: dcrouter,
        if_ip: 10.17.252.12,
        if_prefix: 22,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: void,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_ingress: true,
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
        if_ip: 10.17.1.11,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: dcrouter,
        if_ip: 10.17.252.13,
        if_prefix: 22,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-e,
    ssh_interface: eth0,
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
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
    ]
  },
  {
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
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
        if_ip: 10.17.1.12,
        if_prefix: 24,
      },
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-docker-d,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#
}

#[cfg(test)]
pub fn scenario_aws_multi_dc_env() -> &'static str {
  r#"
DATA STRUCT EXCLUSIVE datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
            availability_zone: us-west-2a
        ',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
            availability_zone: us-west-2b
        ',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
            availability_zone: us-west-2c
        ',
    },
]

DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    aws_artefacts_s3_bucket_name: henlo-bois,
}

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
  availability_mode: multi_dc,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
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
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
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
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
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
        if_ip: 77.77.77.12,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.12,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 77.77.77.13,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.13,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
  {
    hostname: server-e,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
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
        if_ip: 77.77.77.14,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.14,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-e,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-f,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 77.77.77.15,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.15,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-c=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-e=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-e=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-e=>minio-docker-e,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#
}

#[cfg(test)]
pub fn scenario_gcloud_single_dc_env() -> &'static str {
  r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-project,
    google_cloud_artefacts_bucket_name: henlo-bois,
}

DATA STRUCT EXCLUSIVE datacenter {
  dc_name: dc1,
  network_cidr: '10.17.0.0/16',
  default_server_kind: gcloud.e2-standard-4,
  implementation: gcloud,
  implementation_settings: '
    availability_zone: us-west1-b
  ',
}

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation gcloud,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    root_disk: sda,
    WITH server_disk {
      disk_id: sda,
      disk_kind: gcloud.pd-balanced,
      capacity_bytes: 21474836480,
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
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
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
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
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 35.233.254.16,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 10.17.0.13,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 34.105.127.135,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-docker-d,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}

"#
}

#[cfg(test)]
pub fn scenario_gcloud_multi_dc_env() -> &'static str {
  r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-project,
    google_cloud_artefacts_bucket_name: henlo-bois,
}

DATA STRUCT EXCLUSIVE datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-a
        ',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-b
        ',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-c
        ',
    },
]

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
  availability_mode: multi_dc,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
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
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 35.235.82.108,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
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
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 35.236.127.66,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 34.94.151.40,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.12,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 34.94.177.60,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.13,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
  {
    hostname: server-e,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
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
        if_ip: 10.19.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 35.235.118.68,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.14,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-e,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-f,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
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
        if_ip: 10.19.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 34.94.6.34,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.15,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-c=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-e=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-e=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-e=>minio-docker-e,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#
}

#[cfg(test)]
pub fn scenario_gcloud_single_dc_multisub_env() -> &'static str {
  r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-project,
    google_cloud_artefacts_bucket_name: henlo-bois,
}

DATA STRUCT EXCLUSIVE datacenter {
  dc_name: dc1,
  allow_small_subnets: true,
  network_cidr: '10.17.0.0/16',
  default_server_kind: gcloud.e2-standard-4,
  implementation: gcloud,
  implementation_settings: '
    availability_zone: us-west1-b
  ',
}

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation gcloud,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
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
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_vault_instance: true,
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
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_slave: true,
    is_vpn_gateway: true,
    is_ingress: true,
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
        if_ip: 10.17.1.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: void,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_ingress: true,
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
        if_ip: 10.17.1.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-d,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-e,
    ssh_interface: eth0,
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
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
    ]
  },
  {
    hostname: server-f,
    ssh_interface: eth0,
    is_consul_master: false,
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
        if_ip: 10.17.1.12,
        if_prefix: 24,
      },
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-docker-d,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#
}

#[cfg(test)]
pub fn scenario_gcloud_aws_multi_dc_env() -> &'static str {
  return r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    google_cloud_project_id: 12345-project,
    google_cloud_artefacts_bucket_name: henlo-bois,
    aws_artefacts_s3_bucket_name: henlo-bois,
}

DATA STRUCT EXCLUSIVE datacenter [
    {
        dc_name: dc1,
        network_cidr: '10.17.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-a
        ',
    },
    {
        dc_name: dc2,
        network_cidr: '10.18.0.0/16',
        default_server_kind: gcloud.e2-standard-4,
        implementation: gcloud,
        implementation_settings: '
            availability_zone: us-west2-b
        ',
    },
    {
        dc_name: dc3,
        network_cidr: '10.19.0.0/16',
        default_server_kind: aws.t2.large,
        implementation: aws,
        implementation_settings: '
            availability_zone: us-west-2c
        ',
    },
]

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation aws,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
  availability_mode: multi_dc,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
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
        if_ip: 10.17.0.10,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.10,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-b,
    dc: dc1,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_vpn_gateway: true,
    is_ingress: true,
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
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
      {
        if_name: void,
        if_network: internet,
        if_ip: 77.77.77.11,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-b,
        zfs_recordsize: 1M,
      },
    ]
  },
  {
    hostname: server-c,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 77.77.77.12,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.12,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-c,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: pgtest1,
      },
    ]
  },
  {
    hostname: server-d,
    dc: dc2,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
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
        if_ip: 77.77.77.13,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.13,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
    ]
  },
  {
    hostname: server-e,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
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
        if_ip: 77.77.77.14,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.14,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: minio-docker-e,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-f,
    dc: dc3,
    ssh_interface: void,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false,
    is_ingress: false,
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
        if_ip: 77.77.77.15,
        if_prefix: 32,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.15,
        if_prefix: 16,
      },
    ]
    WITH server_root_volume [
      {
        volume_name: nats1,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
    ]
  },
  {
    hostname: server-g,
    dc: dc1,
    ssh_interface: eth0,
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
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
    ]
  },
  {
    hostname: server-h,
    dc: dc2,
    ssh_interface: eth0,
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
        if_ip: 10.18.0.12,
        if_prefix: 24,
      },
    ]
  },
  {
    hostname: server-i,
    dc: dc3,
    ssh_interface: eth0,
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
        if_ip: 10.19.0.12,
        if_prefix: 24,
      },
    ]
  },
]

DATA STRUCT pg_deployment [
  {
    docker_image_pg: pg_15.1,
    deployment_name: testdb,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-c=>pgtest1,
      },
    ]
    WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]


DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-e=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-e=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: global=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker-a,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker-b,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker-c,
    },
    {
      instance_id: 4,
      instance_volume: server-e=>minio-docker-e,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#;
}

#[cfg(test)]
pub fn scenario_single_dc_coprocessor() -> &'static str {
    r#"
DATA STRUCT global_settings {
  project_name: single-dc,
  admin_email: admin@epl-infra.net,
}

DATA admin_ssh_keys {
  "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC3AkuDzzPrMaDav0kN7PIoaBU1Vtw1TfkHxWzPMrleocCltYl8TljwCqEJtmizx5DGKbXFQg31mRVswzuAq2vP2RFdPHQxfl5nJnWsQkelvpPO/Q3LUdtrm19zAgbbDL+AtIg3/lif6/2qNiWCSTfaUpjM7WOPszBNmMRGz/UBZTYc7COTt+I3lK8f6sBn5YyD796LBw6tsNpqfqF9NTAsLT8/PqrXeTpdxFe375gMxeIpNWeE5exMGJKgqnZCcOMOoKMJy61+wdEAYzDFNgIX7ZFvpBYQPf/rTs7LWgtyTSw3fqvMDnfwAf7oIF8rZRwYdVnqTGCWA2h3f4lOf6BERIPkKEK7/DGjmekKnXJrRiLSfcgRjri3VuGBxrJ+Va/Dn6e7o7CdzdJ+fkw7KxTFKuf17Z2r3ZFi1xOduIxXW8/QY6zhq2A11e+HsMe/oaBh3bRcpdMFmW5mqQjGm05xvxArSCAARBKkHjywGs6mRLN2PjNPYdzlI2J8nF6bmSk= henlo";
}

DATA STRUCT EXCLUSIVE datacenter [
  {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
  },
  {
    dc_name: dc2,
    network_cidr: '10.18.0.0/16',
    implementation: coprocessor,
  },
]

DATA STRUCT server_kind [
  {
    kind: testvm.cpu8ram8192,
    cores: 8,
    memory_bytes: 8589934592,
    architecture: x86_64,
  },
  {
    kind: testvm.cpu2ram8192,
    cores: 2,
    memory_bytes: 8589934592,
    architecture: x86_64,
  },
  {
    kind: testvm.cpu2ram4096,
    cores: 2,
    memory_bytes: 4294967296,
    architecture: x86_64,
  },
]

DEFAULTS {
  server.dc dc1,
  server.nixpkgs_environment default_nixpkgs,
  datacenter.region us-west,
  datacenter.implementation testvms,
  datacenter.default_server_kind testvm.cpu2ram8192,
  server_disk.disk_kind default-ssd,
  region.tld epl-infra.net,
  rust_compilation_environment.nixpkgs_environment default_nixpkgs,
  frontend_application_deployment.region us-west,
  backend_application_deployment.region us-west,
  pg_deployment.region us-west,
  nats_cluster.region us-west,
  minio_cluster.region us-west,
  monitoring_cluster.region us-west,
  grafana.region us-west,
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

DATA STRUCT region {
  region_name: us-west,
  is_dns_master: true,
  has_coprocessor_dc: true,
}

DATA STRUCT EXCLUSIVE tld [
  {
    domain: epl-infra.net,
    expose_admin: true,
  }
]

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  },
  {
    network_name: vpn,
    cidr: '172.21.0.0/16',
  },
  {
    network_name: internet,
    cidr: '0.0.0.0/0',
  },
]

DATA valid_server_labels {
  loki_reader;
  loki_writer;
}

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA STRUCT server [
  {
    hostname: server-e,
    kind: testvm.cpu2ram4096,
    dc: dc2,
    ssh_interface: 'eth0:1',
    WITH server_disk [
      { disk_id: 'vda' },
    ]
    WITH network_interface [
      {
        if_name: 'eth0',
        if_network: lan,
        if_ip: 10.18.0.10,
        if_prefix: 32,
      },
      {
        if_name: 'eth0:1',
        if_network: internet,
        if_ip: 77.77.77.14,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.8.10,
        if_prefix: 16,
      },
      {
        if_name: wg1,
        if_network: vpn,
        if_ip: 172.21.8.11,
        if_prefix: 16,
      },
    ]
  },
  {
    hostname: server-f,
    kind: testvm.cpu2ram4096,
    dc: dc2,
    ssh_interface: 'eth0:1',
    WITH server_disk [
      { disk_id: 'vda' },
    ]
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.18.0.11,
        if_prefix: 32,
      },
      {
        if_name: 'eth0:1',
        if_network: internet,
        if_ip: 77.77.77.15,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.8.12,
        if_prefix: 16,
      },
      {
        if_name: wg1,
        if_network: vpn,
        if_ip: 172.21.8.13,
        if_prefix: 16,
      },
    ]
  },
]

DATA STRUCT server [
  {
    hostname: server-a,
    kind: testvm.cpu8ram8192,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_label [
      { label_name: loki_reader, label_value: true },
    ]
    WITH server_disk [
      { disk_id: 'vda' },
      { disk_id: 'vdb', xfs_format: true },
      { disk_id: 'vdc' },
      { disk_id: 'vdd' },
      { disk_id: 'vde' },
      { disk_id: 'vdf' },
      { disk_id: 'vdg' },
      { disk_id: 'vdh' },
      { disk_id: 'vdi' },
      { disk_id: 'vdj' },
    ]
    WITH server_zpool {
      zpool_name: extra,
      WITH server_zfs_dataset [
        { dataset_name: bigone },
      ]
      WITH server_zpool_vdev [
        {
          vdev_number: 1,
          vdev_type: mirror,
          WITH server_zpool_vdev_disk [
            { disk_id: vdc },
            { disk_id: vdd },
          ]
        },
        {
          vdev_number: 2,
          vdev_type: mirror,
          WITH server_zpool_vdev_disk [
            { disk_id: vde },
            { disk_id: vdf },
          ]
        },
      ]
      WITH server_zpool_spare [
        { disk_id: vdg },
      ]
      WITH server_zpool_cache [
        { disk_id: vdh },
      ]
      WITH server_zpool_log [
        { disk_id: vdi },
        { disk_id: vdj },
      ]
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_xfs_volume [
      {
        volume_name: minio-docker,
        xfs_disk: vdb,
      },
    ]
    WITH server_root_volume [
      { volume_name: pgtest1 },
      { volume_name: mon-default },
      { volume_name: mon-am },
    ]
  },
  {
    hostname: server-b,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: false,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: false
    WITH server_label [
      { label_name: loki_reader, label_value: true },
      { label_name: loki_writer, label_value: true },
    ]
    WITH server_disk [
      { disk_id: 'vda' },
      { disk_id: 'vdb', xfs_format: true },
    ]
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.11,
        if_prefix: 24,
      },
    ]
    WITH server_xfs_volume [
      {
        volume_name: minio-docker,
        xfs_disk: vdb,
      },
    ]
    WITH server_root_volume [
      { volume_name: pgtest1 },
      { volume_name: nats1 },
    ]
  },
  {
    hostname: server-c,
    ssh_interface: eth1,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: false,
    is_dns_slave: true,
    is_ingress: true,
    is_vpn_gateway: true,
    is_coprocessor_gateway: true,
    WITH server_label [
      { label_name: loki_writer, label_value: true },
    ]
    WITH server_disk [
      { disk_id: 'vda' },
      { disk_id: 'vdb', xfs_format: true },
    ]
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.12,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.12,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.10,
        if_prefix: 16,
      },
    ]
    WITH server_xfs_volume [
      {
        volume_name: minio-docker,
        xfs_disk: vdb,
      },
    ]
    WITH server_root_volume [
      { volume_name: nats1 },
      { volume_name: mon-default },
      { volume_name: mon-am },
      { volume_name: pgtest1 },
    ]
  },
  {
    hostname: server-d,
    ssh_interface: eth1,
    is_consul_master: false,
    is_nomad_master: true,
    is_vault_instance: true,
    is_dns_master: true,
    is_dns_slave: false,
    is_ingress: true,
    is_vpn_gateway: true,
    is_coprocessor_gateway: true,
    WITH server_label [
      { label_name: loki_writer, label_value: true },
    ]
    WITH server_disk [
      { disk_id: 'vda' },
      { disk_id: 'vdb', xfs_format: true },
    ]
    WITH network_interface [
      {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.13,
        if_prefix: 24,
      },
      {
        if_name: eth1,
        if_network: internet,
        if_ip: 77.77.77.13,
        if_prefix: 24,
      },
      {
        if_name: wg0,
        if_network: vpn,
        if_ip: 172.21.7.11,
        if_prefix: 16,
      },
    ]
    WITH server_xfs_volume [
      {
        volume_name: minio-docker,
        xfs_disk: vdb,
      },
    ]
    WITH server_root_volume [
      { volume_name: nats1 },
      { volume_name: mon-default },
      { volume_name: mon-am },
    ]
  }
]

DATA STRUCT nats_cluster [
  {
    cluster_name: main-nats,
    WITH nats_jetstream_stream [
      {
        stream_name: some_test_stream,
        stream_type: test_vtype,
      },
      {
        stream_name: some_output_stream,
        stream_type: test_vtype,
      },
    ]
    WITH nats_deployment_instance [
      {
        instance_id: 1,
        nats_server: server-b=>nats1,
      },
      {
        instance_id: 2,
        nats_server: server-c=>nats1,
      },
      {
        instance_id: 3,
        nats_server: server-d=>nats1,
      },
    ]
  }
]

DATA STRUCT monitoring_cluster [
  {
    cluster_name: default,
    is_region_default: true,
    WITH monitoring_instance [
      { instance_id: 1, monitoring_server: server-a=>mon-default, },
      { instance_id: 2, monitoring_server: server-c=>mon-default, },
      { instance_id: 3, monitoring_server: server-d=>mon-default, },
    ]
    WITH alertmanager_instance [
      { instance_id: 1, alertmanager_server: server-a=>mon-am, },
      { instance_id: 2, alertmanager_server: server-c=>mon-am, },
      { instance_id: 3, alertmanager_server: server-d=>mon-am, },
    ]
    WITH monitoring_cluster_alert_group [
      { alert_group_name: Default, telegram_channel: default, telegram_bot: default, }
    ]
  }
]

DATA STRUCT telegram_channel {
  channel_name: default,
  channel_id: 12345,
}

DATA STRUCT telegram_bot {
  bot_name: default,
  bot_token: 1234567,
}

DATA STRUCT grafana [
  {
    deployment_name: main,
    database: testdb=>grafana,
  }
]

DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
    writer_placement: '
      match_keys_and_values:
        loki_writer: true
    ',
    reader_placement: '
      match_keys_and_values:
        loki_reader: true
    ',
  }
]

DATA STRUCT tempo_cluster [
  {
    cluster_name: us-west,
    storage_bucket: global=>tempo,
  }
]

DATA STRUCT versioned_type [
  {
    type_name: test_vtype,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          some_field @0 :I64,
        }"
      },
      {
        version: 5,
        snapshot_source: "{
          some_field @0 :I64,
          other_field @1 :F64 DEFAULT '1.23',
          coordinates @2 :{
            x @0 :F64 DEFAULT '0.0',
            y @1 :F64 DEFAULT '0.0',
          }?,
          is_nice @3 :Bool DEFAULT 'true',
        }"
      }
    ] WITH versioned_type_migration [
      {
        version: 2,
        migration_source: "
          ADD .other_field @1 F64 DEFAULT '1.23'
        "
      },
      {
        version: 3,
        migration_source: "
          ADD .coordinates?.x @0 F64 DEFAULT '0.0'
          ADD .coordinates.y @1 F64 DEFAULT '0.0'
        "
      },
      {
        version: 4,
        migration_source: "
          ADD .is_good @3 Bool DEFAULT 'true'
          ADD .nickname @4 String DEFAULT 'who knows'
        "
      },
      {
        version: 5,
        migration_source: "
          RENAME .is_good .is_nice
          DROP .nickname
        "
      }
    ]
  },
  {
    type_name: test_output_type,
    WITH versioned_type_snapshot [
      {
        version: 1,
        snapshot_source: "{
          output_field @0 :I64,
        }"
      },
    ]
  }
]

DATA STRUCT pg_deployment [
  {
    deployment_name: testdb,
    synchronous_replication: true,
    docker_image_pg: pg_15.1,
    WITH pg_deployment_instance [
      {
        instance_id: 1,
        pg_server: server-a=>pgtest1,
      },
      {
        instance_id: 2,
        pg_server: server-b=>pgtest1,
      },
      {
        instance_id: 3,
        pg_server: server-c=>pgtest1,
      },
    ] WITH pg_deployment_unmanaged_db [
      {
        db_name: grafana,
      }
    ]
  }
]

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: global=>docker,
}

DATA STRUCT minio_cluster {
  cluster_name: global,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio-docker,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio-docker,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio-docker,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio-docker,
    },
  ]
  WITH minio_bucket [
    {
      bucket_name: docker,
    },
    {
      bucket_name: loki,
    },
    {
      bucket_name: loki2,
    },
    {
      bucket_name: tempo,
    },
    {
      bucket_name: app1,
    },
  ]
}

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 10 10 10 10 10 10 10 10 10 10
                '
            }
        ]
    }
  ]
}
"#
}
