job "pg-testdb" {
  type = "service"
  namespace = "epl"
  region = "us-west"
  datacenters = ["dc1"]

  vault {
    policies = ["epl-pg-testdb"]
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

  group "pg-1" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-pg-server-a-testdb}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "haproxy_metrics" {
        static = 5436
        host_network = "lan"
      }
      port "patroni" {
        static = 5435
        host_network = "lan"
      }
      port "pg_exporter" {
        static = 5437
        host_network = "lan"
      }
      port "pg_ha_master" {
        static = 5433
        host_network = "lan"
      }
      port "pg_ha_slave" {
        static = 5434
        host_network = "lan"
      }
      port "pg_main" {
        static = 5432
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "pgtest1"
      read_only = false
    }

    service {
      name = "epl-pg-testdb-hap-exp"
      port = "haproxy_metrics"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "haproxy_metrics"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-pg-testdb-pg-exp"
      port = "pg_exporter"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "pg_exporter"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "pg-testdb-1-haproxy" {
      driver = "docker"
      resources {
        memory = 32
        memory_max = 160
      }
      config {
        image = "haproxy@sha256:ebdd23975d25d1fb360ee54f81af26ca9fff6fa05516d43980c99ee5a88ff56e"
        network_mode = "host"
        entrypoint = [
          "/usr/local/sbin/haproxy",
        ]
        args = [
          "-W",
          "-f",
          "/secrets/haproxy.cfg",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/haproxy.cfg"
        perms = "644"
        data = <<EOL

global
  maxconn         100
  ulimit-n        300
  nbthread        4

defaults
    log global
    retries 2
    timeout client 30m
    timeout connect 4s
    timeout server 30m
    timeout check 5s

frontend stats
  mode http
  bind {{ env "meta.private_ip" }}:5436
  http-request use-service prometheus-exporter if { path /metrics }
  http-request return status 200 content-type text/plain string ok if { path /health }
  stats enable
  stats uri /stats
  stats refresh 10s

resolvers consul
  nameserver consul 127.0.0.1:8600
  accepted_payload_size 8192
  hold valid 5s

listen postgres_write
    bind {{ env "meta.private_ip" }}:5433
    mode tcp
    default-server inter 3s fall 3 rise 2 on-marked-down shutdown-sessions
    server-template pg-master 1 master.epl-pg-testdb.service.consul:5432 resolvers consul resolve-prefer ipv4 check

listen postgres_read
    bind {{ env "meta.private_ip" }}:5434
    mode tcp
    default-server inter 3s fall 4 rise 2 on-marked-down shutdown-sessions
    server-template pg-master 4 replica.epl-pg-testdb.service.consul:5432 resolvers consul resolve-prefer ipv4 check
EOL
      }
    }

    task "pg-testdb-1-patroni" {
      driver = "docker"
      resources {
        memory = 464
        memory_max = 592
      }
      env {
        PATRONICTL_CONFIG_FILE = "/secrets/patroni.yml"
      }
      config {
        image = "cultleader777/patroni-pg@sha256:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0c"
        network_mode = "host"
        entrypoint = [
          "/usr/local/bin/patroni",
        ]
        args = [
          "/secrets/patroni.yml",
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
        destination = "secrets/patroni.yml"
        perms = "644"
        data = <<EOL

scope: epl-pg-testdb
name: instance-1
namespace: /epl-patroni

restapi:
  listen: {{ env "meta.private_ip" }}:5435
  connect_address: {{ env "meta.private_ip" }}:5435

consul:
  host: 127.0.0.1
  port: 8500
  scheme: http
  register_service: true
  token: {{ with secret "epl/data/pg/testdb" }}{{ .Data.data.consul_token }}{{ end }}

bootstrap:
  # this section will be written into Etcd:/<namespace>/<scope>/config after initializing new cluster
  # and all other cluster members will use it as a `global configuration`
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576
    postgresql:
      use_pg_rewind: true

  initdb:
  - encoding: UTF8
  - data-checksums

  pg_hba:
  - host replication replicator 10.0.0.0/8 md5
  - host all all 10.0.0.0/8 md5

  users:
    admin:
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_admin_password }}{{ end }}"
      options:
        - createrole
        - createdb
    replicator:
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_replicator_password }}{{ end }}"
      options:
        - replication

postgresql:
  parameters:
    work_mem: 16MB
    shared_buffers: 256MB
    maintenance_work_mem: 128MB
    max_connections: 400
  listen: {{ env "meta.private_ip" }}:5432
  connect_address: {{ env "meta.private_ip" }}:5432
  data_dir: /data/postgresql
  pgpass: /secrets/pgpass
  authentication:
    replication:
      username: replicator
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_replicator_password }}{{ end }}"
    superuser:
      username: postgres
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_superuser_password }}{{ end }}"
    rewind:
      username: rewind_user
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_rewind_password }}{{ end }}"
  pg_hba:
  - local all all trust
  - host replication replicator 10.0.0.0/8 md5
  - host all all 10.0.0.0/8 md5

