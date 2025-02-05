job "bb-moonbeam-dev" {
  type = "service"
  namespace = "apps"
  region = "us-west"
  datacenters = ["dc1"]

  vault {
    policies = ["epl-bb-depl-moonbeam-dev"]
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

  group "moonbeam" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-bb-moonbeam-dev-server-d-moonbeam}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "port_1" {
        static = 9610
        host_network = "lan"
      }
      port "port_2" {
        static = 9611
        host_network = "lan"
      }
      port "port_3" {
        static = 9612
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "moonbeam-dev"
      read_only = false
    }

    service {
      name = "epl-bb-moonbeam-prom"
      port = "port_2"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "port_2"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "bb-moonbeam-node" {
      driver = "docker"
      resources {
        memory = 512
        memory_max = 640
      }
      env {
        TEST_RAW_VALUE = "hello"
      }
      config {
        image = "moonbeamfoundation/moonbeam@sha256:099e885c4601c8f7ba4408492f2df142920a794baf019cf71cf3a3a16810f504"
        network_mode = "host"
        args = [
          "--dev",
          "--rpc-port=9610",
          "--rpc-external",
          "--prometheus-port=9611",
          "--prometheus-external",
          "--port=9612",
          "--base-path=/data",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/data"
      }

      template {
        destination = "secrets/epl-env-secrets.env"
        perms = "644"
        env = true
        data = <<EOL
TEST_POSTGRESQL=postgresql://bbtest:{{ with secret "epl/data/bb-depl/moonbeam-dev" }}{{ .Data.data.env_var_test_postgresql }}{{ end }}@epl-pg-testdb.service.consul:5433/bbtest
TEST_MINIO=s3://bb-depl-moonbeam-dev:{{ with secret "epl/data/bb-depl/moonbeam-dev" }}{{ .Data.data.env_var_test_minio }}{{ end }}@epl-minio-global.service.consul:9002/bb-app1
EOL
      }
    }
  }

}
