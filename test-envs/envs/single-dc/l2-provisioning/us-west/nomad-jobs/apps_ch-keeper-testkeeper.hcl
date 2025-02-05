job "ch-keeper-testkeeper" {
  type = "service"
  namespace = "apps"
  region = "us-west"
  datacenters = ["dc1"]
  update {
    auto_revert = false
    max_parallel = 1
    health_check = "checks"
    min_healthy_time = "30s"
    stagger = "30s"
    healthy_deadline = "300s"
    progress_deadline = "600s"
  }

  group "chk-1" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-ch-keeper-server-a-testkeeper}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "prom_port" {
        static = 9183
        host_network = "lan"
      }
      port "raft_port" {
        static = 9182
        host_network = "lan"
      }
      port "tcp_port" {
        static = 9181
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "ch-keeper"
      read_only = false
    }

    service {
      name = "epl-ch-keeper-testkeeper"
      port = "prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "chk-testkeeper-1" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "clickhouse/clickhouse-server@sha256:2e6587b81a267c6152cf2112c3532516424d3eaa36f1b150d5b8847c0e3d5b01"
        network_mode = "host"
        entrypoint = [
          "/usr/bin/clickhouse-keeper",
        ]
        args = [
          "--config-file=/local/keeper_config.xml",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/clickhouse"
      }

      template {
        destination = "local/keeper_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>10.17.0.10</listen_host>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>9183</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <keeper_server>
        <tcp_port>9181</tcp_port>
        <server_id>1</server_id>
        <log_storage_path>/var/lib/clickhouse/coordination/log</log_storage_path>
        <snapshot_storage_path>/var/lib/clickhouse/coordination/snapshots</snapshot_storage_path>
        <enable_reconfiguration>true</enable_reconfiguration>

        <coordination_settings>
            <operation_timeout_ms>10000</operation_timeout_ms>
            <session_timeout_ms>30000</session_timeout_ms>
            <raft_logs_level>information</raft_logs_level>
        </coordination_settings>

        <raft_configuration>

            <server>
                <id>1</id>
                <hostname>10.17.0.10</hostname>
                <port>9182</port>
            </server>

            <server>
                <id>2</id>
                <hostname>10.17.0.11</hostname>
                <port>9182</port>
            </server>

            <server>
                <id>3</id>
                <hostname>10.17.0.13</hostname>
                <port>9182</port>
            </server>

        </raft_configuration>
    </keeper_server>
</clickhouse>
EOL
      }
    }
  }

  group "chk-2" {
    count = 1
    shutdown_delay = "15s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-ch-keeper-server-b-testkeeper}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "prom_port" {
        static = 9183
        host_network = "lan"
      }
      port "raft_port" {
        static = 9182
        host_network = "lan"
      }
      port "tcp_port" {
        static = 9181
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "ch-keeper"
      read_only = false
    }

    service {
      name = "epl-ch-keeper-testkeeper"
      port = "prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "chk-testkeeper-2" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "clickhouse/clickhouse-server@sha256:2e6587b81a267c6152cf2112c3532516424d3eaa36f1b150d5b8847c0e3d5b01"
        network_mode = "host"
        entrypoint = [
          "/usr/bin/clickhouse-keeper",
        ]
        args = [
          "--config-file=/local/keeper_config.xml",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/clickhouse"
      }

      template {
        destination = "local/keeper_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>10.17.0.11</listen_host>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>9183</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <keeper_server>
        <tcp_port>9181</tcp_port>
        <server_id>2</server_id>
        <log_storage_path>/var/lib/clickhouse/coordination/log</log_storage_path>
        <snapshot_storage_path>/var/lib/clickhouse/coordination/snapshots</snapshot_storage_path>
        <enable_reconfiguration>true</enable_reconfiguration>

        <coordination_settings>
            <operation_timeout_ms>10000</operation_timeout_ms>
            <session_timeout_ms>30000</session_timeout_ms>
            <raft_logs_level>information</raft_logs_level>
        </coordination_settings>

        <raft_configuration>

            <server>
                <id>1</id>
                <hostname>10.17.0.10</hostname>
                <port>9182</port>
            </server>

            <server>
                <id>2</id>
                <hostname>10.17.0.11</hostname>
                <port>9182</port>
            </server>

            <server>
                <id>3</id>
                <hostname>10.17.0.13</hostname>
                <port>9182</port>
            </server>

        </raft_configuration>
    </keeper_server>
</clickhouse>
EOL
      }
    }
  }

  group "chk-3" {
    count = 1
    shutdown_delay = "30s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-ch-keeper-server-d-testkeeper}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "prom_port" {
        static = 9183
        host_network = "lan"
      }
      port "raft_port" {
        static = 9182
        host_network = "lan"
      }
      port "tcp_port" {
        static = 9181
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "ch-keeper"
      read_only = false
    }

    service {
      name = "epl-ch-keeper-testkeeper"
      port = "prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "chk-testkeeper-3" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "clickhouse/clickhouse-server@sha256:2e6587b81a267c6152cf2112c3532516424d3eaa36f1b150d5b8847c0e3d5b01"
        network_mode = "host"
        entrypoint = [
          "/usr/bin/clickhouse-keeper",
        ]
        args = [
          "--config-file=/local/keeper_config.xml",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/clickhouse"
      }

      template {
        destination = "local/keeper_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>10.17.0.13</listen_host>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>9183</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <keeper_server>
        <tcp_port>9181</tcp_port>
        <server_id>3</server_id>
        <log_storage_path>/var/lib/clickhouse/coordination/log</log_storage_path>
        <snapshot_storage_path>/var/lib/clickhouse/coordination/snapshots</snapshot_storage_path>
        <enable_reconfiguration>true</enable_reconfiguration>

        <coordination_settings>
            <operation_timeout_ms>10000</operation_timeout_ms>
            <session_timeout_ms>30000</session_timeout_ms>
            <raft_logs_level>information</raft_logs_level>
        </coordination_settings>

        <raft_configuration>

            <server>
                <id>1</id>
                <hostname>10.17.0.10</hostname>
                <port>9182</port>
            </server>

            <server>
                <id>2</id>
                <hostname>10.17.0.11</hostname>
                <port>9182</port>
            </server>

            <server>
                <id>3</id>
                <hostname>10.17.0.13</hostname>
                <port>9182</port>
            </server>

        </raft_configuration>
    </keeper_server>
</clickhouse>
EOL
      }
    }
  }

}
