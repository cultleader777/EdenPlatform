#!/bin/sh
umask 0077
mkdir -p /var/lib/epl-l1-prov
mkdir -p /var/log/epl-l1-prov
find /var/log/epl-l1-prov/*.log -type f -ctime +7 -exec rm -rf {} \; || true
grep -q epl-prov /etc/group && chgrp epl-prov /var/log/epl-l1-prov || true
chmod 750 /var/log/epl-l1-prov
echo '

CREATE TABLE IF NOT EXISTS l1_provisionings (
  provisioning_id INTEGER PRIMARY KEY,
  is_finished INTEGER DEFAULT 0,
  exit_code INTEGER DEFAULT 0,
  time_started TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  time_ended TIMESTAMP DEFAULT 0
);

-- for checking if l1 provisionings exist now
CREATE INDEX IF NOT EXISTS l1_provisionings_is_finished_index ON l1_provisionings (is_finished);

CREATE TABLE IF NOT EXISTS consul_l1_payloads (
  consul_modify_index INT PRIMARY KEY
);

' | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite

HIGHER_ID=$( echo "
    SELECT provisioning_id, 'found_higher'
    FROM l1_provisionings
    WHERE provisioning_id > L1_EPL_PROVISIONING_ID
    LIMIT 1
" | sqlite3 -csv /var/lib/epl-l1-prov/provisionings.sqlite | grep 'found_higher' || true )

if [ -n "$HIGHER_ID" ]
then
    HIGHER_ID_CUT=$( echo "$HIGHER_ID" | awk -F, '{print $1}' )
    echo "This provisioning id is L1_EPL_PROVISIONING_ID, found higher l1 provisioning id $HIGHER_ID_CUT in the database, refusing to evaluate"
    exit 7
fi

cat > /run/epl-l1-prov <<'ThisIsEplProvL1Script'
#!/bin/sh
set -e
function epl_l1_track_state {
    EPL_PROV_EXIT_CODE=$1

    if [ -d /var/lib/node_exporter ]
    then
        echo "
epl_l1_provisioning_id L1_EPL_PROVISIONING_ID
epl_l1_provisioning_status $EPL_PROV_EXIT_CODE
"      > /var/lib/node_exporter/epl_l1_provisioning.prom.tmp
        chmod 644 /var/lib/node_exporter/epl_l1_provisioning.prom.tmp
        mv -f /var/lib/node_exporter/epl_l1_provisioning.prom.tmp /var/lib/node_exporter/epl_l1_provisioning.prom
    fi
}
function trap_exit {
    EXIT_CODE=$?
    rm -f /run/epl-l1-provisioning.lock

    epl_l1_track_state $EXIT_CODE

    echo "
      UPDATE l1_provisionings
      SET exit_code = $EXIT_CODE,
          time_ended = CURRENT_TIMESTAMP,
          is_finished = 1
      WHERE provisioning_id = L1_EPL_PROVISIONING_ID
    " | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite
}
function generate_l1_secrets() {
rm -f /run/tmpsec-*
mkdir -p /run/keys
chmod 755 /run/keys
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-config.json START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
{
  "acl": {
    "default_policy": "deny",
    "enable_token_persistence": true,
    "enabled": true,
    "tokens": {
      "agent": "5fcc4617-cd1f-40ac-afcf-92ba8f3a724d",
      "default": "027dc9c1-2d75-4534-8dc3-4f42b49f101a",
      "initial_management": "23040270-1e3a-4308-aa44-7d94263d7a35"
    }
  },
  "addresses": {
    "dns": "127.0.0.1",
    "grpc": "127.0.0.1",
    "http": "127.0.0.1",
    "https": "10.17.0.10"
  },
  "advertise_addr": "10.17.0.10",
  "advertise_addr_wan": "10.17.0.10",
  "auto_encrypt": {
    "allow_tls": true
  },
  "bind_addr": "10.17.0.10",
  "bootstrap": false,
  "bootstrap_expect": 3,
  "client_addr": "127.0.0.1",
  "connect": {
    "enabled": true
  },
  "data_dir": "/var/lib/consul",
  "datacenter": "us-west",
  "disable_update_check": false,
  "domain": "consul",
  "enable_local_script_checks": false,
  "enable_script_checks": false,
  "encrypt": "ym3++1rmzzwsffgh8EnMkSDkq9wX6FPG4b8dE+lRQ9I=",
  "encrypt_verify_incoming": true,
  "encrypt_verify_outgoing": true,
  "log_level": "INFO",
  "log_rotate_bytes": 0,
  "log_rotate_duration": "24h",
  "log_rotate_max_files": 0,
  "node_name": "server-a",
  "performance": {
    "leave_drain_time": "5s",
    "raft_multiplier": 1,
    "rpc_hold_timeout": "7s"
  },
  "ports": {
    "dns": 8600,
    "grpc": -1,
    "http": 8500,
    "https": 8501,
    "serf_lan": 8301,
    "serf_wan": 8302,
    "server": 8300
  },
  "raft_protocol": 3,
  "retry_interval": "30s",
  "retry_join": [
    "10.17.0.10",
    "10.17.0.11",
    "10.17.0.12"
  ],
  "retry_join_wan": [],
  "retry_max": 0,
  "server": true,
  "tls": {
    "defaults": {
      "ca_file": "/run/keys/consul-tls-ca-cert.pem",
      "cert_file": "/run/keys/consul-tls-server-cert.pem",
      "key_file": "/run/keys/consul-tls-server-pkey.pem",
      "tls_min_version": "TLSv1_2",
      "verify_incoming": false,
      "verify_outgoing": true
    },
    "https": {
      "verify_incoming": false
    },
    "internal_rpc": {
      "verify_incoming": false,
      "verify_server_hostname": true
    }
  },
  "translate_wan_addrs": false,
  "ui_config": {
    "enabled": true
  }
}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-config.json END
if id -u consul &>/dev/null && id -g consul &>/dev/null; then
  chown consul $TMP_SECRET_PATH
  chgrp consul $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-config.json || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-config.json')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-config.json
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-config.hcl START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'


name = "server-a"
region = "us-west"
datacenter = "dc1"

enable_debug = false
disable_update_check = false


bind_addr = "10.17.0.10"
advertise {
    http = "10.17.0.10:4646"
    rpc = "10.17.0.10:4647"
    serf = "10.17.0.10:4648"
}

ports {
    http = 4646
    rpc = 4647
    serf = 4648
}

consul {
    # The address to the Consul agent.
    address = "127.0.0.1:8500"
    ssl = false
    ca_file = ""
    cert_file = ""
    key_file = ""
    token = "ff525c11-adfb-4648-a2ab-1fcdd47b8cb8"
    # The service name to register the server and client with Consul.
    server_service_name = "nomad-servers"
    client_service_name = "nomad-clients"
    tags = {
    }

    # Enables automatically registering the services.
    auto_advertise = true

    # Enabling the server and client to bootstrap using Consul.
    server_auto_join = true
    client_auto_join = true
}

data_dir = "/var/lib/nomad"

log_level = "INFO"
enable_syslog = true

leave_on_terminate = true
leave_on_interrupt = false


tls {
    http = true
    rpc = true
    ca_file = "/run/keys/nomad-ca.crt"
    cert_file = "/run/keys/nomad-server.crt"
    key_file = "/run/keys/nomad-server.key"
    rpc_upgrade_mode = false
    verify_server_hostname = "true"
    verify_https_client = "false"
}


vault {
    enabled = true
    address = "https://vault.service.consul:8200"
    allow_unauthenticated = false
    create_from_role = "nomad-cluster"
    task_token_ttl = ""
    ca_file = "/run/keys/vault-ca.crt"
    ca_path = ""
    cert_file = ""
    key_file = ""
    tls_server_name = ""
    tls_skip_verify = false
    namespace = ""
    token = "hvs.CAESILpIv6E1q16pmveNHkNDyZ6e1Xj_jjQEutZIFJl9_s7jGh4KHGh2cy5JUEJEbWQ4cGFOcFpxWFlDbW01SUtMNEM"

}

client {
  enabled = true

  node_class = ""
  no_host_uuid = false

  max_kill_timeout = "3600s"

  network_speed = 0
  cpu_total_compute = 0

  gc_interval = "1m"
  gc_disk_usage_threshold = 80
  gc_inode_usage_threshold = 70
  gc_parallel_destroys = 2

  reserved {
    cpu = 0
    memory = 0
    disk = 0
  }

  network_interface = "eth0"

  meta = {
    "private_ip" = "10.17.0.10"
    "run_unassigned_workloads" = "1"
    "lock_epl-minio-server-a-global" = "1"
    "lock_epl-mon-server-a-am-default" = "1"
    "lock_epl-mon-server-a-default" = "1"
    "lock_epl-pg-server-a-testdb" = "1"
  }

  host_volume "minio-docker-a" {
    path = "/srv/volumes/minio-docker-a"
    read_only = false
  }

  host_volume "mon-am" {
    path = "/srv/volumes/mon-am"
    read_only = false
  }

  host_volume "mon-default" {
    path = "/srv/volumes/mon-default"
    read_only = false
  }

  host_volume "pgtest1" {
    path = "/srv/volumes/pgtest1"
    read_only = false
  }

  host_volume "ssl_certs" {
    path = "/run/sec_volumes/ssl_certs"
    read_only = true
  }

  host_network "lan" {
    cidr = "10.0.0.0/8"
  }

}

acl {
    enabled = true
    token_ttl = "30s"
    policy_ttl = "30s"
    replication_token = ""
}

telemetry {
    disable_hostname = "false"
    collection_interval = "1s"
    use_node_name = "false"
    publish_allocation_metrics = "true"
    publish_node_metrics = "true"
    filter_default = "true"
    prefix_filter = []
    disable_dispatched_job_summary_metrics = "false"
    statsite_address = ""
    statsd_address = ""
    datadog_address = ""
    datadog_tags = []
    prometheus_metrics = "true"
    circonus_api_token = ""
    circonus_api_app = "nomad"
    circonus_api_url = "https://api.circonus.com/v2"
    circonus_submission_interval = "10s"
    circonus_submission_url = ""
    circonus_check_id = ""
    circonus_check_force_metric_activation = "false"
    circonus_check_instance_id = ""
    circonus_check_search_tag = ""
    circonus_check_display_name = ""
    circonus_check_tags = ""
    circonus_broker_id = ""
    circonus_broker_select_tag = ""
}

plugin "docker" {
    config {

        # this might use a lot of memory if enabled, we have grafana loki
        # and this can be disabled for memory savings, but if grafana loki doesn't
        # work this is nice for debugging
        disable_log_collection = true

        extra_labels = ["*"]
        logging {
            type = "json-file"
            config {
                max-file = 3
                max-size = "30m"
            }
        }
    }
}

server {
    enabled = true

    bootstrap_expect = 3

    rejoin_after_leave = false

    enabled_schedulers = ["service", "batch", "system"]
    num_schedulers = 128

    node_gc_threshold = "24h"
    eval_gc_threshold = "1h"
    job_gc_threshold = "4h"
    deployment_gc_threshold = "1h"

    encrypt = "hmmM6KajM4gC0Fk7iEazxp2tvDketbBfpCsLih+2J1I="

    raft_protocol = 3

    default_scheduler_config {
        scheduler_algorithm             = "binpack"
        memory_oversubscription_enabled = true
        reject_job_registration         = false
        pause_eval_broker               = false

        preemption_config {
            batch_scheduler_enabled    = true
            system_scheduler_enabled   = true
            service_scheduler_enabled  = true
            sysbatch_scheduler_enabled = true
        }
    }
}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-config.hcl END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-config.hcl || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-config.hcl')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-config.hcl
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_epl-node-exporter-service.hcl START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

service {
  name = "epl-node-exporter"
  id   = "epl-node-exporter"
  port = 9100
  tags = ["epl-mon-default"]

  meta = {
    metrics_path = "/metrics"
  }

  tagged_addresses = {
    lan = {
      address = "10.17.0.10"
      port    = 9100
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.17.0.10:9100/"
        interval = "15s"
    },
  ]
}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-node-exporter-service.hcl END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-node-exporter-service.hcl || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-node-exporter-service.hcl')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-node-exporter-service.hcl
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_epl-cadvisor-service.hcl START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

service {
  name = "epl-cadvisor"
  id   = "epl-cadvisor"
  port = 9280
  tags = ["epl-mon-default"]

  tagged_addresses = {
    lan = {
      address = "10.17.0.10"
      port    = 9280
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "healthcheck"
        name     = "/healthz"
        http     = "http://10.17.0.10:9280/"
        interval = "15s"
    },
  ]
}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-cadvisor-service.hcl END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-cadvisor-service.hcl || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-cadvisor-service.hcl')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-cadvisor-service.hcl
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_epl-vector-service.hcl START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

service {
  name = "epl-vector"
  id   = "epl-vector"
  port = 9281
  tags = ["epl-mon-default"]

  tagged_addresses = {
    lan = {
      address = "10.17.0.10"
      port    = 9281
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "home"
        tcp      = "10.17.0.10:9281"
        interval = "15s"
    },
  ]
}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-vector-service.hcl END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-vector-service.hcl || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-vector-service.hcl')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-vector-service.hcl
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_vector.toml START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

# ----------------------------------
# prometheus metrics
# ----------------------------------
[sources.internal_metrics]
type = "internal_metrics"
scrape_interval_secs = 2

[sinks.prometheus_exporter_sink]
inputs = ["internal_metrics"]
type = "prometheus_exporter"
address = "10.17.0.10:9281"

# ---------------------------------------------------------
# journald source
# ---------------------------------------------------------
[sources.journald]
type = "journald"
current_boot_only = true
exclude_units = [
  "dbus.service",
  "init.scope",
  "systemd-journald.service",
  "systemd-udevd.service",
]

# ----------------------------------
# docker source
# ----------------------------------
[sources.docker]
type = "docker_logs"

# ----------------------------------
# l1 provisioning sources
# ----------------------------------
[sources.l1_provisioning_logs]
type = "file"
include = [ "/var/log/epl-l1-prov/*.log" ]
read_from = "beginning"
remove_after_secs = 86400

[transforms.l1_provisioning_logs_extra]
type = "remap"
inputs = ["l1_provisioning_logs"]
source = """
segments = split!(.file, "/")
fname = split!(get!(segments, [-1]), ".")
.filename = get!(segments, [-1])
.provisioning_id = get!(fname, [-2])
"""

# ----------------------------------
# l2 provisioning sources
# ----------------------------------
[sources.l2_provisioning_logs]
type = "file"
include = [ "/var/log/epl-l2-prov/*/*.log" ]
read_from = "beginning"

