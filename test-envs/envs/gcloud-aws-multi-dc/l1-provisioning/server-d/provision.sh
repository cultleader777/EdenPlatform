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
      "agent": "0665799a-0b6f-4cd3-8057-d59263b47e41",
      "default": "993b059e-bd3a-44a2-928e-e0e1334a6886"
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
  "encrypt": "lBAN12Hwsv/VFoH/uSHTuPfodHrV9dX8CBqVM6V2H/Q=",
  "encrypt_verify_incoming": true,
  "encrypt_verify_outgoing": true,
  "limits": {
    "rpc_max_conns_per_client": 1000
  },
  "log_level": "INFO",
  "log_rotate_bytes": 0,
  "log_rotate_duration": "24h",
  "log_rotate_max_files": 0,
  "node_name": "server-d",
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


name = "server-d"
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
    token = "d28ab4b4-160a-4ace-aada-48bce760070c"
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
    "lock_epl-ingress-us-west" = "1"
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

    encrypt = "OidbfUSyNnOqe7rEM8/1pMci911eith1n2lr/f6zvC4="

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
host = "server-d.us-west.epl-infra.net"
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
host = "server-d.us-west.epl-infra.net"
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
host = "server-d.us-west.epl-infra.net"
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
host = "server-d.us-west.epl-infra.net"
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
  unicast_src_ip 10.18.0.11
  unicast_peer {
    10.18.0.10
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
XKcsl/FVpQFaepVZ3tP02zRe7JVAbuxb9a0CFDSCkBo=
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
MMxjB/3U1icOa+ixgZpQJjDiZQBToGLcTx856LHR43s=
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
MHcCAQEEIBEyRzHybBUKUdvn7xW1OJD0AOdsx4tw+/dUM1AKjy4soAoGCCqGSM49
AwEHoUQDQgAEYYJ7taBPKXqiTrRH2jSGtWYzp0FVKtHbvdp2o0VTybboOS2gaJz3
lkAely7qgZNv+he8TzY5ndSY3tQf9F5B8g==
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
MIIB0jCCAXmgAwIBAgIUOVRp67uSGyFiPhDJyG1+3nd6CyowCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MTExMzE1NTYwMFoXDTQxMTEwOTE1NTYw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABGGCe7WgTyl6ok60R9o0hrVm
M6dBVSrR273adqNFU8m26DktoGic95ZAHpcu6oGTb/oXvE82OZ3UmN7UH/ReQfKj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBT/GJqNPoMFur0EEMiTbFJNTyVt
TTAfBgNVHSMEGDAWgBT2TfyVisALQfqguf1vM8vgAM+8tzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNHADBEAiBkC77f7lhdLvpU40V59XkUYs05nuIaWzMAOKFp
jGsqWgIgeLqKUom004H+l92CrSB4ZPlWAfUB6L+t9yemSH5V/3Y=
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

# NIX REGION secret_value_K10-in-addr-arpa--015-04646-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: fdIvWWo/EIbJgOjYpv2WIv7DdjKIajQW7EB7lrNP7lY=
Created: 20241113160045
Publish: 20241113160045
Activate: 20241113160045
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-04646-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04646-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-04646-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04646-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-32779-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: br/RD2QTu2dFO3Qy8Kdxjyatn2XvU38RDzd3aZsE13M=
Created: 20241113160045
Publish: 20241113160045
Activate: 20241113160045
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-32779-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-32779-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-32779-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-32779-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-04646-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 4646, for 10.in-addr.arpa.
; Created: 20241113160045 (Wed Nov 13 18:00:45 2024)
; Publish: 20241113160045 (Wed Nov 13 18:00:45 2024)
; Activate: 20241113160045 (Wed Nov 13 18:00:45 2024)
10.in-addr.arpa. IN DNSKEY 256 3 15 nSUu2Nap5Viu4OeE4Luu1w1rZeeMbY9yaCRaVVT9uXM=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-04646-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04646-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-04646-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04646-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-32779-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 32779, for 10.in-addr.arpa.
; Created: 20241113160045 (Wed Nov 13 18:00:45 2024)
; Publish: 20241113160045 (Wed Nov 13 18:00:45 2024)
; Activate: 20241113160045 (Wed Nov 13 18:00:45 2024)
10.in-addr.arpa. IN DNSKEY 257 3 15 /9VNyAR/OCaafL3Yrpra1C4FM54FpAy514yxABi6+qo=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-32779-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-32779-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-32779-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-32779-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-15757-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: cINcCT5wSyR3/CT9HKROfVu82M32Vg1HXtwn8/Dqdrw=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-15757-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-15757-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-15757-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-15757-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-39925-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: aVnKzJoIOX5U58qjDrO4Fem1Gx9DVyeds9vk3JuU2k0=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-39925-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-39925-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-39925-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-39925-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-15757-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 15757, for 17.10.in-addr.arpa.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
17.10.in-addr.arpa. IN DNSKEY 256 3 15 nHpZUorbUsNEH9mLVIROIYUdmmSsEvPw2JlvR1o+RBw=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-15757-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-15757-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-15757-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-15757-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-39925-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 39925, for 17.10.in-addr.arpa.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
17.10.in-addr.arpa. IN DNSKEY 257 3 15 fSn3WSw+rvl/snYdXqbO+33N43Kdrpy4rXlKBm8/IlE=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-39925-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-39925-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-39925-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-39925-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-01164-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: Gd7FdZc8a1dVgGrHHJh/318AIDLlgrb/QsG6SehK9bs=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-01164-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-01164-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-01164-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-01164-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-44410-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: WyTDA8Wbnyd5cLEuoEbjWm3z14esk302cWXBDqEgyws=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-44410-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-44410-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-44410-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-44410-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-01164-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 1164, for 18.10.in-addr.arpa.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
18.10.in-addr.arpa. IN DNSKEY 256 3 15 h7UqIhEsdttnvYHhY/mF6rhk2o7U3HVx8BhzvEi6aUk=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-01164-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-01164-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-01164-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-01164-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-44410-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 44410, for 18.10.in-addr.arpa.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
18.10.in-addr.arpa. IN DNSKEY 257 3 15 /OS7qDr60vCC7/Y2Zn2KJeqAAyOFfPtVodRRhlAhxzU=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-44410-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-44410-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-44410-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-44410-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-38376-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: L1B/GMJv6h90L0ku0zfSR4gLh/J65BFFfYuSSUcD3R4=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-38376-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-38376-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-38376-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-38376-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-31880-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: t7HOdMb8fnWpAdDo4ap+2A+dbKqrR6LFY85/Xk734yM=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-31880-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-31880-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-31880-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-31880-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-38376-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 38376, for 19.10.in-addr.arpa.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
19.10.in-addr.arpa. IN DNSKEY 256 3 15 sDN/ijOGEtfYH2jRJtW7X9WyW3Oz8r7pv27mrdXv2Ig=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-38376-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-38376-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-38376-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-38376-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K19-10-in-addr-arpa--015-31880-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 31880, for 19.10.in-addr.arpa.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
19.10.in-addr.arpa. IN DNSKEY 257 3 15 w/mFoOy4B8rm/BWhe99z/DUy1205Q76PxHWAegWT/uo=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K19-10-in-addr-arpa--015-31880-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-31880-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K19-10-in-addr-arpa--015-31880-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K19-10-in-addr-arpa--015-31880-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-25092-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: e5AT82tXgnpyMpt3gCB8Q85YXmm7PNU4iHEVS5whDac=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-25092-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-25092-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-25092-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-25092-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-49856-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: TMw66Ne2h70Nrl7LomM9XEGCUZ38RYWXXkNWqsa5Gl4=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-49856-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-49856-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-49856-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-49856-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-25092-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 25092, for epl-infra.net.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
epl-infra.net. IN DNSKEY 256 3 15 hRBr1u25tPJNqk09OirA85rw1rY2JvfPpicm8TU4km0=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-25092-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-25092-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-25092-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-25092-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-49856-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 49856, for epl-infra.net.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
epl-infra.net. IN DNSKEY 257 3 15 cEnK6luP7tcwXWMHy8LSuy7LJbN2cg527QymP8tYzyU=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-49856-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-49856-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-49856-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-49856-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-51435-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: izrkAMOFHflwJWRpWm6gm4Rz7cohzk+KRooWXang+sM=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-51435-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-51435-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-51435-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-51435-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06040-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 6+/QuoSQF3SShTM+rz4aXwmywlubpWzeaRKM4t6x1Oo=
Created: 20241113160046
Publish: 20241113160046
Activate: 20241113160046
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06040-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06040-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-06040-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06040-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-51435-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 51435, for us-west.epl-infra.net.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
us-west.epl-infra.net. IN DNSKEY 256 3 15 s8jXnlMd+YV6TZP3oKsXMIJbF0wVX5gdfdALKeACdpA=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-51435-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-51435-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-51435-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-51435-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06040-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 6040, for us-west.epl-infra.net.
; Created: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Publish: 20241113160046 (Wed Nov 13 18:00:46 2024)
; Activate: 20241113160046 (Wed Nov 13 18:00:46 2024)
us-west.epl-infra.net. IN DNSKEY 257 3 15 E89hMzBmYjGCw/COGh39yXprrfzDbqsWNWYpajvzTwM=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06040-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06040-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-06040-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06040-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-ca-cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIC7TCCApSgAwIBAgIRAPHQlnY4UkK9q/62hD4T0ZEwCgYIKoZIzj0EAwIwgbkx
CzAJBgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNj
bzEaMBgGA1UECRMRMTAxIFNlY29uZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcw
FQYDVQQKEw5IYXNoaUNvcnAgSW5jLjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENB
IDMyMTQyNjk5NjQ1ODUzNDMwMTU5MjUyNzAzOTU2MjE4NjQ3Nzk2OTAeFw0yNDEx
MTMxNjAwNDZaFw00MTExMDkxNjAwNDZaMIG5MQswCQYDVQQGEwJVUzELMAkGA1UE
CBMCQ0ExFjAUBgNVBAcTDVNhbiBGcmFuY2lzY28xGjAYBgNVBAkTETEwMSBTZWNv
bmQgU3RyZWV0MQ4wDAYDVQQREwU5NDEwNTEXMBUGA1UEChMOSGFzaGlDb3JwIElu
Yy4xQDA+BgNVBAMTN0NvbnN1bCBBZ2VudCBDQSAzMjE0MjY5OTY0NTg1MzQzMDE1
OTI1MjcwMzk1NjIxODY0Nzc5NjkwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATM
uquHEwL+my2rYgyJd/BRgEBk6USbelBRkwFCVzQmCF+MkybtcC0ExrKWvxT8nf/A
cO0hh74PhbwpfuhNzZ4Do3sweTAOBgNVHQ8BAf8EBAMCAYYwDwYDVR0TAQH/BAUw
AwEB/zApBgNVHQ4EIgQgkFAxl4OuEjAwf8I/VsnHI+fEsrlBBBoT4c8u+nIP4rIw
KwYDVR0jBCQwIoAgkFAxl4OuEjAwf8I/VsnHI+fEsrlBBBoT4c8u+nIP4rIwCgYI
KoZIzj0EAwIDRwAwRAIgaQsVvN+ANmSeH4IScpDO2/CnYqfnPnOVgFlmJQ9gGBgC
IAPsOXnKOelVuSdjqmTQHJvUlEvIDtdQQrpBnRD8ZxxS
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
0665799a-0b6f-4cd3-8057-d59263b47e41
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
MIIBZDCCAQqgAwIBAgIUZQsXaNv36QnkwJ1GOgMadgUIEVowCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQxMTEzMTU1NjAwWhcNNDExMTA5MTU1NjAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABNIz
FpB3q+GpZHzrH4tOcOX+fRGI3thIMXIpDikibedU73aIpujCCQLBbJn+c4HIvumm
83muuihA40ieiKRIbdijQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBT+jXj08/ibrpdTJPrdOm9pyHceoTAKBggqhkjOPQQDAgNI
ADBFAiEA2GyEg/eg2Dn1OkAxqOTzK5l1DH4qDz4IhHvQb7Q30IcCIGCYILlXPBVd
4wiYWPBXMft1ENdgWWjCK5uQOo1NfqfA
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
MIIB5zCCAYygAwIBAgIUI2rmUS5y5BXxOJ1JDncPxYtZ4wYwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQxMTEzMTU1NjAwWhcNMjUxMTEzMTU1NjAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEG3UYLMjm+zCfSlMw/kVC/sN0
5G9yFmxIeiitxosf5CchNmgRp8CzPhhcdGqms0ZvEZ+QRuSuUTUTnhZttdKtdKOB
0zCB0DAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFMBembfLLiU3NYlS2nCcXd2ZPUua
MB8GA1UdIwQYMBaAFP6NePTz+Juul1Mk+t06b2nIdx6hMFEGA1UdEQEB/wRHMEWC
FHNlcnZlci51cy13ZXN0Lm5vbWFkghxub21hZC1zZXJ2ZXJzLnNlcnZpY2UuY29u
c3Vsgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0EAwIDSQAwRgIhAJhTia1ZTUbQ
R8ocXNI0pgYTe7jnCxrGU3Q2rRb0cCMlAiEAqnxmBs/ek9dao83rVphsyafJEHgF
DQ9faJbs3yInRD0=
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
MHcCAQEEIJgvyw2Bk4NMdeiREhhR6bDcfulp4Y02vY2bMt94cjUtoAoGCCqGSM49
AwEHoUQDQgAEG3UYLMjm+zCfSlMw/kVC/sN05G9yFmxIeiitxosf5CchNmgRp8Cz
PhhcdGqms0ZvEZ+QRuSuUTUTnhZttdKtdA==
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
MIIBkDCCATegAwIBAgIUbgj6jLrtK0FbOkFQVkAyhGPImLkwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQxMTEzMTU1NjAwWhcNMjUxMTEzMTU1NjAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE5QQMpyZcmHoFfGg5VipoQ3at
0ryqkKRzA1uRTZ5ivRBCbeH1d8ymMH9jBbIiDqePTJk7kluX81lsQoC+3FvDeqN/
MH0wDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcD
AjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSZcbAaI9+FZ667q62HqqLx8qsqdTAf
BgNVHSMEGDAWgBT+jXj08/ibrpdTJPrdOm9pyHceoTAKBggqhkjOPQQDAgNHADBE
AiAFU+pTFLRBSt60rd4IRUXn8ce05dCLYoZ0StoIIiwpIwIgMqWA8VQYLMGHoR83
tmTIfRrLdZutUiv4btEk+sDre0E=
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
MHcCAQEEINZnoNqV3bbAt0UrxCbKyYSRkOMVV0/2O4K+eNTAh2XaoAoGCCqGSM49
AwEHoUQDQgAE5QQMpyZcmHoFfGg5VipoQ3at0ryqkKRzA1uRTZ5ivRBCbeH1d8ym
MH9jBbIiDqePTJk7kluX81lsQoC+3FvDeg==
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
MIIBZTCCAQqgAwIBAgIUFx7nNX8GaKl1b75hby6DOwzdujMwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQxMTEzMTU1NjAwWhcNNDExMTA5MTU1NjAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABK6a
IshvWc5dXp4iHolpWhToFQUmsY9Tci/XgciqWr4flR1qqFh1jtqpd1ZWJTFc1KFY
UACK1f2r47NyledWrzKjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRcinhG7Fb0K3zRQuWZsUtwX2iAWzAKBggqhkjOPQQDAgNJ
ADBGAiEA3RkbJvx55+k05Arnrp+/unmA6N7hbTKcOqTH+Nah8yACIQDeDwxd569z
FKD1ediXQ707FbNxf0vBI+58D0dyWUN7RQ==
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

# NIX REGION secret_value_epl-wireguard-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
eBJszvWF8B+LNWscdIJ+sqp56TBhTYyxD9gj5XEQQGs=
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
3671ce90-8054-4649-9e30-955485289b8a
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
# ns1.epl-infra.net.	IN	A	10.18.0.11
# ns2.epl-infra.net.	IN	A	10.17.0.10
# ns3.epl-infra.net.	IN	A	10.19.0.10
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns3.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.18.0.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.10
# ns3.us-west.epl-infra.net.	IN	A	10.19.0.10
# us-west.epl-infra.net.	IN	DS	6040 15 2 2BC3A7E1108A50558913F781D6B20E5F8E87F816478EEC8343B787C91BCE978D
#
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTguMC4xMQpuczIuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xNy4wLjEwCm5zMy5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOC4wLjExCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMApuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglEUwk2MDQwIDE1IDIgMkJDM0E3RTExMDhBNTA1NTg5MTNGNzgxRDZCMjBFNUY4RTg3RjgxNjQ3OEVFQzgzNDNCNzg3QzkxQkNFOTc4RAoK /run/named/private-epl-infra.net.zone
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
# 10.0.17.10.in-addr.arpa.	IN	PTR	ns2.epl-infra.net.
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns3.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	ns1.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMTAuMC4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIuZXBsLWluZnJhLm5ldC4KMTAuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczMuZXBsLWluZnJhLm5ldC4KMTEuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEuZXBsLWluZnJhLm5ldC4KMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4K /run/named/private-10.in-addr.arpa.zone
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
# ns1.us-west.epl-infra.net.	IN	A	10.18.0.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.10
# ns3.us-west.epl-infra.net.	IN	A	10.19.0.10
# server-a.us-west.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-west.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-west.epl-infra.net.	IN	A	10.18.0.10
# server-d.us-west.epl-infra.net.	IN	A	10.18.0.11
# server-e.us-west.epl-infra.net.	IN	A	10.19.0.10
# server-f.us-west.epl-infra.net.	IN	A	10.19.0.11
# server-g.us-west.epl-infra.net.	IN	A	10.17.0.12
# server-h.us-west.epl-infra.net.	IN	A	10.18.0.12
# server-i.us-west.epl-infra.net.	IN	A	10.19.0.12
maybe_update_dns_file JFRUTCAzNjAwCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOC4wLjExCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMApuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWQudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTEKc2VydmVyLWUudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKc2VydmVyLWYudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTEKc2VydmVyLWcudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTIKc2VydmVyLWgudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTIKc2VydmVyLWkudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTIK /run/named/private-us-west.epl-infra.net.zone
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
# 10.0.17.10.in-addr.arpa.	IN	PTR	ns2.us-west.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	server-b.us-west.epl-infra.net.
# 12.0.17.10.in-addr.arpa.	IN	PTR	server-g.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTcuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMyLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItYi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTcuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWcudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-17.10.in-addr.arpa.zone
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
# 10.0.18.10.in-addr.arpa.	IN	PTR	server-c.us-west.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	ns1.us-west.epl-infra.net.
# 12.0.18.10.in-addr.arpa.	IN	PTR	server-h.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMS4wLjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWgudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-18.10.in-addr.arpa.zone
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
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns3.us-west.epl-infra.net.
# 11.0.19.10.in-addr.arpa.	IN	PTR	server-f.us-west.epl-infra.net.
# 12.0.19.10.in-addr.arpa.	IN	PTR	server-i.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTkuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTkuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWkudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-19.10.in-addr.arpa.zone
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
# ns1.epl-infra.net.	IN	A	35.235.83.50
# ns2.epl-infra.net.	IN	A	34.102.96.253
# ns3.epl-infra.net.	IN	A	34.212.47.215
# adm-alertmanager-default	IN	A	34.102.96.253
# adm-alertmanager-default	IN	A	34.102.72.86
# adm-alertmanager-default	IN	A	35.235.83.50
# adm-alertmanager-default	IN	A	34.212.47.215
# adm-consul-us-west	IN	A	34.102.96.253
# adm-consul-us-west	IN	A	34.102.72.86
# adm-consul-us-west	IN	A	35.235.83.50
# adm-consul-us-west	IN	A	34.212.47.215
# adm-grafana-main	IN	A	34.102.96.253
# adm-grafana-main	IN	A	34.102.72.86
# adm-grafana-main	IN	A	35.235.83.50
# adm-grafana-main	IN	A	34.212.47.215
# adm-minio-global	IN	A	34.102.96.253
# adm-minio-global	IN	A	34.102.72.86
# adm-minio-global	IN	A	35.235.83.50
# adm-minio-global	IN	A	34.212.47.215
# adm-nomad-us-west	IN	A	34.102.96.253
# adm-nomad-us-west	IN	A	34.102.72.86
# adm-nomad-us-west	IN	A	35.235.83.50
# adm-nomad-us-west	IN	A	34.212.47.215
# adm-prometheus-default	IN	A	34.102.96.253
# adm-prometheus-default	IN	A	34.102.72.86
# adm-prometheus-default	IN	A	35.235.83.50
# adm-prometheus-default	IN	A	34.212.47.215
# adm-vault-us-west	IN	A	34.102.96.253
# adm-vault-us-west	IN	A	34.102.72.86
# adm-vault-us-west	IN	A	35.235.83.50
# adm-vault-us-west	IN	A	34.212.47.215
# admin	IN	A	34.102.96.253
# admin	IN	A	34.102.72.86
# admin	IN	A	35.235.83.50
# admin	IN	A	34.212.47.215
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJMzUuMjM1LjgzLjUwCm5zMi5lcGwtaW5mcmEubmV0LglJTglBCTM0LjEwMi45Ni4yNTMKbnMzLmVwbC1pbmZyYS5uZXQuCUlOCUEJMzQuMjEyLjQ3LjIxNQphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQkzNC4xMDIuOTYuMjUzCmFkbS1hbGVydG1hbmFnZXItZGVmYXVsdAlJTglBCTM0LjEwMi43Mi44NgphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQkzNS4yMzUuODMuNTAKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJMzQuMjEyLjQ3LjIxNQphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQkzNC4xMDIuOTYuMjUzCmFkbS1jb25zdWwtdXMtd2VzdAlJTglBCTM0LjEwMi43Mi44NgphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQkzNS4yMzUuODMuNTAKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJMzQuMjEyLjQ3LjIxNQphZG0tZ3JhZmFuYS1tYWluCUlOCUEJMzQuMTAyLjk2LjI1MwphZG0tZ3JhZmFuYS1tYWluCUlOCUEJMzQuMTAyLjcyLjg2CmFkbS1ncmFmYW5hLW1haW4JSU4JQQkzNS4yMzUuODMuNTAKYWRtLWdyYWZhbmEtbWFpbglJTglBCTM0LjIxMi40Ny4yMTUKYWRtLW1pbmlvLWdsb2JhbAlJTglBCTM0LjEwMi45Ni4yNTMKYWRtLW1pbmlvLWdsb2JhbAlJTglBCTM0LjEwMi43Mi44NgphZG0tbWluaW8tZ2xvYmFsCUlOCUEJMzUuMjM1LjgzLjUwCmFkbS1taW5pby1nbG9iYWwJSU4JQQkzNC4yMTIuNDcuMjE1CmFkbS1ub21hZC11cy13ZXN0CUlOCUEJMzQuMTAyLjk2LjI1MwphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTM0LjEwMi43Mi44NgphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTM1LjIzNS44My41MAphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTM0LjIxMi40Ny4yMTUKYWRtLXByb21ldGhldXMtZGVmYXVsdAlJTglBCTM0LjEwMi45Ni4yNTMKYWRtLXByb21ldGhldXMtZGVmYXVsdAlJTglBCTM0LjEwMi43Mi44NgphZG0tcHJvbWV0aGV1cy1kZWZhdWx0CUlOCUEJMzUuMjM1LjgzLjUwCmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQkzNC4yMTIuNDcuMjE1CmFkbS12YXVsdC11cy13ZXN0CUlOCUEJMzQuMTAyLjk2LjI1MwphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTM0LjEwMi43Mi44NgphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTM1LjIzNS44My41MAphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTM0LjIxMi40Ny4yMTUKYWRtaW4JSU4JQQkzNC4xMDIuOTYuMjUzCmFkbWluCUlOCUEJMzQuMTAyLjcyLjg2CmFkbWluCUlOCUEJMzUuMjM1LjgzLjUwCmFkbWluCUlOCUEJMzQuMjEyLjQ3LjIxNQo= /run/named/public-epl-infra.net.zone
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
# 215.47.212.34.in-addr.arpa.	IN	PTR	admin
# 253.96.102.34.in-addr.arpa.	IN	PTR	admin
# 50.83.235.35.in-addr.arpa.	IN	PTR	admin
# 86.72.102.34.in-addr.arpa.	IN	PTR	admin
maybe_update_dns_file JFRUTCAzNjAwCmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQppbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMjE1LjQ3LjIxMi4zNC5pbi1hZGRyLmFycGEuCUlOCVBUUglhZG1pbgoyNTMuOTYuMTAyLjM0LmluLWFkZHIuYXJwYS4JSU4JUFRSCWFkbWluCjUwLjgzLjIzNS4zNS5pbi1hZGRyLmFycGEuCUlOCVBUUglhZG1pbgo4Ni43Mi4xMDIuMzQuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4K /run/named/public-in-addr.arpa.zone


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
provision_zfs_dataset rpool acme /var/lib/acme 128k on no s1CnOdJ4Df8XPOdW7jtscc8lvSYr41IQF754YItOdj
provision_zfs_dataset rpool docker /var/lib/docker 128k on no
provision_zfs_dataset rpool nats1 /srv/volumes/nats1 4k on yes eeSE6FWOvCI40lRvL0xBLxkiKaNJlN0I8AuSwEzIkB
provision_zfs_dataset rpool nomad /var/lib/nomad 4k on no z7MMnpKHTflYpomkma9JEMmJf5xqDCCzvkBMM5ennK

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
        "epl-nix-cache:W9BBOh3S33CuuDIvYZHGC9MdVkJKLOZGDp5MVzdC01o="
      ];

    };

    networking.hostId = "e8d7250b";


    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFbis+BzEk38EuddzQHZiFVsw8ATqnLYO0eFcWm/5xRG epl-root-ssh-key"
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
MIIB0jCCAXmgAwIBAgIUOVRp67uSGyFiPhDJyG1+3nd6CyowCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MTExMzE1NTYwMFoXDTQxMTEwOTE1NTYw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABGGCe7WgTyl6ok60R9o0hrVm
M6dBVSrR273adqNFU8m26DktoGic95ZAHpcu6oGTb/oXvE82OZ3UmN7UH/ReQfKj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBT/GJqNPoMFur0EEMiTbFJNTyVt
TTAfBgNVHSMEGDAWgBT2TfyVisALQfqguf1vM8vgAM+8tzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNHADBEAiBkC77f7lhdLvpU40V59XkUYs05nuIaWzMAOKFp
jGsqWgIgeLqKUom004H+l92CrSB4ZPlWAfUB6L+t9yemSH5V/3Y=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUZQsXaNv36QnkwJ1GOgMadgUIEVowCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQxMTEzMTU1NjAwWhcNNDExMTA5MTU1NjAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABNIz
FpB3q+GpZHzrH4tOcOX+fRGI3thIMXIpDikibedU73aIpujCCQLBbJn+c4HIvumm
83muuihA40ieiKRIbdijQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBT+jXj08/ibrpdTJPrdOm9pyHceoTAKBggqhkjOPQQDAgNI
ADBFAiEA2GyEg/eg2Dn1OkAxqOTzK5l1DH4qDz4IhHvQb7Q30IcCIGCYILlXPBVd
4wiYWPBXMft1ENdgWWjCK5uQOo1NfqfA
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUFx7nNX8GaKl1b75hby6DOwzdujMwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQxMTEzMTU1NjAwWhcNNDExMTA5MTU1NjAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABK6a
IshvWc5dXp4iHolpWhToFQUmsY9Tci/XgciqWr4flR1qqFh1jtqpd1ZWJTFc1KFY
UACK1f2r47NyledWrzKjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRcinhG7Fb0K3zRQuWZsUtwX2iAWzAKBggqhkjOPQQDAgNJ
ADBGAiEA3RkbJvx55+k05Arnrp+/unmA6N7hbTKcOqTH+Nah8yACIQDeDwxd569z
FKD1ediXQ707FbNxf0vBI+58D0dyWUN7RQ==
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
ip route add 10.19.0.0/16 via 10.18.128.11

