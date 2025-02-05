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
      "agent": "fb8298ae-e2ce-4da0-939d-8d6b84d4c7dd",
      "default": "475b6cd0-e040-4d93-a233-470f4a2566b9"
    }
  },
  "addresses": {
    "dns": "127.0.0.1",
    "grpc": "127.0.0.1",
    "http": "127.0.0.1",
    "https": "10.18.0.11"
  },
  "advertise_addr": "10.18.0.11",
  "advertise_addr_wan": "10.18.0.11",
  "auto_encrypt": {
    "tls": true
  },
  "bind_addr": "10.18.0.11",
  "client_addr": "127.0.0.1",
  "data_dir": "/var/lib/consul",
  "datacenter": "us-west",
  "disable_update_check": false,
  "domain": "consul",
  "enable_local_script_checks": false,
  "enable_script_checks": false,
  "encrypt": "7roptiRBnaz6QNOPLFnHwRsIhARr9B24m7uzR1Yzh7Y=",
  "encrypt_verify_incoming": true,
  "encrypt_verify_outgoing": true,
  "limits": {
    "rpc_max_conns_per_client": 1000
  },
  "log_level": "INFO",
  "log_rotate_bytes": 0,
  "log_rotate_duration": "24h",
  "log_rotate_max_files": 0,
  "node_name": "server-f",
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
  "retry_max": 0,
  "server": false,
  "tls": {
    "defaults": {
      "ca_file": "/run/keys/consul-tls-ca-cert.pem",
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


name = "server-f"
region = "us-west"
datacenter = "dc2"

enable_debug = false
disable_update_check = false


bind_addr = "10.18.0.11"
advertise {
    http = "10.18.0.11:4646"
    rpc = "10.18.0.11:4647"
    serf = "10.18.0.11:4648"
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
    token = "7bfb1eef-2c44-45b6-be97-be0bdee67afb"
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
    "private_ip" = "10.18.0.11"
    "run_unassigned_workloads" = "1"
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
      address = "10.18.0.11"
      port    = 9100
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.18.0.11:9100/"
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
      address = "10.18.0.11"
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
        http     = "http://10.18.0.11:9280/"
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
      address = "10.18.0.11"
      port    = 9281
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "home"
        tcp      = "10.18.0.11:9281"
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
address = "10.18.0.11:9281"

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
host = "server-f.us-west.epl-infra.net"
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
host = "server-f.us-west.epl-infra.net"
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
host = "server-f.us-west.epl-infra.net"
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
host = "server-f.us-west.epl-infra.net"
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
      address = "10.18.0.11"
      port    = 9134
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.18.0.11:9134/"
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
rNWJ1ZbY7Jn9v/4skjaG+9yFDCZ40WCc/fri5uh0ANc=
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
JZzKafLUtgqL4x6FNWGYSSx/+1gX7vxcYm/N35+Vomc=
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
MHcCAQEEICtR0n4LIRh1gTZg1OaerovFe/jk0hHNgdirjOflkvcHoAoGCCqGSM49
AwEHoUQDQgAEEqCDKLlz2H3z4KWySgR7GKT9y/CsVNVDh+D4/222F9+hXuqzOrrv
nNkrbHy+A7oUQPvmkkqLs6pfO8sVBO/EfA==
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
MIIB0zCCAXmgAwIBAgIUMvxkI8+bEOnpBAWXLg9czaa7F/8wCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MDQwMzA3MTAwMFoXDTQxMDMzMDA3MTAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABBKggyi5c9h98+ClskoEexik
/cvwrFTVQ4fg+P9tthffoV7qszq675zZK2x8vgO6FED75pJKi7OqXzvLFQTvxHyj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBQpGsZvhTwlD5lLo036bgJ55LUQ
uzAfBgNVHSMEGDAWgBS0jNMdLrZav7vHTLN1AbKbMEWefDA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEA5UqkfyzWAitEsQsC0MYlo/pmsTYUGi3PR9wM
9q/GejMCIFZjkfCAxGVp8rbK9dnSGAxiBIs+8t6OuYsdvoocdHaO
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
MIIC7DCCApOgAwIBAgIQYTq3Bw40zijnd0a9o5d0/TAKBggqhkjOPQQDAjCBuTEL
MAkGA1UEBhMCVVMxCzAJBgNVBAgTAkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2Nv
MRowGAYDVQQJExExMDEgU2Vjb25kIFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAV
BgNVBAoTDkhhc2hpQ29ycCBJbmMuMUAwPgYDVQQDEzdDb25zdWwgQWdlbnQgQ0Eg
MTI5MjM5OTgxMDQ4ODgzMDkzMzcxODI0OTAyNDM1OTA2MjIxMzA5MB4XDTI0MDQw
MzA3MTQ0MFoXDTQxMDMzMDA4MTQ0MFowgbkxCzAJBgNVBAYTAlVTMQswCQYDVQQI
EwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNjbzEaMBgGA1UECRMRMTAxIFNlY29u
ZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcwFQYDVQQKEw5IYXNoaUNvcnAgSW5j
LjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENBIDEyOTIzOTk4MTA0ODg4MzA5MzM3
MTgyNDkwMjQzNTkwNjIyMTMwOTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABLeD
F6ul7bRANvId2MYOaw/7w8H30ekU8biedKbDd5eSfDYm2Fp3TwVv4/cF7JAg95Ol
XZ+fCDN47T8Rxuu+TZyjezB5MA4GA1UdDwEB/wQEAwIBhjAPBgNVHRMBAf8EBTAD
AQH/MCkGA1UdDgQiBCDoiwp0tgxhat4WX9YI+znmRFULViIbZ5aNPcv2u4leVjAr
BgNVHSMEJDAigCDoiwp0tgxhat4WX9YI+znmRFULViIbZ5aNPcv2u4leVjAKBggq
hkjOPQQDAgNHADBEAiBr3TsKz+WLh3iQ3IZXBRMrL++6Klr0F4ZaAvouZQ19awIg
P0//hm/fbE6cGYOLFAW9z3BBZB01gURS7kuWvQrFk3U=
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
fb8298ae-e2ce-4da0-939d-8d6b84d4c7dd
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

# NIX REGION secret_value_nomad-ca.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBYzCCAQqgAwIBAgIUep8qv+l1GuhVJul4tib/IGKMdfIwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwNDAzMDcxMDAwWhcNNDEwMzMwMDcxMDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABJKw
4hR5TKzwD5a4q+v7GH13xqNGDtwkv9EBSyP7H/+dE/3AngX299xolbTA1oM1oZup
6R5Rpl7qQrLkp20gCJmjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRGIrvYldZMfcTV2ZxtBeNWeCJdRzAKBggqhkjOPQQDAgNH
ADBEAiA5Jew8y6K0bGxfOdTHZo4mN89THhkMbyhNP66s5TziVgIgaAlhCtTY5l0d
gzfZAWquhaUNTB3S1c4n8So10mp6i34=
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
MIIByTCCAW6gAwIBAgIUNcDT0B/y2dWM5PXBIUAJWW2GlTYwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwNDAzMDcxMDAwWhcNMjUwNDAzMDcxMDAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEEZZ+nZVtGxDqEUK6h5l5QU3E
nTOnBT4St7BNGXQhYipxiFoUsmdejexiZvGYhpLzmEX7BN7hqumJI+5d6MYJHqOB
tTCBsjAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFD7Q3ghsWj+euMtfBqoPgJezNdGV
MB8GA1UdIwQYMBaAFEYiu9iV1kx9xNXZnG0F41Z4Il1HMDMGA1UdEQEB/wQpMCeC
FGNsaWVudC51cy13ZXN0Lm5vbWFkgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0E
AwIDSQAwRgIhAPSIzTYLR/copjMMQOyvcpBwgE/+VGZHmdpJKXdjNhdoAiEAk4Lb
IJsE58/F23Ep/B4cBJtBcrAqo/yqi/nyKF9YqPs=
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
MHcCAQEEILxIeOdeEf6N/t/sIw35aFV5H2ZNsDWjRY9diaG/7Z42oAoGCCqGSM49
AwEHoUQDQgAEEZZ+nZVtGxDqEUK6h5l5QU3EnTOnBT4St7BNGXQhYipxiFoUsmde
jexiZvGYhpLzmEX7BN7hqumJI+5d6MYJHg==
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
MIIBZTCCAQqgAwIBAgIUSM1A4DEIn7ux+gTlOYs27jyVHi4wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwNDAzMDcxMDAwWhcNNDEwMzMwMDcxMDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABALb
KEsmBeVdW9h/fCKHzIMAv34XeUTLh+45YYz7rDAGA6R8OxbJHm5b4+eA/8t4Fb9s
zYSON9QBlfA/MckVPlejQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSMfKnds9foJ2NwJ+YlMDy6+fnfcDAKBggqhkjOPQQDAgNJ
ADBGAiEA53D2cNJ0hfIY/hpj3nWgnwQlx6acV+8G55YAo4TiNncCIQCJzUNQadzZ
ju/mxDO4WZfWjdVXVwWmSqn+V9zUKUu2SA==
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

# NIX REGION secret_value_epl-wireguard-key-wg0 START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
QNHGjNlYVsnbPJy4N3VFjC5SpLdxf7ipUaIMi20jyGM=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-wireguard-key-wg0 END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-wireguard-key-wg0 || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-wireguard-key-wg0')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-wireguard-key-wg0
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_epl-wireguard-key-wg1 START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
eNEQ0GNXzaogCV9pKokkAGIOumptVBnAOybFH700Y0c=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-wireguard-key-wg1 END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-wireguard-key-wg1 || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-wireguard-key-wg1')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-wireguard-key-wg1
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
provision_zfs_dataset rpool docker /var/lib/docker 128k on no

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
        "http://10.17.0.1:12777/"
        "https://cache.nixos.org/"
      ];
      trusted-public-keys = [
        "epl-nix-cache:sQyvNhOoGKYp4ZlxZR0WNJ5p42bKUOmMGmVa13vdQks="
      ];

    };

    networking.hostId = "13381ede";


    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGTqVUxJ8L3C/Yr295FIfMkNT72qqlyCOHWdwLynTCz1 epl-root-ssh-key"
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
MIIB0zCCAXmgAwIBAgIUMvxkI8+bEOnpBAWXLg9czaa7F/8wCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MDQwMzA3MTAwMFoXDTQxMDMzMDA3MTAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABBKggyi5c9h98+ClskoEexik
/cvwrFTVQ4fg+P9tthffoV7qszq675zZK2x8vgO6FED75pJKi7OqXzvLFQTvxHyj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBQpGsZvhTwlD5lLo036bgJ55LUQ
uzAfBgNVHSMEGDAWgBS0jNMdLrZav7vHTLN1AbKbMEWefDA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEA5UqkfyzWAitEsQsC0MYlo/pmsTYUGi3PR9wM
9q/GejMCIFZjkfCAxGVp8rbK9dnSGAxiBIs+8t6OuYsdvoocdHaO
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBYzCCAQqgAwIBAgIUep8qv+l1GuhVJul4tib/IGKMdfIwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwNDAzMDcxMDAwWhcNNDEwMzMwMDcxMDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABJKw
4hR5TKzwD5a4q+v7GH13xqNGDtwkv9EBSyP7H/+dE/3AngX299xolbTA1oM1oZup
6R5Rpl7qQrLkp20gCJmjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRGIrvYldZMfcTV2ZxtBeNWeCJdRzAKBggqhkjOPQQDAgNH
ADBEAiA5Jew8y6K0bGxfOdTHZo4mN89THhkMbyhNP66s5TziVgIgaAlhCtTY5l0d
gzfZAWquhaUNTB3S1c4n8So10mp6i34=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUSM1A4DEIn7ux+gTlOYs27jyVHi4wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwNDAzMDcxMDAwWhcNNDEwMzMwMDcxMDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABALb
KEsmBeVdW9h/fCKHzIMAv34XeUTLh+45YYz7rDAGA6R8OxbJHm5b4+eA/8t4Fb9s
zYSON9QBlfA/MckVPlejQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSMfKnds9foJ2NwJ+YlMDy6+fnfcDAKBggqhkjOPQQDAgNJ
ADBGAiEA53D2cNJ0hfIY/hpj3nWgnwQlx6acV+8G55YAo4TiNncCIQCJzUNQadzZ
ju/mxDO4WZfWjdVXVwWmSqn+V9zUKUu2SA==
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

if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
then
    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
    key_prefix "epl-interdc-routes/dc2" {
        policy = "write"
    }
EOL

    ${pkgs.consul}/bin/consul acl policy create \
        -name "vrrp-policy-dc2" \
        -description "VRRP policy for datacenter dc2" \
        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

    ${pkgs.consul}/bin/consul acl token create \
        -description "VRRP Token for datacenter dc2" \
        -policy-name "vrrp-policy-dc2" \
        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
fi



# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc2.txt


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
        epl-nomad-vault-policies
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "10.17.0.1"; prefixLength = 32; via = "77.77.77.1"; }

      { address = "0.0.0.0"; prefixLength = 0; via = "77.77.77.1"; }

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
           iifname eth0 ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 77.77.77.15/32 } drop comment "Disallow traffic from internet to internal networks";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-internet-fw END

