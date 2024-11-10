job "docker-registry" {
  type = "system"
  namespace = "epl"
  region = "us-west"
  datacenters = ["dc1"]

  vault {
    policies = ["epl-docker-registry"]
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

  group "registry" {
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
      port "reg" {
        static = 5000
        host_network = "lan"
      }
    }

    service {
      name = "epl-docker-registry"
      port = "reg"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "reg"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "docker-registry" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      config {
        image = "registry@sha256:ce14a6258f37702ff3cd92232a6f5b81ace542d9f1631966999e9f7c1ee6ddba"
        network_mode = "host"
        args = [
          "/secrets/config.yml",
        ]
      }

      template {
        destination = "secrets/config.yml"
        perms = "644"
        data = <<EOL

version: 0.1
log:
  level: info
  formatter: text
  fields:
    service: registry
    environment: staging
loglevel: info
storage:
  s3:
    accesskey: docker_registry
    secretkey: {{ with secret "epl/data/docker-registry" }}{{ .Data.data.minio_bucket_password }}{{ end }}
    region: us-east-1
    regionendpoint: http://epl-minio-global.service.consul:9002
    bucket: docker
    encrypt: false
    secure: false
    v4auth: true
    chunksize: 5242880
    rootdirectory: /
  delete:
    enabled: true
  maintenance:
    uploadpurging:
      enabled: false
      age: 5040h
      interval: 24h
      dryrun: false
    readonly:
      enabled: false
  cache:
    blobdescriptor: inmemory
    blobdescriptorsize: 10000
http:
  addr: {{ env "meta.private_ip" }}:5000
  host: http://epl-docker-registry.service.consul:5000
EOL
      }

      template {
        destination = "secrets/env"
        perms = "644"
        env = true
        data = <<EOL
REGISTRY_HTTP_SECRET={{ with secret "epl/data/docker-registry" }}{{ .Data.data.registry_http_secret }}{{ end }}
EOL
      }
    }
  }

}