# ROUTES DELETE
ip route del 10.19.0.0/16

# FINISH
' | \
  CONSUL_HTTP_TOKEN=$( ${pkgs.coreutils}/bin/cat /run/keys/consul-vrrp-token.txt ) \
  ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/10.18.0.0p24 -

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
        epl-consul-vrrp-switch
        epl-nomad-acl-bootstrap
        epl-nomad-acl-policies
        epl-nomad-vault-policies
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "0.0.0.0"; prefixLength = 0; via = "10.18.0.1"; }

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
           iifname void ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 35.235.83.50/32 } drop comment "Disallow traffic from internet to internal networks";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-internet-fw END

# NIX REGION firewall START

  networking.hostName = "server-d";
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
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-d.dc2.us-west.gcloud-aws-multi-dc:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-d.dc2.us-west.gcloud-aws-multi-dc: \w\a\]\u@server-d.dc2.us-west.gcloud-aws-multi-dc:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-d.dc2.us-west.gcloud-aws-multi-dc:\u:\w\007\]$PS1"
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
         while ! ${pkgs.consul}/bin/consul kv get epl-l1-plans/server-d
         do
           sleep 7
         done

         ${pkgs.consul}/bin/consul watch \
           -type=key -key=epl-l1-plans/server-d \
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
  epl-infra.net. initial-key 257 3 15 "cEnK6luP7tcwXWMHy8LSuy7LJbN2cg527QymP8tYzyU=";
  us-west.epl-infra.net. initial-key 257 3 15 "E89hMzBmYjGCw/COGh39yXprrfzDbqsWNWYpajvzTwM=";
  10.in-addr.arpa. initial-key 257 3 15 "/9VNyAR/OCaafL3Yrpra1C4FM54FpAy514yxABi6+qo=";
  17.10.in-addr.arpa. initial-key 257 3 15 "fSn3WSw+rvl/snYdXqbO+33N43Kdrpy4rXlKBm8/IlE=";
  18.10.in-addr.arpa. initial-key 257 3 15 "/OS7qDr60vCC7/Y2Zn2KJeqAAyOFfPtVodRRhlAhxzU=";
  19.10.in-addr.arpa. initial-key 257 3 15 "w/mFoOy4B8rm/BWhe99z/DUy1205Q76PxHWAegWT/uo=";
  in-addr.arpa. initial-key 257 3 15 "dqKIja/yAVZ7/QpRWPpPoEXn0+UtTECL/DbkL15md00=";
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
    10.17.0.10;
    10.19.0.10;
  };
};
zone "10.in-addr.arpa." {
  type master;
  file "/run/named/private-10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.10;
    10.19.0.10;
  };
};
zone "us-west.epl-infra.net." {
  type master;
  file "/run/named/private-us-west.epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.10;
    10.19.0.10;
  };
};
zone "17.10.in-addr.arpa." {
  type master;
  file "/run/named/private-17.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.10;
    10.19.0.10;
  };
};
zone "18.10.in-addr.arpa." {
  type master;
  file "/run/named/private-18.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.10;
    10.19.0.10;
  };
};
zone "19.10.in-addr.arpa." {
  type master;
  file "/run/named/private-19.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.10;
    10.19.0.10;
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
    34.102.96.253;
    34.212.47.215;
  };
};
zone "in-addr.arpa." {
  type master;
  file "/run/named/public-in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    34.102.96.253;
    34.212.47.215;
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
ip address add "172.21.7.13/16" dev "wg0" || true
wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
ip link set up dev "wg0"

# peers
wg set wg0 peer "siicT83i7PoJLy1Xo/t5QVLIdk4u+iu9YIIh++YK0Ts=" allowed-ips "172.21.7.254/32"

wg set wg0 peer "WgN5tDYjbE6RQm7ry9KYl+FRgIiChE0lWDbIdxWPrXs=" allowed-ips "172.21.7.15/32,10.19.0.0/16" endpoint "34.214.1.46:51820"

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
        ip tunnel add vpnGre mode gre local 10.18.0.11 key 17
        ip addr add 10.18.128.11/17 dev vpnGre


        ip neighbor add 10.18.128.12 lladdr 10.18.0.12 dev vpnGre

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

            ip daddr 10.18.0.12 ip saddr 10.0.0.0/8 ip daddr set 10.18.128.12;
            ip saddr 10.18.128.12 ip saddr set 10.18.0.12;
        }

              '';
            };

