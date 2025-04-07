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
      "agent": "edaf13c7-6efe-484d-a19a-639fb89a4186",
      "default": "f8823b41-8d84-4b0e-b72e-ea405aa42d2d",
      "initial_management": "085bc3a7-c50e-4091-b3a1-14e6b78a8486"
    }
  },
  "addresses": {
    "dns": "127.0.0.1",
    "grpc": "127.0.0.1",
    "http": "127.0.0.1",
    "https": "10.19.0.10"
  },
  "advertise_addr": "10.19.0.10",
  "advertise_addr_wan": "10.19.0.10",
  "auto_encrypt": {
    "allow_tls": true
  },
  "bind_addr": "10.19.0.10",
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
  "encrypt": "aOpV+IZzfh3euz5cfUD1nALYcKOquepYYE663TCrFdA=",
  "encrypt_verify_incoming": true,
  "encrypt_verify_outgoing": true,
  "limits": {
    "rpc_max_conns_per_client": 1000
  },
  "log_level": "INFO",
  "log_rotate_bytes": 0,
  "log_rotate_duration": "24h",
  "log_rotate_max_files": 0,
  "node_name": "server-e",
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
    "10.18.0.10",
    "10.19.0.10"
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


name = "server-e"
region = "us-west"
datacenter = "dc3"

enable_debug = false
disable_update_check = false


