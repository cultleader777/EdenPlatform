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
      "agent": "aa9c049d-56ab-4a63-87a1-71e38983315f",
      "default": "ce9be2ca-5865-4eb3-9bb0-2bcace5d9262",
      "initial_management": "e667d3e2-75cf-4938-b854-9f4a574a9c49"
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
  "encrypt": "h9wq4SR+F45G2pCi0urIW16sdkjlpxmb3tLN1RO/qJU=",
  "encrypt_verify_incoming": true,
  "encrypt_verify_outgoing": true,
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
    token = "c04780fa-40d9-4220-bfe3-74bc53ee3024"
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
    "private_ip" = "10.19.0.10"
    "run_unassigned_workloads" = "1"
    "lock_epl-ingress-us-west" = "1"
    "lock_epl-minio-server-e-global" = "1"
    "lock_epl-mon-server-e-am-default" = "1"
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

# NIX REGION secret_value_keepalived.conf START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

global_defs {
  enable_script_security
  script_user consul
}

vrrp_instance vpnRouter {
  interface eth0
  state MASTER
  virtual_router_id 1
  priority 50
  unicast_src_ip 10.19.0.10
  unicast_peer {
    10.19.0.11
  }
  virtual_ipaddress {

  }

  notify_master /run/current-system/sw/bin/epl-consul-vrrp-switch

}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_keepalived.conf END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/keepalived.conf || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/keepalived.conf')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/keepalived.conf
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
DJnabrXMsQKCIvS6yzpUeb1ET6UM5yUeRTgSGjhHLBs=
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
4Zo8C0m7rHAbKvQK9bq9x2qAwEwwqMKOzyYAEulKz28=
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
MHcCAQEEIC4IVC96CJMs58Jv9UQMMtvdkic3Ivlzzx6AZiOTIo6qoAoGCCqGSM49
AwEHoUQDQgAErvOhTbZ6YOeBqzoBkot+erfU+As2k06Rjb3vYHEp92x0deyVRq3C
MRXQPrpvO8vWq+LszWjmHVocy/V+ryyjFw==
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
MIIB0zCCAXmgAwIBAgIUFXKgm49joHEO2/JgwMTGkqxYucswCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMTIxNzA2MTAwMFoXDTQwMTIxMjA2MTAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABK7zoU22emDngas6AZKLfnq3
1PgLNpNOkY2972BxKfdsdHXslUatwjEV0D66bzvL1qvi7M1o5h1aHMv1fq8soxej
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBQY7DF/fuqV2uraikN0w7zZdDoD
VTAfBgNVHSMEGDAWgBT8ui2xHlr0+GQMkGXxm7Y6ZSVGOzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAhvuw0GpetbyMgepkcZZV0rTT2uQ/iyKXHuDF
gdlBjG4CIG1L4SCFCw7rIRNcFfsfLskAwb7K7SqftCeQHJ9Z3Qec
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