# NIX REGION epl_nft_rules_l3-vpn-hop-address-translation END

# NIX REGION frr_ospf_config START

  services.frr.ospf = {
      enable = true;
      config = ''
        !
        router ospf
          ospf router-id 10.18.0.11
          redistribute bgp
          network 10.18.0.0/16 area 10.18.0.0
          area 10.18.0.0 range 10.18.0.0/16 advertise
          area 10.18.0.0 range 0.0.0.0/0 not-advertise
          area 10.18.0.0 authentication message-digest
          neighbor 10.18.0.10
        !
        interface eth0
          ip ospf cost 500
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 0zfLIkGGsqRGu2GG
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
          peer 10.18.0.11
            no shutdown
          peer 172.21.7.13
            no shutdown
      '';
  };
# NIX REGION frr_bfd_config END

# NIX REGION frr_bgp_config START

  services.frr.bgp = {
      enable = true;
      config = ''
        !
        router bgp 64530
          bgp router-id 10.18.0.11
          address-family ipv4 unicast
            network 10.18.0.0/16
          exit-address-family
          neighbor 10.18.0.10 remote-as 64530
          neighbor 10.18.0.10 password 5GtjphImQld0EfmdzZMpcxNVcYiQFvYUzHR2OPW3fg
          neighbor 10.18.0.10 bfd
          neighbor 172.21.7.11 remote-as 64529
          neighbor 172.21.7.11 password yqPdL0hE0M5p2LesNMCSb8KAIJHsmygEltZ0uJghMl
          neighbor 172.21.7.11 bfd
          neighbor 172.21.7.15 remote-as 64531
          neighbor 172.21.7.15 password ZflQSGW4GGhzcPyL4lOcTyOnGxqD8kfKLqfSJVGo0j
          neighbor 172.21.7.15 bfd
          address-family ipv4 unicast
            network 10.18.0.0/16
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
          set src 10.18.0.11
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol ospf route-map LANRM
        !
        ip protocol bgp route-map LANRM
        !
        ip prefix-list INTERSUBNET seq 100 permit 10.18.0.0/16 le 24
        !
        ip prefix-list INTERSUBNET seq 101 permit 10.18.0.0/16 le 24
        !
        route-map LANRM deny 90
          match ip address prefix-list INTERSUBNET
        !
        interface eth0
          ip address 10.18.0.11/24
      '';
  };