bind_addr = "10.19.0.10"
advertise {
    http = "10.19.0.10:4646"
    rpc = "10.19.0.10:4647"
    serf = "10.19.0.10:4648"
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
    token = "29fa2b93-6aaf-4ca1-935e-b102bf331d9b"
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
    cert_file = "/run/keys/nomad-client.crt"
    key_file = "/run/keys/nomad-client.key"
    rpc_upgrade_mode = false
    verify_server_hostname = "true"
    verify_https_client = "false"
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
    "private_ip" = "10.19.0.10"
    "run_unassigned_workloads" = "1"
    "lock_epl-ingress-us-west" = "1"
    "lock_epl-minio-server-e-global" = "1"
  }

  host_volume "minio-docker-e" {
    path = "/srv/volumes/minio-docker-e"
    read_only = false
  }

  host_volume "mon-am" {
    path = "/srv/volumes/mon-am"
    read_only = false
  }

  host_volume "nats1" {
    path = "/srv/volumes/nats1"
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
      address = "10.19.0.10"
      port    = 9100
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.19.0.10:9100/"
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
      address = "10.19.0.10"
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
        http     = "http://10.19.0.10:9280/"
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
      address = "10.19.0.10"
      port    = 9281
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "home"
        tcp      = "10.19.0.10:9281"
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
address = "10.19.0.10:9281"

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
host = "server-e.us-west.epl-infra.net"
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
host = "server-e.us-west.epl-infra.net"
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
host = "server-e.us-west.epl-infra.net"
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
host = "server-e.us-west.epl-infra.net"
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
      address = "10.19.0.10"
      port    = 9134
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.19.0.10:9134/"
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
zHbQrBAph4feyLtq3yWlXv8z2sRlKZPCgLsmu3WOsFE=
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
2cHKNM6ZDa+EonSiZ/8zv2Xko7o1SmXkbDLisuF2tB8=
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
MHcCAQEEIHZOhG7uAnYEQZuvRVkOdCHFdxJAkxjBOQX/sbbY2bwIoAoGCCqGSM49
AwEHoUQDQgAEE0EJpTjyQr7elx/cYnSTNiPLNUZNQbzzwHztX8L6gR1Hp6CGxxJw
WLXwN5n6KeiC8N4rshR6xSwlpy3uswxfOQ==
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
MIIB0zCCAXmgAwIBAgIUbiD8I8IyzT783hwO1XBOopPvP8cwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMTIxMDA4NDAwMFoXDTQwMTIwNTA4NDAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABBNBCaU48kK+3pcf3GJ0kzYj
yzVGTUG888B87V/C+oEdR6eghscScFi18DeZ+inogvDeK7IUesUsJact7rMMXzmj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSpna8r2/KxjqtMjNVYNjob4DJo
djAfBgNVHSMEGDAWgBS8xb/JxxGG4yHM0LOG3F6dddmgYTA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAx5ug99RrnV//6EsY/RV9GqxY2j/rKPD8Nfj3
+ECg3IoCICwhKh86NzitxL9Av43JMy1DVnjg5OUCKSHgaVtKCvHc
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

# NIX REGION secret_value_K10-in-addr-arpa--015-17699-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: nVSiiRR5UEfFghu0k9gxNp4mFjK5OvFgEqTtnUq78gQ=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-17699-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-17699-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-17699-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-17699-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-61551-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: Vb1AkHoaiIMgY7HEIP1i5pAnqJsHeXGtI9GxNjjsEVE=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-61551-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-61551-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-61551-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-61551-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-17699-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 17699, for 10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
10.in-addr.arpa. IN DNSKEY 256 3 15 4cwx9MRER5qB77SS46gUUN1vqPHv4B9h3qQVAobh4sw=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-17699-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-17699-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-17699-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-17699-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-61551-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 61551, for 10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
10.in-addr.arpa. IN DNSKEY 257 3 15 bl8iC8SU8SfruHIeH1Ae01G3Q32Itfzwpdyr8PO4qdw=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-61551-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-61551-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-61551-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-61551-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-43679-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 1UCPrdQIlVMn4LQtyRv4j8ABv02knha+Pe8sscZ9nX8=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-43679-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-43679-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-43679-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-43679-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-04193-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: LYZ/5s5qzsxcfgGTMx5BLjMNmJBsNO+gGFHhFd6b/SY=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-04193-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-04193-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-04193-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-04193-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-43679-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 43679, for 17.10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
17.10.in-addr.arpa. IN DNSKEY 256 3 15 gA1gMTN7+x3VKyEwqCiAClI7Q5S4GjwPTYKtqLfFPD8=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-43679-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-43679-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-43679-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-43679-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-04193-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 4193, for 17.10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
17.10.in-addr.arpa. IN DNSKEY 257 3 15 drvbe3R6GQMoQxeQDMOyNilYpl7s/o/khjJXzlyfppQ=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-04193-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-04193-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-04193-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-04193-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-37739-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: PxO250/oixDMWeFR6koEedThz9OjsQBr5wJfak6R7d8=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-37739-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-37739-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-37739-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-37739-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-16507-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: BvlhoKRRPue0NNwZTjU+xUYukv6eFLNipCaxUoEGg8s=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-16507-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-16507-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-16507-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-16507-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-37739-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 37739, for 18.10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
18.10.in-addr.arpa. IN DNSKEY 256 3 15 wiEWyXK8dDYih9m/ErakAOzBwwN70kSgEZhMBp6GsCM=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-37739-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-37739-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-37739-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-37739-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-16507-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 16507, for 18.10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
18.10.in-addr.arpa. IN DNSKEY 257 3 15 sr71+iR3W4xAD7UfCjM2Fgo8hpynY/YhzAd4nKdBw/E=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-16507-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-16507-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-16507-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-16507-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-64643-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: LMQwHRXDcvP0aJZKmLigqMep3oHqWc+XxjNts4OWIiU=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-64643-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-64643-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-64643-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-64643-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56398-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 6MeUa3QDLfHI6tMVI93+RlayEFjDG5dVvjSdxvIL7NI=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56398-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56398-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-56398-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56398-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-64643-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 64643, for 19.10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
19.10.in-addr.arpa. IN DNSKEY 256 3 15 lPwdK6+YZ+GTLleMNMGLeIHhJEG7TCZn2QcM6QvEClM=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-64643-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-64643-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-64643-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-64643-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56398-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 56398, for 19.10.in-addr.arpa.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
19.10.in-addr.arpa. IN DNSKEY 257 3 15 +SVbqQWacRqV6CHGgPyK8NJ98udi5fMrPsUz2MXT9TY=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56398-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56398-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-56398-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56398-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-35219-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: X8W3hSHHmc5pUz7kS2OHm1S4RwgJM6CpX3ahW/nR8xM=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-35219-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-35219-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-35219-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-35219-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-58038-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: GeV3zLJNovQCnJA9+EBY4puO0AlQK2VZzeR2+Flk6gM=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-58038-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-58038-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-58038-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-58038-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-35219-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 35219, for epl-infra.net.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
epl-infra.net. IN DNSKEY 256 3 15 AcGnOInPReGMI5NRL6iwH5b0JvTc7s1H6LfqWhr0t3Y=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-35219-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-35219-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-35219-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-35219-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-58038-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 58038, for epl-infra.net.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
epl-infra.net. IN DNSKEY 257 3 15 RdMqIAn9Igdoj1/WuaS1Ax5GrmWBgj0BYP5k+k/HarE=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-58038-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-58038-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-58038-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-58038-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-24870-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: tGkpMGY06xZjR9GcmY70oz3IffAvaVgFerBrb5uGeSI=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-24870-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-24870-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-24870-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-24870-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-48961-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: /Juy/+v90o9r4Rp3om3I8AvuJtDgVQekcHnsSGMXYFI=
Created: 20231210084433
Publish: 20231210084433
Activate: 20231210084433
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-48961-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-48961-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-48961-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-48961-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-24870-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 24870, for us-west.epl-infra.net.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
us-west.epl-infra.net. IN DNSKEY 256 3 15 tRdH0VJXoS9B3K7vnicKJwYJd52Fj63rX78McOGU1KY=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-24870-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-24870-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-24870-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-24870-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-48961-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 48961, for us-west.epl-infra.net.
; Created: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Publish: 20231210084433 (Sun Dec 10 10:44:33 2023)
; Activate: 20231210084433 (Sun Dec 10 10:44:33 2023)
us-west.epl-infra.net. IN DNSKEY 257 3 15 LEXLv/EILvpz18YApP3LSwFg5Btgo2NLhzf1eozDRic=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-48961-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-48961-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-48961-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-48961-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-ca-cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIC7DCCApOgAwIBAgIQTG+xxul0oTyDrwn76tA23zAKBggqhkjOPQQDAjCBuTEL
MAkGA1UEBhMCVVMxCzAJBgNVBAgTAkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2Nv
MRowGAYDVQQJExExMDEgU2Vjb25kIFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAV
BgNVBAoTDkhhc2hpQ29ycCBJbmMuMUAwPgYDVQQDEzdDb25zdWwgQWdlbnQgQ0Eg
MTAxNjAxMjc4Mzc2ODc4MDAwNDc4MjA2MDAyMjA5MDk1Njk0MDQ3MB4XDTIzMTIx
MDA4NDQzNFoXDTQwMTIwNTA4NDQzNFowgbkxCzAJBgNVBAYTAlVTMQswCQYDVQQI
EwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNjbzEaMBgGA1UECRMRMTAxIFNlY29u
ZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcwFQYDVQQKEw5IYXNoaUNvcnAgSW5j
LjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENBIDEwMTYwMTI3ODM3Njg3ODAwMDQ3
ODIwNjAwMjIwOTA5NTY5NDA0NzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABHfc
0iOhg1QVN2OIq3Qb2WhX9Y3rHLrNdHeG0Xz/n2ltHLAauiKWalxgOXRbBmAtEJUk
UHRFJDzWJrMZTmeueB2jezB5MA4GA1UdDwEB/wQEAwIBhjAPBgNVHRMBAf8EBTAD
AQH/MCkGA1UdDgQiBCCztbgjUDgNOMTEynYTcSWobb2CBiLBLFLdKrhj6VungDAr
BgNVHSMEJDAigCCztbgjUDgNOMTEynYTcSWobb2CBiLBLFLdKrhj6VungDAKBggq
hkjOPQQDAgNHADBEAiBVqXP2mG8KbrBo+S3kYmBs4TUVzzZkzrS67HPwipsXrAIg
NaAH90rh5KxmSBaRZnYZkqXVeiCS4xcG7r3ZZfl+5iE=
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
edaf13c7-6efe-484d-a19a-639fb89a4186
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
MIICxTCCAmugAwIBAgIRAJwsRj3X2+hGhm/G5FkAv7owCgYIKoZIzj0EAwIwgbkx
CzAJBgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNj
bzEaMBgGA1UECRMRMTAxIFNlY29uZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcw
FQYDVQQKEw5IYXNoaUNvcnAgSW5jLjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENB
IDEwMTYwMTI3ODM3Njg3ODAwMDQ3ODIwNjAwMjIwOTA5NTY5NDA0NzAeFw0yNTA0
MDcxNDQ2MTRaFw0yNjA0MDcxNDQ2MTRaMCAxHjAcBgNVBAMTFXNlcnZlci51cy13
ZXN0LmNvbnN1bDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABN6VF+D8Oj6sv8O8
meHsOqGh5wAe9GpJ6bNfHORiaizoZtsCoPlIvKhzi4QDpBKLMU2Kd6Xb3FIsbCvu
WAMbeCyjgeswgegwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMB
BggrBgEFBQcDAjAMBgNVHRMBAf8EAjAAMCkGA1UdDgQiBCALldzwNDgzh/gEcb8U
H/L1/mjfoy/+EeW2D8pWBh2NUzArBgNVHSMEJDAigCCztbgjUDgNOMTEynYTcSWo
bb2CBiLBLFLdKrhj6VungDBRBgNVHREESjBIgh5zZXJ2ZXItZS5zZXJ2ZXIudXMt
d2VzdC5jb25zdWyCFXNlcnZlci51cy13ZXN0LmNvbnN1bIIJbG9jYWxob3N0hwR/
AAABMAoGCCqGSM49BAMCA0gAMEUCIQC2f7LG09II0hJXbYUB4atsydbY/QrXWizm
Rm6r/PPgvQIgN7uNPOYw/fSN1/qUFfrdKyJecLJvZTbY2Va6kQcFoEE=
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
MHcCAQEEIHnnrvAbRrK5GK7+FUkbdqXTZ3b1GknvTOHb4VYltUOtoAoGCCqGSM49
AwEHoUQDQgAE3pUX4Pw6Pqy/w7yZ4ew6oaHnAB70aknps18c5GJqLOhm2wKg+Ui8
qHOLhAOkEosxTYp3pdvcUixsK+5YAxt4LA==
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
085bc3a7-c50e-4091-b3a1-14e6b78a8486
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
f8823b41-8d84-4b0e-b72e-ea405aa42d2d
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
d976a8ee-0707-47af-ae35-d6c9ec1fbab8
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
13eaa54f-9169-43ed-8aa3-fb36216c2f35
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

# NIX REGION secret_value_consul-vrrp-token-dc2.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
a3284ce6-6602-4eba-96c2-714f8cc022ba
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-vrrp-token-dc2.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-vrrp-token-dc2.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-vrrp-token-dc2.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-vrrp-token-dc2.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-vrrp-token-dc3.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
01f87256-fc47-4c01-bedb-bef43a4f7d8d
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-vrrp-token-dc3.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-vrrp-token-dc3.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-vrrp-token-dc3.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-vrrp-token-dc3.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-server-consul-acl-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
8405a59d-9bc0-4705-abcf-dac84f8b8ae8
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
29fa2b93-6aaf-4ca1-935e-b102bf331d9b
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
MIIBZTCCAQqgAwIBAgIUA1Rf+QzWDRp7TOCcm77pJrufaX0wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABMEG
BMbE77RYrk44Sx6N0iRvrDemC60NFF5mSOmqd5ISiL9HnmxSesSuLUD2CimRonBa
b3CwHUXc19fCUIUvcZmjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSIaEmGc1TZqroZHSDON2TLjgFazDAKBggqhkjOPQQDAgNJ
ADBGAiEA96Kbui7gZAtmLFWC25/SLeYWtLmhHhiX/SX8bviWtTMCIQDv0h32ruvR
d8U8yrMaNQ7XFbDBnHeoKbiIg7t/kww1kQ==
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

# NIX REGION secret_value_nomad-client.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBxzCCAW6gAwIBAgIUELKSRUHgTYJGdF9+syXBn63SC1gwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjUwNDA3MTQ0MTAwWhcNMjYwNDA3MTQ0MTAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEKVfFEGMdyUCe5p38JUVgmVSh
fAEZyxwY+PYOLFQwzwm4nPJf5zpRZvtxRU2dFlwyq+NrjN4mc0zoRWfNjPi6ZqOB
tTCBsjAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFFRK8yuaTcziyputtq9UaOa9x1Of
MB8GA1UdIwQYMBaAFIhoSYZzVNmquhkdIM43ZMuOAVrMMDMGA1UdEQEB/wQpMCeC
FGNsaWVudC51cy13ZXN0Lm5vbWFkgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0E
AwIDRwAwRAIgdBVDbAWAik8G3/M1qoDkqmElGf7fLnJYbQDQ8IR2V5ICIHGxpCVF
tcL8bOoVXgB2CTs+Gz05qi17cbqs1Nf/l5V8
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-client.crt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-client.crt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-client.crt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-client.crt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_nomad-client.key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIBk4L5If23Ts2xYcmj7tG3Y8vFyNe3SLlwbgE/U4pwe7oAoGCCqGSM49
AwEHoUQDQgAEKVfFEGMdyUCe5p38JUVgmVShfAEZyxwY+PYOLFQwzwm4nPJf5zpR
ZvtxRU2dFlwyq+NrjN4mc0zoRWfNjPi6Zg==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_nomad-client.key END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/nomad-client.key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-client.key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-client.key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_vault-ca.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUEG/FUYCY2e7t5RpR2Fiob2DRYWgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABO0S
uQyFy64bbzPnt8SXlEMMG5F7w6bK3c+7WbhDlmxdtL7G5T4F0jQZa9tYMzZWdPJy
bcFk/D0d+njRx2dfEFujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQro1VtoctSvekfgXtFOeiOXNn21zAKBggqhkjOPQQDAgNJ
ADBGAiEAwJ8BoWndiIp6UTQg10YI3dUj0OBMlk3EbODNSaBi894CIQDvMfK6uu0c
vtgvVueNMmbOlTGoFOi0xZjX2tK3KVmbRg==
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
cb914c84-309a-4a4d-9a07-2433e4172bbe
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_vault-service-consul-acl-token.txt END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/vault-service-consul-acl-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vault-service-consul-acl-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vault-service-consul-acl-token.txt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_epl-wireguard-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
GARa3AQkQxtUOaH1OVuMBLJZLCn+zeW/WixQkpVrp2E=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-wireguard-key END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-wireguard-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-wireguard-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-wireguard-key
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
  touch /run/restart-bind
}

