job "grafana-main" {
  type = "service"
  namespace = "epl"
  region = "us-west"
  datacenters = ["dc1", "dc2", "dc3"]

  vault {
    policies = ["epl-grafana-main"]
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

  group "grafana" {
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
      port "svc" {
        static = 3000
        host_network = "lan"
      }
    }

    service {
      name = "epl-grafana-main"
      port = "svc"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "svc"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "epl-grafana-promxy-main" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "giantswarm/promxy@sha256:9d53be3c6cad0a791bf5eee64103f7c402f20cd19ca9d5afe6208c11033e605f"
        network_mode = "host"
        args = [
          "--bind-addr=127.0.0.1:3001",
          "--config=/secrets/promxy-conf.yml",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/promxy-conf.yml"
        perms = "644"
        data = <<EOL

global:
  evaluation_interval: 5s
  external_labels:
    source: promxy

promxy:
  server_groups:

    - static_configs:
        - targets:
          - epl-mon-default-victoriametrics.service.consul:9091
      # labels to be added to metrics retrieved from this server_group
      labels:
        epl_mc: default
      anti_affinity: 10s
      timeout: 5s
      query_params:
        nocache: 1
EOL
      }
    }

    task "epl-grafana-service-main" {
      driver = "docker"
      resources {
        memory = 256
        memory_max = 384
      }
      env {
        GF_PATHS_CONFIG = "/secrets/grafana.ini"
        GF_PATHS_PROVISIONING = "/secrets/provisioning"
      }
      config {
        image = "grafana/grafana@sha256:39c849cebccccb22c0a5194f07c535669386190e029aa440ad535226974a5809"
        network_mode = "host"
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/grafana.ini"
        perms = "644"
        data = <<EOL

[paths]
data = /var/lib/grafana
plugins = /var/lib/grafana/plugins

[server]
protocol = http
http_port = 3000

[database]
type = postgres
host = epl-pg-testdb.service.consul:5433
user = grafana
password = {{ with secret "epl/data/grafana/main" }}{{ .Data.data.postgres_password }}{{ end }}
name = grafana

[datasources]

[users]
allow_sign_up = false

[security]
admin_user = admin
admin_password = {{ with secret "epl/data/grafana/main" }}{{ .Data.data.admin_password }}{{ end }}

[auth]

[auth.anonymous]
enabled = false

[auth.basic]
enabled = true

[log]
mode = console

[metrics]
enabled = true
EOL
      }

      template {
        destination = "secrets/provisioning/datasources/datasources.yml"
        perms = "644"
        data = <<EOL
apiVersion: 1
datasources:

  - name: promxy all victoria metrics
    type: prometheus
    access: proxy
    url: http://127.0.0.1:3001
    isDefault: true

  - name: default victoria metrics
    type: prometheus
    access: proxy
    url: http://epl-mon-default-victoriametrics.service.us-west.consul:9091

  - name: default prometheus
    type: prometheus
    access: proxy
    url: http://epl-mon-default-victoriametrics.service.us-west.consul:9090

  - name: main loki cluster
    type: loki
    access: proxy
    url: http://epl-loki-main-loki-reader.service.us-west.consul:3012
    jsonData:
      maxLines: 1000

  - name: r1-tempo tempo cluster
    type: tempo
    url: http://epl-tempo-r1-tempo.service.us-west.consul:4310
    access: proxy
    basicAuth: false

EOL
      }
    }
  }

}