# NIX REGION frr_zebra_config END

# NIX REGION frr_static_routes START

  # You gotta be kidding me... https://github.com/NixOS/nixpkgs/issues/274286
  services.frr.mgmt.enable = true;
  environment.etc."frr/staticd.conf".text = ''
        !
        ip route 10.18.0.0/16 10.18.0.1

        !
        ip route 10.17.0.0/16 10.18.0.1

        !
        ip route 0.0.0.0/0 10.18.0.1

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
      { address = "10.18.0.11"; prefixLength = 24; }

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
# ns1.epl-infra.net.	IN	A	10.18.0.11
# ns2.epl-infra.net.	IN	A	10.17.0.10
# ns3.epl-infra.net.	IN	A	10.19.0.10
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns3.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.18.0.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.10
# ns3.us-west.epl-infra.net.	IN	A	10.19.0.10
# us-west.epl-infra.net.	IN	DS	6040 15 2 2BC3A7E1108A50558913F781D6B20E5F8E87F816478EEC8343B787C91BCE978D
#
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTguMC4xMQpuczIuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xNy4wLjEwCm5zMy5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOC4wLjExCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMApuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglEUwk2MDQwIDE1IDIgMkJDM0E3RTExMDhBNTA1NTg5MTNGNzgxRDZCMjBFNUY4RTg3RjgxNjQ3OEVFQzgzNDNCNzg3QzkxQkNFOTc4RAoK /run/named/private-epl-infra.net.zone
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
# 10.0.17.10.in-addr.arpa.	IN	PTR	ns2.epl-infra.net.
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns3.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	ns1.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
# 19.10.in-addr.arpa.	IN	NS	ns3.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMTAuMC4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIuZXBsLWluZnJhLm5ldC4KMTAuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczMuZXBsLWluZnJhLm5ldC4KMTEuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEuZXBsLWluZnJhLm5ldC4KMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4K /run/named/private-10.in-addr.arpa.zone
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
# ns1.us-west.epl-infra.net.	IN	A	10.18.0.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.0.10
# ns3.us-west.epl-infra.net.	IN	A	10.19.0.10
# server-a.us-west.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-west.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-west.epl-infra.net.	IN	A	10.18.0.10
# server-d.us-west.epl-infra.net.	IN	A	10.18.0.11
# server-e.us-west.epl-infra.net.	IN	A	10.19.0.10
# server-f.us-west.epl-infra.net.	IN	A	10.19.0.11
# server-g.us-west.epl-infra.net.	IN	A	10.17.0.12
# server-h.us-west.epl-infra.net.	IN	A	10.18.0.12
# server-i.us-west.epl-infra.net.	IN	A	10.19.0.12
maybe_update_dns_file JFRUTCAzNjAwCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KbnMxLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xOC4wLjExCm5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMC4xMApuczMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWQudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTEKc2VydmVyLWUudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTAKc2VydmVyLWYudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTEKc2VydmVyLWcudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTIKc2VydmVyLWgudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTIKc2VydmVyLWkudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE5LjAuMTIK /run/named/private-us-west.epl-infra.net.zone
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
# 10.0.17.10.in-addr.arpa.	IN	PTR	ns2.us-west.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	server-b.us-west.epl-infra.net.
# 12.0.17.10.in-addr.arpa.	IN	PTR	server-g.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTcuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMyLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItYi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTcuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWcudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-17.10.in-addr.arpa.zone
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
# 10.0.18.10.in-addr.arpa.	IN	PTR	server-c.us-west.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	ns1.us-west.epl-infra.net.
# 12.0.18.10.in-addr.arpa.	IN	PTR	server-h.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMS4wLjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWgudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-18.10.in-addr.arpa.zone
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
# 10.0.19.10.in-addr.arpa.	IN	PTR	ns3.us-west.epl-infra.net.
# 11.0.19.10.in-addr.arpa.	IN	PTR	server-f.us-west.epl-infra.net.
# 12.0.19.10.in-addr.arpa.	IN	PTR	server-i.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTkuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxOS4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjE5LjEwLmluLWFkZHIuYXJwYS4JSU4JTlMJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KCjEwLjAuMTkuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMzLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOS4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTkuMTAuaW4tYWRkci5hcnBhLglJTglQVFIJc2VydmVyLWkudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-19.10.in-addr.arpa.zone
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
# ns1.epl-infra.net.	IN	A	35.235.83.50
# ns2.epl-infra.net.	IN	A	34.102.96.253
# ns3.epl-infra.net.	IN	A	34.212.47.215
# adm-alertmanager-default	IN	A	34.102.96.253
# adm-alertmanager-default	IN	A	34.102.72.86
# adm-alertmanager-default	IN	A	35.235.83.50
# adm-alertmanager-default	IN	A	34.212.47.215
# adm-consul-us-west	IN	A	34.102.96.253
# adm-consul-us-west	IN	A	34.102.72.86
# adm-consul-us-west	IN	A	35.235.83.50
# adm-consul-us-west	IN	A	34.212.47.215
# adm-grafana-main	IN	A	34.102.96.253
# adm-grafana-main	IN	A	34.102.72.86
# adm-grafana-main	IN	A	35.235.83.50
# adm-grafana-main	IN	A	34.212.47.215
# adm-minio-global	IN	A	34.102.96.253
# adm-minio-global	IN	A	34.102.72.86
# adm-minio-global	IN	A	35.235.83.50
# adm-minio-global	IN	A	34.212.47.215
# adm-nomad-us-west	IN	A	34.102.96.253
# adm-nomad-us-west	IN	A	34.102.72.86
# adm-nomad-us-west	IN	A	35.235.83.50
# adm-nomad-us-west	IN	A	34.212.47.215
# adm-prometheus-default	IN	A	34.102.96.253
# adm-prometheus-default	IN	A	34.102.72.86
# adm-prometheus-default	IN	A	35.235.83.50
# adm-prometheus-default	IN	A	34.212.47.215
# adm-vault-us-west	IN	A	34.102.96.253
# adm-vault-us-west	IN	A	34.102.72.86
# adm-vault-us-west	IN	A	35.235.83.50
# adm-vault-us-west	IN	A	34.212.47.215
# admin	IN	A	34.102.96.253
# admin	IN	A	34.102.72.86
# admin	IN	A	35.235.83.50
# admin	IN	A	34.212.47.215
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgplcGwtaW5mcmEubmV0LglJTglOUwluczMuZXBsLWluZnJhLm5ldC4KbnMxLmVwbC1pbmZyYS5uZXQuCUlOCUEJMzUuMjM1LjgzLjUwCm5zMi5lcGwtaW5mcmEubmV0LglJTglBCTM0LjEwMi45Ni4yNTMKbnMzLmVwbC1pbmZyYS5uZXQuCUlOCUEJMzQuMjEyLjQ3LjIxNQphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQkzNC4xMDIuOTYuMjUzCmFkbS1hbGVydG1hbmFnZXItZGVmYXVsdAlJTglBCTM0LjEwMi43Mi44NgphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQkzNS4yMzUuODMuNTAKYWRtLWFsZXJ0bWFuYWdlci1kZWZhdWx0CUlOCUEJMzQuMjEyLjQ3LjIxNQphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQkzNC4xMDIuOTYuMjUzCmFkbS1jb25zdWwtdXMtd2VzdAlJTglBCTM0LjEwMi43Mi44NgphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQkzNS4yMzUuODMuNTAKYWRtLWNvbnN1bC11cy13ZXN0CUlOCUEJMzQuMjEyLjQ3LjIxNQphZG0tZ3JhZmFuYS1tYWluCUlOCUEJMzQuMTAyLjk2LjI1MwphZG0tZ3JhZmFuYS1tYWluCUlOCUEJMzQuMTAyLjcyLjg2CmFkbS1ncmFmYW5hLW1haW4JSU4JQQkzNS4yMzUuODMuNTAKYWRtLWdyYWZhbmEtbWFpbglJTglBCTM0LjIxMi40Ny4yMTUKYWRtLW1pbmlvLWdsb2JhbAlJTglBCTM0LjEwMi45Ni4yNTMKYWRtLW1pbmlvLWdsb2JhbAlJTglBCTM0LjEwMi43Mi44NgphZG0tbWluaW8tZ2xvYmFsCUlOCUEJMzUuMjM1LjgzLjUwCmFkbS1taW5pby1nbG9iYWwJSU4JQQkzNC4yMTIuNDcuMjE1CmFkbS1ub21hZC11cy13ZXN0CUlOCUEJMzQuMTAyLjk2LjI1MwphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTM0LjEwMi43Mi44NgphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTM1LjIzNS44My41MAphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTM0LjIxMi40Ny4yMTUKYWRtLXByb21ldGhldXMtZGVmYXVsdAlJTglBCTM0LjEwMi45Ni4yNTMKYWRtLXByb21ldGhldXMtZGVmYXVsdAlJTglBCTM0LjEwMi43Mi44NgphZG0tcHJvbWV0aGV1cy1kZWZhdWx0CUlOCUEJMzUuMjM1LjgzLjUwCmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQkzNC4yMTIuNDcuMjE1CmFkbS12YXVsdC11cy13ZXN0CUlOCUEJMzQuMTAyLjk2LjI1MwphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTM0LjEwMi43Mi44NgphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTM1LjIzNS44My41MAphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTM0LjIxMi40Ny4yMTUKYWRtaW4JSU4JQQkzNC4xMDIuOTYuMjUzCmFkbWluCUlOCUEJMzQuMTAyLjcyLjg2CmFkbWluCUlOCUEJMzUuMjM1LjgzLjUwCmFkbWluCUlOCUEJMzQuMjEyLjQ3LjIxNQo= /run/named/public-epl-infra.net.zone
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
# 215.47.212.34.in-addr.arpa.	IN	PTR	admin
# 253.96.102.34.in-addr.arpa.	IN	PTR	admin
# 50.83.235.35.in-addr.arpa.	IN	PTR	admin
# 86.72.102.34.in-addr.arpa.	IN	PTR	admin
maybe_update_dns_file JFRUTCAzNjAwCmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQppbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMy5lcGwtaW5mcmEubmV0LgoKMjE1LjQ3LjIxMi4zNC5pbi1hZGRyLmFycGEuCUlOCVBUUglhZG1pbgoyNTMuOTYuMTAyLjM0LmluLWFkZHIuYXJwYS4JSU4JUFRSCWFkbWluCjUwLjgzLjIzNS4zNS5pbi1hZGRyLmFycGEuCUlOCVBUUglhZG1pbgo4Ni43Mi4xMDIuMzQuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4K /run/named/public-in-addr.arpa.zone