function maybe_update_dns_file() {
  SOURCE_BASE64=$1
  TARGET_FILE=$2
  CHECKSUM=$( echo $SOURCE_BASE64 | base64 -d | sha256sum | awk '{print $1}' )

  # bind journalfile will clash with zone file, we have the source
  # so journalfile is irrelevant for us
  if [ -f "$TARGET_FILE.jnl" ]
  then
    # just delete journal file as under normal circumstances it is not needed
    # only for acme update keys
    rm -f $TARGET_FILE.jnl
    touch /run/restart-bind
  fi

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


# first installation bind not installed yet
if id named
then


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


# $TTL 3600
# epl-infra.net.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# epl-infra.net.	IN	NS	ns1.epl-infra.net.
# epl-infra.net.	IN	NS	ns2.epl-infra.net.
# epl-infra.net.	IN	NS	ns3.epl-infra.net.
# ns1.epl-infra.net.	IN	A	10.19.0.10
# ns2.epl-infra.net.	IN	A	10.17.0.11
# ns3.epl-infra.net.	IN	A	10.18.0.10
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns3.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.19.0.10
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.11
# ns3.us-west.epl-infra.net.	IN	A	10.18.0.10
# us-west.epl-infra.net.	IN	DS	48961 15 2 17535CBEB01A009AB3D9D28505D49313C7EF731E66F7EB84B1E2EEBCA000EC4A
#
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTkuMC4xMApuczIuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xNy4wLjExCm5zMy5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOS4wLjEwCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMQpuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglEUwk0ODk2MSAxNSAyIDE3NTM1Q0JFQjAxQTAwOUFCM0Q5RDI4NTA1RDQ5MzEzQzdFRjczMUU2NkY3RUI4NEIxRTJFRUJDQTAwMEVDNEEKCg== /run/named/private-epl-infra.net.zone
# $TTL 3600
# 10.in-addr.arpa.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 10.in-addr.arpa.	IN	NS	ns1.epl-infra.net.
# 10.in-addr.arpa.	IN	NS	ns2.epl-infra.net.
# 10.in-addr.arpa.	IN	NS	ns3.epl-infra.net.
#
# 10.0.18.10.in-addr.arpa.	IN	PTR	ns3.epl-infra.net.
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns1.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	ns2.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMTAuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczMuZXBsLWluZnJhLm5ldC4KMTAuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEuZXBsLWluZnJhLm5ldC4KMTEuMC4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIuZXBsLWluZnJhLm5ldC4KMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4K /run/named/private-10.in-addr.arpa.zone
# $TTL 3600
# us-west.epl-infra.net.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns3.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.19.0.10
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.11
# ns3.us-west.epl-infra.net.	IN	A	10.18.0.10
# server-a.us-west.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-west.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-west.epl-infra.net.	IN	A	10.18.0.10
# server-d.us-west.epl-infra.net.	IN	A	10.18.0.11
# server-e.us-west.epl-infra.net.	IN	A	10.19.0.10
# server-f.us-west.epl-infra.net.	IN	A	10.19.0.11
maybe_update_dns_file JFRUTCAzNjAwCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOS4wLjEwCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMQpuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWQudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTEKc2VydmVyLWUudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKc2VydmVyLWYudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTEK /run/named/private-us-west.epl-infra.net.zone
# $TTL 3600
# 17.10.in-addr.arpa.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
#
# 10.0.17.10.in-addr.arpa.	IN	PTR	server-a.us-west.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	ns2.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTcuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMS4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-17.10.in-addr.arpa.zone
# $TTL 3600
# 18.10.in-addr.arpa.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 18.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
#
# 10.0.18.10.in-addr.arpa.	IN	PTR	ns3.us-west.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	server-d.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZC51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-18.10.in-addr.arpa.zone
# $TTL 3600
# 19.10.in-addr.arpa.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 19.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
#
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns1.us-west.epl-infra.net.
# 11.0.19.10.in-addr.arpa.	IN	PTR	server-f.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTkuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-19.10.in-addr.arpa.zone
# $TTL 3600
# epl-infra.net.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# epl-infra.net.	IN	NS	ns1.epl-infra.net.
# epl-infra.net.	IN	NS	ns2.epl-infra.net.
# epl-infra.net.	IN	NS	ns3.epl-infra.net.
# ns1.epl-infra.net.	IN	A	77.77.77.14
# ns2.epl-infra.net.	IN	A	77.77.77.11
# ns3.epl-infra.net.	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.11
# adm-alertmanager-default	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.14
# adm-alertmanager-default	IN	A	77.77.77.15
# adm-consul-us-west	IN	A	77.77.77.11
# adm-consul-us-west	IN	A	77.77.77.12
# adm-consul-us-west	IN	A	77.77.77.14
# adm-consul-us-west	IN	A	77.77.77.15
# adm-grafana-main	IN	A	77.77.77.11
# adm-grafana-main	IN	A	77.77.77.12
# adm-grafana-main	IN	A	77.77.77.14
# adm-grafana-main	IN	A	77.77.77.15
# adm-minio-global	IN	A	77.77.77.11
# adm-minio-global	IN	A	77.77.77.12
# adm-minio-global	IN	A	77.77.77.14
# adm-minio-global	IN	A	77.77.77.15
# adm-nomad-us-west	IN	A	77.77.77.11
# adm-nomad-us-west	IN	A	77.77.77.12
# adm-nomad-us-west	IN	A	77.77.77.14
# adm-nomad-us-west	IN	A	77.77.77.15
# adm-prometheus-default	IN	A	77.77.77.11
# adm-prometheus-default	IN	A	77.77.77.12
# adm-prometheus-default	IN	A	77.77.77.14
# adm-prometheus-default	IN	A	77.77.77.15
# adm-vault-us-west	IN	A	77.77.77.11
# adm-vault-us-west	IN	A	77.77.77.12
# adm-vault-us-west	IN	A	77.77.77.14
# adm-vault-us-west	IN	A	77.77.77.15
# admin	IN	A	77.77.77.11
# admin	IN	A	77.77.77.12
# admin	IN	A	77.77.77.14
# admin	IN	A	77.77.77.15
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJNzcuNzcuNzcuMTQKbnMyLmVwbC1pbmZyYS5uZXQuCUlOCUEJNzcuNzcuNzcuMTEKbnMzLmVwbC1pbmZyYS5uZXQuCUlOCUEJNzcuNzcuNzcuMTIKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTEKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTQKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTUKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTEKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTQKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTUKYWRtLWdyYWZhbmEtbWFpbglJTglBCTc3Ljc3Ljc3LjExCmFkbS1ncmFmYW5hLW1haW4JSU4JQQk3Ny43Ny43Ny4xMgphZG0tZ3JhZmFuYS1tYWluCUlOCUEJNzcuNzcuNzcuMTQKYWRtLWdyYWZhbmEtbWFpbglJTglBCTc3Ljc3Ljc3LjE1CmFkbS1taW5pby1nbG9iYWwJSU4JQQk3Ny43Ny43Ny4xMQphZG0tbWluaW8tZ2xvYmFsCUlOCUEJNzcuNzcuNzcuMTIKYWRtLW1pbmlvLWdsb2JhbAlJTglBCTc3Ljc3Ljc3LjE0CmFkbS1taW5pby1nbG9iYWwJSU4JQQk3Ny43Ny43Ny4xNQphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjExCmFkbS1ub21hZC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLW5vbWFkLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xNAphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjE1CmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMQphZG0tcHJvbWV0aGV1cy1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLXByb21ldGhldXMtZGVmYXVsdAlJTglBCTc3Ljc3Ljc3LjE0CmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xNQphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjExCmFkbS12YXVsdC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLXZhdWx0LXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xNAphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjE1CmFkbWluCUlOCUEJNzcuNzcuNzcuMTEKYWRtaW4JSU4JQQk3Ny43Ny43Ny4xMgphZG1pbglJTglBCTc3Ljc3Ljc3LjE0CmFkbWluCUlOCUEJNzcuNzcuNzcuMTUK /run/named/public-epl-infra.net.zone
# $TTL 3600
# in-addr.arpa.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# in-addr.arpa.	IN	NS	ns1.epl-infra.net.
# in-addr.arpa.	IN	NS	ns2.epl-infra.net.
# in-addr.arpa.	IN	NS	ns3.epl-infra.net.
#
# 11.77.77.77.in-addr.arpa.	IN	PTR	admin
# 12.77.77.77.in-addr.arpa.	IN	PTR	admin
# 14.77.77.77.in-addr.arpa.	IN	PTR	admin
# 15.77.77.77.in-addr.arpa.	IN	PTR	admin
maybe_update_dns_file JFRUTCAzNjAwCmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQppbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMTEuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTIuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTQuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTUuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4K /run/named/public-in-addr.arpa.zone


fi
# ------------- DNS END   ---------------


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
provision_zfs_dataset rpool acme /var/lib/acme 128k on no 1q1QE28jQkhLqUh7wwwv28cRFcJXMraTbdPcRzLbfC
provision_zfs_dataset rpool consul /var/lib/consul 4k on no LhtK4GiwKX9k6gBdUo0mz9TvVOa149crhebbQdTRAq
provision_zfs_dataset rpool docker /var/lib/docker 128k on no
provision_zfs_dataset rpool minio-docker-e /srv/volumes/minio-docker-e 1M on yes 28pTbWmPoN0TxE24OxB4qaiIfZ8eRsLnZUWm8ToaAG
provision_zfs_dataset rpool mon-am /srv/volumes/mon-am 4k on yes hzBXmFrt9VvldXrFZj8y7n85r8I5TvOGqEkXKdpKwl
provision_zfs_dataset rpool nats1 /srv/volumes/nats1 4k on yes 66kce6nQIwcwLWWKXFWjR8UfdTn4obk3yO9YxwjST2

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
        "epl-nix-cache:TKuTt1vtCbpAtW6YmzN2doZvTsoPunGMpXZzh2nAu2Q="
      ];

    };

    networking.hostId = "b32a6c8e";


    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIWH2F//Ff/MIYhKzwx+CYP3wJ5h9/h+VMQkk/uyKfo+ epl-root-ssh-key"
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
      HISTCONTROL = "ignoreboth";
      NOMAD_ADDR = "https://nomad-servers.service.consul:4646";
      VAULT_ADDR = "https://vault.service.consul:8200";
    };

    security.pki.certificates = [
      ''-----BEGIN CERTIFICATE-----
MIIB0zCCAXmgAwIBAgIUbiD8I8IyzT783hwO1XBOopPvP8cwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMTIxMDA4NDAwMFoXDTQwMTIwNTA4NDAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABBNBCaU48kK+3pcf3GJ0kzYj
yzVGTUG888B87V/C+oEdR6eghscScFi18DeZ+inogvDeK7IUesUsJact7rMMXzmj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSpna8r2/KxjqtMjNVYNjob4DJo
djAfBgNVHSMEGDAWgBS8xb/JxxGG4yHM0LOG3F6dddmgYTA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAx5ug99RrnV//6EsY/RV9GqxY2j/rKPD8Nfj3
+ECg3IoCICwhKh86NzitxL9Av43JMy1DVnjg5OUCKSHgaVtKCvHc
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUA1Rf+QzWDRp7TOCcm77pJrufaX0wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABMEG
BMbE77RYrk44Sx6N0iRvrDemC60NFF5mSOmqd5ISiL9HnmxSesSuLUD2CimRonBa
b3CwHUXc19fCUIUvcZmjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSIaEmGc1TZqroZHSDON2TLjgFazDAKBggqhkjOPQQDAgNJ
ADBGAiEA96Kbui7gZAtmLFWC25/SLeYWtLmhHhiX/SX8bviWtTMCIQDv0h32ruvR
d8U8yrMaNQ7XFbDBnHeoKbiIg7t/kww1kQ==
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUEG/FUYCY2e7t5RpR2Fiob2DRYWgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABO0S
uQyFy64bbzPnt8SXlEMMG5F7w6bK3c+7WbhDlmxdtL7G5T4F0jQZa9tYMzZWdPJy
bcFk/D0d+njRx2dfEFujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQro1VtoctSvekfgXtFOeiOXNn21zAKBggqhkjOPQQDAgNJ
ADBGAiEAwJ8BoWndiIp6UTQg10YI3dUj0OBMlk3EbODNSaBi894CIQDvMfK6uu0c
vtgvVueNMmbOlTGoFOi0xZjX2tK3KVmbRg==
-----END CERTIFICATE-----
''
    ];

    environment.systemPackages =
      let
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

      in
      [
        pkgs.bmon
        pkgs.cadvisor
        pkgs.curl
        pkgs.dig
        pkgs.git
        pkgs.google-cloud-sdk
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
        pkgs.wireguard-tools
        pkgs.zstd
        epl-consul-bootstrap
        epl-consul-vrrp-acl
        epl-nomad-acl-bootstrap
        epl-nomad-acl-policies
        epl-nomad-consul-acl-bootstrap
        epl-nomad-vault-policies
        epl-vault-consul-acl-bootstrap
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "0.0.0.0"; prefixLength = 0; via = "10.19.0.1"; }

    ];

