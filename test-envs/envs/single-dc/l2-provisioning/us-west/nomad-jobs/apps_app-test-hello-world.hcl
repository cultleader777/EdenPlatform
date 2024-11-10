job "app-test-hello-world" {
  type = "service"
  namespace = "apps"
  region = "us-west"
  datacenters = ["dc1"]

  vault {
    policies = ["epl-app-test-hello-world"]
  }
  update {
    auto_revert = false
    max_parallel = 1
    health_check = "checks"
    min_healthy_time = "30s"
    stagger = "30s"
    healthy_deadline = "300s"
    progress_deadline = "600s"
  }

  group "app" {
    count = 2
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }

    constraint {
        operator  = "distinct_hosts"
        value     = "true"
    }

    constraint {
      attribute = "${meta.run_unassigned_workloads}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "app" {
        static = 7347
        host_network = "lan"
      }
    }

    service {
      name = "epl-app-test-hello-world"
      port = "app"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "app"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "app-test-hello-world" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      env {
        EPL_CFG_SOME_BOOL = "true"
        EPL_CFG_SOME_FLOAT = "3.14"
        EPL_CFG_SOME_INT = "42"
        EPL_CFG_SOME_STRING = "henlo bois"
        EPL_CH_CHSHARD_DATABASE = "chdb_a"
        EPL_CH_CHSHARD_URL = "http://epl-clickhouse-testch.service.consul:8121"
        EPL_CH_CHSHARD_USER = "db_chdb_a_rw"
        EPL_DEPLOYMENT_NAME = "test-hello-world"
        EPL_HTTP_SOCKET = "${meta.private_ip}:7347"
        EPL_NATS_CONN_SIMPLE_MSG_STREAM = "nats://epl-nats-main-nats.service.consul:4222"
        EPL_NATS_CONN_SOME_TEST_STREAM_CONSUMER = "nats://epl-nats-main-nats.service.consul:4222"
        EPL_NATS_CONN_SOME_TEST_STREAM_PRODUCER = "nats://epl-nats-main-nats.service.consul:4222"
        EPL_NATS_STREAM_SIMPLE_MSG_STREAM = "chdb_a_sink"
        EPL_NATS_STREAM_SOME_TEST_STREAM_CONSUMER = "some_test_stream"
        EPL_NATS_STREAM_SOME_TEST_STREAM_PRODUCER = "some_output_stream"
        EPL_S3_STORAGE_BUCKET = "app1"
        EPL_S3_STORAGE_URI = "http://epl-minio-global.service.consul:9002"
        EPL_S3_STORAGE_USER = "epl_app_test_hello_world"
        OTEL_EXPORTER_OTLP_ENDPOINT = "http://epl-tempo-us-west.service.consul:4314"
        RUST_LOG = "info"
      }
      config {
        image = "@@EPL_APP_IMAGE_x86_64:hello-world@@"
        network_mode = "host"
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/dbcreds"
        perms = "644"
        env = true
        data = <<EOL
EPL_PG_CONN_DEFAULT={{ with secret "epl/data/app/test-hello-world" }}{{ .Data.data.pg_shard_default }}{{ end }}
EPL_CH_CHSHARD_PASSWORD={{ with secret "epl/data/app/test-hello-world" }}{{ .Data.data.ch_shard_chshard_password }}{{ end }}
EOL
      }

      template {
        destination = "secrets/minio"
        perms = "644"
        env = true
        data = <<EOL
EPL_S3_STORAGE_PASSWORD={{ with secret "epl/data/app/test-hello-world" }}{{ .Data.data.minio_bucket_storage_password }}{{ end }}
EOL
      }
    }
  }

}