tags:
    nofailover: false
    noloadbalance: false
    clonefrom: false
    nosync: false
EOL
      }
    }

    task "pg-testdb-1-prom-exp" {
      driver = "docker"
      resources {
        memory = 32
        memory_max = 160
      }
      env {
        DATA_SOURCE_URI = "${meta.private_ip}:5432/postgres?sslmode=disable"
        DATA_SOURCE_USER = "postgres_exporter"
      }
      config {
        image = "quay.io/prometheuscommunity/postgres-exporter@sha256:f34d50a64a4d558ad118ffc73be45a359ac8f30b8daba4b241458bcb9f94e254"
        network_mode = "host"
        args = [
          "--web.listen-address=${meta.private_ip}:5437",
          "--config.file=/dev/null",
          "--collector.database",
          "--collector.bgwriter",
          "--collector.replication_slot",
          "--auto-discover-databases",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/exporter.env"
        perms = "644"
        env = true
        data = <<EOL
DATA_SOURCE_PASS={{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_exporter_password }}{{ end }}
EOL
      }
    }
  }

  group "pg-2" {
    count = 1
    shutdown_delay = "60s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-pg-server-b-testdb}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "haproxy_metrics" {
        static = 5436
        host_network = "lan"
      }
      port "patroni" {
        static = 5435
        host_network = "lan"
      }
      port "pg_exporter" {
        static = 5437
        host_network = "lan"
      }
      port "pg_ha_master" {
        static = 5433
        host_network = "lan"
      }
      port "pg_ha_slave" {
        static = 5434
        host_network = "lan"
      }
      port "pg_main" {
        static = 5432
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "pgtest1"
      read_only = false
    }

    service {
      name = "epl-pg-testdb-hap-exp"
      port = "haproxy_metrics"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "haproxy_metrics"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-pg-testdb-pg-exp"
      port = "pg_exporter"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "pg_exporter"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "pg-testdb-2-haproxy" {
      driver = "docker"
      resources {
        memory = 32
        memory_max = 160
      }
      config {
        image = "haproxy@sha256:ebdd23975d25d1fb360ee54f81af26ca9fff6fa05516d43980c99ee5a88ff56e"
        network_mode = "host"
        entrypoint = [
          "/usr/local/sbin/haproxy",
        ]
        args = [
          "-W",
          "-f",
          "/secrets/haproxy.cfg",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/haproxy.cfg"
        perms = "644"
        data = <<EOL

global
  maxconn         100
  ulimit-n        300
  nbthread        4

defaults
    log global
    retries 2
    timeout client 30m
    timeout connect 4s
    timeout server 30m
    timeout check 5s

frontend stats
  mode http
  bind {{ env "meta.private_ip" }}:5436
  http-request use-service prometheus-exporter if { path /metrics }
  http-request return status 200 content-type text/plain string ok if { path /health }
  stats enable
  stats uri /stats
  stats refresh 10s

resolvers consul
  nameserver consul 127.0.0.1:8600
  accepted_payload_size 8192
  hold valid 5s

listen postgres_write
    bind {{ env "meta.private_ip" }}:5433
    mode tcp
    default-server inter 3s fall 3 rise 2 on-marked-down shutdown-sessions
    server-template pg-master 1 master.epl-pg-testdb.service.consul:5432 resolvers consul resolve-prefer ipv4 check

listen postgres_read
    bind {{ env "meta.private_ip" }}:5434
    mode tcp
    default-server inter 3s fall 4 rise 2 on-marked-down shutdown-sessions
    server-template pg-master 4 replica.epl-pg-testdb.service.consul:5432 resolvers consul resolve-prefer ipv4 check
EOL
      }
    }

    task "pg-testdb-2-patroni" {
      driver = "docker"
      resources {
        memory = 464
        memory_max = 592
      }
      env {
        PATRONICTL_CONFIG_FILE = "/secrets/patroni.yml"
      }
      config {
        image = "cultleader777/patroni-pg@sha256:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0c"
        network_mode = "host"
        entrypoint = [
          "/usr/local/bin/patroni",
        ]
        args = [
          "/secrets/patroni.yml",
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
        destination = "secrets/patroni.yml"
        perms = "644"
        data = <<EOL

scope: epl-pg-testdb
name: instance-2
namespace: /epl-patroni

restapi:
  listen: {{ env "meta.private_ip" }}:5435
  connect_address: {{ env "meta.private_ip" }}:5435

consul:
  host: 127.0.0.1
  port: 8500
  scheme: http
  register_service: true
  token: {{ with secret "epl/data/pg/testdb" }}{{ .Data.data.consul_token }}{{ end }}

bootstrap:
  # this section will be written into Etcd:/<namespace>/<scope>/config after initializing new cluster
  # and all other cluster members will use it as a `global configuration`
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576
    postgresql:
      use_pg_rewind: true

  initdb:
  - encoding: UTF8
  - data-checksums

  pg_hba:
  - host replication replicator 10.0.0.0/8 md5
  - host all all 10.0.0.0/8 md5

  users:
    admin:
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_admin_password }}{{ end }}"
      options:
        - createrole
        - createdb
    replicator:
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_replicator_password }}{{ end }}"
      options:
        - replication

postgresql:
  parameters:
    work_mem: 16MB
    shared_buffers: 256MB
    maintenance_work_mem: 128MB
    max_connections: 400
  listen: {{ env "meta.private_ip" }}:5432
  connect_address: {{ env "meta.private_ip" }}:5432
  data_dir: /data/postgresql
  pgpass: /secrets/pgpass
  authentication:
    replication:
      username: replicator
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_replicator_password }}{{ end }}"
    superuser:
      username: postgres
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_superuser_password }}{{ end }}"
    rewind:
      username: rewind_user
      password: "{{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_rewind_password }}{{ end }}"
  pg_hba:
  - local all all trust
  - host replication replicator 10.0.0.0/8 md5
  - host all all 10.0.0.0/8 md5

tags:
    nofailover: false
    noloadbalance: false
    clonefrom: false
    nosync: false
EOL
      }
    }

    task "pg-testdb-2-prom-exp" {
      driver = "docker"
      resources {
        memory = 32
        memory_max = 160
      }
      env {
        DATA_SOURCE_URI = "${meta.private_ip}:5432/postgres?sslmode=disable"
        DATA_SOURCE_USER = "postgres_exporter"
      }
      config {
        image = "quay.io/prometheuscommunity/postgres-exporter@sha256:f34d50a64a4d558ad118ffc73be45a359ac8f30b8daba4b241458bcb9f94e254"
        network_mode = "host"
        args = [
          "--web.listen-address=${meta.private_ip}:5437",
          "--config.file=/dev/null",
          "--collector.database",
          "--collector.bgwriter",
          "--collector.replication_slot",
          "--auto-discover-databases",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/exporter.env"
        perms = "644"
        env = true
        data = <<EOL
DATA_SOURCE_PASS={{ with secret "epl/data/pg/testdb" }}{{ .Data.data.pg_exporter_password }}{{ end }}
EOL
      }
    }
  }

}