[transforms.l2_provisioning_logs_extra]
type = "remap"
inputs = ["l2_provisioning_logs"]
source = """
segments = split!(.file, "/")
.filename = get!(segments, [-1])
.provisioning_id = get!(segments, [-2])
"""

# ----------------------------------
# loki journald sink
# ----------------------------------
[sinks.loki_journald]
type = "loki"
inputs = [ "journald" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_journald.buffer]
type = "disk"
max_size = 268435488
when_full = "block"

[sinks.loki_journald.labels]
source_type = "journald"
host = "server-a.us-west.epl-infra.net"
systemd_unit = "{{ _SYSTEMD_UNIT }}"

# ----------------------------------
# loki l1 provisioning sink
# ----------------------------------
[sinks.loki_l1_provisioning]
type = "loki"
inputs = [ "l1_provisioning_logs_extra" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l1_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l1_provisioning.labels]
source_type = "l1_provisioning"
host = "server-a.us-west.epl-infra.net"
file = "{{ filename }}"
provisioning_id = "{{ provisioning_id }}"

# ----------------------------------
# loki l2 provisioning sink
# ----------------------------------
[sinks.loki_l2_provisioning]
type = "loki"
inputs = [ "l2_provisioning_logs_extra" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l2_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l2_provisioning.labels]
source_type = "l2_provisioning"
host = "server-a.us-west.epl-infra.net"
file = "{{ filename }}"
provisioning_id = "{{ provisioning_id }}"

# ----------------------------------
# loki nomad docker sink for main
# ----------------------------------
[transforms.loki_nomad_docker_router]
type = "route"
inputs = [ "docker" ]
[transforms.loki_nomad_docker_router.route]
main = '.label.epl_loki_cluster == "main"'


# ----------------------------------
# loki nomad docker sink for main
# ----------------------------------
[sinks.loki_nomad_docker_main]
type = "loki"
inputs = [ "loki_nomad_docker_router.main", "loki_nomad_docker_router._unmatched" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_nomad_docker_main.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_nomad_docker_main.labels]
source_type = "nomad_docker"
host = "server-a.us-west.epl-infra.net"
namespace = "{{ label.\"com.hashicorp.nomad.namespace\" }}"
job_name = "{{ label.\"com.hashicorp.nomad.job_name\" }}"
task_group_name = "{{ label.\"com.hashicorp.nomad.task_group_name\" }}"
task_name = "{{ label.\"com.hashicorp.nomad.task_name\" }}"
alloc_id = "{{ label.\"com.hashicorp.nomad.alloc_id\" }}"
image = "{{ image }}"
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_vector.toml END
if id -u vector &>/dev/null && id -g vector &>/dev/null; then
  chown vector $TMP_SECRET_PATH
  chgrp vector $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/vector.toml || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vector.toml')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vector.toml
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_epl-zfs-exporter-service.hcl START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