# NIX REGION static_node_routes END

    boot.kernel.sysctl = {
      # for loki ScyllaDB
      "fs.aio-max-nr" = 1048576;
    };

# NIX REGION epl_nft_rules_epl-internet-fw START

            networking.nftables.tables.epl-internet-fw = {
              family = "ip";
              content = ''

       chain EPL_INTERNET_FIREWALL {
           type filter hook prerouting priority mangle + 20; policy accept;
           iifname void ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 77.77.77.14/32 } drop comment "Disallow traffic from internet to internal networks";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-internet-fw END

# NIX REGION firewall START

  networking.hostName = "server-e";
  networking.firewall.allowPing = true;
  networking.firewall.enable = true;
  networking.firewall.checkReversePath = false;
  networking.firewall.trustedInterfaces = [

    "eth0"

    "wg0"

    "vpnGre"

  ];

  networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];

  networking.firewall.allowedUDPPorts = [ 53 51820 ];

# NIX REGION firewall END

   programs.bash.promptInit = ''
     # Provide a nice prompt if the terminal supports it.
     if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
       PROMPT_COLOR="1;31m"
       ((UID)) && PROMPT_COLOR="1;32m"
       if [ -n "$INSIDE_EMACS" ]; then
         # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-e.dc3.us-west.gcloud-multi-dc:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-e.dc3.us-west.gcloud-multi-dc: \w\a\]\u@server-e.dc3.us-west.gcloud-multi-dc:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-e.dc3.us-west.gcloud-multi-dc:\u:\w\007\]$PS1"
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
         while ! ${pkgs.consul}/bin/consul kv get epl-l1-plans/server-e
         do
           sleep 7
         done

         ${pkgs.consul}/bin/consul watch \
           -type=key -key=epl-l1-plans/server-e \
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
# NIX REGION custom_hardware START
    imports = [ "${modulesPath}/virtualisation/google-compute-image.nix" ];


  services.zfs.expandOnBoot = "all";

  fileSystems."/" =
    # force because google-compute-config.nix makes it ext4
    pkgs.lib.mkForce
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
# NIX REGION custom_hardware END
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
  epl-infra.net. initial-key 257 3 15 "RdMqIAn9Igdoj1/WuaS1Ax5GrmWBgj0BYP5k+k/HarE=";
  us-west.epl-infra.net. initial-key 257 3 15 "LEXLv/EILvpz18YApP3LSwFg5Btgo2NLhzf1eozDRic=";
  10.in-addr.arpa. initial-key 257 3 15 "bl8iC8SU8SfruHIeH1Ae01G3Q32Itfzwpdyr8PO4qdw=";
  17.10.in-addr.arpa. initial-key 257 3 15 "drvbe3R6GQMoQxeQDMOyNilYpl7s/o/khjJXzlyfppQ=";
  18.10.in-addr.arpa. initial-key 257 3 15 "sr71+iR3W4xAD7UfCjM2Fgo8hpynY/YhzAd4nKdBw/E=";
  19.10.in-addr.arpa. initial-key 257 3 15 "+SVbqQWacRqV6CHGgPyK8NJ98udi5fMrPsUz2MXT9TY=";
  in-addr.arpa. initial-key 257 3 15 "HGNILHENJtwtEUPeRwTJg5pXPxg8fWiW5kcbuEtfkh8=";
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