# NIX REGION secret_value_K10-in-addr-arpa--015-44961-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 4RP6aNkR+u0VjJyc/qGooI7Qk8qP/2CZpTesmKnlAxs=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-44961-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-44961-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-44961-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-44961-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-07750-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: XJiddZ8/kfCTtHMW9D60eYoWHENOPBGAOpMD1n+lOV0=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-07750-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-07750-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-07750-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-07750-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-44961-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 44961, for 10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
10.in-addr.arpa. IN DNSKEY 256 3 15 sTv3M7iJtOwMZjpb1fKjh7I2Pqh5vNeiRDThd3+h7eQ=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-44961-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-44961-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-44961-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-44961-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-07750-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 7750, for 10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
10.in-addr.arpa. IN DNSKEY 257 3 15 EDcnSNMxM8jFNAzwt7sdpd2osA90HOAOw3OobPEe9VM=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-07750-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-07750-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-07750-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-07750-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-16823-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 3zT8XCeoY4IcXnJ5j3O0iQqR0M3WCjwcR41P0P000Rg=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-16823-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-16823-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-16823-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-16823-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-61728-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: P0L3KqOt9LcSbEzh8D+mLxBxw0o5i4FfjzmS5pMlJvQ=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-61728-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-61728-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-61728-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-61728-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-16823-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 16823, for 17.10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
17.10.in-addr.arpa. IN DNSKEY 256 3 15 SbbK0Eg9QFGdlfnrNyFuAnPO8EnKWOMqeuO8xwgEEqI=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-16823-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-16823-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-16823-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-16823-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-61728-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 61728, for 17.10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
17.10.in-addr.arpa. IN DNSKEY 257 3 15 azn82AbZk0vfsYtNcKTofCYwfBqjmlWeLEtKqVHxwk8=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-61728-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-61728-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-61728-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-61728-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-47690-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: BwjyXAM38AUz7b3+hZXLQJmZohjivoMfEgCY0lZUpe4=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-47690-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-47690-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-47690-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-47690-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-60947-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: LjH91Nf6eNSW8aeZo9jecTq2XISmMahQ4mpKueZ5nHw=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-60947-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-60947-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-60947-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-60947-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-47690-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 47690, for 18.10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
18.10.in-addr.arpa. IN DNSKEY 256 3 15 eEP0EzdXzZBCp1aajWIoKc1D3Yz9sAVOi/H2/s3l94g=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-47690-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-47690-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-47690-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-47690-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-60947-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 60947, for 18.10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
18.10.in-addr.arpa. IN DNSKEY 257 3 15 Xf1YznaHMJkisSXa6QunCmivRgtYxPylLhpOs3uBtwE=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-60947-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-60947-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-60947-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-60947-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56324-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: mbZe4K+4rJhIf75OMs9rpMZVqQybGaomcbWgnmmey0Q=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56324-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56324-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-56324-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56324-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-06031-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 3+adt2P22kDmLCMAnQgNQ7yZ39UGY36IfoGMkOFEYII=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-06031-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-06031-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-06031-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-06031-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56324-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 56324, for 19.10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
19.10.in-addr.arpa. IN DNSKEY 256 3 15 Ta9KCnFbfLpxhoMzj1vutMOI8NvPJrrZwRv/sHgQaBk=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-56324-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56324-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-56324-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-56324-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-06031-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 6031, for 19.10.in-addr.arpa.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
19.10.in-addr.arpa. IN DNSKEY 257 3 15 20BS8hmIKY1gzYl7RHPoNTcB1luZ2Yay88E/PT9l6vY=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-06031-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-06031-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-06031-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-06031-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-26492-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: HMqMw201m1oYF61FvkgLbGwPKuNlbT9WrDEdTCrqRN0=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-26492-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-26492-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-26492-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-26492-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-64830-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: IMgxdN834ZBWo4D01uELge2I041lZo9VR+ykEpq++Ls=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-64830-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-64830-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-64830-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-64830-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-26492-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 26492, for epl-infra.net.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
epl-infra.net. IN DNSKEY 256 3 15 fXPBOHWnd7Kq76WKyzzKr7lw+cHXs64y4wsyh8EUQT8=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-26492-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-26492-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-26492-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-26492-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-64830-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 64830, for epl-infra.net.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
epl-infra.net. IN DNSKEY 257 3 15 3V0DydHMIEmN+PDJZxVEOHHU1QPVKkIxS6Y2iPyaHuQ=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-64830-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-64830-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-64830-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-64830-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-00692-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: Rz5esIqnn1eXJxLW0d3hIcle+Nzm8bw2+SDKtzxCcNE=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-00692-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-00692-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-00692-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-00692-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-46722-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: EsvpJJOqa4Im509W+w3ampJ4CvOtc2nauu6+4jJ5psE=
Created: 20231217061451
Publish: 20231217061451
Activate: 20231217061451
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-46722-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-46722-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-46722-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-46722-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-00692-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 692, for us-west.epl-infra.net.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
us-west.epl-infra.net. IN DNSKEY 256 3 15 jGsPZQPTjUFT8BUZjTE4U5/H9qr/X4wzhwzb504g0BY=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-00692-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-00692-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-00692-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-00692-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-46722-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 46722, for us-west.epl-infra.net.
; Created: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Publish: 20231217061451 (Sun Dec 17 08:14:51 2023)
; Activate: 20231217061451 (Sun Dec 17 08:14:51 2023)
us-west.epl-infra.net. IN DNSKEY 257 3 15 k5nxYOGcYIC66fP80Z7nVeVWmZQGaJvlCQGncoKvLyM=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-46722-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-46722-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-46722-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-46722-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-ca-cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIC7jCCApSgAwIBAgIRAJoVxci0dDJpuCcdtLCxhxowCgYIKoZIzj0EAwIwgbkx
CzAJBgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNj
bzEaMBgGA1UECRMRMTAxIFNlY29uZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcw
FQYDVQQKEw5IYXNoaUNvcnAgSW5jLjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENB
IDIwNDgxNDE2MTEyMTA3ODQ3MzI3MDc0NTc1MjA4ODgzNjQwOTExNDAeFw0yMzEy
MTcwNjE0NTFaFw00MDEyMTIwNjE0NTFaMIG5MQswCQYDVQQGEwJVUzELMAkGA1UE
CBMCQ0ExFjAUBgNVBAcTDVNhbiBGcmFuY2lzY28xGjAYBgNVBAkTETEwMSBTZWNv
bmQgU3RyZWV0MQ4wDAYDVQQREwU5NDEwNTEXMBUGA1UEChMOSGFzaGlDb3JwIElu
Yy4xQDA+BgNVBAMTN0NvbnN1bCBBZ2VudCBDQSAyMDQ4MTQxNjExMjEwNzg0NzMy
NzA3NDU3NTIwODg4MzY0MDkxMTQwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQ6
i3rpK6UAXSHpyWCHs6CjQ9uCqN6BNBDXEwYerWLiG9dFxbYWl5iVeta0UpbvUKIc
xMyguOkQiCN+BwEvWcn8o3sweTAOBgNVHQ8BAf8EBAMCAYYwDwYDVR0TAQH/BAUw
AwEB/zApBgNVHQ4EIgQgiXDTwKJ4uBcjHX5M24vShXjO5COxj5pICd62722i23Uw
KwYDVR0jBCQwIoAgiXDTwKJ4uBcjHX5M24vShXjO5COxj5pICd62722i23UwCgYI
KoZIzj0EAwIDSAAwRQIhAKbbgbzuoHRyMbn5zmx4x8REnZ0y47y1i05nZ4JvYEP5
AiAcfEUWJBvT7DYtLdCiXQucFHJsdltZ0P7yhBDYjaipmw==
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
aa9c049d-56ab-4a63-87a1-71e38983315f
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
MIICxTCCAmugAwIBAgIRALQ1Zm5LZJOgPyHi71mTgV8wCgYIKoZIzj0EAwIwgbkx
CzAJBgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNj
bzEaMBgGA1UECRMRMTAxIFNlY29uZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcw
FQYDVQQKEw5IYXNoaUNvcnAgSW5jLjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENB
IDIwNDgxNDE2MTEyMTA3ODQ3MzI3MDc0NTc1MjA4ODgzNjQwOTExNDAeFw0yMzEy
MTcwNjE0NTJaFw0yNDEyMTYwNjE0NTJaMCAxHjAcBgNVBAMTFXNlcnZlci51cy13
ZXN0LmNvbnN1bDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABJSpU6WGA+K/srH+
PikOdBhM74FD54OgCzQ+NIeKT82YKLj6yVY84T3LbAG3Jhmmzvs2rZ6dn/H/20nP
29ZgLr2jgeswgegwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMB
BggrBgEFBQcDAjAMBgNVHRMBAf8EAjAAMCkGA1UdDgQiBCB8ZPbWOU5FBH4SrvzR
WLZa6PPtRmcdlTRjgRdvJek57TArBgNVHSMEJDAigCCJcNPAoni4FyMdfkzbi9KF
eM7kI7GPmkgJ3rbvbaLbdTBRBgNVHREESjBIgh5zZXJ2ZXItZS5zZXJ2ZXIudXMt
d2VzdC5jb25zdWyCFXNlcnZlci51cy13ZXN0LmNvbnN1bIIJbG9jYWxob3N0hwR/
AAABMAoGCCqGSM49BAMCA0gAMEUCIFZ89hxr4hCXs4486dmblIiM727Smd2rQ8sK
73gocQnDAiEAh04ByQWuLbTp/Nl/Yek7Mnl4KCddeDHd8RFJIhspjcI=
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
MHcCAQEEIJnLXuNM8Ix+a9+MDvhKN4FuqZCVVP8jwtUqqyAN6dmvoAoGCCqGSM49
AwEHoUQDQgAElKlTpYYD4r+ysf4+KQ50GEzvgUPng6ALND40h4pPzZgouPrJVjzh
PctsAbcmGabO+zatnp2f8f/bSc/b1mAuvQ==
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
e667d3e2-75cf-4938-b854-9f4a574a9c49
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
ce9be2ca-5865-4eb3-9bb0-2bcace5d9262
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
cbe7fc30-1393-4bf8-b718-a22f306f1bd3
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
09cf448a-7737-4e9f-a0bc-3b85d2b5bfda
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
8609c2f9-d67f-4d67-b361-bbded98bd0a4
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
124abcb0-230f-4793-a6f2-4d07a06ce810
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
906dfc6b-9138-41ca-a37d-6ca899ce6861
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
c04780fa-40d9-4220-bfe3-74bc53ee3024
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
MIIBYzCCAQqgAwIBAgIUcwf2Wx0BAcab9GKxA/3NBEqkYLgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjE3MDYxMDAwWhcNNDAxMjEyMDYxMDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABLVj
Pe+gqLwyF3DQMRAabqVvVBJ8+BhSHpF5F9ps9x8pO9oym8WUsMEDDQy5MWOqryIY
kcm2w6yeFqQtI2wkGy6jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQZUCUg2M10b1hICyW+TrTBDwZpBjAKBggqhkjOPQQDAgNH
ADBEAiAsPC7WS/zkA2vtfdQsEkENH9qeLOIAqLdoCbi+N+9ktAIgYcMpR0b5tfQr
yfNQWyGeMzoFRZ8sgpeOXVWXki2Cqmw=
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
MIIByTCCAW6gAwIBAgIUVy6pBeFL0brYMIn699CNGCZsHHswCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjE3MDYxMDAwWhcNMjQxMjE2MDYxMDAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEqynoZZr29DGRHQr2AtmX+QxV
n9t6ohgWGYS/O2muj8H6OMWhhazXdI58suz7OPyyaTXTa2LAjzvSmE0I3wYoQaOB
tTCBsjAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFBi11rACTQk3RxSyZgY7mu2DnIvO
MB8GA1UdIwQYMBaAFBlQJSDYzXRvWEgLJb5OtMEPBmkGMDMGA1UdEQEB/wQpMCeC
FGNsaWVudC51cy13ZXN0Lm5vbWFkgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0E
AwIDSQAwRgIhAICprW8BT50Ve42OSxtrrgIukM+QDKSspciG+Owgt4nkAiEAzmO0
FCtlKhlBJUOi2IpcN/rVkEdVXVQMCIkQTrx/GhE=
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
MHcCAQEEIEu00vyEr5PNNBI0EzOcawOaTuHqH5Ad+qLITmsbPdZMoAoGCCqGSM49
AwEHoUQDQgAEqynoZZr29DGRHQr2AtmX+QxVn9t6ohgWGYS/O2muj8H6OMWhhazX
dI58suz7OPyyaTXTa2LAjzvSmE0I3wYoQQ==
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
MIIBZDCCAQqgAwIBAgIUKJGRPQ0w0nLzoNYftEqb9Qqk7AwwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMxMjE3MDYxMDAwWhcNNDAxMjEyMDYxMDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABI0r
mro4rc4aaNwfFajPMuDVsfsoHuOw4K1FF4524iZ5Yfw4mlOU0PDWMjTjNHAUQhdU
JETmg35q6Tn5imq5v82jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSfhC32z/CuhNhrc5hGF2wVJrLDFjAKBggqhkjOPQQDAgNI
ADBFAiEAo+RsRay1tItvzyeJDfDb2gja7DVsL/cXCzJb6gn6B6UCIAZ+O5ZoouUS
EinlUnb7MGKl0z5/dCH8pzSnisY3Amto
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
43a9c688-f364-406a-8173-5ab26225531a
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
WBw3fpUUMbnJ24wvK2dUX53dpDghfH1bfrVcgFFqk2c=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_epl-wireguard-key END
if id -u root &>/dev/null && id -g root &>/dev/null; then
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/epl-wireguard-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-wireguard-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-wireguard-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-vrrp-token.txt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
124abcb0-230f-4793-a6f2-4d07a06ce810
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_consul-vrrp-token.txt END
if id -u consul &>/dev/null && id -g consul &>/dev/null; then
  chown consul $TMP_SECRET_PATH
  chgrp consul $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/consul-vrrp-token.txt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-vrrp-token.txt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-vrrp-token.txt
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
provision_zfs_dataset rpool consul /var/lib/consul 4k on no BszkDkIIPSpT7bZzI8LU2yDHDMq8R760yVlzXJzMI4
provision_zfs_dataset rpool docker /var/lib/docker 128k on no
provision_zfs_dataset rpool minio-docker-e /srv/volumes/minio-docker-e 1M on yes dPtmNEm5rlwK5j4bGHmLyXQ1RGiA9NrOz4aQel5qI6
provision_zfs_dataset rpool mon-am /srv/volumes/mon-am 4k on yes ZkURMvg0bhSGC0pop28CiMhwH8lLLdoqy6oa9X1ag6
provision_zfs_dataset rpool nats1 /srv/volumes/nats1 4k on yes s2zbbgGklKptI8cSxEEOyw9X707jCUDB1ofn5mgS8P

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
        "epl-nix-cache:knUE4VbRh9LazjdBw2qsBAMPgtyf5OI7cAVqF1hLuyw="
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
      HISTCONTROL = "ignoreboth";
      NOMAD_ADDR = "https://nomad-servers.service.consul:4646";
      VAULT_ADDR = "https://vault.service.consul:8200";
    };

    security.pki.certificates = [
      ''-----BEGIN CERTIFICATE-----
MIIB0zCCAXmgAwIBAgIUFXKgm49joHEO2/JgwMTGkqxYucswCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMTIxNzA2MTAwMFoXDTQwMTIxMjA2MTAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABK7zoU22emDngas6AZKLfnq3
1PgLNpNOkY2972BxKfdsdHXslUatwjEV0D66bzvL1qvi7M1o5h1aHMv1fq8soxej
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBQY7DF/fuqV2uraikN0w7zZdDoD
VTAfBgNVHSMEGDAWgBT8ui2xHlr0+GQMkGXxm7Y6ZSVGOzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAhvuw0GpetbyMgepkcZZV0rTT2uQ/iyKXHuDF
gdlBjG4CIG1L4SCFCw7rIRNcFfsfLskAwb7K7SqftCeQHJ9Z3Qec
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBYzCCAQqgAwIBAgIUcwf2Wx0BAcab9GKxA/3NBEqkYLgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjE3MDYxMDAwWhcNNDAxMjEyMDYxMDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABLVj
Pe+gqLwyF3DQMRAabqVvVBJ8+BhSHpF5F9ps9x8pO9oym8WUsMEDDQy5MWOqryIY
kcm2w6yeFqQtI2wkGy6jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQZUCUg2M10b1hICyW+TrTBDwZpBjAKBggqhkjOPQQDAgNH
ADBEAiAsPC7WS/zkA2vtfdQsEkENH9qeLOIAqLdoCbi+N+9ktAIgYcMpR0b5tfQr
yfNQWyGeMzoFRZ8sgpeOXVWXki2Cqmw=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUKJGRPQ0w0nLzoNYftEqb9Qqk7AwwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMxMjE3MDYxMDAwWhcNNDAxMjEyMDYxMDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABI0r
mro4rc4aaNwfFajPMuDVsfsoHuOw4K1FF4524iZ5Yfw4mlOU0PDWMjTjNHAUQhdU
JETmg35q6Tn5imq5v82jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSfhC32z/CuhNhrc5hGF2wVJrLDFjAKBggqhkjOPQQDAgNI
ADBFAiEAo+RsRay1tItvzyeJDfDb2gja7DVsL/cXCzJb6gn6B6UCIAZ+O5ZoouUS
EinlUnb7MGKl0z5/dCH8pzSnisY3Amto
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


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
            # ROUTES CREATE
            ip route add 10.19.0.0/16 via 10.17.128.10

            # ROUTES DELETE
            ip route del 10.19.0.0/16

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc1.txt


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


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
            # ROUTES CREATE
            ip route add 10.19.0.0/16 via 10.18.128.10

            # ROUTES DELETE
            ip route del 10.19.0.0/16

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc2.txt


if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
then
    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
    key_prefix "epl-interdc-routes/dc3" {
        policy = "write"
    }
EOL

    ${pkgs.consul}/bin/consul acl policy create \
        -name "vrrp-policy-dc3" \
        -description "VRRP policy for datacenter dc3" \
        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

    ${pkgs.consul}/bin/consul acl token create \
        -description "VRRP Token for datacenter dc3" \
        -policy-name "vrrp-policy-dc3" \
        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
fi


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
            # ROUTES CREATE
            ip route add 10.0.0.0/8 via 10.19.0.10
            ip route add 0.0.0.0/0 via 10.19.0.10

            # ROUTES DELETE
            ip route del 10.0.0.0/8
            ip route del 0.0.0.0/0

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc3.txt


# NIX REGION consul_vrrp_bootstrap_script END

        '';
        epl-consul-vrrp-switch = pkgs.writeShellScriptBin "epl-consul-vrrp-switch" ''

# NIX REGION consul_vrrp_switch_script START

/run/current-system/sw/bin/echo '
# ROUTES CREATE
ip route add 10.0.0.0/8 via 10.19.0.10
ip route add 0.0.0.0/0 via 10.19.0.10

# ROUTES DELETE
ip route del 10.0.0.0/8
ip route del 0.0.0.0/0

# FINISH
' | \
  CONSUL_HTTP_TOKEN=$( ${pkgs.coreutils}/bin/cat /run/keys/consul-vrrp-token.txt ) \
  ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -

# NIX REGION consul_vrrp_switch_script END

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
        pkgs.wireguard-tools
        pkgs.zstd
        epl-consul-bootstrap
        epl-consul-vrrp-acl
        epl-consul-vrrp-switch
        epl-nomad-acl-bootstrap
        epl-nomad-acl-policies
        epl-nomad-consul-acl-bootstrap
        epl-nomad-vault-policies
        epl-vault-consul-acl-bootstrap
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "169.254.169.254"; prefixLength = 32; via = "10.19.0.1"; }

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
           iifname void ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 34.212.47.215/32 } drop comment "Disallow traffic from internet to internal networks";
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
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-e.dc3.us-west.gcloud-aws-multi-dc:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-e.dc3.us-west.gcloud-aws-multi-dc: \w\a\]\u@server-e.dc3.us-west.gcloud-aws-multi-dc:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-e.dc3.us-west.gcloud-aws-multi-dc:\u:\w\007\]$PS1"
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
  epl-infra.net. initial-key 257 3 15 "3V0DydHMIEmN+PDJZxVEOHHU1QPVKkIxS6Y2iPyaHuQ=";
  us-west.epl-infra.net. initial-key 257 3 15 "k5nxYOGcYIC66fP80Z7nVeVWmZQGaJvlCQGncoKvLyM=";
  10.in-addr.arpa. initial-key 257 3 15 "EDcnSNMxM8jFNAzwt7sdpd2osA90HOAOw3OobPEe9VM=";
  17.10.in-addr.arpa. initial-key 257 3 15 "azn82AbZk0vfsYtNcKTofCYwfBqjmlWeLEtKqVHxwk8=";
  18.10.in-addr.arpa. initial-key 257 3 15 "Xf1YznaHMJkisSXa6QunCmivRgtYxPylLhpOs3uBtwE=";
  19.10.in-addr.arpa. initial-key 257 3 15 "20BS8hmIKY1gzYl7RHPoNTcB1luZ2Yay88E/PT9l6vY=";
  in-addr.arpa. initial-key 257 3 15 "pFGF/hEQu2aUEkeLvjCqKF7meN68e/b1hSMFmec7+7o=";
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
  type slave;
  file "/run/named/private-epl-infra.net.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.19.0.10;
};
zone "10.in-addr.arpa." {
  type slave;
  file "/run/named/private-10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.19.0.10;
};
zone "us-west.epl-infra.net." {
  type slave;
  file "/run/named/private-us-west.epl-infra.net.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.19.0.10;
};
zone "17.10.in-addr.arpa." {
  type slave;
  file "/run/named/private-17.10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.19.0.10;
};
zone "18.10.in-addr.arpa." {
  type slave;
  file "/run/named/private-18.10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.19.0.10;
};
zone "19.10.in-addr.arpa." {
  type slave;
  file "/run/named/private-19.10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.19.0.10;
};

};

