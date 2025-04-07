job "loki-main" {
  type = "service"
  namespace = "epl"
  region = "us-west"
  datacenters = ["dc1", "dc2", "dc3"]

  vault {
    policies = ["epl-loki-main"]
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

  group "backend" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.run_unassigned_workloads}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "grpc" {
        static = 3015
        host_network = "lan"
      }
      port "http" {
        static = 3014
        host_network = "lan"
      }
    }

    service {
      name = "epl-loki-main-loki-backend"
      port = "http"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "http"
        port = "http"
        path = "/ready"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "loki-backend-main" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "grafana/loki@sha256:22caa5cdd21d227145acf3cca49db63898152ba470744e2b6962eed7c3469f9e"
        network_mode = "host"
        args = [
          "-config.file=/secrets/config.yml",
          "-target=backend",
          "-server.http-listen-port=3014",
          "-server.http-listen-address=${meta.private_ip}",
          "-server.grpc-listen-port=3015",
          "-server.grpc-listen-address=${meta.private_ip}",
          "-legacy-read-mode=false",
          "-memberlist.advertise-addr=${meta.private_ip}",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/config.yml"
        perms = "644"
        data = <<EOL

# TODO: enable auth maybe?
auth_enabled: false

limits_config:
    retention_period: 720h

common:
  replication_factor: 1
  ring:
    instance_addr: {{ env "meta.private_ip" }}
    kvstore:
      store: consul
      prefix: nomad-loki/epl-loki-main-loki/
      consul:
        host: 127.0.0.1:8500
        acl_token: {{ with secret "epl/data/loki/main" }}{{ .Data.data.consul_token }}{{ end }}

ingester:
  # https://github.com/grafana/loki/issues/8615
  autoforget_unhealthy: true

schema_config:
  configs:
  - from: 2020-05-15
    store: tsdb
    object_store: s3
    schema: v13
    index:
      prefix: index_
      period: 24h

compactor:
  working_directory: /alloc/tmp/compactor
  compaction_interval: 5m

storage_config:
  tsdb_shipper:
    active_index_directory: /alloc/tmp/index
    cache_location: /alloc/tmp/index_cache
  aws:
    s3: s3://loki_main:{{ with secret "epl/data/loki/main" }}{{ .Data.data.minio_bucket_password }}{{ end }}@epl-minio-global.service.consul:9002/loki
    s3forcepathstyle: true
EOL
      }
    }
  }

  group "reader" {
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
        static = 3013
        host_network = "lan"
      }
      port "http" {
        static = 3012
        host_network = "lan"
      }
    }

    service {
      name = "epl-loki-main-loki-reader"
      port = "http"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "http"
        port = "http"
        path = "/ready"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "loki-reader-main" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "grafana/loki@sha256:22caa5cdd21d227145acf3cca49db63898152ba470744e2b6962eed7c3469f9e"
        network_mode = "host"
        args = [
          "-config.file=/secrets/config.yml",
          "-target=read",
          "-server.http-listen-port=3012",
          "-server.http-listen-address=${meta.private_ip}",
          "-server.grpc-listen-port=3013",
          "-server.grpc-listen-address=${meta.private_ip}",
          "-ring.prefix=nomad-loki/epl-loki-main-loki/",
          "-legacy-read-mode=false",
          "-common.compactor-grpc-address=epl-loki-main-loki-backend.service.consul:3015",
          "-memberlist.advertise-addr=${meta.private_ip}",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/config.yml"
        perms = "644"
        data = <<EOL

# TODO: enable auth maybe?
auth_enabled: false

limits_config:
    retention_period: 720h

common:
  replication_factor: 1
  ring:
    instance_addr: {{ env "meta.private_ip" }}
    kvstore:
      store: consul
      prefix: nomad-loki/epl-loki-main-loki/
      consul:
        host: 127.0.0.1:8500
        acl_token: {{ with secret "epl/data/loki/main" }}{{ .Data.data.consul_token }}{{ end }}

ingester:
  # https://github.com/grafana/loki/issues/8615
  autoforget_unhealthy: true

schema_config:
  configs:
  - from: 2020-05-15
    store: tsdb
    object_store: s3
    schema: v13
    index:
      prefix: index_
      period: 24h

compactor:
  working_directory: /alloc/tmp/compactor
  compaction_interval: 5m

storage_config:
  tsdb_shipper:
    active_index_directory: /alloc/tmp/index
    cache_location: /alloc/tmp/index_cache
  aws:
    s3: s3://loki_main:{{ with secret "epl/data/loki/main" }}{{ .Data.data.minio_bucket_password }}{{ end }}@epl-minio-global.service.consul:9002/loki
    s3forcepathstyle: true
EOL
      }
    }
  }

  group "writer" {
    count = 3
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
        static = 3011
        host_network = "lan"
      }
      port "http" {
        static = 3010
        host_network = "lan"
      }
    }

    service {
      name = "epl-loki-main-loki-writer"
      port = "http"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "http"
        port = "http"
        path = "/ready"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "loki-writer-main" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "grafana/loki@sha256:22caa5cdd21d227145acf3cca49db63898152ba470744e2b6962eed7c3469f9e"
        network_mode = "host"
        args = [
          "-config.file=/secrets/config.yml",
          "-target=write",
          "-server.http-listen-port=3010",
          "-server.http-listen-address=${meta.private_ip}",
          "-server.grpc-listen-port=3011",
          "-server.grpc-listen-address=${meta.private_ip}",
          "-ingester.wal-enabled=false",
          "-ring.prefix=nomad-loki/epl-loki-main-loki/",
          "-legacy-read-mode=false",
          "-common.compactor-grpc-address=epl-loki-main-loki-backend.service.consul:3015",
          "-memberlist.advertise-addr=${meta.private_ip}",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/config.yml"
        perms = "644"
        data = <<EOL

# TODO: enable auth maybe?
auth_enabled: false

limits_config:
    retention_period: 720h

common:
  replication_factor: 1
  ring:
    instance_addr: {{ env "meta.private_ip" }}
    kvstore:
      store: consul
      prefix: nomad-loki/epl-loki-main-loki/
      consul:
        host: 127.0.0.1:8500
        acl_token: {{ with secret "epl/data/loki/main" }}{{ .Data.data.consul_token }}{{ end }}

ingester:
  # https://github.com/grafana/loki/issues/8615
  autoforget_unhealthy: true

schema_config:
  configs:
  - from: 2020-05-15
    store: tsdb
    object_store: s3
    schema: v13
    index:
      prefix: index_
      period: 24h

compactor:
  working_directory: /alloc/tmp/compactor
  compaction_interval: 5m

storage_config:
  tsdb_shipper:
    active_index_directory: /alloc/tmp/index
    cache_location: /alloc/tmp/index_cache
  aws:
    s3: s3://loki_main:{{ with secret "epl/data/loki/main" }}{{ .Data.data.minio_bucket_password }}{{ end }}@epl-minio-global.service.consul:9002/loki
    s3forcepathstyle: true
EOL
      }
    }
  }

}