service {
  name = "epl-zfs-exporter"
  id   = "epl-zfs-exporter"
  port = 9134
  tags = ["epl-mon-default"]

  meta = {
    metrics_path = "/metrics"
  }

  tagged_addresses = {
    lan = {
      address = "10.17.0.10"
      port    = 9134
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.17.0.10:9134/"
        interval = "15s"
    },
  ]
}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-zfs-exporter-service.hcl END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-zfs-exporter-service.hcl || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-zfs-exporter-service.hcl')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-zfs-exporter-service.hcl
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_l1-fast-prov-decryption-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
B0UZKv1Q8W5x0Q6zKoZLHeHvb3zL1xqJ6Wu+ByG7bjk=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_l1-fast-prov-decryption-key END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/l1-fast-prov-decryption-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/l1-fast-prov-decryption-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/l1-fast-prov-decryption-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_l1-fast-prov-admin-pub-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
WMrzlJEWOfXbbaA9N3dX9bmT/PWL5J/wqnbrG4uxFDA=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_l1-fast-prov-admin-pub-key END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/l1-fast-prov-admin-pub-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/l1-fast-prov-admin-pub-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/l1-fast-prov-admin-pub-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_public_tls_key.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIF7qVV9xyXk7LfBiTys4Jeq6l/BxuZlHnj7W7+WWo4tUoAoGCCqGSM49
AwEHoUQDQgAEBwVBjlxSRPzNKa5okjOXIn3GpEPtJQRZmNDuUYSQERFRZ/n1gPpt
gbOPoMkXMpLpIxX0QbfbZwovAa7fnvrjAw==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_public_tls_key.pem END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/public_tls_key.pem || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/public_tls_key.pem')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/public_tls_key.pem
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_public_tls_cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIB0zCCAXmgAwIBAgIURHWOkn9KNvRoY5arvExMwaj3N1gwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMDkxMjE1MTUwMFoXDTQwMDkwNzE1MTUw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABAcFQY5cUkT8zSmuaJIzlyJ9
xqRD7SUEWZjQ7lGEkBERUWf59YD6bYGzj6DJFzKS6SMV9EG322cKLwGu35764wOj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBTt5kaLFeDVf/2szJ5jlNYJM4fR
jDAfBgNVHSMEGDAWgBTfb6fjpcV+ROLt+ZPtUxzbN/hLfzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAkH2+mhpST9bWU6axj+5/fw9fBqykLhZEFKIt
vm5euRgCIGg0j2D88wDdyrG6utTy9n8V3Rtr2BbELmyHvVU6hQ0r
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_public_tls_cert.pem END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/public_tls_cert.pem || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/public_tls_cert.pem')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/public_tls_cert.pem
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-ca-cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIC6zCCApGgAwIBAgIQKpKba5h2dLI5ZiN07xUqMDAKBggqhkjOPQQDAjCBuDEL
MAkGA1UEBhMCVVMxCzAJBgNVBAgTAkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2Nv
MRowGAYDVQQJExExMDEgU2Vjb25kIFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAV
BgNVBAoTDkhhc2hpQ29ycCBJbmMuMT8wPQYDVQQDEzZDb25zdWwgQWdlbnQgQ0Eg
NTY1ODg4MDM0NjIzOTk0MzIzNTI2NTI2NTAzMDk4NDAzNDk3NDQwHhcNMjMwOTEy
MTUxOTQ3WhcNNDAwOTA3MTUxOTQ3WjCBuDELMAkGA1UEBhMCVVMxCzAJBgNVBAgT
AkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2NvMRowGAYDVQQJExExMDEgU2Vjb25k
IFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAVBgNVBAoTDkhhc2hpQ29ycCBJbmMu
MT8wPQYDVQQDEzZDb25zdWwgQWdlbnQgQ0EgNTY1ODg4MDM0NjIzOTk0MzIzNTI2
NTI2NTAzMDk4NDAzNDk3NDQwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASgdUxy
OjOq+EatX1Dk7rRDGe+h7iwosJoC1nYhDNq/wx2cMz6PbSUTzFH18iKSAHkWFsek
VZvJJOO71fI0rmawo3sweTAOBgNVHQ8BAf8EBAMCAYYwDwYDVR0TAQH/BAUwAwEB
/zApBgNVHQ4EIgQg/mtrQQ0SkAIzZ4ZC/HfnuGdk4E44KiX1pSdOrkI2LMUwKwYD
VR0jBCQwIoAg/mtrQQ0SkAIzZ4ZC/HfnuGdk4E44KiX1pSdOrkI2LMUwCgYIKoZI
zj0EAwIDSAAwRQIhAJAWuWKRN/wC2GZO0vfJDfJmeNGni8a5aldCPpmGNEZ5AiBf
IGuUoKDAS/1pJSUbOv4isHC5Q6D9FRX+CTSwmSZeLg==
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-tls-ca-cert.pem END
if id -u consul &>/dev/null && id -g consul &>/dev/null; then
  chown consul $TMP_SECRET_PATH
  chgrp consul $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-tls-ca-cert.pem || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-tls-ca-cert.pem')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-tls-ca-cert.pem
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-agent-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
5fcc4617-cd1f-40ac-afcf-92ba8f3a724d
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-agent-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-agent-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-agent-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-agent-token.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-server-cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIICwjCCAmmgAwIBAgIQK6gMUM63pRL2WSgKL6NzqDAKBggqhkjOPQQDAjCBuDEL
MAkGA1UEBhMCVVMxCzAJBgNVBAgTAkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2Nv
MRowGAYDVQQJExExMDEgU2Vjb25kIFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAV
BgNVBAoTDkhhc2hpQ29ycCBJbmMuMT8wPQYDVQQDEzZDb25zdWwgQWdlbnQgQ0Eg
NTY1ODg4MDM0NjIzOTk0MzIzNTI2NTI2NTAzMDk4NDAzNDk3NDQwHhcNMjMwOTEy
MTUxOTQ3WhcNMjQwOTExMTUxOTQ3WjAgMR4wHAYDVQQDExVzZXJ2ZXIudXMtd2Vz
dC5jb25zdWwwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATIgF4NCXShgLCcIJM5
hVIsrQE/eYoCqo6V5xxJZaKycs8oFbD789I4fAXxTA/jlOA90+TdrQo19lZptcwv
dZ/jo4HrMIHoMA4GA1UdDwEB/wQEAwIFoDAdBgNVHSUEFjAUBggrBgEFBQcDAQYI
KwYBBQUHAwIwDAYDVR0TAQH/BAIwADApBgNVHQ4EIgQgxsa+UYg0mOeT4qT/60tP
CwzyQwPMFLMFhpvzzYVkVnMwKwYDVR0jBCQwIoAg/mtrQQ0SkAIzZ4ZC/HfnuGdk
4E44KiX1pSdOrkI2LMUwUQYDVR0RBEowSIIec2VydmVyLWEuc2VydmVyLnVzLXdl
c3QuY29uc3VsghVzZXJ2ZXIudXMtd2VzdC5jb25zdWyCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNHADBEAiA7QjLsrB/JUF3x1peP/u+c5JQzIe5huSjFqlqr
StOuDgIgPBRC8q/zYItczWUjhi/mpOpatPksaFboQo4SivIkCWU=
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-tls-server-cert.pem END
if id -u consul &>/dev/null && id -g consul &>/dev/null; then
  chown consul $TMP_SECRET_PATH
  chgrp consul $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-tls-server-cert.pem || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-tls-server-cert.pem')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-tls-server-cert.pem
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-server-pkey.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIGMfBFX3w1XVQrBKJQZ4nY3+E+Wytq3pSSwTRyA7E8xkoAoGCCqGSM49
AwEHoUQDQgAEyIBeDQl0oYCwnCCTOYVSLK0BP3mKAqqOleccSWWisnLPKBWw+/PS
OHwF8UwP45TgPdPk3a0KNfZWabXML3Wf4w==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-tls-server-pkey.pem END
if id -u consul &>/dev/null && id -g consul &>/dev/null; then
  chown consul $TMP_SECRET_PATH
  chgrp consul $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-tls-server-pkey.pem || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-tls-server-pkey.pem')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-tls-server-pkey.pem
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-management-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
23040270-1e3a-4308-aa44-7d94263d7a35
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-management-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-management-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-management-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-management-token.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-default-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
027dc9c1-2d75-4534-8dc3-4f42b49f101a
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-default-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-default-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-default-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-default-token.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-fast-l1-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
c859e665-d7d3-4475-82df-33872f99a792
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-fast-l1-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-fast-l1-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-fast-l1-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-fast-l1-token.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-vrrp-token-dc1.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
4731a9af-b854-49cd-83c0-4ec362c2f3a2
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-vrrp-token-dc1.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-vrrp-token-dc1.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-vrrp-token-dc1.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-vrrp-token-dc1.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-server-consul-acl-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
ff525c11-adfb-4648-a2ab-1fcdd47b8cb8
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-server-consul-acl-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-server-consul-acl-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-server-consul-acl-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-server-consul-acl-token.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-client-consul-acl-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
6da2ec1e-ce23-4ff1-8482-5514d10e67c2
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-client-consul-acl-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-client-consul-acl-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-client-consul-acl-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-client-consul-acl-token.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-ca.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBYzCCAQqgAwIBAgIUYZcmr8mhyAEDZl4kPGxhIvNHo3MwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMwOTEyMTUxNTAwWhcNNDAwOTA3MTUxNTAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABApg
1JUoScfNeESvAx/ILIl6PjYP1a7+GNKp4swMlqIyT0e4e5KikH9FD7tbcK7dCtB6
jzfnHz/tOsuhHKUIMlWjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBR4Rgztzg831f5un/j+MvQW3JdrNjAKBggqhkjOPQQDAgNH
ADBEAiBfB8de1oRPSQGf/wfb3B6hInfn/5FFDOWh/qWS5q0++gIgb7k9IIDaVSyW
hIpvOwajg7S9WViDnm7xyc5QE9AuDV8=
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-ca.crt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-ca.crt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-ca.crt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-ca.crt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-server.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIB5zCCAYygAwIBAgIUPC5Lx0g1EU7ATRp9qJgeKDgq28AwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMwOTEyMTUxNTAwWhcNMjQwOTExMTUxNTAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEFYjIKyNNanLu83lnNq/egoLM
jaGqN6KUhUKjXacKHsflIzFXjpA1iGjWVNOJUdKqW2cK1iI9O7tHFWoIUbtojaOB
0zCB0DAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFDBjxRoAoBxtgcHwFCX6Zz+Nlcem
MB8GA1UdIwQYMBaAFHhGDO3ODzfV/m6f+P4y9Bbcl2s2MFEGA1UdEQEB/wRHMEWC
FHNlcnZlci51cy13ZXN0Lm5vbWFkghxub21hZC1zZXJ2ZXJzLnNlcnZpY2UuY29u
c3Vsgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0EAwIDSQAwRgIhAMDgFUpp22Ih
+LuN/8n6uhTtPEdjbMXVqt3vDX2OkMfiAiEAzqBh3NOwCBYbzLgh8bwDeSAU9GAM
4h2VHY/amqbKizY=
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-server.crt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-server.crt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-server.crt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-server.crt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-server.key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIDqlnoTiWw/8VlPFqx5g+eKd2RpFrdf1FznTnDX4WydboAoGCCqGSM49
AwEHoUQDQgAEFYjIKyNNanLu83lnNq/egoLMjaGqN6KUhUKjXacKHsflIzFXjpA1
iGjWVNOJUdKqW2cK1iI9O7tHFWoIUbtojQ==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-server.key END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-server.key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-server.key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-server.key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-cli.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBkTCCATegAwIBAgIUP2PC2CN8jnZlzOp93K+i09Obg+wwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMwOTEyMTUxNTAwWhcNMjQwOTExMTUxNTAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE8ivg6hMYDVAwQx94Rfv7l1if
WN1hsLsyLDQmS/GDdPn5bLqel5NVv921cMPvQsdeE27/9n+K5HMM0ORXjTGOKKN/
MH0wDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcD
AjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBQrCMD9hxzwzCOgzlCtNgKBg1L1nTAf
BgNVHSMEGDAWgBR4Rgztzg831f5un/j+MvQW3JdrNjAKBggqhkjOPQQDAgNIADBF
AiBXtjCP6cMfJ3zBxZOeRyJFEiWQ9nidnG0DQe2xdCinVAIhAMQVa/qJpeWnnLuE
TGM5xiGLr/jC4MJ7kQhur/tMDCkA
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-cli.crt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-cli.crt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-cli.crt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-cli.crt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-cli.key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIGgEmh28eS/bwLngyyVtPc6497Sx9Iy6N/b6Y8tew9QqoAoGCCqGSM49
AwEHoUQDQgAE8ivg6hMYDVAwQx94Rfv7l1ifWN1hsLsyLDQmS/GDdPn5bLqel5NV
v921cMPvQsdeE27/9n+K5HMM0ORXjTGOKA==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-cli.key END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-cli.key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-cli.key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-cli.key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_vault-ca.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUBDue0YEIfd2qbo4XRzWHvaV9BEYwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMwOTEyMTUxNTAwWhcNNDAwOTA3MTUxNTAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABNTO
yPyt3tguJq5Hovlxr6IQ60lGfwMDWMSc1uv1/dTL3TjprbP9t4Ic+iReWhiJRc0Z
FQ7MDfU45lE14Z+k2k6jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRuPY04BMtoFfZ603s4NbO+FqGOkzAKBggqhkjOPQQDAgNI
ADBFAiEAnVfAzlAORNtoN+EItPWaGwsK6jV+NSO37f2PiH+p44YCIDQ4Bq1aFZNH
Ytpm8Qlf0liWTW3yOqJx/IESsgAakE+m
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_vault-ca.crt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  chmod 0644 $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/vault-ca.crt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vault-ca.crt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vault-ca.crt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_vault-service-consul-acl-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
0c68332c-e144-46a3-a34b-a9919c0daf77
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_vault-service-consul-acl-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/vault-service-consul-acl-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vault-service-consul-acl-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vault-service-consul-acl-token.txt
fi
rm -f $TMP_SECRET_PATH || true
}
trap trap_exit ERR