# we could implement some complex mechanism
# to detect if zone files changed later
/run/current-system/sw/bin/systemctl reload bind.service || true

# zone file changed, reload will not reload it
if [ -f /run/restart-bind ]
then
  rm -f /run/restart-bind
  /run/current-system/sw/bin/systemctl restart bind.service || true
fi


cp -pu /run/keys/K10-in-addr-arpa--015-04646-private /run/dnsseckeys/K10.in-addr.arpa.+015+04646.private
cp -pu /run/keys/K10-in-addr-arpa--015-32779-private /run/dnsseckeys/K10.in-addr.arpa.+015+32779.private
cp -pu /run/keys/K10-in-addr-arpa--015-04646-key /run/dnsseckeys/K10.in-addr.arpa.+015+04646.key
cp -pu /run/keys/K10-in-addr-arpa--015-32779-key /run/dnsseckeys/K10.in-addr.arpa.+015+32779.key
cp -pu /run/keys/K17-10-in-addr-arpa--015-15757-private /run/dnsseckeys/K17.10.in-addr.arpa.+015+15757.private
cp -pu /run/keys/K17-10-in-addr-arpa--015-39925-private /run/dnsseckeys/K17.10.in-addr.arpa.+015+39925.private
cp -pu /run/keys/K17-10-in-addr-arpa--015-15757-key /run/dnsseckeys/K17.10.in-addr.arpa.+015+15757.key
cp -pu /run/keys/K17-10-in-addr-arpa--015-39925-key /run/dnsseckeys/K17.10.in-addr.arpa.+015+39925.key
cp -pu /run/keys/K18-10-in-addr-arpa--015-01164-private /run/dnsseckeys/K18.10.in-addr.arpa.+015+01164.private
cp -pu /run/keys/K18-10-in-addr-arpa--015-44410-private /run/dnsseckeys/K18.10.in-addr.arpa.+015+44410.private
cp -pu /run/keys/K18-10-in-addr-arpa--015-01164-key /run/dnsseckeys/K18.10.in-addr.arpa.+015+01164.key
cp -pu /run/keys/K18-10-in-addr-arpa--015-44410-key /run/dnsseckeys/K18.10.in-addr.arpa.+015+44410.key
cp -pu /run/keys/K19-10-in-addr-arpa--015-38376-private /run/dnsseckeys/K19.10.in-addr.arpa.+015+38376.private
cp -pu /run/keys/K19-10-in-addr-arpa--015-31880-private /run/dnsseckeys/K19.10.in-addr.arpa.+015+31880.private
cp -pu /run/keys/K19-10-in-addr-arpa--015-38376-key /run/dnsseckeys/K19.10.in-addr.arpa.+015+38376.key
cp -pu /run/keys/K19-10-in-addr-arpa--015-31880-key /run/dnsseckeys/K19.10.in-addr.arpa.+015+31880.key
cp -pu /run/keys/Kepl-infra-net--015-25092-private /run/dnsseckeys/Kepl-infra.net.+015+25092.private
cp -pu /run/keys/Kepl-infra-net--015-49856-private /run/dnsseckeys/Kepl-infra.net.+015+49856.private
cp -pu /run/keys/Kepl-infra-net--015-25092-key /run/dnsseckeys/Kepl-infra.net.+015+25092.key
cp -pu /run/keys/Kepl-infra-net--015-49856-key /run/dnsseckeys/Kepl-infra.net.+015+49856.key
cp -pu /run/keys/Kus-west-epl-infra-net--015-51435-private /run/dnsseckeys/Kus-west.epl-infra.net.+015+51435.private
cp -pu /run/keys/Kus-west-epl-infra-net--015-06040-private /run/dnsseckeys/Kus-west.epl-infra.net.+015+06040.private
cp -pu /run/keys/Kus-west-epl-infra-net--015-51435-key /run/dnsseckeys/Kus-west.epl-infra.net.+015+51435.key
cp -pu /run/keys/Kus-west-epl-infra-net--015-06040-key /run/dnsseckeys/Kus-west.epl-infra.net.+015+06040.key

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
epl_l1_provisioning_last_hash{hash=\"ef70450d60d02f39b3e01241590947a01db1c5fa8d283317bea6214bc4430310\",hostname=\"server-d\"} $BOOT_TIME
" > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE

fi