zone "epl-infra.net." {
  type master;
  file "/run/named/private-epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.11;
    10.18.0.10;
  };
};
zone "10.in-addr.arpa." {
  type master;
  file "/run/named/private-10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.11;
    10.18.0.10;
  };
};
zone "us-west.epl-infra.net." {
  type master;
  file "/run/named/private-us-west.epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.11;
    10.18.0.10;
  };
};
zone "17.10.in-addr.arpa." {
  type master;
  file "/run/named/private-17.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.11;
    10.18.0.10;
  };
};
zone "18.10.in-addr.arpa." {
  type master;
  file "/run/named/private-18.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.11;
    10.18.0.10;
  };
};
zone "19.10.in-addr.arpa." {
  type master;
  file "/run/named/private-19.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.11;
    10.18.0.10;
  };
};

};

view internet {
          match-clients { any; };
          recursion no;
zone "epl-infra.net." {
  type master;
  file "/run/named/public-epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    77.77.77.11;
    77.77.77.12;
  };
};
zone "in-addr.arpa." {
  type master;
  file "/run/named/public-in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    77.77.77.11;
    77.77.77.12;
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
          " --web.listen-address=10.19.0.10:9100" +
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
          " --listen_ip=10.19.0.10" +
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

# NIX REGION wireguard_configs START

  systemd.services.wireguard-wg0 = {
    description = "WireGuard Tunnel - wg0";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    environment.DEVICE = "wg0";
    path = with pkgs; [ kmod iproute2 wireguard-tools ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    };

    script = ''

modprobe wireguard || true
ip link add dev "wg0" type wireguard

# this might fail as kernel seems to remember ip address from previously
ip address add "172.21.7.14/16" dev "wg0" || true
wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
ip link set up dev "wg0"

# peers
wg set wg0 peer "z/dz9TXusSGuvEuMgN9uZ7KIAURxbWA4hJBhaF/7ygw=" allowed-ips "172.21.7.254/32"

    '';

    postStop = ''
      ip link del dev "wg0"
    '';
  };

# NIX REGION wireguard_configs END

# NIX REGION epl_nft_rules_epl-nat START

            networking.nftables.tables.epl-nat = {
              family = "ip";
              content = ''

       chain EPL_POSTROUTING {
           type nat hook postrouting priority 0;

               ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
               ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-nat END

    boot.kernelModules = [ "gre" ];

# NIX REGION l3_vpn_hop_interface START

  systemd.services.vpn-gre-tunnel = {
    description = "VPN GRE Tunnel - vpnGre";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    path = with pkgs; [ kmod iproute2 ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    };

    script = ''
        ip tunnel add vpnGre mode gre local 10.19.0.10 key 17
        ip addr add 10.19.128.10/17 dev vpnGre



        ip link set dev vpnGre up
    '';

    postStop = ''
      ip link del dev vpnGre
    '';
  };

# NIX REGION l3_vpn_hop_interface END

# NIX REGION epl_nft_rules_l3-vpn-hop-address-translation START

            networking.nftables.tables.l3-vpn-hop-address-translation = {
              family = "ip";
              content = ''

        chain PREROUTING {
            type filter hook prerouting priority -300; policy accept;

        }

              '';
            };

# NIX REGION epl_nft_rules_l3-vpn-hop-address-translation END

    services.prometheus.exporters.zfs.enable = true;
    services.prometheus.exporters.zfs.port = 9134;

    networking.useDHCP = false;

    networking.interfaces.eth0.ipv4.addresses = [
      { address = "10.19.0.10"; prefixLength = 24; }

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
  touch /run/restart-bind
}

function maybe_update_dns_file() {
  SOURCE_BASE64=$1
  TARGET_FILE=$2
  CHECKSUM=$( echo $SOURCE_BASE64 | base64 -d | sha256sum | awk '{print $1}' )

  # bind journalfile will clash with zone file, we have the source
  # so journalfile is irrelevant for us
  if [ -f "$TARGET_FILE.jnl" ]
  then
    # just delete journal file as under normal circumstances it is not needed
    # only for acme update keys
    rm -f $TARGET_FILE.jnl
    touch /run/restart-bind
  fi

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


# $TTL 3600
# epl-infra.net.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# epl-infra.net.	IN	NS	ns1.epl-infra.net.
# epl-infra.net.	IN	NS	ns2.epl-infra.net.
# epl-infra.net.	IN	NS	ns3.epl-infra.net.
# ns1.epl-infra.net.	IN	A	10.19.0.10
# ns2.epl-infra.net.	IN	A	10.17.0.11
# ns3.epl-infra.net.	IN	A	10.18.0.10
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns3.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.19.0.10
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.11
# ns3.us-west.epl-infra.net.	IN	A	10.18.0.10
# us-west.epl-infra.net.	IN	DS	48961 15 2 17535CBEB01A009AB3D9D28505D49313C7EF731E66F7EB84B1E2EEBCA000EC4A
#
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTkuMC4xMApuczIuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xNy4wLjExCm5zMy5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOS4wLjEwCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMQpuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglEUwk0ODk2MSAxNSAyIDE3NTM1Q0JFQjAxQTAwOUFCM0Q5RDI4NTA1RDQ5MzEzQzdFRjczMUU2NkY3RUI4NEIxRTJFRUJDQTAwMEVDNEEKCg== /run/named/private-epl-infra.net.zone
# $TTL 3600
# 10.in-addr.arpa.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 10.in-addr.arpa.	IN	NS	ns1.epl-infra.net.
# 10.in-addr.arpa.	IN	NS	ns2.epl-infra.net.
# 10.in-addr.arpa.	IN	NS	ns3.epl-infra.net.
#
# 10.0.18.10.in-addr.arpa.	IN	PTR	ns3.epl-infra.net.
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns1.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	ns2.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMTAuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczMuZXBsLWluZnJhLm5ldC4KMTAuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEuZXBsLWluZnJhLm5ldC4KMTEuMC4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIuZXBsLWluZnJhLm5ldC4KMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4K /run/named/private-10.in-addr.arpa.zone
# $TTL 3600
# us-west.epl-infra.net.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns3.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.19.0.10
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.11
# ns3.us-west.epl-infra.net.	IN	A	10.18.0.10
# server-a.us-west.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-west.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-west.epl-infra.net.	IN	A	10.18.0.10
# server-d.us-west.epl-infra.net.	IN	A	10.18.0.11
# server-e.us-west.epl-infra.net.	IN	A	10.19.0.10
# server-f.us-west.epl-infra.net.	IN	A	10.19.0.11
maybe_update_dns_file JFRUTCAzNjAwCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOS4wLjEwCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMQpuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWQudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTEKc2VydmVyLWUudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKc2VydmVyLWYudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTEK /run/named/private-us-west.epl-infra.net.zone
# $TTL 3600
# 17.10.in-addr.arpa.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
#
# 10.0.17.10.in-addr.arpa.	IN	PTR	server-a.us-west.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	ns2.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTcuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMS4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-17.10.in-addr.arpa.zone
# $TTL 3600
# 18.10.in-addr.arpa.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 18.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
#
# 10.0.18.10.in-addr.arpa.	IN	PTR	ns3.us-west.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	server-d.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZC51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-18.10.in-addr.arpa.zone
# $TTL 3600
# 19.10.in-addr.arpa.	IN	SOA	ns1.us-west.epl-infra.net. us-west.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 19.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
#
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns1.us-west.epl-infra.net.
# 11.0.19.10.in-addr.arpa.	IN	PTR	server-f.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTkuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-19.10.in-addr.arpa.zone
# $TTL 3600
# epl-infra.net.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# epl-infra.net.	IN	NS	ns1.epl-infra.net.
# epl-infra.net.	IN	NS	ns2.epl-infra.net.
# epl-infra.net.	IN	NS	ns3.epl-infra.net.
# ns1.epl-infra.net.	IN	A	77.77.77.14
# ns2.epl-infra.net.	IN	A	77.77.77.11
# ns3.epl-infra.net.	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.11
# adm-alertmanager-default	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.14
# adm-alertmanager-default	IN	A	77.77.77.15
# adm-consul-us-west	IN	A	77.77.77.11
# adm-consul-us-west	IN	A	77.77.77.12
# adm-consul-us-west	IN	A	77.77.77.14
# adm-consul-us-west	IN	A	77.77.77.15
# adm-grafana-main	IN	A	77.77.77.11
# adm-grafana-main	IN	A	77.77.77.12
# adm-grafana-main	IN	A	77.77.77.14
# adm-grafana-main	IN	A	77.77.77.15
# adm-minio-global	IN	A	77.77.77.11
# adm-minio-global	IN	A	77.77.77.12
# adm-minio-global	IN	A	77.77.77.14
# adm-minio-global	IN	A	77.77.77.15
# adm-nomad-us-west	IN	A	77.77.77.11
# adm-nomad-us-west	IN	A	77.77.77.12
# adm-nomad-us-west	IN	A	77.77.77.14
# adm-nomad-us-west	IN	A	77.77.77.15
# adm-prometheus-default	IN	A	77.77.77.11
# adm-prometheus-default	IN	A	77.77.77.12
# adm-prometheus-default	IN	A	77.77.77.14
# adm-prometheus-default	IN	A	77.77.77.15
# adm-vault-us-west	IN	A	77.77.77.11
# adm-vault-us-west	IN	A	77.77.77.12
# adm-vault-us-west	IN	A	77.77.77.14
# adm-vault-us-west	IN	A	77.77.77.15
# admin	IN	A	77.77.77.11
# admin	IN	A	77.77.77.12
# admin	IN	A	77.77.77.14
# admin	IN	A	77.77.77.15
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJNzcuNzcuNzcuMTQKbnMyLmVwbC1pbmZyYS5uZXQuCUlOCUEJNzcuNzcuNzcuMTEKbnMzLmVwbC1pbmZyYS5uZXQuCUlOCUEJNzcuNzcuNzcuMTIKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTEKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTQKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTUKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTEKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTQKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTUKYWRtLWdyYWZhbmEtbWFpbglJTglBCTc3Ljc3Ljc3LjExCmFkbS1ncmFmYW5hLW1haW4JSU4JQQk3Ny43Ny43Ny4xMgphZG0tZ3JhZmFuYS1tYWluCUlOCUEJNzcuNzcuNzcuMTQKYWRtLWdyYWZhbmEtbWFpbglJTglBCTc3Ljc3Ljc3LjE1CmFkbS1taW5pby1nbG9iYWwJSU4JQQk3Ny43Ny43Ny4xMQphZG0tbWluaW8tZ2xvYmFsCUlOCUEJNzcuNzcuNzcuMTIKYWRtLW1pbmlvLWdsb2JhbAlJTglBCTc3Ljc3Ljc3LjE0CmFkbS1taW5pby1nbG9iYWwJSU4JQQk3Ny43Ny43Ny4xNQphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjExCmFkbS1ub21hZC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLW5vbWFkLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xNAphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjE1CmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMQphZG0tcHJvbWV0aGV1cy1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLXByb21ldGhldXMtZGVmYXVsdAlJTglBCTc3Ljc3Ljc3LjE0CmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xNQphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjExCmFkbS12YXVsdC11cy13ZXN0CUlOCUEJNzcuNzcuNzcuMTIKYWRtLXZhdWx0LXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xNAphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjE1CmFkbWluCUlOCUEJNzcuNzcuNzcuMTEKYWRtaW4JSU4JQQk3Ny43Ny43Ny4xMgphZG1pbglJTglBCTc3Ljc3Ljc3LjE0CmFkbWluCUlOCUEJNzcuNzcuNzcuMTUK /run/named/public-epl-infra.net.zone
# $TTL 3600
# in-addr.arpa.	IN	SOA	ns1.epl-infra.net. epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# in-addr.arpa.	IN	NS	ns1.epl-infra.net.
# in-addr.arpa.	IN	NS	ns2.epl-infra.net.
# in-addr.arpa.	IN	NS	ns3.epl-infra.net.
#
# 11.77.77.77.in-addr.arpa.	IN	PTR	admin
# 12.77.77.77.in-addr.arpa.	IN	PTR	admin
# 14.77.77.77.in-addr.arpa.	IN	PTR	admin
# 15.77.77.77.in-addr.arpa.	IN	PTR	admin
maybe_update_dns_file JFRUTCAzNjAwCmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQppbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMTEuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTIuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTQuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTUuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4K /run/named/public-in-addr.arpa.zone



# we could implement some complex mechanism
# to detect if zone files changed later
/run/current-system/sw/bin/systemctl reload bind.service || true

# zone file changed, reload will not reload it
if [ -f /run/restart-bind ]
then
  rm -f /run/restart-bind
  /run/current-system/sw/bin/systemctl restart bind.service || true
fi


cp -pu /run/keys/K10-in-addr-arpa--015-17699-private /run/dnsseckeys/K10.in-addr.arpa.+015+17699.private
cp -pu /run/keys/K10-in-addr-arpa--015-61551-private /run/dnsseckeys/K10.in-addr.arpa.+015+61551.private
cp -pu /run/keys/K10-in-addr-arpa--015-17699-key /run/dnsseckeys/K10.in-addr.arpa.+015+17699.key
cp -pu /run/keys/K10-in-addr-arpa--015-61551-key /run/dnsseckeys/K10.in-addr.arpa.+015+61551.key
cp -pu /run/keys/K17-10-in-addr-arpa--015-43679-private /run/dnsseckeys/K17.10.in-addr.arpa.+015+43679.private
cp -pu /run/keys/K17-10-in-addr-arpa--015-04193-private /run/dnsseckeys/K17.10.in-addr.arpa.+015+04193.private
cp -pu /run/keys/K17-10-in-addr-arpa--015-43679-key /run/dnsseckeys/K17.10.in-addr.arpa.+015+43679.key
cp -pu /run/keys/K17-10-in-addr-arpa--015-04193-key /run/dnsseckeys/K17.10.in-addr.arpa.+015+04193.key
cp -pu /run/keys/K18-10-in-addr-arpa--015-37739-private /run/dnsseckeys/K18.10.in-addr.arpa.+015+37739.private
cp -pu /run/keys/K18-10-in-addr-arpa--015-16507-private /run/dnsseckeys/K18.10.in-addr.arpa.+015+16507.private
cp -pu /run/keys/K18-10-in-addr-arpa--015-37739-key /run/dnsseckeys/K18.10.in-addr.arpa.+015+37739.key
cp -pu /run/keys/K18-10-in-addr-arpa--015-16507-key /run/dnsseckeys/K18.10.in-addr.arpa.+015+16507.key
cp -pu /run/keys/K19-10-in-addr-arpa--015-64643-private /run/dnsseckeys/K19.10.in-addr.arpa.+015+64643.private
cp -pu /run/keys/K19-10-in-addr-arpa--015-56398-private /run/dnsseckeys/K19.10.in-addr.arpa.+015+56398.private
cp -pu /run/keys/K19-10-in-addr-arpa--015-64643-key /run/dnsseckeys/K19.10.in-addr.arpa.+015+64643.key
cp -pu /run/keys/K19-10-in-addr-arpa--015-56398-key /run/dnsseckeys/K19.10.in-addr.arpa.+015+56398.key
cp -pu /run/keys/Kepl-infra-net--015-35219-private /run/dnsseckeys/Kepl-infra.net.+015+35219.private
cp -pu /run/keys/Kepl-infra-net--015-58038-private /run/dnsseckeys/Kepl-infra.net.+015+58038.private
cp -pu /run/keys/Kepl-infra-net--015-35219-key /run/dnsseckeys/Kepl-infra.net.+015+35219.key
cp -pu /run/keys/Kepl-infra-net--015-58038-key /run/dnsseckeys/Kepl-infra.net.+015+58038.key
cp -pu /run/keys/Kus-west-epl-infra-net--015-24870-private /run/dnsseckeys/Kus-west.epl-infra.net.+015+24870.private
cp -pu /run/keys/Kus-west-epl-infra-net--015-48961-private /run/dnsseckeys/Kus-west.epl-infra.net.+015+48961.private
cp -pu /run/keys/Kus-west-epl-infra-net--015-24870-key /run/dnsseckeys/Kus-west.epl-infra.net.+015+24870.key
cp -pu /run/keys/Kus-west-epl-infra-net--015-48961-key /run/dnsseckeys/Kus-west.epl-infra.net.+015+48961.key

journalctl _SYSTEMD_INVOCATION_ID=$( systemctl show --value -p InvocationID bind.service ) \
  | grep -e 'zone_rekey:dns_zone_getdnsseckeys failed: permission denied' -e "key-directory: '/run/dnsseckeys' does not exist" \
  && systemctl restart bind.service || true

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

if [ -d /var/lib/node_exporter ]
then
  # l1 last hash
  METRICS_FILE=/var/lib/node_exporter/epl_l1_last_hash.prom
  BOOT_TIME=$( cat /proc/stat | grep btime | awk '{ print $2 }' )
  echo "
epl_l1_provisioning_last_hash{hash=\"4336cf127a2ec590e373f4078bfe5d282feefdb4f829b3f7d058588f7d0eec0b\",hostname=\"server-e\"} $BOOT_TIME
" > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE

fi
