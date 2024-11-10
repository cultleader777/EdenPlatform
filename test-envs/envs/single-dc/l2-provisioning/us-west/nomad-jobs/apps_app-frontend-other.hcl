job "app-frontend-other" {
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

  group "app" {
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
      port "app" {
        static = 7438
        host_network = "lan"
      }
    }

    service {
      name = "epl-app-frontend-other"
      port = "app"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "app"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "app-frontend-other" {
      driver = "docker"
      resources {
        memory = 16
        memory_max = 144
      }
      env {
        EPL_DEPLOYMENT_NAME = "frontend-other"
        EPL_ENDPOINT_MAPPING = "{}"
        EPL_EXTLINKS_MAPPING = "{}"
        EPL_EXTPAGES_MAPPING = "{}"
        EPL_HTTP_SOCKET = "${meta.private_ip}:7438"
        OVERRIDE_ROOT_PATH = "/other"
      }
      config {
        image = "@@EPL_APP_IMAGE_x86_64:frontend-other@@"
        network_mode = "host"
        labels {
          epl_loki_cluster = "main"
        }
      }
    }
  }

}