view internet {
          match-clients { any; };
          recursion no;
zone "epl-infra.net." {
  type slave;
  file "/run/named/public-epl-infra.net.zone.signed";
  masters { 35.235.83.50; };

  allow-query { any; };
};
zone "in-addr.arpa." {
  type slave;
  file "/run/named/public-in-addr.arpa.zone.signed";
  masters { 35.235.83.50; };
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
wg set wg0 peer "DyMxArxtervvFETX/B0o5DFW0hw9FxQAMQ8zVdW/xWs=" allowed-ips "172.21.7.254/32"

wg set wg0 peer "UzA5OExvNNlJj+u2m6chcwY79umWT+wAJeTzZfNvokk=" allowed-ips "172.21.7.10/32,10.17.0.0/16" endpoint "34.102.96.253:51820"

wg set wg0 peer "ohohvUJwez3/Wg5wgNO+iQsiiGcf+DYj0yv+9V0VriI=" allowed-ips "172.21.7.12/32,10.18.0.0/16" endpoint "34.102.72.86:51820"

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
               ip saddr 10.19.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-nat END

# NIX REGION frr_ospf_config START

  services.frr.ospf = {
      enable = true;
      config = ''
        !
        router ospf
          ospf router-id 10.19.0.10
          redistribute bgp
          network 10.19.0.0/16 area 10.19.0.0
          area 10.19.0.0 range 10.19.0.0/16 advertise
          area 10.19.0.0 range 0.0.0.0/0 not-advertise
          area 10.19.0.0 authentication message-digest
          default-information originate always
          neighbor 10.19.0.11
        !
        interface eth0
          ip ospf cost 500
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 ndWfzQA5JHtJ9bJS
          ip ospf authentication message-digest
          ip ospf network non-broadcast
      '';
  };
# NIX REGION frr_ospf_config END

# NIX REGION frr_bfd_config START

  services.frr.bfd = {
      enable = true;
      config = ''
        !
        bfd
          peer 10.19.0.10
            no shutdown
          peer 172.21.7.14
            no shutdown
      '';
  };
# NIX REGION frr_bfd_config END

# NIX REGION frr_bgp_config START

  services.frr.bgp = {
      enable = true;
      config = ''
        !
        router bgp 64531
          bgp router-id 10.19.0.10
          address-family ipv4 unicast
            network 10.19.0.0/16
          exit-address-family
          neighbor 10.19.0.11 remote-as 64531
          neighbor 10.19.0.11 password 1ZfrfQ2Q699cZwH0lgo5LI8uw5UtiKs4p7g0TymLCf
          neighbor 10.19.0.11 bfd
          neighbor 172.21.7.10 remote-as 64529
          neighbor 172.21.7.10 password DkzwtWffsOlzKnmR0WOvMn6QsMlfWZAwGa8wfvm3Ey
          neighbor 172.21.7.10 bfd
          neighbor 172.21.7.12 remote-as 64530
          neighbor 172.21.7.12 password xhfDhWT286D0NF73DM75yAbG3xHpLxko1ECge1omts
          neighbor 172.21.7.12 bfd
          address-family ipv4 unicast
            network 10.19.0.0/16
          exit-address-family
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
          set src 10.19.0.10
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol ospf route-map LANRM
        !
        ip protocol bgp route-map LANRM
        !
        ip prefix-list INTERSUBNET seq 100 permit 10.19.0.0/16 le 24
        !
        route-map LANRM deny 90
          match ip address prefix-list INTERSUBNET
        !
        interface eth0
          ip address 10.19.0.10/24
      '';
  };
# NIX REGION frr_zebra_config END

# NIX REGION frr_static_routes START

  # You gotta be kidding me... https://github.com/NixOS/nixpkgs/issues/274286
  services.frr.mgmt.enable = true;
  environment.etc."frr/staticd.conf".text = ''
        !
        ip route 10.19.0.0/16 10.19.0.1

        !
        ip route 0.0.0.0/0 10.19.0.1

  '';
  systemd.services.staticd.serviceConfig.ExecStart = lib.mkForce "${pkgs.frr}/libexec/frr/staticd -A localhost";
  services.frr.static.enable = true;
# NIX REGION frr_static_routes END

# NIX REGION keepalived START

  systemd.services.keepalived = {
    description = "Keepalive Daemon (LVS and VRRP)";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" "network-online.target" "syslog.target" ];
    wants = [ "network-online.target" ];
    serviceConfig = {
      Type = "forking";
      PIDFile = "/run/keepalived.pid";
      KillMode = "process";
      RuntimeDirectory = "keepalived";
      ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
      ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
      Restart = "always";
      RestartSec = "1s";
    };
  };

# NIX REGION keepalived END


    # reload service on file change
    systemd.services.keepalived-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/keepalived.conf -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl reload keepalived.service || true'";
      };

      enable = true;
    };
    systemd.paths.keepalived-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/keepalived.conf";
        Unit = "keepalived-restart.service";
      };

      enable = true;
    };

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







if [[ ! $( timeout 0.5s dig @127.0.0.1 server-e.us-west.epl-infra.net +dnssec +short ) ]]
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
epl_l1_provisioning_last_hash{hash=\"8c44259e835c7bccd140c84dd29064f95bf77427d42cc159a675d03fdd4b6645\",hostname=\"server-e\"} $BOOT_TIME
" > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE

fi