# -1 means in progress
epl_l1_track_state -1

which lockfile && lockfile /run/epl-l1-provisioning.lock || true
umask 0077
generate_l1_secrets

mkdir -m 0700 -p /run/secdir
chmod 0700 /run/secdir
chown admin /run/secdir

mkdir -m 0750 -p /var/log/epl-l2-prov
chown admin /var/log/epl-l2-prov
grep -q epl-prov /etc/group && chgrp epl-prov /var/log/epl-l2-prov || true
chmod 0750 /var/log/epl-l2-prov

mkdir -m 0750 -p /var/log/epl-l1-upload
find /var/log/epl-l1-upload/*.log -type f -ctime +7 -exec rm -rf {} \; || true
chown admin /var/log/epl-l1-upload
grep -q epl-prov /etc/group && chgrp epl-prov /var/log/epl-l1-upload || true
chmod 0750 /var/log/epl-l1-upload


mkdir -p /etc/nixos/l1-checker
pushd /etc/nixos/l1-checker
echo H4sICJgeJWYCA2wxLXNpZy1jaGVja2VyLnRhcgDtWW1v2zgSzmf/ilndFbWLWrby4gDJprdtLwWK7TZFs7vAIRsItEQ7WkukjqQcu03++86QtCIrzva+NMXumUBjkTPivJDzzIwaDlI+YVVuQpEtdr7OGOIY7e/Tb3R4MLTzyM3tIxJ3ov1RFOHiwUG0M4z2hofRDgx3HmFU2jAFsJPybJ6lTD/Mx5Xe+duNz1DOphr+BVlRSmXge7wGtPICPt/C7VGnQ5NQm5SLeVjM/s1VNmcmkwI+dwAEKzicQJBHfZ1N+8kVT2ZcBcdImqO7iA2pwzAKh3ZRqwQX7JZ5Ng61rFTC32Q516+W59VkkiVcQzgI4QKCsMgDCNJKcP/TL5X8nScmgEvaa1xlefrhimnS4OlTXAEgNkfA6dOnxJYJPOA8bzEWszRT8E9ZGTtNSojta6toGIwzMShYJkK+4JZv4I2rN7b8b0VZGY37Xmfmyhp2DBd2y3+ATFiRQ8rnPJdlwYUTZVdrZeO9u8UPLJmxKdfhJBMpumcDRcs0q4oNhDFaN9rfQFjK37UUGwhluYhTOs1MTOMHuews1v/NM8NJVfT8bWdnO/42Ixw0Q+ub4P9wFB0S/g8P9w8PR7v7hP+7W/x/nNHNmZg63NwLR71Op2sxPY9iBPTYYx4tT7ngihkey5IV8YQwG4yqONEcjnegO0UUrMZAvqJtBoqXkh56xMUqcyWVhuClfYD3SAiIQDhr8B8n4k/15I4hx7wgEL7fvX19+v78lJZSmVSEqS4XXRlT6qPBoFL5wMjBGpG4S4dnqOFG66Crl0KWOiPlQF9RIlytBEROuU5UVlpZyJFLMUX9Gqueq+Qi1SvgR5/SqmFTjcjZNbLMEhRgpH9xjIltiZ4DH3w99NI3iH/KdKTr15Pxpfjf3R+t6r+DvSHSo929/YNt/D9K/GN9k1SGjXOKjrIaY6zFDwSJXaZgpQkWKIqpDFHAFSXgShBwpQRsKC9CVQmT4Ra+nKBdSkQIJbHq0/hc6k2vfZOw+D/K/3WlW+TfJv5Hu3sHLv53o8PhaETxj3XANv4fY5hlySGRQld5XJUpJnhsZqi1m/HlEWijMBDh4gecQfAjXwa29UoUp0oA2xS+OMIOy6w4XlvCW1p3nAViw2S5gfMnS2hw5jKZbeB7h8sNrkmO2XSN4Q2tOOKc5RVvK/0rLTq6RpjBXN3mOHfLxHOLaz+s8MdD2WWnk3NjcQ+6PTghXXHOlEJPnS91yNR0jhpRrzmBl0qxZZhzMcV+kHi+O4F9MFccX7b9FSKeUjGnDg9rruB0UWLqRwbFOfJPbd2CFpYM38dSwdUH1HNzOhKgwut5TXV4vYGAAsHwosQudTLBUoWotgMH4IvMwCE994DnWFU5vcgmFBZ7YTHuaas8NBLNCLtRz9norbclXGaWsU8ZbfbdNfaUJ2pZmjWOvTWOlmjrZhpvBWYgJgTPQ2qxsfrkIkbI2qxrgzujzjzGzt9vdPMCzu2xh/i3oOkrm7BC1E2mPOYLYReXhtreSezvCL1m81v4Si5CRzUy1mgQN1bTllcaDvmyEQ+78dEsaajrLek+gwJDAutlmEhVMAPPeivChZAi4ZcXXNgD5SneuGUuWXrpecgNfim2hcBJ0xRtUi/EmvHnRqI4Z9GqtJgoWXisArvTs96aJxonsVKhZEqjkifwH1eCnLMJD2mflVfWdEWha1AYo/d8PeP3Rm8kV+3d6Uwt9QbOZuAX+y98YPkIGBNauMInTMf2CkAwmDM1wEpqwMu8n0fUhs8H9CcjREL9dOiKpQBqv6FbhIRrqWzxxcayMlRRIQM1EdShjHnCMF2BqIoxxj52NWz17E/J6ZRhT6VMbF18pxwVhKRu9wM6yExCXdpf+HzjX8X7jE3Yx5/x5+ezlcOwWvRe0V2/1MT+Hvz68t0vp+fdJ+lKg5tb76pwnbE2lFSM172W5BLtQuVqJoTczWZ8TMKzH5vA27oa9B3Sibdp438Oo1oygHeQ90/wkSdYQ9yFBPgs8CT9TQTQdRv5Nc/SO15TzQYXKuZYNXbSq62Gzdi1bFicf+IPBbXbqKEq7V7HbHzngvuSNsvZqD30N3P32pI9/KPkMW1Dp9RWeiwXLibaiaCNqfetuGfsGSLJfbyd4uYXZ/Rky5djsM/XSop86ScIyUwtL2EoR8PhetLqTirULcGwrsW0RWEgEoY5E5G1ZfVx48XWuVNKru8vVhSJLAom0nshGFTiEwIf9BN4ohFtbO2ur6Dfz+U0E8egCuhPkBasK9+crB0ORs+d7O9f4DVrBUx9yWsV3jDcJAXXNlJA1GaCV/u5KzFozyO8/EFDBF7Xtfrnzg1rtYgbVmKzUrISEd90lVDHOKny4O79zgP7tPV/7fKHwxywmINKwjVDlMzxYqRLrBO5eA56lpUlGhhsgql7lnSaavR8NjhVSipArmY2aJeA+Fy/1HO1pi8zrep8kYQKEUnhNWLJzCiGl50+vJHvjFrWhanPQjdAwFXLw2zxns8R+e2Z2P+lwHjpf+JKPodsKqTCLI9HWim8tEIaKh7xEXsiPEFMx3TCq5TL1TxLuE8hbb96WwlMmb0XE6YN5NEKLI7stezWNlER5RAV9e1t8ue27X+07/9fV8aXvv8NRwf19//diL7/RaNo+/3/kb7/iTl9nU753OGF7bChe4RuESlTmGyumRJ9bsO7/7K3/Ry3HduxHduxHdvxlx9/ABCNZdMAKAAA | base64 -d | gunzip | tar x -C .
popd


mkdir -m 700 -p /run/sec_volumes/ssl_certs
cp /run/keys/public_tls_key.pem /run/sec_volumes/ssl_certs/
cp /run/keys/public_tls_cert.pem /run/sec_volumes/ssl_certs/


function provision_zfs_dataset() {
  VOLUME_ZPOOL=$1
  VOLUME_NAME=$2
  MOUNTPOINT=$3
  RECORDSIZE=$4
  COMPRESSION=$5
  EXPOSE_TO_CONTAINERS=$6
  ENCRYPTION_PASSPHRASE=$7
  EXPECTED_DATASET_NAME="$VOLUME_ZPOOL/epl-$VOLUME_NAME"
  KEY_LOCATION="/run/keys/zfs-volume-$VOLUME_NAME-passphrase"

  CREATION_ARGS="-o compression=$COMPRESSION -o mountpoint=$MOUNTPOINT -o recordsize=$RECORDSIZE"

  if ! zfs list -H -o name $EXPECTED_DATASET_NAME
  then
    # dataset doesnt exist, create

    ENCRYPTION_ARGS=""
    if [ -n "$ENCRYPTION_PASSPHRASE" ];
    then
        # we rely on umask created for l1 provisioning script to create file with 0600 permissions
        echo -n "$ENCRYPTION_PASSPHRASE" > $KEY_LOCATION
        ENCRYPTION_ARGS="-o encryption=on -o keylocation=file://$KEY_LOCATION -o keyformat=passphrase"
    fi

    zfs create $CREATION_ARGS $ENCRYPTION_ARGS $EXPECTED_DATASET_NAME

    if [ -n "$ENCRYPTION_PASSPHRASE" ];
    then
        shred $KEY_LOCATION
        rm -f $KEY_LOCATION
    fi
  else
    # dataset exists, check if not mounted then mount
    IS_MOUNTED=$( zfs list -H -o mounted $EXPECTED_DATASET_NAME )
    if [ "$IS_MOUNTED" == "no" ]
    then
        if [ -n "$ENCRYPTION_PASSPHRASE" ];
        then
            echo -n "$ENCRYPTION_PASSPHRASE" > $KEY_LOCATION
            zfs load-key $EXPECTED_DATASET_NAME || true
        fi

        zfs mount $EXPECTED_DATASET_NAME

        if [ -n "$ENCRYPTION_PASSPHRASE" ];
        then
            shred $KEY_LOCATION
            rm -f $KEY_LOCATION
        fi
    fi
  fi

  if [ "$EXPOSE_TO_CONTAINERS" == "yes" ]
  then
    chmod 777 $MOUNTPOINT
  else
    chmod 700 $MOUNTPOINT
  fi
}
mkdir -m 700 -p /srv/volumes
zpool import -af
provision_zfs_dataset rpool consul /var/lib/consul 4k on no NlWMzUL9tqJGu8oe9XEGWZHYK6eq9CNeSmfEkFz03g
provision_zfs_dataset rpool docker /var/lib/docker 128k on no
provision_zfs_dataset rpool minio-docker-a /srv/volumes/minio-docker-a 1M on yes ujH3pcvMVZcyTIgRJjq4poPbRZsz7fNJqJb6s0R8yX
provision_zfs_dataset rpool mon-am /srv/volumes/mon-am 4k on yes ATykOoODZvy30f4c4tVrYqfXRyipwqkhNlCQhr8iiv
provision_zfs_dataset rpool mon-default /srv/volumes/mon-default 4k on yes fJKv82MMKCN1wFYk51ysJH2C5hFOyIZQt0SJCY8oiV
provision_zfs_dataset rpool nomad /var/lib/nomad 4k on no RnmMPkboUYAwJq3ojX2v4lCZvI322ai99WzApTGhTR
provision_zfs_dataset rpool pgtest1 /srv/volumes/pgtest1 4k on yes e5s0hgFvxYlV6GMvYESV1HheM3EwYgrWDiSq0htweW

mkdir -p /etc/nixos
pushd /etc/nixos
git config --global init.defaultBranch master
git config --global user.name 'EPL L1 provisioner'
git config --global user.email 'epl@example.com'
git init
if ! cat /etc/nixos/configuration.nix | head -n 2 | grep '# EDEN PLATFORM GENERATED NIX CONFIG'
then
  mv -f /etc/nixos/configuration.nix /etc/nixos/configuration-initial.nix || true
fi
cat > /etc/nixos/configuration.nix <<'LilBoiPeepLikesBenzTruck'
# EDEN PLATFORM GENERATED NIX CONFIG
# changes done to this file will be overwritten by Eden platform
let
  pkgs = import (fetchTarball { url = "https://github.com/NixOS/nixpkgs/archive/057f9aecfb71c4437d2b27d3323df7f93c010b7e.tar.gz"; sha256 = "1ndiv385w1qyb3b18vw13991fzb9wg4cl21wglk89grsfsnra41k"; }) {};
  lib = pkgs.lib;
  modulesPath = pkgs.path + "/nixos/modules";
in

{ ... }:
{

    nix.settings = {
      tarball-ttl = 60 * 60 * 7;
      experimental-features = [ "nix-command" "flakes" ];
      substituters = [
        "https://cache.nixos.org/"
      ];
      trusted-public-keys = [
        "epl-nix-cache:iOjMfZuBw6v4/nhD+WTutCbfs/p9cl2ZZLnfVMdxpEQ="
      ];

    };

    networking.hostId = "a79b8498";


    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIONUZdMtNbaNBA+F2IS18RAcVToqkvGVDw4/3nFvE9TR epl-root-ssh-key"
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC3AkuDzzPrMaDav0kN7PIoaBU1Vtw1TfkHxWzPMrleocCltYl8TljwCqEJtmizx5DGKbXFQg31mRVswzuAq2vP2RFdPHQxfl5nJnWsQkelvpPO/Q3LUdtrm19zAgbbDL+AtIg3/lif6/2qNiWCSTfaUpjM7WOPszBNmMRGz/UBZTYc7COTt+I3lK8f6sBn5YyD796LBw6tsNpqfqF9NTAsLT8/PqrXeTpdxFe375gMxeIpNWeE5exMGJKgqnZCcOMOoKMJy61+wdEAYzDFNgIX7ZFvpBYQPf/rTs7LWgtyTSw3fqvMDnfwAf7oIF8rZRwYdVnqTGCWA2h3f4lOf6BERIPkKEK7/DGjmekKnXJrRiLSfcgRjri3VuGBxrJ+Va/Dn6e7o7CdzdJ+fkw7KxTFKuf17Z2r3ZFi1xOduIxXW8/QY6zhq2A11e+HsMe/oaBh3bRcpdMFmW5mqQjGm05xvxArSCAARBKkHjywGs6mRLN2PjNPYdzlI2J8nF6bmSk= henlo"

      ];
    };
    services.sshd.enable = true;
    services.openssh.settings.PermitRootLogin = "prohibit-password";
    services.getty.autologinUser = lib.mkDefault "root";

    swapDevices = [ ];

    nixpkgs.config.allowUnfreePredicate = pkg: builtins.elem (lib.getName pkg) [
        "consul"
        "nomad"
        "vault"
        "vault-bin"
     ];

    system.stateVersion = "23.11";

    environment.sessionVariables = {
      NOMAD_ADDR = "https://nomad-servers.service.consul:4646";
      VAULT_ADDR = "https://vault.service.consul:8200";
    };

    security.pki.certificates = [
      ''-----BEGIN CERTIFICATE-----
MIIB0zCCAXmgAwIBAgIURHWOkn9KNvRoY5arvExMwaj3N1gwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMDkxMjE1MTUwMFoXDTQwMDkwNzE1MTUw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABAcFQY5cUkT8zSmuaJIzlyJ9
xqRD7SUEWZjQ7lGEkBERUWf59YD6bYGzj6DJFzKS6SMV9EG322cKLwGu35764wOj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBTt5kaLFeDVf/2szJ5jlNYJM4fR
jDAfBgNVHSMEGDAWgBTfb6fjpcV+ROLt+ZPtUxzbN/hLfzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAkH2+mhpST9bWU6axj+5/fw9fBqykLhZEFKIt
vm5euRgCIGg0j2D88wDdyrG6utTy9n8V3Rtr2BbELmyHvVU6hQ0r
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBYzCCAQqgAwIBAgIUYZcmr8mhyAEDZl4kPGxhIvNHo3MwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMwOTEyMTUxNTAwWhcNNDAwOTA3MTUxNTAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABApg
1JUoScfNeESvAx/ILIl6PjYP1a7+GNKp4swMlqIyT0e4e5KikH9FD7tbcK7dCtB6
jzfnHz/tOsuhHKUIMlWjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBR4Rgztzg831f5un/j+MvQW3JdrNjAKBggqhkjOPQQDAgNH
ADBEAiBfB8de1oRPSQGf/wfb3B6hInfn/5FFDOWh/qWS5q0++gIgb7k9IIDaVSyW
hIpvOwajg7S9WViDnm7xyc5QE9AuDV8=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUBDue0YEIfd2qbo4XRzWHvaV9BEYwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMwOTEyMTUxNTAwWhcNNDAwOTA3MTUxNTAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABNTO
yPyt3tguJq5Hovlxr6IQ60lGfwMDWMSc1uv1/dTL3TjprbP9t4Ic+iReWhiJRc0Z
FQ7MDfU45lE14Z+k2k6jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRuPY04BMtoFfZ603s4NbO+FqGOkzAKBggqhkjOPQQDAgNI
ADBFAiEAnVfAzlAORNtoN+EItPWaGwsK6jV+NSO37f2PiH+p44YCIDQ4Bq1aFZNH
Ytpm8Qlf0liWTW3yOqJx/IESsgAakE+m
-----END CERTIFICATE-----
''
    ];

    environment.systemPackages =
      let
        epl-acme-vault-policies = pkgs.writeShellScriptBin "epl-acme-vault-policies" ''

                if [ -z "$VAULT_TOKEN" ]
                then
                    echo Must set VAULT_TOKEN for this script
                    exit 7
                fi

                while ! curl -f -s https://vault.service.consul:8200 &>/dev/null
                do
                    sleep 1
                done

                cat > /tmp/epl-acme-vault-token-policy.json<<EOL
                {
                    "token_explicit_max_ttl": 0,
                    "name": "epl-acme-certs",
                    "orphan": true,
                    "token_period": 259200,
                    "renewable": true
                }
                EOL

                cat > /tmp/epl-acme-vault-policy.hcl<<EOL
                path "epl/data/certs/*" {
                    capabilities = ["read", "list", "create", "patch", "update", "delete"]
                }

                # Allow looking up the token passed to validate the token has the
                # proper capabilities. This is provided by the "default" policy.
                path "auth/token/lookup-self" {
                    capabilities = ["read"]
                }

                # Allow checking the capabilities of our own token. This is used to validate the
                # token upon startup.
                path "sys/capabilities-self" {
                    capabilities = ["update"]
                }

                # Allow our own token to be renewed.
                path "auth/token/renew-self" {
                    capabilities = ["update"]
                }
                EOL

                vault policy write epl-acme-certs /tmp/epl-acme-vault-policy.hcl
                vault write /auth/token/roles/epl-acme-certs @/tmp/epl-acme-vault-token-policy.json

                ORIGINAL_TOKEN=$VAULT_TOKEN
                export VAULT_TOKEN=$1
                if ! vault token lookup
                then
                    # token invalid, needs to be created
                    export VAULT_TOKEN=$ORIGINAL_TOKEN
                    NEW_TOKEN=$( vault token create -policy epl-acme-certs -period 168h -orphan | grep 'hvs.' | sed -E 's/^.* hvs/hvs/' )
                    echo "ACME_CERTS_VAULT_TOKEN $NEW_TOKEN"
                fi

        '';
        epl-consul-bootstrap = pkgs.writeShellScriptBin "epl-consul-bootstrap" ''

            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^agent-policy:$' &>/dev/null && exit 0

            cat > /tmp/epl-consul-agent-policy.hcl<<EOL
            node_prefix "" {
                policy = "write"
            }
            service_prefix "" {
                policy = "write"
            }
            EOL

            cat > /tmp/epl-consul-default-policy.hcl<<EOL
            # allow prometheus target scrapes
            agent_prefix "" {
                policy = "read"
            }
            node_prefix "" {
                policy = "read"
            }
            service_prefix "" {
                policy = "read"
            }
            # For DNS policiy, remove in the future when
            # we separate default token from DNS token
            query_prefix "" {
                policy = "read"
            }

            # inter DC routing, allow every node to access routes
            key_prefix "epl-interdc-routes/" {
                policy = "list"
            }

            # all l1 provisioning plans are sodium encrypted doesnt matter
            # if anyone reads, only intended node can decrypt
            key_prefix "epl-l1-plans/" {
                policy = "list"
            }
            EOL

            cat > /tmp/epl-consul-fast-l1-admin-policy.hcl<<EOL
            # allow plans upload for every server
            key_prefix "epl-l1-plans/" {
                policy = "write"
            }
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "agent-policy" \
                -description "Agent Token Policy" \
                -rules @/tmp/epl-consul-agent-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Agent Token" \
                -policy-name "agent-policy" \
                -secret=$( sudo cat /run/keys/consul-agent-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "default-token" \
                -description "Default Token Policy" \
                -rules @/tmp/epl-consul-default-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Default Token" \
                -policy-name "default-token" \
                -secret=$( sudo cat /run/keys/consul-default-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "fast-l1-token" \
                -description "Fast L1 Admin Policy" \
                -rules @/tmp/epl-consul-fast-l1-admin-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Fast L1 Admin" \
                -policy-name "fast-l1-token" \
                -secret=$( sudo cat /run/keys/consul-fast-l1-token.txt )

        '';
        epl-consul-vrrp-acl = pkgs.writeShellScriptBin "epl-consul-vrrp-acl" ''

# NIX REGION consul_vrrp_bootstrap_script START

export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

while :
do
    consul members | grep alive &>/dev/null && break
    sleep 1
done

if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
then
    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
    key_prefix "epl-interdc-routes/dc1" {
        policy = "write"
    }
EOL

    ${pkgs.consul}/bin/consul acl policy create \
        -name "vrrp-policy-dc1" \
        -description "VRRP policy for datacenter dc1" \
        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

    ${pkgs.consul}/bin/consul acl token create \
        -description "VRRP Token for datacenter dc1" \
        -policy-name "vrrp-policy-dc1" \
        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
fi


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
            # ROUTES CREATE
            ip route add 0.0.0.0/0 via 10.17.0.12

            # ROUTES DELETE
            ip route del 0.0.0.0/0

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc1.txt


# NIX REGION consul_vrrp_bootstrap_script END

        '';
        epl-nomad-acl-bootstrap = pkgs.writeShellScriptBin "epl-nomad-acl-bootstrap" ''

            while ! curl -f -s https://nomad-servers.service.consul:4646 &>/dev/null
            do
                sleep 1
            done

            while [ "$( dig +short nomad-servers.service.consul | wc -l )" -lt 3 ]
            do
                sleep 1
            done

            while true
            do
              nomad acl bootstrap &> /run/secdir/nomad-bootstrap-output.txt.tmp
              if cat /run/secdir/nomad-bootstrap-output.txt.tmp | grep 'No cluster leader'
              then
                sleep 2
                continue
              fi

              if cat /run/secdir/nomad-bootstrap-output.txt.tmp | grep 'Secret ID'
              then
                mv -f /run/secdir/nomad-bootstrap-output.txt.tmp /run/secdir/nomad-bootstrap-output.txt
              fi

              break
            done

        '';
        epl-nomad-acl-policies = pkgs.writeShellScriptBin "epl-nomad-acl-policies" ''


            if [ -z "$NOMAD_TOKEN" ]
            then
                echo Must set NOMAD_TOKEN for this script
                exit 7
            fi

            while ! curl -f -s https://nomad-servers.service.consul:4646 &>/dev/null
            do
                sleep 1
            done

            cat > /tmp/epl-nomad-anonymous-policy.hcl<<EOL
              namespace "*" {
                policy       = "read"
                capabilities = [
                  "list-jobs"
                ]
              }

              agent {
                policy = "read"
              }

              operator {
                policy = "read"
              }

              quota {
                policy = "read"
              }

              node {
                policy = "read"
              }

              host_volume "*" {
                policy = "read"
              }
            EOL

            nomad acl policy apply -description "Anonymous policy" anonymous /tmp/epl-nomad-anonymous-policy.hcl

            nomad namespace apply -description "Eden platform" epl

        '';
        epl-nomad-consul-acl-bootstrap = pkgs.writeShellScriptBin "epl-nomad-consul-acl-bootstrap" ''

            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^nomad-server:$' &>/dev/null && exit 0

            cat > /tmp/epl-nomad-server-consul-acl-policy.hcl<<EOL
            agent_prefix "" {
                policy = "read"
            }

            node_prefix "" {
                policy = "read"
            }

            service_prefix "" {
                policy = "write"
            }

            # TODO: remove after nomad 1.9 and use consul identities instead
            key_prefix "epl-kv/" {
                policy = "read"
            }

            acl = "write"
            EOL

            cat > /tmp/epl-nomad-client-consul-acl-policy.hcl<<EOL
            agent_prefix "" {
                policy = "read"
            }

            node_prefix "" {
                policy = "read"
            }

            service_prefix "" {
                policy = "write"
            }

            acl = "write"
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "nomad-server" \
                -description "Nomad Server Policy" \
                -rules @/tmp/epl-nomad-server-consul-acl-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Nomad Server Token" \
                -policy-name "nomad-server" \
                -secret=$( sudo cat /run/keys/nomad-server-consul-acl-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "nomad-client" \
                -description "Nomad Client Policy" \
                -rules @/tmp/epl-nomad-client-consul-acl-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Nomad Client Token" \
                -policy-name "nomad-client" \
                -secret=$( sudo cat /run/keys/nomad-client-consul-acl-token.txt )

        '';
        epl-nomad-vault-policies = pkgs.writeShellScriptBin "epl-nomad-vault-policies" ''

            if [ -z "$VAULT_TOKEN" ]
            then
                echo Must set VAULT_TOKEN for this script
                exit 7
            fi

            while ! curl -f -s https://vault.service.consul:8200 &>/dev/null
            do
                sleep 1
            done

            cat > /tmp/epl-nomad-vault-token-policy.json<<EOL
            {
                "disallowed_policies": "nomad-server",
                "token_explicit_max_ttl": 0,
                "name": "nomad-cluster",
                "orphan": true,
                "token_period": 259200,
                "renewable": true
            }
            EOL

            cat > /tmp/epl-nomad-vault-policy.hcl<<EOL
            # Allow creating tokens under "nomad-cluster" token role. The token role name
            # should be updated if "nomad-cluster" is not used.
            path "auth/token/create/nomad-cluster" {
                capabilities = ["update"]
            }

            # Allow looking up "nomad-cluster" token role. The token role name should be
            # updated if "nomad-cluster" is not used.
            path "auth/token/roles/nomad-cluster" {
                capabilities = ["read"]
            }

            # Allow looking up the token passed to Nomad to validate # the token has the
            # proper capabilities. This is provided by the "default" policy.
            path "auth/token/lookup-self" {
                capabilities = ["read"]
            }

            # Allow looking up incoming tokens to validate they have permissions to access
            # the tokens they are requesting. This is only required if
            # allow_unauthenticated is set to false.
            path "auth/token/lookup" {
                capabilities = ["update"]
            }

            # Allow revoking tokens that should no longer exist. This allows revoking
            # tokens for dead tasks.
            path "auth/token/revoke-accessor" {
                capabilities = ["update"]
            }

            # Allow checking the capabilities of our own token. This is used to validate the
            # token upon startup.
            path "sys/capabilities-self" {
                capabilities = ["update"]
            }

            # Allow our own token to be renewed.
            path "auth/token/renew-self" {
                capabilities = ["update"]
            }
            EOL

            vault policy write nomad-server /tmp/epl-nomad-vault-policy.hcl
            vault write /auth/token/roles/nomad-cluster @/tmp/epl-nomad-vault-token-policy.json

            ORIGINAL_TOKEN=$VAULT_TOKEN
            export VAULT_TOKEN=$1
            if ! vault token lookup
            then
                # token invalid, needs to be created
                export VAULT_TOKEN=$ORIGINAL_TOKEN
                NEW_TOKEN=$( vault token create -policy nomad-server -period 72h -orphan | grep 'hvs.' | sed -E 's/^.* hvs/hvs/' )
                echo "NOMAD_VAULT_TOKEN $NEW_TOKEN"
            fi

        '';
        epl-process-route-data = pkgs.writeShellScriptBin "epl-process-route-data" ''

PREFIX=/run/current-system/sw/bin
$PREFIX/cat /dev/stdin > /run/epl-routes-tmp

# delete old routes if exist
if [ -f /run/epl-routes ];
then
    OLD_SCRIPT=$( $PREFIX/cat /run/epl-routes | $PREFIX/jq -r '.Value' | $PREFIX/base64 -d )
    # delete old routes
    DELETE_BLOCK=$( $PREFIX/echo "$OLD_SCRIPT" | $PREFIX/sed -n '/ROUTES DELETE/,/FINISH/p' )
    $PREFIX/echo "export PATH=/run/current-system/sw/bin/:\$PATH; $DELETE_BLOCK" | /bin/sh
    $PREFIX/echo old routes deleted
    $PREFIX/echo "$DELETE_BLOCK"
fi

NEW_SCRIPT=$( $PREFIX/cat /run/epl-routes-tmp | $PREFIX/jq -r '.Value' | $PREFIX/base64 -d )
NEW_ADD_BLOCK=$( $PREFIX/echo "$NEW_SCRIPT" | $PREFIX/sed -n '/ROUTES CREATE/,/ROUTES DELETE/p' )
# add new routes
$PREFIX/echo "export PATH=/run/current-system/sw/bin/:\$PATH; $NEW_ADD_BLOCK" | /bin/sh

# set new file in place for old route deletion
$PREFIX/mv -f /run/epl-routes-tmp /run/epl-routes

$PREFIX/echo routes were changed
$PREFIX/echo "$NEW_ADD_BLOCK"


        '';
        epl-vault-consul-acl-bootstrap = pkgs.writeShellScriptBin "epl-vault-consul-acl-bootstrap" ''

            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^vault-service:$' &>/dev/null && exit 0

            cat > /tmp/epl-consul-vault-service-policy.hcl<<EOL
            service "vault" {
              policy = "write"
            }

            agent_prefix "" {
              policy = "read"
            }

            session_prefix "" {
              policy = "write"
            }
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "vault-service" \
                -description "Vault Service Policy" \
                -rules @/tmp/epl-consul-vault-service-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Vault Service Token" \
                -policy-name "vault-service" \
                -secret=$( sudo cat /run/keys/vault-service-consul-acl-token.txt )

        '';
        epl-wait-for-consul = pkgs.writeShellScriptBin "epl-wait-for-consul" ''

while ! ${pkgs.consul}/bin/consul members
do
  sleep 5
done

        '';
        epl-watch-route-data = pkgs.writeShellScriptBin "epl-watch-route-data" ''

# wait for consul to become available
while ! ${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24
do
  sleep 7
done

exec ${pkgs.consul}/bin/consul watch -type=key -key=epl-interdc-routes/dc1/10.17.0.0p24 /run/current-system/sw/bin/epl-process-route-data

        '';

      in
      [
        pkgs.awscli2
        pkgs.bmon
        pkgs.cadvisor
        pkgs.curl
        pkgs.dig
        pkgs.git
        pkgs.gzip
        pkgs.htop
        pkgs.iftop
        pkgs.inetutils
        pkgs.iotop
        pkgs.iperf
        pkgs.jq
        pkgs.moreutils
        pkgs.natscli
        pkgs.netcat
        pkgs.nftables
        pkgs.nomad
        pkgs.parted
        pkgs.postgresql
        pkgs.procmail
        pkgs.prometheus-node-exporter
        pkgs.sqlite
        pkgs.sysstat
        pkgs.tmux
        pkgs.vault
        pkgs.vector
        pkgs.vim
        pkgs.wget
        pkgs.zstd
        epl-acme-vault-policies
        epl-consul-bootstrap
        epl-consul-vrrp-acl
        epl-nomad-acl-bootstrap
        epl-nomad-acl-policies
        epl-nomad-consul-acl-bootstrap
        epl-nomad-vault-policies
        epl-process-route-data
        epl-vault-consul-acl-bootstrap
        epl-wait-for-consul
        epl-watch-route-data
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "169.254.169.254"; prefixLength = 32; via = "10.17.0.1"; }

      { address = "10.17.0.0"; prefixLength = 16; via = "10.17.0.1"; }

    ];

# NIX REGION static_node_routes END

    boot.kernel.sysctl = {
      # for loki ScyllaDB
      "fs.aio-max-nr" = 1048576;
    };

# NIX REGION firewall START

  networking.hostName = "server-a";
  networking.firewall.allowPing = true;
  networking.firewall.enable = true;
  networking.firewall.checkReversePath = true;
  networking.firewall.trustedInterfaces = [

    "eth0"

  ];

# NIX REGION firewall END

   programs.bash.promptInit = ''
     # Provide a nice prompt if the terminal supports it.
     if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
       PROMPT_COLOR="1;31m"
       ((UID)) && PROMPT_COLOR="1;32m"
       if [ -n "$INSIDE_EMACS" ]; then
         # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-a.dc1.us-west.aws-single-dc:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-a.dc1.us-west.aws-single-dc: \w\a\]\u@server-a.dc1.us-west.aws-single-dc:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-a.dc1.us-west.aws-single-dc:\u:\w\007\]$PS1"
       fi
     fi
   '';

     # l1 agent
     systemd.services.l1-fast-agent = {
       wantedBy = [ "multi-user.target" ];
       requires = [ "network-online.target" ];
       after = [ "network-online.target" "consul.service" ];
       script =
       let
         l1Checker = import ./l1-checker/default.nix { pkgs = pkgs; };
       in
       ''
         export PATH=/run/current-system/sw/bin:$PATH
         # wait for consul to become available
         while ! ${pkgs.consul}/bin/consul kv get epl-l1-plans/server-a
         do
           sleep 7
         done

         ${pkgs.consul}/bin/consul watch \
           -type=key -key=epl-l1-plans/server-a \
           ${l1Checker}/checker \
             /run/keys/l1-fast-prov-decryption-key \
             /run/keys/l1-fast-prov-admin-pub-key \
             /run/secdir/l1-fast-plan.zst
       '';

       serviceConfig = {
         User = "root";
         Group = "root";
         Type = "simple";
         Restart = "always";
         RestartSec = "20";
       };

       enable = true;
     };

    imports = [ "${modulesPath}/virtualisation/amazon-image.nix" ];


  ec2.zfs.enable = true;

  fileSystems."/" =
    { device = "rpool/root";
      fsType = "zfs";
    };

  fileSystems."/nix" =
    { device = "rpool/nix";
      fsType = "zfs";
    };

  fileSystems."/var" =
    { device = "rpool/var";
      fsType = "zfs";
    };

  fileSystems."/var/log" =
    { device = "rpool/var/log";
      fsType = "zfs";
    };

  fileSystems."/boot" = {
    # The ZFS image uses a partition labeled ESP whether or not we're
    # booting with EFI.
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };


    networking.usePredictableInterfaceNames = false;

    users.users.named.extraGroups = ["keys"];
    services.bind =
    {
        enable = true;
        extraOptions = ''
          recursion yes;
          dnssec-validation auto;
          validate-except { consul.; };
          key-directory "/run/dnsseckeys";
        '';
        forwarders = [ "1.1.1.1" ];
        cacheNetworks = [
          # bind can be internet
          # facing depending on DC
          "0.0.0.0/0"
        ];
        extraConfig = ''
          trust-anchors {
  epl-infra.net. initial-key 257 3 15 "TCXRFxjm2uqUKpKn6eABN5NNf1FtyY8a9H6uZR7K6KI=";
  us-west.epl-infra.net. initial-key 257 3 15 "OXxs15tmj8FtSQsZdBwa1SM1MiAeEg/C7e3f/IczJNQ=";
  10.in-addr.arpa. initial-key 257 3 15 "Y+yibE5ACePmUjX87fv8Z7T9ayF6OAIlsxAQC7FX2a8=";
  17.10.in-addr.arpa. initial-key 257 3 15 "/wiAJKQkhlzvci06DLGhDg9SVUS462dou5TqD1wD5QI=";
  in-addr.arpa. initial-key 257 3 15 "Udz7YhewT0Pttkx4Wfy75TaQFegWkT666WcX3zlo1no=";
};




          dnssec-policy epl {
            keys {
              ksk key-directory lifetime unlimited algorithm ED25519;
              zsk key-directory lifetime unlimited algorithm ED25519;
            };
            dnskey-ttl 300;
            max-zone-ttl 3600;
            parent-ds-ttl 3600;
            parent-propagation-delay 2h;
            publish-safety 7d;
            retire-safety 7d;
            signatures-refresh 1439h;
            signatures-validity 90d;
            signatures-validity-dnskey 90d;
            zone-propagation-delay 2h;
          };

view lan {
          # add VPN address so local user integration tests pass
          match-clients { 10.0.0.0/8; 172.21.0.0/16; localhost; };
          zone "consul." IN {
              type forward;
              forward only;
              forwarders { 127.0.0.1 port 8600; };
          };

          zone "us-west.epl-infra.net" IN {
              type forward;
              forward only;
              forwarders {
                10.17.0.13 port 53;
                10.17.0.12 port 53;
              };

          };

          zone "17.10.in-addr.arpa." IN {
              type forward;
              forward only;
              forwarders {
                10.17.0.13 port 53;
                10.17.0.12 port 53;
              };

          };


          zone "epl-infra.net." IN {
              type forward;
              forward only;
              forwarders {
                10.17.0.13 port 53;
                10.17.0.12 port 53;
              };

          };

          zone "10.in-addr.arpa." IN {
              type forward;
              forward only;
              forwarders {
                10.17.0.13 port 53;
                10.17.0.12 port 53;
              };

          };


};

        '';
    };

    virtualisation.docker.daemon.settings = { "registry-mirrors" = [ "https://registry-1.docker.io" "http://epl-docker-registry.service.consul:5000" ]; };
    virtualisation.docker.extraOptions = "--insecure-registry http://epl-docker-registry.service.consul:5000";

    users.groups.epl-prov = {};

    services.consul = {
      enable = true;
      webUi = true;
      forceAddrFamily = "ipv4";
      extraConfigFiles = [
        "/run/keys/consul-config.json"
      ];
    };
    users.users.consul.extraGroups = ["keys"];


    # reload service on file change
    systemd.services.consul-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/consul-config.json -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl restart consul.service || true'";
      };

      enable = true;
    };
    systemd.paths.consul-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/consul-config.json";
        Unit = "consul-restart.service";
      };

      enable = true;
    };

    systemd.services.nomad = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];
      path = [ pkgs.iproute2 ];

      serviceConfig = {
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStartPre = [
            "+${pkgs.coreutils}/bin/mkdir -p /var/lib/nomad"
            "+${pkgs.coreutils}/bin/chmod 700 /var/lib/nomad"
        ];
        ExecStart = "${pkgs.nomad}/bin/nomad agent -config=/run/keys/nomad-config.hcl";
        ExecReload = "/bin/kill -HUP $MAINPID";
        KillMode = "process";
        KillSignal = "SIGINT";
        LimitNOFILE = "infinity";
        LimitNPROC = "infinity";
        Restart = "always";
        RestartSec = "20";
        TasksMax = "infinity";
      };

      enable = true;
    };


    # reload service on file change
    systemd.services.nomad-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/nomad-config.hcl -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl restart nomad.service || true'";
      };

      enable = true;
    };
    systemd.paths.nomad-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/nomad-config.hcl";
        Unit = "nomad-restart.service";
      };

      enable = true;
    };

    users.users.node-exp = {
        isSystemUser = true;
        description = "Vault service";
        extraGroups = ["keys"];
        group = "node-exp";
    };
    users.groups.node-exp = {};

    systemd.services.node_exporter = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" ];

      serviceConfig = {
        User = "node-exp";
        Group = "node-exp";
        Type = "simple";
        ExecStart = "${pkgs.prometheus-node-exporter}/bin/node_exporter" +
          " --collector.systemd" +
          " --collector.textfile" +
          " --collector.textfile.directory=/var/lib/node_exporter" +
          " --web.listen-address=10.17.0.10:9100" +
          " --web.telemetry-path=/metrics";
        Restart = "always";
        RestartSec = "1";
        SyslogIdentifier = "node_exporter";
        ProtectHome = "yes";
        NoNewPrivileges = "yes";
        ProtectSystem = "strict";
        ProtectControlGroups = "true";
        ProtectKernelModules = "true";
        ProtectKernelTunables = "yes";
      };

      enable = true;
    };

    systemd.services.cadvisor = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStart = "${pkgs.cadvisor}/bin/cadvisor" +
          " --listen_ip=10.17.0.10" +
          " --port=9280" +
          " --prometheus_endpoint=/metrics" +
          " --docker_only" +
          " --store_container_labels=false" +
          " --whitelisted_container_labels=com.hashicorp.nomad.job.name,com.hashicorp.nomad.node_name,com.hashicorp.nomad.namespace";
        Restart = "always";
        RestartSec = "1";
      };

      enable = true;
    };


    # reload service on file change
    systemd.services.vector-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/vector.toml -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl restart vector.service || true'";
      };

      enable = true;
    };
    systemd.paths.vector-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/vector.toml";
        Unit = "vector-restart.service";
      };

      enable = true;
    };

    users.users.vector = {
        isSystemUser = true;
        description = "Vector service";
        extraGroups = ["keys" "systemd-journal" "docker" "epl-prov" ];
        group = "vector";
    };
    users.groups.vector = {};

    systemd.services.vector = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        User = "vector";
        Group = "vector";
        Type = "simple";
        ExecStartPre = "${pkgs.vector}/bin/vector validate --config-toml=/run/keys/vector.toml";
        ExecStart = "${pkgs.vector}/bin/vector --threads=4 --config-toml=/run/keys/vector.toml";
        Restart = "always";
        RestartSec = "10";
      };

      enable = true;
    };

    networking.nftables.enable = true;

    systemd.services.epl-route-watcher = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];

      serviceConfig = {
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStart = "/run/current-system/sw/bin/epl-watch-route-data";
        Restart = "always";
        RestartSec = "10";
      };

      enable = true;
    };

    services.prometheus.exporters.zfs.enable = true;
    services.prometheus.exporters.zfs.port = 9134;

    networking.useDHCP = false;

    networking.interfaces.eth0.ipv4.addresses = [
      { address = "10.17.0.10"; prefixLength = 24; }

    ];

}

LilBoiPeepLikesBenzTruck
echo L1_EPL_PROVISIONING_ID > /etc/nixos/epl-prov-id
chown root:root /etc/nixos/configuration.nix
chmod 0600 /etc/nixos/configuration.nix
git add .
git commit -am 'Update L1_EPL_PROVISIONING_ID' || true
popd
if ! nixos-version | grep -E '^23.11'
then
  nix-channel --add https://channels.nixos.org/nixos-23.11 nixos
  nix-channel --update nixos
fi
nixos-rebuild switch || L1_PROVISIONING_TOLERATE_REBUILD_FAIL
generate_l1_secrets
L1_RESTART_CONSUL_POST_SECRETS && echo restarting consul after sleeping 10 seconds... && sleep 10 && systemctl restart consul.service || true

mkdir -m 700 -p /run/sec_volumes/ssl_certs
cp /run/keys/public_tls_key.pem /run/sec_volumes/ssl_certs/
cp /run/keys/public_tls_cert.pem /run/sec_volumes/ssl_certs/


# ------------- DNS START ---------------

function update_dns_file() {
  SOURCE_BASE64=$1
  TARGET_FILE=$2
  SOURCE_CHECKSUM=$3
  # Serial replacement will work only until exhausting
  # last number of 32 bit space for 42 years
  # we cannot provision more often than every
  # minute for different serials. We win time by subtracting
  # 23 years - year of when this line was written
  DATE=$( date +%y%m%d%H%M -d '23 years ago' )
  echo $SOURCE_BASE64 | \
    base64 -d | \
    sed "s/SERIAL_TO_REPLACE/$DATE/g" > $TARGET_FILE
  echo ";CHECKSUM $SOURCE_CHECKSUM" >> $TARGET_FILE
  chown named:named $TARGET_FILE
  chmod 644 $TARGET_FILE
}

function maybe_update_dns_file() {
  SOURCE_BASE64=$1
  TARGET_FILE=$2
  CHECKSUM=$( echo $SOURCE_BASE64 | base64 -d | sha256sum | awk '{print $1}' )
  if [ ! -f $TARGET_FILE ]
  then
     echo zone target $TARGET_FILE doesnt exist, installing to $TARGET_FILE
     update_dns_file $SOURCE_BASE64 $TARGET_FILE $CHECKSUM
     return 0
  fi
  if ! grep ";CHECKSUM $CHECKSUM" $TARGET_FILE
  then
     echo Source file changed, installing to $TARGET_FILE
     update_dns_file $SOURCE_BASE64 $TARGET_FILE $CHECKSUM
     return 0
  fi
}



# in first provisioning might not work
NAMED_UID=$( id -u named )
NAMED_GID=$( id -g named )

# prepare for dnssec
mkdir -p /run/named
chown $NAMED_UID:$NAMED_GID /run/named
chmod 700 /run/named
mkdir -p /run/dnsseckeys
chown $NAMED_UID:$NAMED_GID /run/dnsseckeys
chmod 700 /run/dnsseckeys







if [[ ! $( timeout 0.5s dig @127.0.0.1 server-a.us-west.epl-infra.net +dnssec +short ) ]]
then
    systemctl restart bind.service
fi

# ------------- DNS END   ---------------


# wait for consul to be up and running if restarted for up to 10 seconds
for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

export CONSUL_HTTP_TOKEN=$( cat /run/keys/consul-agent-token.txt )
for I in $(seq 1 5); do
  consul services register /run/keys/epl-node-exporter-service.hcl && break || true
  # try a few times if consul is down
  sleep 1
done


mkdir -p /var/lib/node_exporter
chown -R node-exp:node-exp /var/lib/node_exporter
chmod 700 /var/lib/node_exporter


# wait for consul to be up and running if restarted for up to 10 seconds
for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

export CONSUL_HTTP_TOKEN=$(cat /run/keys/consul-agent-token.txt)
for I in $(seq 1 5); do
  consul services register /run/keys/epl-cadvisor-service.hcl && break || true
  # try a few times if consul is down
  sleep 1
done


# wait for consul to be up and running if restarted for up to 10 seconds
for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

export CONSUL_HTTP_TOKEN=$( cat /run/keys/consul-agent-token.txt )
for I in $(seq 1 5); do
  consul services register /run/keys/epl-vector-service.hcl && break || true
  # try a few times if consul is down
  sleep 1
done


# create for vector
mkdir --mode 700 -p /var/lib/vector
chown vector:vector /var/lib/vector


# bootstrap inter dc routes
function epl_bootstrap_routes() {
    ORIG_ROUTES='
            # ROUTES CREATE
            ip route add 0.0.0.0/0 via 10.17.0.12

            # ROUTES DELETE
            ip route del 0.0.0.0/0

            # FINISH'
    echo -n '{"Value":"' > /run/epl-routes-tmp2
    echo -n "$ORIG_ROUTES" | base64 -w0 >> /run/epl-routes-tmp2
    echo -n '"}' >> /run/epl-routes-tmp2
    mv -f /run/epl-routes-tmp2 /run/epl-routes
    echo "$ORIG_ROUTES" | sed -n '/ROUTES CREATE/,/ROUTES DELETE/p' | sh || true
}

consul kv get epl-interdc-routes/dc1 || epl_bootstrap_routes


# wait for consul to be up and running if restarted for up to 10 seconds
for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

export CONSUL_HTTP_TOKEN=$(cat /run/keys/consul-agent-token.txt)
for I in $(seq 1 5); do
  consul services register /run/keys/epl-zfs-exporter-service.hcl && break || true
  # try a few times if consul is down
  sleep 1
done

rm -f /run/epl-l1-prov
rm -f /run/epl-l1-provisioning.lock

echo "
    UPDATE l1_provisionings
    SET exit_code = 0,
        time_ended = CURRENT_TIMESTAMP,
        is_finished = 1
    WHERE provisioning_id = L1_EPL_PROVISIONING_ID
" | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite


epl_l1_track_state 0

chmod 644 /var/log/epl-l1-prov/L1_EPL_PROVISIONING_ID.log

ThisIsEplProvL1Script
chmod 700 /run/epl-l1-prov
echo "SELECT 'running provisioning id is unfinished', provisioning_id FROM l1_provisionings WHERE is_finished = 0;" | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite | grep unfinished && exit 27 || true
echo 'INSERT INTO l1_provisionings(provisioning_id) VALUES (L1_EPL_PROVISIONING_ID);' | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite
tmux new-session -d '/run/epl-l1-prov |& tee /var/log/epl-l1-prov/L1_EPL_PROVISIONING_ID.log'
