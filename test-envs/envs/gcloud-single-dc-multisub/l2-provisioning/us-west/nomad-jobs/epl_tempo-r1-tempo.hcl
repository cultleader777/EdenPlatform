job "tempo-r1-tempo" {
  type = "service"
  namespace = "epl"
  region = "us-west"
  datacenters = ["dc1"]

  vault {
    policies = ["epl-tempo-r1-tempo"]
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

  group "tempo" {
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
      port "grpc" {
        static = 4311
        host_network = "lan"
      }
      port "http" {
        static = 4310
        host_network = "lan"
      }
      port "otlp_grpc" {
        static = 4314
        host_network = "lan"
      }
      port "otlp_http" {
        static = 4313
        host_network = "lan"
      }
      port "peer" {
        static = 4312
        host_network = "lan"
      }
    }

    service {
      name = "epl-tempo-r1-tempo"
      port = "http"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "http"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "tempo-r1-tempo" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "grafana/tempo@sha256:4443be217c396b065ee34845534199c36fdba4dc619cb96550e228d73fba6e69"
        network_mode = "host"
        args = [
          "-target=scalable-single-binary",
          "-config.file=/secrets/config.yml",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/config.yml"
        perms = "644"
        data = <<EOL

# for grafana
stream_over_http_enabled: true

server:
  http_listen_port: 4310
  http_listen_address: {{ env "meta.private_ip" }}
  grpc_listen_port: 4311
  grpc_listen_address: {{ env "meta.private_ip" }}

distributor:
  ring:
    instance_addr: {{ env "meta.private_ip" }}
  receivers:
    otlp:
      protocols:
        http:
          endpoint: {{ env "meta.private_ip" }}:4313
        grpc:
          endpoint: {{ env "meta.private_ip" }}:4314

ingester:
  max_block_duration: 5m

compactor:
  ring:
    instance_addr: {{ env "meta.private_ip" }}
  compaction:
    block_retention: 720h

memberlist:
  node_name: {{ env "node.unique.name" }}
  abort_if_cluster_join_fails: false
  bind_port: 4312
  bind_addr:
  - {{ env "meta.private_ip" }}
  join_members:
  - epl-tempo-r1-tempo.service.consul:4312

metrics_generator:
  ring:
    instance_addr: {{ env "meta.private_ip" }}
  registry:
    external_labels:
      source: tempo
      cluster: r1-tempo
  storage:
    path: /tmp/tempo/generator/wal
    remote_write:
      - url: http://epl-mon-default-victoriametrics.service.consul:9091/api/v1/write
        send_exemplars: true

storage:
  trace:
    backend: s3                        # backend configuration to use
    wal:
      path: /tmp/tempo/wal             # where to store the the wal locally
    s3:
      bucket: tempo                    # how to store data in s3
      endpoint: epl-minio-global.service.consul:9002
      access_key: tempo_r1_tempo
      secret_key: {{ with secret "epl/data/tempo/r1-tempo" }}{{ .Data.data.minio_bucket_password }}{{ end }}
      insecure: true

querier:
  frontend_worker:
    frontend_address: epl-tempo-r1-tempo.service.consul:4311

overrides:
  defaults:
    metrics_generator:
      processors: ['service-graphs', 'span-metrics']
EOL
      }
    }
  }

}
