job "mon-default" {
  type = "service"
  namespace = "epl"
  region = "us-west"
  datacenters = ["dc1", "dc2"]
  update {
    auto_revert = false
    max_parallel = 1
    health_check = "checks"
    min_healthy_time = "30s"
    stagger = "30s"
    healthy_deadline = "300s"
    progress_deadline = "600s"
  }

  group "alertmanager-1" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-mon-server-a-am-default}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "am" {
        static = 9092
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "mon-am"
      read_only = false
    }

    service {
      name = "epl-mon-default-alertmanager"
      port = "am"
      address = "${meta.private_ip}"
      check {
        type = "http"
        port = "am"
        path = "/-/healthy"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "mon-default-am" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      config {
        image = "bitnami/alertmanager@sha256:a676ae08b0d1e06d5cf3e0e9a4d5cc90e919d1ebce895bea9dcfcc41bffcc0ca"
        network_mode = "host"
        args = [
          "--web.listen-address=10.17.0.10:9092",
          "--config.file=/secrets/alertmanager.yml",
          "--storage.path=/volume/alertmanager",
          "--cluster.listen-address=10.17.0.10:9093",
          "--cluster.advertise-address=10.17.0.10:9093",
          "--cluster.peer=10.17.0.12:9093",
          "--cluster.peer=10.17.0.13:9093",
        ]
      }

      volume_mount {
        volume = "v_1"
        destination = "/volume"
      }

      template {
        destination = "secrets/alertmanager.yml"
        perms = "644"
        data = <<EOL

# Inhibition rules allow to mute a set of alerts given that another alert is
# firing.
# We use this to mute any warning-level notifications if the same alert is
# already critical.
inhibit_rules:
  - source_matchers: [severity="critical"]
    target_matchers: [severity="warning"]
    equal: [alertname, cluster, service]

# The root route on which each incoming alert enters.
route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 1h
  # this receiver will never be used as we explicitly list all alert groups
  # to which channel they belong
  receiver: unused

  routes:
  - receiver: Default
    matchers:
    - alertname=~"^(FilesystemSpaceLow)$"
receivers:
  - name: 'Default'
    telegram_configs:
      - bot_token: 1234567
        chat_id: 12345
        api_url: https://api.telegram.org
        parse_mode: ''

  - name: 'unused'
    telegram_configs:
      - bot_token: bad_bot_token
        chat_id: -123456789
        api_url: https://api.telegram.org
        parse_mode: ''
EOL
      }
    }
  }

  group "alertmanager-2" {
    count = 1
    shutdown_delay = "60s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-mon-server-c-am-default}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "am" {
        static = 9092
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "mon-am"
      read_only = false
    }

    service {
      name = "epl-mon-default-alertmanager"
      port = "am"
      address = "${meta.private_ip}"
      check {
        type = "http"
        port = "am"
        path = "/-/healthy"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "mon-default-am" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      config {
        image = "bitnami/alertmanager@sha256:a676ae08b0d1e06d5cf3e0e9a4d5cc90e919d1ebce895bea9dcfcc41bffcc0ca"
        network_mode = "host"
        args = [
          "--web.listen-address=10.17.0.12:9092",
          "--config.file=/secrets/alertmanager.yml",
          "--storage.path=/volume/alertmanager",
          "--cluster.listen-address=10.17.0.12:9093",
          "--cluster.advertise-address=10.17.0.12:9093",
          "--cluster.peer=10.17.0.10:9093",
          "--cluster.peer=10.17.0.13:9093",
        ]
      }

      volume_mount {
        volume = "v_1"
        destination = "/volume"
      }

      template {
        destination = "secrets/alertmanager.yml"
        perms = "644"
        data = <<EOL

# Inhibition rules allow to mute a set of alerts given that another alert is
# firing.
# We use this to mute any warning-level notifications if the same alert is
# already critical.
inhibit_rules:
  - source_matchers: [severity="critical"]
    target_matchers: [severity="warning"]
    equal: [alertname, cluster, service]

# The root route on which each incoming alert enters.
route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 1h
  # this receiver will never be used as we explicitly list all alert groups
  # to which channel they belong
  receiver: unused

  routes:
  - receiver: Default
    matchers:
    - alertname=~"^(FilesystemSpaceLow)$"
receivers:
  - name: 'Default'
    telegram_configs:
      - bot_token: 1234567
        chat_id: 12345
        api_url: https://api.telegram.org
        parse_mode: ''

  - name: 'unused'
    telegram_configs:
      - bot_token: bad_bot_token
        chat_id: -123456789
        api_url: https://api.telegram.org
        parse_mode: ''
EOL
      }
    }
  }

  group "alertmanager-3" {
    count = 1
    shutdown_delay = "120s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-mon-server-d-am-default}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "am" {
        static = 9092
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "mon-am"
      read_only = false
    }

    service {
      name = "epl-mon-default-alertmanager"
      port = "am"
      address = "${meta.private_ip}"
      check {
        type = "http"
        port = "am"
        path = "/-/healthy"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "mon-default-am" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      config {
        image = "bitnami/alertmanager@sha256:a676ae08b0d1e06d5cf3e0e9a4d5cc90e919d1ebce895bea9dcfcc41bffcc0ca"
        network_mode = "host"
        args = [
          "--web.listen-address=10.17.0.13:9092",
          "--config.file=/secrets/alertmanager.yml",
          "--storage.path=/volume/alertmanager",
          "--cluster.listen-address=10.17.0.13:9093",
          "--cluster.advertise-address=10.17.0.13:9093",
          "--cluster.peer=10.17.0.10:9093",
          "--cluster.peer=10.17.0.12:9093",
        ]
      }

      volume_mount {
        volume = "v_1"
        destination = "/volume"
      }

      template {
        destination = "secrets/alertmanager.yml"
        perms = "644"
        data = <<EOL

# Inhibition rules allow to mute a set of alerts given that another alert is
# firing.
# We use this to mute any warning-level notifications if the same alert is
# already critical.
inhibit_rules:
  - source_matchers: [severity="critical"]
    target_matchers: [severity="warning"]
    equal: [alertname, cluster, service]

# The root route on which each incoming alert enters.
route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 1h
  # this receiver will never be used as we explicitly list all alert groups
  # to which channel they belong
  receiver: unused

  routes:
  - receiver: Default
    matchers:
    - alertname=~"^(FilesystemSpaceLow)$"
receivers:
  - name: 'Default'
    telegram_configs:
      - bot_token: 1234567
        chat_id: 12345
        api_url: https://api.telegram.org
        parse_mode: ''

  - name: 'unused'
    telegram_configs:
      - bot_token: bad_bot_token
        chat_id: -123456789
        api_url: https://api.telegram.org
        parse_mode: ''
EOL
      }
    }
  }

  group "monitoring-1" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-mon-server-c-default}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "prom" {
        static = 9090
        host_network = "lan"
      }
      port "vm" {
        static = 9091
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "mon-default"
      read_only = false
    }

    volume "v_2" {
      type = "host"
      source = "mon-default"
      read_only = false
    }

    service {
      name = "epl-mon-default-prometheus"
      port = "prom"
      address = "${meta.private_ip}"
      check {
        type = "http"
        port = "prom"
        path = "/-/healthy"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-mon-default-victoriametrics"
      port = "vm"
      address = "${meta.private_ip}"
      check {
        type = "http"
        port = "vm"
        path = "/health"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "mon-default-prometheus" {
      driver = "docker"
      resources {
        memory = 768
        memory_max = 896
      }
      config {
        image = "bitnami/prometheus@sha256:fa5181c0bb2843c181bdbc97571a7938f7ee2778d198b4be3b4f2ad43297e8a7"
        network_mode = "host"
        args = [
          "--web.listen-address=10.17.0.12:9090",
          "--config.file=/secrets/prometheus.yml",
          "--storage.tsdb.path=/volume/prometheus-data",
          "--storage.tsdb.retention.time=15d",
          "--web.console.libraries=/opt/bitnami/prometheus/conf/console_libraries",
          "--web.console.templates=/opt/bitnami/prometheus/conf/consoles",
          "--web.external-url=https://adm-prometheus-default.epl-infra.net",
        ]
      }

      volume_mount {
        volume = "v_1"
        destination = "/volume"
      }

      template {
        destination = "secrets/alert_rules.yml"
        perms = "644"
        data = <<EOL
groups:
- name: Default
  rules:

  - alert: FilesystemSpaceLow
    expr: round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100           / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
    for: 5m
    labels:
      severity: 50
    annotations:
      description: "Filesystem {{"{{"}} $labels.device {{"}}"}} at {{"{{"}} $labels.instance {{"}}"}} has less than 20% disk space remaining"
EOL
      }

      template {
        destination = "secrets/prometheus.yml"
        perms = "644"
        data = <<EOL

global:
  scrape_interval: 15s
  evaluation_interval: 15s

remote_write:
  - url: http://10.17.0.12:9091/api/v1/write


rule_files:
  - /secrets/alert_rules.yml


scrape_configs:
  - job_name: "prometheus"
    static_configs:
      - targets: ["10.17.0.12:9090"]

  - job_name: 'consul'
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        tags:
          - epl-mon-default
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance
      - source_labels: [__meta_consul_service_metadata_metrics_path]
        regex: ^(/.+)$
        action: replace
        target_label: __metrics_path__


  - job_name: 'consul-nomad'
    scheme: https
    tls_config:
      insecure_skip_verify: true
    metrics_path: /v1/metrics
    params:
      format: ['prometheus']
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        services:
          - nomad-clients
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance

  - job_name: 'consul-vault'
    scheme: https
    tls_config:
      insecure_skip_verify: true
    metrics_path: /v1/sys/metrics
    params:
      format: ['prometheus']
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        services:
          - vault
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance



alerting:
  alertmanagers:
    - static_configs:
      - targets:
        - 10.17.0.10:9092
        - 10.17.0.12:9092
        - 10.17.0.13:9092
EOL
      }
    }

    task "mon-default-vm" {
      driver = "docker"
      resources {
        memory = 256
        memory_max = 384
      }
      config {
        image = "victoriametrics/victoria-metrics@sha256:8b57b33434c062bfdfc2e8993bfac0158db1c351929a3c69419a30e39fb95713"
        network_mode = "host"
        args = [
          "-storageDataPath=/volume/victoriametrics-data",
          "-retentionPeriod=24",
          "-httpListenAddr=10.17.0.12:9091",
        ]
      }

      volume_mount {
        volume = "v_2"
        destination = "/volume"
      }
    }
  }

  group "monitoring-2" {
    count = 1
    shutdown_delay = "60s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-mon-server-d-default}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "prom" {
        static = 9090
        host_network = "lan"
      }
      port "vm" {
        static = 9091
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "mon-default"
      read_only = false
    }

    volume "v_2" {
      type = "host"
      source = "mon-default"
      read_only = false
    }

    service {
      name = "epl-mon-default-prometheus"
      port = "prom"
      address = "${meta.private_ip}"
      check {
        type = "http"
        port = "prom"
        path = "/-/healthy"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-mon-default-victoriametrics"
      port = "vm"
      address = "${meta.private_ip}"
      check {
        type = "http"
        port = "vm"
        path = "/health"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "mon-default-prometheus" {
      driver = "docker"
      resources {
        memory = 768
        memory_max = 896
      }
      config {
        image = "bitnami/prometheus@sha256:fa5181c0bb2843c181bdbc97571a7938f7ee2778d198b4be3b4f2ad43297e8a7"
        network_mode = "host"
        args = [
          "--web.listen-address=10.17.0.13:9090",
          "--config.file=/secrets/prometheus.yml",
          "--storage.tsdb.path=/volume/prometheus-data",
          "--storage.tsdb.retention.time=15d",
          "--web.console.libraries=/opt/bitnami/prometheus/conf/console_libraries",
          "--web.console.templates=/opt/bitnami/prometheus/conf/consoles",
          "--web.external-url=https://adm-prometheus-default.epl-infra.net",
        ]
      }

      volume_mount {
        volume = "v_1"
        destination = "/volume"
      }

      template {
        destination = "secrets/alert_rules.yml"
        perms = "644"
        data = <<EOL
groups:
- name: Default
  rules:

  - alert: FilesystemSpaceLow
    expr: round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100           / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
    for: 5m
    labels:
      severity: 50
    annotations:
      description: "Filesystem {{"{{"}} $labels.device {{"}}"}} at {{"{{"}} $labels.instance {{"}}"}} has less than 20% disk space remaining"
EOL
      }

      template {
        destination = "secrets/prometheus.yml"
        perms = "644"
        data = <<EOL

global:
  scrape_interval: 15s
  evaluation_interval: 15s

remote_write:
  - url: http://10.17.0.13:9091/api/v1/write


rule_files:
  - /secrets/alert_rules.yml


scrape_configs:
  - job_name: "prometheus"
    static_configs:
      - targets: ["10.17.0.13:9090"]

  - job_name: 'consul'
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        tags:
          - epl-mon-default
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance
      - source_labels: [__meta_consul_service_metadata_metrics_path]
        regex: ^(/.+)$
        action: replace
        target_label: __metrics_path__


  - job_name: 'consul-nomad'
    scheme: https
    tls_config:
      insecure_skip_verify: true
    metrics_path: /v1/metrics
    params:
      format: ['prometheus']
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        services:
          - nomad-clients
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance

  - job_name: 'consul-vault'
    scheme: https
    tls_config:
      insecure_skip_verify: true
    metrics_path: /v1/sys/metrics
    params:
      format: ['prometheus']
    consul_sd_configs:
      - server: '127.0.0.1:8500'
        services:
          - vault
    relabel_configs:
      - source_labels: [__meta_consul_service]
        target_label: job
      - source_labels: [__meta_consul_node, __meta_consul_service_port]
        separator: ':'
        target_label: instance



alerting:
  alertmanagers:
    - static_configs:
      - targets:
        - 10.17.0.10:9092
        - 10.17.0.12:9092
        - 10.17.0.13:9092
EOL
      }
    }

    task "mon-default-vm" {
      driver = "docker"
      resources {
        memory = 256
        memory_max = 384
      }
      config {
        image = "victoriametrics/victoria-metrics@sha256:8b57b33434c062bfdfc2e8993bfac0158db1c351929a3c69419a30e39fb95713"
        network_mode = "host"
        args = [
          "-storageDataPath=/volume/victoriametrics-data",
          "-retentionPeriod=24",
          "-httpListenAddr=10.17.0.13:9091",
        ]
      }

      volume_mount {
        volume = "v_2"
        destination = "/volume"
      }
    }
  }

}
