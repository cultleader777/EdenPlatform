job "nats-main-nats" {
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

  group "nats-1" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-nats-server-b-main-nats}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "nats_client" {
        static = 4222
        host_network = "lan"
      }
      port "nats_cluster" {
        static = 4223
        host_network = "lan"
      }
      port "nats_http_mon" {
        static = 4224
        host_network = "lan"
      }
      port "nats_prom_port" {
        static = 4225
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "nats1"
      read_only = false
    }

    service {
      name = "epl-nats-main-nats"
      port = "nats_cluster"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "nats_cluster"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-nats-main-nats-prom"
      port = "nats_prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "nats_prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "nats-main-nats-daemon" {
      driver = "docker"
      resources {
        memory = 256
        memory_max = 384
      }
      env {
        GOMEMLIMIT = "256MiB"
      }
      config {
        image = "nats@sha256:110974826bc2099b037c3324e47e5bdd743472bff304920ab92c881646d656e3"
        network_mode = "host"
        args = [
          "--name=main-nats-server-b",
          "--jetstream",
          "--config=/local/js.conf",
          "--port=4222",
          "--http_port=4224",
          "--addr=${meta.private_ip}",
          "--cluster_name=main-nats",
          "--cluster=nats://${meta.private_ip}:4223",
          "--cluster_advertise=${meta.private_ip}:4223",
          "--routes=nats://10.17.0.11:4223,nats://10.17.0.12:4223,nats://10.17.0.13:4223",
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
        destination = "local/js.conf"
        perms = "644"
        data = <<EOL

jetstream {
  store_dir: /data/nats
  max_file_store: 1125899906842624
}
EOL
      }
    }

    task "nats-main-nats-prom-exp" {
      driver = "docker"
      resources {
        memory = 32
        memory_max = 160
      }
      config {
        image = "natsio/prometheus-nats-exporter@sha256:26c826662ac8424597cc9bdf89ea5b606eb66e3c11db9b1215c27d2076bbb01b"
        network_mode = "host"
        args = [
          "-addr=${meta.private_ip}",
          "-accstatz",
          "-connz_detailed",
          "-healthz",
          "-gatewayz",
          "-leafz",
          "-routez",
          "-subz",
          "-varz",
          "-jsz=all",
          "-use_internal_server_id",
          "-use_internal_server_name",
          "-p=4225",
          "http://${meta.private_ip}:4224",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }
    }
  }

  group "nats-2" {
    count = 1
    shutdown_delay = "60s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-nats-server-c-main-nats}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "nats_client" {
        static = 4222
        host_network = "lan"
      }
      port "nats_cluster" {
        static = 4223
        host_network = "lan"
      }
      port "nats_http_mon" {
        static = 4224
        host_network = "lan"
      }
      port "nats_prom_port" {
        static = 4225
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "nats1"
      read_only = false
    }

    service {
      name = "epl-nats-main-nats"
      port = "nats_cluster"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "nats_cluster"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-nats-main-nats-prom"
      port = "nats_prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "nats_prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "nats-main-nats-daemon" {
      driver = "docker"
      resources {
        memory = 256
        memory_max = 384
      }
      env {
        GOMEMLIMIT = "256MiB"
      }
      config {
        image = "nats@sha256:110974826bc2099b037c3324e47e5bdd743472bff304920ab92c881646d656e3"
        network_mode = "host"
        args = [
          "--name=main-nats-server-c",
          "--jetstream",
          "--config=/local/js.conf",
          "--port=4222",
          "--http_port=4224",
          "--addr=${meta.private_ip}",
          "--cluster_name=main-nats",
          "--cluster=nats://${meta.private_ip}:4223",
          "--cluster_advertise=${meta.private_ip}:4223",
          "--routes=nats://10.17.0.11:4223,nats://10.17.0.12:4223,nats://10.17.0.13:4223",
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
        destination = "local/js.conf"
        perms = "644"
        data = <<EOL

jetstream {
  store_dir: /data/nats
  max_file_store: 1125899906842624
}
EOL
      }
    }

    task "nats-main-nats-prom-exp" {
      driver = "docker"
      resources {
        memory = 32
        memory_max = 160
      }
      config {
        image = "natsio/prometheus-nats-exporter@sha256:26c826662ac8424597cc9bdf89ea5b606eb66e3c11db9b1215c27d2076bbb01b"
        network_mode = "host"
        args = [
          "-addr=${meta.private_ip}",
          "-accstatz",
          "-connz_detailed",
          "-healthz",
          "-gatewayz",
          "-leafz",
          "-routez",
          "-subz",
          "-varz",
          "-jsz=all",
          "-use_internal_server_id",
          "-use_internal_server_name",
          "-p=4225",
          "http://${meta.private_ip}:4224",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }
    }
  }

  group "nats-3" {
    count = 1
    shutdown_delay = "120s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-nats-server-d-main-nats}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "nats_client" {
        static = 4222
        host_network = "lan"
      }
      port "nats_cluster" {
        static = 4223
        host_network = "lan"
      }
      port "nats_http_mon" {
        static = 4224
        host_network = "lan"
      }
      port "nats_prom_port" {
        static = 4225
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "nats1"
      read_only = false
    }

    service {
      name = "epl-nats-main-nats"
      port = "nats_cluster"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "nats_cluster"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-nats-main-nats-prom"
      port = "nats_prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "nats_prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "nats-main-nats-daemon" {
      driver = "docker"
      resources {
        memory = 256
        memory_max = 384
      }
      env {
        GOMEMLIMIT = "256MiB"
      }
      config {
        image = "nats@sha256:110974826bc2099b037c3324e47e5bdd743472bff304920ab92c881646d656e3"
        network_mode = "host"
        args = [
          "--name=main-nats-server-d",
          "--jetstream",
          "--config=/local/js.conf",
          "--port=4222",
          "--http_port=4224",
          "--addr=${meta.private_ip}",
          "--cluster_name=main-nats",
          "--cluster=nats://${meta.private_ip}:4223",
          "--cluster_advertise=${meta.private_ip}:4223",
          "--routes=nats://10.17.0.11:4223,nats://10.17.0.12:4223,nats://10.17.0.13:4223",
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
        destination = "local/js.conf"
        perms = "644"
        data = <<EOL

jetstream {
  store_dir: /data/nats
  max_file_store: 1125899906842624
}
EOL
      }
    }

    task "nats-main-nats-prom-exp" {
      driver = "docker"
      resources {
        memory = 32
        memory_max = 160
      }
      config {
        image = "natsio/prometheus-nats-exporter@sha256:26c826662ac8424597cc9bdf89ea5b606eb66e3c11db9b1215c27d2076bbb01b"
        network_mode = "host"
        args = [
          "-addr=${meta.private_ip}",
          "-accstatz",
          "-connz_detailed",
          "-healthz",
          "-gatewayz",
          "-leafz",
          "-routez",
          "-subz",
          "-varz",
          "-jsz=all",
          "-use_internal_server_id",
          "-use_internal_server_name",
          "-p=4225",
          "http://${meta.private_ip}:4224",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }
    }
  }

}