# NIX REGION firewall START

  networking.hostName = "server-f";
  networking.firewall.allowPing = true;
  networking.firewall.enable = true;
  networking.firewall.checkReversePath = true;
  networking.firewall.trustedInterfaces = [

    "eth0"

    "wg0"

    "wg1"

  ];

  networking.firewall.interfaces."eth0".allowedTCPPorts = [ 22 ];

  networking.firewall.interfaces."eth0".allowedUDPPorts = [ 51820 ];

# NIX REGION firewall END

   programs.bash.promptInit = ''
     # Provide a nice prompt if the terminal supports it.
     if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
       PROMPT_COLOR="1;31m"
       ((UID)) && PROMPT_COLOR="1;32m"
       if [ -n "$INSIDE_EMACS" ]; then
         # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-f.dc2.us-west.single-dc:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-f.dc2.us-west.single-dc: \w\a\]\u@server-f.dc2.us-west.single-dc:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-f.dc2.us-west.single-dc:\u:\w\007\]$PS1"
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
         while ! ${pkgs.consul}/bin/consul kv get epl-l1-plans/server-f
         do
           sleep 7
         done

         ${pkgs.consul}/bin/consul watch \
           -type=key -key=epl-l1-plans/server-f \
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
    imports =
      [ "${modulesPath}/profiles/qemu-guest.nix" ];


  boot.zfs.devNodes = "/dev/disk/by-label/rpool";
  services.zfs.expandOnBoot = "all";

  boot.loader.grub = {
    enable = true;
    zfsSupport = true;
    efiSupport = false;
    efiInstallAsRemovable = false;
    mirroredBoots = [
      { devices = [ "nodev"]; path = "/boot"; }
    ];
  };

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

  fileSystems."/boot" =
    { device = "/dev/vda2";
      fsType = "vfat";
    };


    boot.initrd.availableKernelModules = [ "zfs" "ahci" "xhci_pci" "virtio_pci" "virtio_blk" "virtio_console" ];
    boot.initrd.kernelModules = [ ];
    boot.kernelModules = [ "zfs" "kvm-amd" ];
    boot.kernelParams = [ "console=ttyS0,115200n8" ];
    boot.extraModulePackages = [ ];
    boot.loader.grub.extraConfig = "
      serial --speed=115200 --unit=0 --word=8 --parity=no --stop=1
      terminal_input serial
      terminal_output serial
    ";

    hardware.cpu.amd.updateMicrocode = lib.mkDefault true;
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
  epl-infra.net. initial-key 257 3 15 "S3kHCK5EdZ1RvETLEjN7eOEDwakb38uTOthXy9rHqP0=";
  us-west.epl-infra.net. initial-key 257 3 15 "T2U2NydENI+AZ7uG/TpJLo+aiMWjKCelzwS2Elz3hD8=";
  10.in-addr.arpa. initial-key 257 3 15 "4xGVntgXL372RUkl0lKYblo9LsjbIeya4YM7XcUcm3I=";
  17.10.in-addr.arpa. initial-key 257 3 15 "fwNKxEmsPcB0jduOrROiv41wkuS4Rw2ZNaBTuT5kS8I=";
  18.10.in-addr.arpa. initial-key 257 3 15 "FIbgakRzqSt4KPefJ4a/fb6qTjM3qp+hAI2bVvVzCOs=";
  in-addr.arpa. initial-key 257 3 15 "tmlquEYo+6MM+ELfYhtK9OrkyIGe8UkU0VfxbiGC7PE=";
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

          zone "18.10.in-addr.arpa." IN {
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

    virtualisation.docker.daemon.settings = { "registry-mirrors" = [ "http://10.17.0.1:12778" "http://epl-docker-registry.service.consul:5000" ]; };
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
          " --web.listen-address=10.18.0.11:9100" +
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
          " --listen_ip=10.18.0.11" +
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
ip address add "172.21.8.12/16" dev "wg0" || true
wg set "wg0" private-key "/run/keys/epl-wireguard-key-wg0" listen-port "51820"
ip link set up dev "wg0"

# peers
wg set "wg0" peer "LtUFP97hFFSXpPKHWUcJWSGy6Ke7W9xT0Qy2OBghjVo=" allowed-ips "172.21.7.10/32,10.0.0.0/8" endpoint "77.77.77.12:51820" persistent-keepalive 10
wg set "wg0" peer "kaGdqKOrOKaX+4Ycb8SuJu10aRIX8YeGYZFwRw6NkW0=" allowed-ips "172.21.8.10/32,10.18.0.10/32" endpoint "77.77.77.14:51820" persistent-keepalive 10

# routes
ip route del '172.21.0.0/16' dev wg0 || true
ip route add '172.21.7.10/32' dev wg0 || true
ip route add '172.21.8.10/32' dev wg0 || true
ip route add '10.18.0.10/32' via 172.21.8.10 src 10.18.0.11 || true
    '';

    postStop = ''

      ip route del '172.21.8.10/32' || true
      ip route del '10.18.0.10/32' || true
      ip route del '172.21.7.10/32' dev wg0 || true

      ip link del dev "wg0"
    '';
  };

  systemd.services.wireguard-wg1 = {
    description = "WireGuard Tunnel - wg1";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    environment.DEVICE = "wg1";
    path = with pkgs; [ kmod iproute2 wireguard-tools ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    };

    script = ''

modprobe wireguard || true
ip link add dev "wg1" type wireguard

# this might fail as kernel seems to remember ip address from previously
ip address add "172.21.8.13/16" dev "wg1" || true
wg set "wg1" private-key "/run/keys/epl-wireguard-key-wg1" listen-port "51821"
ip link set up dev "wg1"

# peers
wg set "wg1" peer "EBTehQ3aNOQ/p1xau3j0h8jgLiFp2PFZl83xCGJGC3E=" allowed-ips "172.21.7.11/32,10.0.0.0/8" endpoint "77.77.77.13:51820" persistent-keepalive 10

# routes
ip route del '172.21.0.0/16' dev wg1 || true
ip route add '172.21.7.11/32' dev wg1 || true
    '';

    postStop = ''

      ip route del '172.21.7.11/32' dev wg1 || true

      ip link del dev "wg1"
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
           ip saddr 10.18.0.11/32 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-nat END

# NIX REGION frr_bfd_config START

  services.frr.bfd = {
      enable = true;
      config = ''
        !
        bfd peer 172.21.8.12
          no shutdown
        bfd peer 172.21.8.13
          no shutdown
      '';
  };
# NIX REGION frr_bfd_config END

# NIX REGION frr_bgp_config START

  services.frr.bgp = {
      enable = true;
      config = ''
        !
        router bgp 4201179659
          bgp router-id 10.18.0.11
          address-family ipv4 unicast
            redistribute connected
            redistribute static
            network 10.18.0.11/32
          exit-address-family
          neighbor 172.21.7.10 remote-as 64529
          neighbor 172.21.7.10 password XEU7x1jMEnOqNbz26BZ2jOGwArxsDBr8RnIeyOvEwJ
          neighbor 172.21.7.10 bfd
          neighbor 172.21.7.10 disable-connected-check
          neighbor 172.21.7.11 remote-as 64529
          neighbor 172.21.7.11 password vwTT5LUZ3KJQajRca9f4nzBRPx7g2s0kvcqlRX3yHQ
          neighbor 172.21.7.11 bfd
          neighbor 172.21.7.11 disable-connected-check
      '';
  };
# NIX REGION frr_bgp_config END

# NIX REGION frr_zebra_config START

  services.frr.zebra = {
      enable = true;
      config = ''
        !
        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
        !
        ip prefix-list ANY seq 100 permit 0.0.0.0/0
        !
        route-map LANRM permit 100
          match ip address prefix-list LAN
          set src 10.18.0.11
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol bgp route-map LANRM
        !
        interface eth0
          ip address 10.18.0.11/32
      '';
  };
# NIX REGION frr_zebra_config END

    services.prometheus.exporters.zfs.enable = true;
    services.prometheus.exporters.zfs.port = 9134;

    networking.useDHCP = false;

    networking.interfaces.eth0.ipv4.addresses = [
      { address = "10.18.0.11"; prefixLength = 32; }

      { address = "77.77.77.15"; prefixLength = 24; }

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







if [[ ! $( timeout 0.5s dig @127.0.0.1 server-f.us-west.epl-infra.net +dnssec +short ) ]]
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
epl_l1_provisioning_last_hash{hash=\"8a427a71e8c2cf7fb882e19277e116ebe139b9f307c18b3d1a8ab60626224a76\",hostname=\"server-f\"} $BOOT_TIME
" > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE

fi
