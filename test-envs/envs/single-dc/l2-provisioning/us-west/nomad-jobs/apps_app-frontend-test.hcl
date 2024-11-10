job "app-frontend-test" {
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
        static = 7437
        host_network = "lan"
      }
    }

    service {
      name = "epl-app-frontend-test"
      port = "app"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "app"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "app-frontend-test" {
      driver = "docker"
      resources {
        memory = 16
        memory_max = 144
      }
      env {
        EPL_DEPLOYMENT_NAME = "frontend-test"
        EPL_ENDPOINT_MAPPING = "{first:'/muh/app/'}"
        EPL_EXTLINKS_MAPPING = "{be_hello_world:'https://www.epl-infra.net/muh/app/'}"
        EPL_EXTPAGES_MAPPING = "{fe_all_arg:'https://www.epl-infra.net/other/'}"
        EPL_HTTP_SOCKET = "${meta.private_ip}:7437"
      }
      config {
        image = "@@EPL_APP_IMAGE_x86_64:frontend-test@@"
        network_mode = "host"
        labels {
          epl_loki_cluster = "main"
        }
      }
    }
  }

}
