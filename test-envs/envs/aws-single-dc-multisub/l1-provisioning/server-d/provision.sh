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
      "agent": "f73b3251-0583-4a10-b535-e9588a5089ed",
      "default": "0d66764e-2428-4bb2-ae3f-ffb1563e894a"
    }
  },
  "addresses": {
    "dns": "127.0.0.1",
    "grpc": "127.0.0.1",
    "http": "127.0.0.1",
    "https": "10.17.1.11"
  },
  "advertise_addr": "10.17.1.11",
  "advertise_addr_wan": "10.17.1.11",
  "auto_encrypt": {
    "tls": true
  },
  "bind_addr": "10.17.1.11",
  "client_addr": "127.0.0.1",
  "data_dir": "/var/lib/consul",
  "datacenter": "us-west",
  "disable_update_check": false,
  "domain": "consul",
  "enable_local_script_checks": false,
  "enable_script_checks": false,
  "encrypt": "v+e3fkAWTctR8t5JlR/tKKk4Lxs+q9r5dwb+5m7liB0=",
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
    "10.17.0.11",
    "10.17.1.10"
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
datacenter = "dc1"

enable_debug = false
disable_update_check = false


bind_addr = "10.17.1.11"
advertise {
    http = "10.17.1.11:4646"
    rpc = "10.17.1.11:4647"
    serf = "10.17.1.11:4648"
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
    token = "c96829a0-5eee-4638-ba0d-3aef14138e54"
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
    "private_ip" = "10.17.1.11"
    "run_unassigned_workloads" = "1"
    "lock_epl-ingress-us-west" = "1"
    "lock_epl-minio-server-d-global" = "1"
    "lock_epl-mon-server-d-am-default" = "1"
  }

  host_volume "minio-docker-d" {
    path = "/srv/volumes/minio-docker-d"
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

    encrypt = "AjJtEjiZ9kLS9PXWa8OMbdly1x+n5vLo10yJkgg+OFA="

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

# NIX REGION secret_value_vault_config.hcl START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

# Eden platform machines assume we never use swap
disable_mlock = true

cluster_name = "us-west"
max_lease_ttl = "768h"
default_lease_ttl = "768h"

disable_clustering = "False"
cluster_addr = "https://server-d.us-west.epl-infra.net:8201"
api_addr = "https://server-d.us-west.epl-infra.net:8200"

plugin_directory = "/usr/local/lib/vault/plugins"

listener "tcp" {
  address = "10.17.1.11:8200"
  cluster_address = "10.17.1.11:8201"
  tls_cert_file = "/run/keys/vault-instance.crt"
  tls_key_file = "/run/keys/vault-instance.key"
  tls_client_ca_file = "/run/keys/vault-ca.crt"
  tls_min_version  = "tls12"
  tls_disable = "false"
  telemetry {
    unauthenticated_metrics_access = "true"
  }
}

storage "raft" {
  path = "/var/lib/vault"
  node_id = "server-d.us-west.epl-infra.net"

  retry_join {
    leader_api_addr = "https://server-b.us-west.epl-infra.net:8200"
    leader_ca_cert_file = "/run/keys/vault-ca.crt"
    leader_client_cert_file = "/run/keys/vault-instance.crt"
    leader_client_key_file = "/run/keys/vault-instance.key"
  }

  retry_join {
    leader_api_addr = "https://server-c.us-west.epl-infra.net:8200"
    leader_ca_cert_file = "/run/keys/vault-ca.crt"
    leader_client_cert_file = "/run/keys/vault-instance.crt"
    leader_client_key_file = "/run/keys/vault-instance.key"
  }

}

service_registration "consul" {
  address = "127.0.0.1:8500"
  check_timeout = "5s"
  disable_registration = "False"
  scheme = "http"
  service = "vault"
  service_tags = ""
  service_address = "10.17.1.11"
  token = "58985e73-128d-43fb-890a-1690b0955bad"
}

ui = true

telemetry {
  prometheus_retention_time = "3h"
}
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_vault_config.hcl END
if id -u vault &>/dev/null && id -g vault &>/dev/null; then
  chown vault $TMP_SECRET_PATH
  chgrp vault $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/vault_config.hcl || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vault_config.hcl')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vault_config.hcl
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
      address = "10.17.1.11"
      port    = 9100
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.17.1.11:9100/"
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
      address = "10.17.1.11"
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
        http     = "http://10.17.1.11:9280/"
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
      address = "10.17.1.11"
      port    = 9281
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "home"
        tcp      = "10.17.1.11:9281"
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
address = "10.17.1.11:9281"

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
  unicast_src_ip 10.17.1.11
  unicast_peer {
    10.17.1.10
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
      address = "10.17.1.11"
      port    = 9134
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.17.1.11:9134/"
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
Y94hNcSmB6Y7wcGRFFu5wiGr7XnitLUoZGxqC0jSIg4=
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
8QzJ8VP5iMdJTHjDJNjR3MDHxpJaWK4UaCss6ZN2KD0=
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
MHcCAQEEIIS7RVZTPQzmOF4DMGHScSQm2TCQuQYrUkt6e8Xlpq9yoAoGCCqGSM49
AwEHoUQDQgAECsDBp1ql/Fwlp9Oi8ZW0xDB07RnEIBkbeqfuKKkn8zgZAv+ernCd
qPrcVbW/FsGVWSdaTdNSea+4alS465Mnsg==
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
MIIB0jCCAXmgAwIBAgIUZcywHAjaf3VMsJX5eEYP4qKj0wwwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MDEwMjA5NDIwMFoXDTQwMTIyODA5NDIw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABArAwadapfxcJafTovGVtMQw
dO0ZxCAZG3qn7iipJ/M4GQL/nq5wnaj63FW1vxbBlVknWk3TUnmvuGpUuOuTJ7Kj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSJO5NCIqik6pSW9N+T/xWK6AzR
RDAfBgNVHSMEGDAWgBSamtoWbzJ+jIAB8MhJ1OI6c6Jo7jA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNHADBEAiAC5oEUuyFaJmajsmStVfLX3PdNGEZjVvfWG3LH
a6CuSAIgfjNyzX2BOvhn/K2y46JgFpfj/nu/C3W2Fpd4kDWHiSI=
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

# NIX REGION secret_value_K10-in-addr-arpa--015-39018-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: ZzaD9cqK6rUMrEXgpGs4PjokXVHdccESE4YwWhzk4Bs=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-39018-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-39018-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-39018-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-39018-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-04946-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: DrDRQYgaZN7hrLOl9FlgNc6fXcOncJvgWNxe2Egq9ig=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-04946-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04946-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-04946-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04946-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-39018-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 39018, for 10.in-addr.arpa.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
10.in-addr.arpa. IN DNSKEY 256 3 15 XfC8P0Iptw4foBtqzPwGJSW1V3PJ7IZJy0dDSDGXZUE=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-39018-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-39018-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-39018-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-39018-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-04946-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 4946, for 10.in-addr.arpa.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
10.in-addr.arpa. IN DNSKEY 257 3 15 RCuSlvf+tmrNojBnBYX6B51k0SkQjfOTkZ7cAJ6rDYU=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-04946-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04946-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-04946-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-04946-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-18308-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: Se0c19/RKLdNqgKYfWQMpKavuuLRUUEn23zz/C+Yphg=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-18308-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-18308-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-18308-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-18308-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-57593-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: w1u9DI9fFhfAp5sl4dX6tKktKx4ME9WX9LJK/BddCYc=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-57593-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-57593-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-57593-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-57593-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-18308-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 18308, for 17.10.in-addr.arpa.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
17.10.in-addr.arpa. IN DNSKEY 256 3 15 YLBeWf/+sDRzpXyfs8fSbo/TabGnHvny9hVi+4Tz5SA=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-18308-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-18308-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-18308-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-18308-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K17-10-in-addr-arpa--015-57593-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 57593, for 17.10.in-addr.arpa.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
17.10.in-addr.arpa. IN DNSKEY 257 3 15 VyB/VcVOYDm43EAd333l+PWTsNKFp8PqARCJHOX7wlk=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K17-10-in-addr-arpa--015-57593-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-57593-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K17-10-in-addr-arpa--015-57593-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K17-10-in-addr-arpa--015-57593-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-56109-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 8UniX6QQKNkSeqeLn8YAgP4KFNhfTC0dN7nxy9gGxjY=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-56109-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56109-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-56109-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56109-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-02768-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: oMTUay8y5v4GAE5udOovsAjxZETbcvcL6vcI+jjsh0E=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-02768-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-02768-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-02768-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-02768-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-56109-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 56109, for epl-infra.net.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
epl-infra.net. IN DNSKEY 256 3 15 /Z+hUQ7GxwUVN/Rg87syzOXAErAu3KzO6XFFDV070mo=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-56109-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56109-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-56109-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56109-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-02768-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 2768, for epl-infra.net.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
epl-infra.net. IN DNSKEY 257 3 15 X0AjsCmQ1cpIltZ18OKk7NfRUhOib8zj3uT1/T6OIu8=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-02768-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-02768-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-02768-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-02768-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-13820-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: eJcH49VRYdxiCDg4B5rulc2gB4pU/g49mnuR5wSVlM4=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-13820-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-13820-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-13820-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-13820-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06484-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: cZeyMn4SJ7TjThyijr9UP2b1BNgqtkSQ/J2tzBMn6GM=
Created: 20240102094650
Publish: 20240102094650
Activate: 20240102094650
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06484-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06484-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-06484-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06484-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-13820-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 13820, for us-west.epl-infra.net.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
us-west.epl-infra.net. IN DNSKEY 256 3 15 glNSWPcZcNYIiS+oCUOA84ZwqnU8OW4XxVk46IU21Dk=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-13820-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-13820-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-13820-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-13820-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06484-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 6484, for us-west.epl-infra.net.
; Created: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Publish: 20240102094650 (Tue Jan  2 11:46:50 2024)
; Activate: 20240102094650 (Tue Jan  2 11:46:50 2024)
us-west.epl-infra.net. IN DNSKEY 257 3 15 66IGH5NhA/ezssjM92SrnTkIkwqKlLeqYMHdzFKszRo=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-west-epl-infra-net--015-06484-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06484-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-west-epl-infra-net--015-06484-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-west-epl-infra-net--015-06484-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-ca-cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIC6zCCApGgAwIBAgIQHQvCnTYTR/VUGr0ZpxbxCTAKBggqhkjOPQQDAjCBuDEL
MAkGA1UEBhMCVVMxCzAJBgNVBAgTAkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2Nv
MRowGAYDVQQJExExMDEgU2Vjb25kIFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAV
BgNVBAoTDkhhc2hpQ29ycCBJbmMuMT8wPQYDVQQDEzZDb25zdWwgQWdlbnQgQ0Eg
Mzg2MDg2NzQzODYyMjY1NjY1MTYxNDU3NDU3OTY0NjQ1NzA2MzMwHhcNMjQwMTAy
MDk0NjUxWhcNNDAxMjI4MDk0NjUxWjCBuDELMAkGA1UEBhMCVVMxCzAJBgNVBAgT
AkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2NvMRowGAYDVQQJExExMDEgU2Vjb25k
IFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAVBgNVBAoTDkhhc2hpQ29ycCBJbmMu
MT8wPQYDVQQDEzZDb25zdWwgQWdlbnQgQ0EgMzg2MDg2NzQzODYyMjY1NjY1MTYx
NDU3NDU3OTY0NjQ1NzA2MzMwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASGQoew
kToP62KpxGZjtSPjFgwgkddZBG/Mh8zsD/b9/GBzf7M7rYbVwiLsK1X7k1NmwXMZ
cAe/Di+vmcO6Jsv+o3sweTAOBgNVHQ8BAf8EBAMCAYYwDwYDVR0TAQH/BAUwAwEB
/zApBgNVHQ4EIgQgcLVD0NQKLvWiUwm94adx6FnkYqXM9h/maxgWxtMx7kMwKwYD
VR0jBCQwIoAgcLVD0NQKLvWiUwm94adx6FnkYqXM9h/maxgWxtMx7kMwCgYIKoZI
zj0EAwIDSAAwRQIgBAd/cI0d3idL3PIujwJtat/pV6osjDJX0nqNP/dAdZ4CIQC9
tDpcMH4f/vdqmM07/CJckWM+A3nfjvZXlxf6y5eIYw==
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
f73b3251-0583-4a10-b535-e9588a5089ed
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
MIIBZTCCAQqgAwIBAgIUHHyb2Jl3Co+G2CbFoaLuGqB1UfwwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwMTAyMDk0MjAwWhcNNDAxMjI4MDk0MjAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABKCY
1U9bIO5x0Sv/q41gNW7zAx7TP/fHYziXTBKj0k9xhQzZPp4t8TIZiA478OaCYIrb
e18Pz8wm+yGuC2x7DdujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRh3xc1NBqq3ZuYRH7qZVeLHEh3ejAKBggqhkjOPQQDAgNJ
ADBGAiEAhr3aStAzY2cCOWFaX+ZyIeBsG34vYE+tI23XD6VkMh4CIQCVFcheXMZs
gCerkeGRQNBsHNY/6r1Jp3IM9Nv6ydkA/A==
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
MIIB5zCCAYygAwIBAgIUD4sssxO+kAo6cAZ20L6oq4KKnL4wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjUwNDA3MTQ0MTAwWhcNMjYwNDA3MTQ0MTAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAElAL0D07t34iHa3U578j0orel
8A5mtXMusXnZoIL8AGD4mHZTwbJiCLjJoFU0HBdAdMyPOtRUqpEDIpjsnFVdZKOB
0zCB0DAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFPKu5yagpFMdWTFcVbMj4cgp+3AG
MB8GA1UdIwQYMBaAFGHfFzU0Gqrdm5hEfuplV4scSHd6MFEGA1UdEQEB/wRHMEWC
FHNlcnZlci51cy13ZXN0Lm5vbWFkghxub21hZC1zZXJ2ZXJzLnNlcnZpY2UuY29u
c3Vsgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0EAwIDSQAwRgIhAPfgNMrWy7mv
rQafJxxG6nu1NEOOouwO304PcaOH1ubTAiEAyL5/5zXH6jZmZ0wMmkV9NVqd3Q2G
MDmpgIekbGTz8xk=
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
MHcCAQEEIHg0BBtI9aD4xrW+Ku+pnHFh2CX2J9Vdz5ewffB46bn4oAoGCCqGSM49
AwEHoUQDQgAElAL0D07t34iHa3U578j0orel8A5mtXMusXnZoIL8AGD4mHZTwbJi
CLjJoFU0HBdAdMyPOtRUqpEDIpjsnFVdZA==
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
MIIBkjCCATegAwIBAgIUamWcp69JzYDAQy/U5bfxD561aBowCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjUwNDA3MTQ0MTAwWhcNMjYwNDA3MTQ0MTAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEibqJO4YQpqDujI5TRFAWT+/A
cIweoeuNkEtgpomZ7HiOz9f1jtpRC6H5bnpKI8rQV2YBPBkSKFT7as1ZmZxVRqN/
MH0wDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcD
AjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSc9ug9PQmzK23q7NAdjjZYLLIQ9zAf
BgNVHSMEGDAWgBRh3xc1NBqq3ZuYRH7qZVeLHEh3ejAKBggqhkjOPQQDAgNJADBG
AiEAt2xM72Y11jB87IxVmwhydDCGy8cHxCkLM4TSoz6xhlACIQCyHGJxIeEDuI3y
kfUJM/dPGS74nuRLxSpthXGGv94hiQ==
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
MHcCAQEEICFtGwzZZus/Nz2nqJEBoXh9ZQ7nLos99BLVv/VU+NUyoAoGCCqGSM49
AwEHoUQDQgAEibqJO4YQpqDujI5TRFAWT+/AcIweoeuNkEtgpomZ7HiOz9f1jtpR
C6H5bnpKI8rQV2YBPBkSKFT7as1ZmZxVRg==
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
MIIBYzCCAQqgAwIBAgIUbhdnPpWBtwSe1fcmpY2ZrFEdJ18wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwMTAyMDk0MjAwWhcNNDAxMjI4MDk0MjAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABC8m
xpk5pJmH+4NQKB6uEumI1nD99PUafiVUcBpLSYJFRVjWSk7VsaDRVk7T9PUWHeIk
HEhU29Ik79MGsZXyK32jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQDHNfmYT+QvTEVjYs7r8p5SFPnPzAKBggqhkjOPQQDAgNH
ADBEAiAzotLfmy91WmKI7UI1MVF94M+ZAZd6ZRwQvs0oT291bwIgQ6Dwxd+l68dE
N1yKhQjTOuvQN9HQeA199ZXhSe1whpc=
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

# NIX REGION secret_value_vault-instance.crt START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIICQjCCAemgAwIBAgIUIermdf7imjI4hZCffgPw8hnEFYEwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjUwNDA3MTQ0MTAwWhcNMjYwNDA3MTQ0MTAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEA+48Ot1wsAsmP0G2jTlZuecb
V8NEmi1jXjj5oVfT812XyJPCtUDJOxa04ZTU54tvnBEjf1g7ClAv35Di8xgi+qOC
AS8wggErMA4GA1UdDwEB/wQEAwIFoDAdBgNVHSUEFjAUBggrBgEFBQcDAQYIKwYB
BQUHAwIwDAYDVR0TAQH/BAIwADAdBgNVHQ4EFgQUv/uZiimRFYX7nNEUVSp8Rv0o
llIwHwYDVR0jBBgwFoAUAxzX5mE/kL0xFY2LO6/KeUhT5z8wgasGA1UdEQEB/wSB
oDCBnYIec2VydmVyLWQudXMtd2VzdC5lcGwtaW5mcmEubmV0ghR2YXVsdC5zZXJ2
aWNlLmNvbnN1bIIWKi52YXVsdC5zZXJ2aWNlLmNvbnN1bIIcdmF1bHQuc2Vydmlj
ZS51cy13ZXN0LmNvbnN1bIIeKi52YXVsdC5zZXJ2aWNlLnVzLXdlc3QuY29uc3Vs
gglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0EAwIDRwAwRAIgaR5EbmoofnIVRNF8
nSlOOPENvrRuZWQ3LpX1Vk4q5fECIH9NhjTkJgslofyz6zqexcMssLPA8mmEWQdf
kgWYvjqL
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_vault-instance.crt END
if id -u vault &>/dev/null && id -g vault &>/dev/null; then
  chown vault $TMP_SECRET_PATH
  chgrp vault $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/vault-instance.crt || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vault-instance.crt')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vault-instance.crt
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_vault-instance.key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIFwLRROjrAuP+da/4QCX0ZKm5OTJtuV+2mn7tRBKWHYEoAoGCCqGSM49
AwEHoUQDQgAEA+48Ot1wsAsmP0G2jTlZuecbV8NEmi1jXjj5oVfT812XyJPCtUDJ
Oxa04ZTU54tvnBEjf1g7ClAv35Di8xgi+g==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_vault-instance.key END
if id -u vault &>/dev/null && id -g vault &>/dev/null; then
  chown vault $TMP_SECRET_PATH
  chgrp vault $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/vault-instance.key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vault-instance.key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vault-instance.key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_epl-wireguard-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
KGSg42/wbjh4lT0jhKPSDY/5istBPuMjaRQrjNenEV0=
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
4078643d-2921-46ad-8e8a-2ec80af32a06
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
# ns1.epl-infra.net.	IN	A	10.17.1.11
# ns2.epl-infra.net.	IN	A	10.17.1.10
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.17.1.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.1.10
# us-west.epl-infra.net.	IN	DS	6484 15 2 1D8CF57B801B017A0552E23734E471BF6FA27392C9794DB63C42CFA8CD833626
#
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgpuczEuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xNy4xLjExCm5zMi5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCm5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMS4xMQpuczIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglEUwk2NDg0IDE1IDIgMUQ4Q0Y1N0I4MDFCMDE3QTA1NTJFMjM3MzRFNDcxQkY2RkEyNzM5MkM5Nzk0REI2M0M0MkNGQThDRDgzMzYyNgoK /run/named/private-epl-infra.net.zone
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
#
# 10.1.17.10.in-addr.arpa.	IN	PTR	ns2.epl-infra.net.
# 11.1.17.10.in-addr.arpa.	IN	PTR	ns1.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoKMTAuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIuZXBsLWluZnJhLm5ldC4KMTEuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEuZXBsLWluZnJhLm5ldC4KMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-10.in-addr.arpa.zone
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
# ns1.us-west.epl-infra.net.	IN	A	10.17.1.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.1.10
# server-a.us-west.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-west.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-west.epl-infra.net.	IN	A	10.17.1.10
# server-d.us-west.epl-infra.net.	IN	A	10.17.1.11
# server-e.us-west.epl-infra.net.	IN	A	10.17.0.12
# server-f.us-west.epl-infra.net.	IN	A	10.17.1.12
maybe_update_dns_file JFRUTCAzNjAwCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCm5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMS4xMQpuczIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKc2VydmVyLWQudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTEKc2VydmVyLWUudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTIKc2VydmVyLWYudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTIK /run/named/private-us-west.epl-infra.net.zone
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
#
# 10.0.17.10.in-addr.arpa.	IN	PTR	server-a.us-west.epl-infra.net.
# 10.1.17.10.in-addr.arpa.	IN	PTR	ns2.us-west.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	server-b.us-west.epl-infra.net.
# 11.1.17.10.in-addr.arpa.	IN	PTR	ns1.us-west.epl-infra.net.
# 12.0.17.10.in-addr.arpa.	IN	PTR	server-e.us-west.epl-infra.net.
# 12.1.17.10.in-addr.arpa.	IN	PTR	server-f.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCgoxMC4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1hLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTAuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMS4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1iLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMi4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1lLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTIuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-17.10.in-addr.arpa.zone
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
# ns1.epl-infra.net.	IN	A	77.77.77.13
# ns2.epl-infra.net.	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.13
# adm-consul-us-west	IN	A	77.77.77.12
# adm-consul-us-west	IN	A	77.77.77.13
# adm-grafana-main	IN	A	77.77.77.12
# adm-grafana-main	IN	A	77.77.77.13
# adm-minio-global	IN	A	77.77.77.12
# adm-minio-global	IN	A	77.77.77.13
# adm-nomad-us-west	IN	A	77.77.77.12
# adm-nomad-us-west	IN	A	77.77.77.13
# adm-prometheus-default	IN	A	77.77.77.12
# adm-prometheus-default	IN	A	77.77.77.13
# adm-vault-us-west	IN	A	77.77.77.12
# adm-vault-us-west	IN	A	77.77.77.13
# admin	IN	A	77.77.77.12
# admin	IN	A	77.77.77.13
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgpuczEuZXBsLWluZnJhLm5ldC4JSU4JQQk3Ny43Ny43Ny4xMwpuczIuZXBsLWluZnJhLm5ldC4JSU4JQQk3Ny43Ny43Ny4xMgphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMgphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMwphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMgphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMwphZG0tZ3JhZmFuYS1tYWluCUlOCUEJNzcuNzcuNzcuMTIKYWRtLWdyYWZhbmEtbWFpbglJTglBCTc3Ljc3Ljc3LjEzCmFkbS1taW5pby1nbG9iYWwJSU4JQQk3Ny43Ny43Ny4xMgphZG0tbWluaW8tZ2xvYmFsCUlOCUEJNzcuNzcuNzcuMTMKYWRtLW5vbWFkLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMgphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjEzCmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMgphZG0tcHJvbWV0aGV1cy1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTMKYWRtLXZhdWx0LXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMgphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjEzCmFkbWluCUlOCUEJNzcuNzcuNzcuMTIKYWRtaW4JSU4JQQk3Ny43Ny43Ny4xMwo= /run/named/public-epl-infra.net.zone
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
#
# 12.77.77.77.in-addr.arpa.	IN	PTR	admin
# 13.77.77.77.in-addr.arpa.	IN	PTR	admin
maybe_update_dns_file JFRUTCAzNjAwCmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQppbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoKMTIuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTMuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4K /run/named/public-in-addr.arpa.zone


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
provision_zfs_dataset rpool acme /var/lib/acme 128k on no B56uihrGwCxSidWTrZGLzcALjmByUU2THNWxzPmuW8
provision_zfs_dataset rpool docker /var/lib/docker 128k on no
provision_zfs_dataset rpool minio-docker-d /srv/volumes/minio-docker-d 1M on yes jnoNPOP6rYmcFRBwJ93VEXPTfaEkpoGKTlwAHHoR5a
provision_zfs_dataset rpool mon-am /srv/volumes/mon-am 4k on yes TNvvZanRVlogctNDdk17s9tIPAd7bRF2f3fTqwpJb2
provision_zfs_dataset rpool nats1 /srv/volumes/nats1 4k on yes Af54fUD15vDR4kCSVIIjQCmHVIHNQO8sAqI9pK63Ef
provision_zfs_dataset rpool nomad /var/lib/nomad 4k on no PGOhtTqDmJguKSZRkV3phaK9GlbxA52OkkMByfMAuw
provision_zfs_dataset rpool vault /var/lib/vault 4k on no HrrvuNTuUD4bGgGS6G06TGZGzFBzPfTTLkAneshyTA

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
        "epl-nix-cache:uatIAjmqeqj2086Q/vBPHwWMhzl1RNv42P0HCODYt7g="
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
MIIB0jCCAXmgAwIBAgIUZcywHAjaf3VMsJX5eEYP4qKj0wwwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MDEwMjA5NDIwMFoXDTQwMTIyODA5NDIw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABArAwadapfxcJafTovGVtMQw
dO0ZxCAZG3qn7iipJ/M4GQL/nq5wnaj63FW1vxbBlVknWk3TUnmvuGpUuOuTJ7Kj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSJO5NCIqik6pSW9N+T/xWK6AzR
RDAfBgNVHSMEGDAWgBSamtoWbzJ+jIAB8MhJ1OI6c6Jo7jA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNHADBEAiAC5oEUuyFaJmajsmStVfLX3PdNGEZjVvfWG3LH
a6CuSAIgfjNyzX2BOvhn/K2y46JgFpfj/nu/C3W2Fpd4kDWHiSI=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBYzCCAQqgAwIBAgIUbhdnPpWBtwSe1fcmpY2ZrFEdJ18wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwMTAyMDk0MjAwWhcNNDAxMjI4MDk0MjAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABC8m
xpk5pJmH+4NQKB6uEumI1nD99PUafiVUcBpLSYJFRVjWSk7VsaDRVk7T9PUWHeIk
HEhU29Ik79MGsZXyK32jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQDHNfmYT+QvTEVjYs7r8p5SFPnPzAKBggqhkjOPQQDAgNH
ADBEAiAzotLfmy91WmKI7UI1MVF94M+ZAZd6ZRwQvs0oT291bwIgQ6Dwxd+l68dE
N1yKhQjTOuvQN9HQeA199ZXhSe1whpc=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUHHyb2Jl3Co+G2CbFoaLuGqB1UfwwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwMTAyMDk0MjAwWhcNNDAxMjI4MDk0MjAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABKCY
1U9bIO5x0Sv/q41gNW7zAx7TP/fHYziXTBKj0k9xhQzZPp4t8TIZiA478OaCYIrb
e18Pz8wm+yGuC2x7DdujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBRh3xc1NBqq3ZuYRH7qZVeLHEh3ejAKBggqhkjOPQQDAgNJ
ADBGAiEAhr3aStAzY2cCOWFaX+ZyIeBsG34vYE+tI23XD6VkMh4CIQCVFcheXMZs
gCerkeGRQNBsHNY/6r1Jp3IM9Nv6ydkA/A==
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


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.0.0p24 || echo '
            # ROUTES CREATE
            ip route add 0.0.0.0/0 via 10.17.0.10

            # ROUTES DELETE
            ip route del 0.0.0.0/0

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/10.17.1.0p24 || echo '
            # ROUTES CREATE
            ip route add 0.0.0.0/0 via 10.17.1.10

            # ROUTES DELETE
            ip route del 0.0.0.0/0

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc1.txt


# NIX REGION consul_vrrp_bootstrap_script END

        '';
        epl-consul-vrrp-switch = pkgs.writeShellScriptBin "epl-consul-vrrp-switch" ''

# NIX REGION consul_vrrp_switch_script START

/run/current-system/sw/bin/echo '
# ROUTES CREATE
ip route add 0.0.0.0/0 via 10.17.1.11

# ROUTES DELETE
ip route del 0.0.0.0/0

# FINISH
' | \
  CONSUL_HTTP_TOKEN=$( ${pkgs.coreutils}/bin/cat /run/keys/consul-vrrp-token.txt ) \
  ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.1.0p24 -

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
        epl-vault-operator-init = pkgs.writeShellScriptBin "epl-vault-operator-init" ''

export VAULT_ADDR=$1
# pass initial text as argument or init vault

${pkgs.vault-bin}/bin/vault operator init &> /run/secdir/vault-init-output.txt.tmp

if grep -e 'Initial Root Token:' /run/secdir/vault-init-output.txt.tmp &> /dev/null
then
    mv /run/secdir/vault-init-output.txt.tmp /run/secdir/vault-init-output.txt

    echo Unsealing initial vault after 3 seconds...
    sleep 3

    for UK in $(seq 1 3)
    do
        UNSEAL_KEY=$( cat /run/secdir/vault-init-output.txt | grep "Unseal Key $UK:" | sed -E 's/^.*: //' )
        ${pkgs.vault-bin}/bin/vault operator unseal $UNSEAL_KEY
    done

    export VAULT_TOKEN=$( cat /run/secdir/vault-init-output.txt | grep "Initial Root Token:" | sed -E 's/^.*: //' )
    ${pkgs.vault-bin}/bin/vault secrets enable -path=epl kv-v2 || true
fi

        '';
        epl-vault-operator-unseal = pkgs.writeShellScriptBin "epl-vault-operator-unseal" ''

export VAULT_ADDR=$1
# pass initial text as argument or init vault
VAULT_KEY_BASE64=$2

export VAULT_INIT_RES=$( echo "$VAULT_KEY_BASE64" | base64 -d )

if [ -z "$VAULT_INIT_RES" ]
then
    echo Failed to initialize vault/key is not set
    exit 7
fi

# make sure vault is responsive before we try to unseal
while ! curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"initialized":'
do
  sleep 3
done

# make sure vault is initialized
while ! curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"initialized":true'
do
  sleep 3
done

# in case we need to restart due to raft logs
if sudo journalctl -u vault.service --since "$(systemctl show vault.service -p ExecMainStartTimestamp | cut -d= -f2)" | grep 'no TLS config found' &>/dev/null
then
  echo "Restarting vault and waiting 10 seconds"
  sudo systemctl restart vault.service
  sleep 10
fi

if curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"sealed":true'
then
  for UK in $(seq 1 3)
  do
    UNSEAL_KEY=$( echo "$VAULT_INIT_RES" | grep "Unseal Key $UK:" | sed -E 's/^.*: //' )
    ${pkgs.vault-bin}/bin/vault operator unseal $UNSEAL_KEY
  done
fi

export VAULT_TOKEN=$( echo "$VAULT_INIT_RES" | grep "Initial Root Token:" | sed -E 's/^.*: //' )
${pkgs.vault-bin}/bin/vault secrets enable -path=epl kv-v2 || true

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
        epl-nomad-vault-policies
        epl-vault-operator-init
        epl-vault-operator-unseal
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "169.254.169.254"; prefixLength = 32; via = "10.17.1.1"; }

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
           iifname void ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 77.77.77.13/32 } drop comment "Disallow traffic from internet to internal networks";
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

    "eth1"

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
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-d.dc1.us-west.aws-single-dc-multisub:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-d.dc1.us-west.aws-single-dc-multisub: \w\a\]\u@server-d.dc1.us-west.aws-single-dc-multisub:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-d.dc1.us-west.aws-single-dc-multisub:\u:\w\007\]$PS1"
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
  epl-infra.net. initial-key 257 3 15 "X0AjsCmQ1cpIltZ18OKk7NfRUhOib8zj3uT1/T6OIu8=";
  us-west.epl-infra.net. initial-key 257 3 15 "66IGH5NhA/ezssjM92SrnTkIkwqKlLeqYMHdzFKszRo=";
  10.in-addr.arpa. initial-key 257 3 15 "RCuSlvf+tmrNojBnBYX6B51k0SkQjfOTkZ7cAJ6rDYU=";
  17.10.in-addr.arpa. initial-key 257 3 15 "VyB/VcVOYDm43EAd333l+PWTsNKFp8PqARCJHOX7wlk=";
  in-addr.arpa. initial-key 257 3 15 "c/YvDQexvTeo7asHWp2P6MDph2DIIDcF9xWjzjL8/IM=";
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
    10.17.1.10;
  };
};
zone "10.in-addr.arpa." {
  type master;
  file "/run/named/private-10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.1.10;
  };
};
zone "us-west.epl-infra.net." {
  type master;
  file "/run/named/private-us-west.epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.1.10;
  };
};
zone "17.10.in-addr.arpa." {
  type master;
  file "/run/named/private-17.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.1.10;
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
    77.77.77.12;
  };
};
zone "in-addr.arpa." {
  type master;
  file "/run/named/public-in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
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

    users.users.vault = {
        isSystemUser = true;
        description = "Vault service";
        extraGroups = ["keys"];
        group = "vault";
    };
    users.groups.vault = {};

    systemd.services.vault = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];

      serviceConfig = {
        User = "vault";
        Group = "vault";
        Type = "simple";
        ExecStartPre = [
            "+${pkgs.coreutils}/bin/mkdir -p /var/lib/vault"
            "+${pkgs.coreutils}/bin/chown vault:vault /var/lib/vault"
            "+${pkgs.coreutils}/bin/chmod 700 /var/lib/vault"
        ];
        ExecStart = "${pkgs.vault-bin}/bin/vault server -config=/run/keys/vault_config.hcl -log-level=info";
        Restart = "always";
        RestartSec = "20";
        TasksMax = "infinity";
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
          " --web.listen-address=10.17.1.11:9100" +
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
          " --listen_ip=10.17.1.11" +
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
ip address add "172.21.7.11/16" dev "wg0" || true
wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
ip link set up dev "wg0"

# peers
wg set wg0 peer "s9VoP4qXE4lFFLk6yiD2gFsAtoNiczDYuIWFCwVGTxk=" allowed-ips "172.21.7.254/32"

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
               ip saddr 10.17.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
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
          ospf router-id 10.17.1.11
          redistribute bgp
          network 10.17.0.0/16 area 10.17.0.0
          area 10.17.0.0 range 10.17.0.0/16 advertise
          area 10.17.0.0 range 0.0.0.0/0 not-advertise
          area 10.17.0.0 authentication message-digest
          default-information originate always
          neighbor 10.17.252.10
          neighbor 10.17.252.11
          neighbor 10.17.252.12
        !
        interface eth1
          ip ospf cost 100
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 KdVBIXIuMHuTxutM
          ip ospf authentication message-digest
          ip ospf network non-broadcast
        !
        interface eth0
          ip ospf cost 500
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 KdVBIXIuMHuTxutM
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
          peer 10.17.1.11
            no shutdown
          peer 172.21.7.11
            no shutdown
      '';
  };
# NIX REGION frr_bfd_config END

# NIX REGION frr_bgp_config START

  services.frr.bgp = {
      enable = true;
      config = ''
        !
        router bgp 64529
          bgp router-id 10.17.1.11
          address-family ipv4 unicast
            network 10.17.0.0/16
          exit-address-family
          neighbor 10.17.252.10 remote-as 64529
          neighbor 10.17.252.10 password KRL2Qd6eEN48G9tbSY0cNwffUl9EBTe9NkHkoxoV6i
          neighbor 10.17.252.10 bfd
          neighbor 10.17.252.11 remote-as 64529
          neighbor 10.17.252.11 password giHJUpmVtpvII6XQjpgH06za4S33h4GgWwULd1Y0Ax
          neighbor 10.17.252.11 bfd
          neighbor 10.17.252.12 remote-as 64529
          neighbor 10.17.252.12 password cj2UNksGECdcxABR5tRaKFfjNDzkFGKYotdCPeRzhk
          neighbor 10.17.252.12 bfd
          address-family ipv4 unicast
            network 10.17.0.0/16
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
          set src 10.17.1.11
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol ospf route-map LANRM
        !
        ip protocol bgp route-map LANRM
        !
        ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
        !
        route-map LANRM deny 90
          match ip address prefix-list INTERSUBNET
        !
        interface eth1
          ip address 10.17.252.13/22
        !
        interface eth0
          ip address 10.17.1.11/24
      '';
  };
# NIX REGION frr_zebra_config END

# NIX REGION frr_static_routes START

  # You gotta be kidding me... https://github.com/NixOS/nixpkgs/issues/274286
  services.frr.mgmt.enable = true;
  environment.etc."frr/staticd.conf".text = ''
        !
        ip route 10.17.0.0/16 10.17.1.1

        !
        ip route 0.0.0.0/0 10.17.1.1

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
      { address = "10.17.1.11"; prefixLength = 24; }

    ];

    networking.interfaces.eth1.ipv4.addresses = [
      { address = "10.17.252.13"; prefixLength = 22; }

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
# ns1.epl-infra.net.	IN	A	10.17.1.11
# ns2.epl-infra.net.	IN	A	10.17.1.10
# us-west.epl-infra.net.	IN	NS	ns1.us-west.epl-infra.net.
# us-west.epl-infra.net.	IN	NS	ns2.us-west.epl-infra.net.
# ns1.us-west.epl-infra.net.	IN	A	10.17.1.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.1.10
# us-west.epl-infra.net.	IN	DS	6484 15 2 1D8CF57B801B017A0552E23734E471BF6FA27392C9794DB63C42CFA8CD833626
#
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgpuczEuZXBsLWluZnJhLm5ldC4JSU4JQQkxMC4xNy4xLjExCm5zMi5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCm5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMS4xMQpuczIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglEUwk2NDg0IDE1IDIgMUQ4Q0Y1N0I4MDFCMDE3QTA1NTJFMjM3MzRFNDcxQkY2RkEyNzM5MkM5Nzk0REI2M0M0MkNGQThDRDgzMzYyNgoK /run/named/private-epl-infra.net.zone
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
#
# 10.1.17.10.in-addr.arpa.	IN	PTR	ns2.epl-infra.net.
# 11.1.17.10.in-addr.arpa.	IN	PTR	ns1.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns1.us-west.epl-infra.net.
# 17.10.in-addr.arpa.	IN	NS	ns2.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0LgoxMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoKMTAuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIuZXBsLWluZnJhLm5ldC4KMTEuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEuZXBsLWluZnJhLm5ldC4KMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-10.in-addr.arpa.zone
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
# ns1.us-west.epl-infra.net.	IN	A	10.17.1.11
# ns2.us-west.epl-infra.net.	IN	A	10.17.1.10
# server-a.us-west.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-west.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-west.epl-infra.net.	IN	A	10.17.1.10
# server-d.us-west.epl-infra.net.	IN	A	10.17.1.11
# server-e.us-west.epl-infra.net.	IN	A	10.17.0.12
# server-f.us-west.epl-infra.net.	IN	A	10.17.1.12
maybe_update_dns_file JFRUTCAzNjAwCnVzLXdlc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0Lgp1cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCm5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTcuMS4xMQpuczIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKc2VydmVyLWEudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTAKc2VydmVyLWQudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTEKc2VydmVyLWUudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTIKc2VydmVyLWYudXMtd2VzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjEuMTIK /run/named/private-us-west.epl-infra.net.zone
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
#
# 10.0.17.10.in-addr.arpa.	IN	PTR	server-a.us-west.epl-infra.net.
# 10.1.17.10.in-addr.arpa.	IN	PTR	ns2.us-west.epl-infra.net.
# 11.0.17.10.in-addr.arpa.	IN	PTR	server-b.us-west.epl-infra.net.
# 11.1.17.10.in-addr.arpa.	IN	PTR	ns1.us-west.epl-infra.net.
# 12.0.17.10.in-addr.arpa.	IN	PTR	server-e.us-west.epl-infra.net.
# 12.1.17.10.in-addr.arpa.	IN	PTR	server-f.us-west.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy13ZXN0LmVwbC1pbmZyYS5uZXQuIHVzLXdlc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTcuMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxNy4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCgoxMC4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1hLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTAuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczIudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMS4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1iLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTEuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEudXMtd2VzdC5lcGwtaW5mcmEubmV0LgoxMi4wLjE3LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1lLnVzLXdlc3QuZXBsLWluZnJhLm5ldC4KMTIuMS4xNy4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy13ZXN0LmVwbC1pbmZyYS5uZXQuCg== /run/named/private-17.10.in-addr.arpa.zone
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
# ns1.epl-infra.net.	IN	A	77.77.77.13
# ns2.epl-infra.net.	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.12
# adm-alertmanager-default	IN	A	77.77.77.13
# adm-consul-us-west	IN	A	77.77.77.12
# adm-consul-us-west	IN	A	77.77.77.13
# adm-grafana-main	IN	A	77.77.77.12
# adm-grafana-main	IN	A	77.77.77.13
# adm-minio-global	IN	A	77.77.77.12
# adm-minio-global	IN	A	77.77.77.13
# adm-nomad-us-west	IN	A	77.77.77.12
# adm-nomad-us-west	IN	A	77.77.77.13
# adm-prometheus-default	IN	A	77.77.77.12
# adm-prometheus-default	IN	A	77.77.77.13
# adm-vault-us-west	IN	A	77.77.77.12
# adm-vault-us-west	IN	A	77.77.77.13
# admin	IN	A	77.77.77.12
# admin	IN	A	77.77.77.13
maybe_update_dns_file JFRUTCAzNjAwCmVwbC1pbmZyYS5uZXQuCUlOCVNPQQluczEuZXBsLWluZnJhLm5ldC4gZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKZXBsLWluZnJhLm5ldC4JSU4JTlMJbnMxLmVwbC1pbmZyYS5uZXQuCmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgpuczEuZXBsLWluZnJhLm5ldC4JSU4JQQk3Ny43Ny43Ny4xMwpuczIuZXBsLWluZnJhLm5ldC4JSU4JQQk3Ny43Ny43Ny4xMgphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMgphZG0tYWxlcnRtYW5hZ2VyLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMwphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMgphZG0tY29uc3VsLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMwphZG0tZ3JhZmFuYS1tYWluCUlOCUEJNzcuNzcuNzcuMTIKYWRtLWdyYWZhbmEtbWFpbglJTglBCTc3Ljc3Ljc3LjEzCmFkbS1taW5pby1nbG9iYWwJSU4JQQk3Ny43Ny43Ny4xMgphZG0tbWluaW8tZ2xvYmFsCUlOCUEJNzcuNzcuNzcuMTMKYWRtLW5vbWFkLXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMgphZG0tbm9tYWQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjEzCmFkbS1wcm9tZXRoZXVzLWRlZmF1bHQJSU4JQQk3Ny43Ny43Ny4xMgphZG0tcHJvbWV0aGV1cy1kZWZhdWx0CUlOCUEJNzcuNzcuNzcuMTMKYWRtLXZhdWx0LXVzLXdlc3QJSU4JQQk3Ny43Ny43Ny4xMgphZG0tdmF1bHQtdXMtd2VzdAlJTglBCTc3Ljc3Ljc3LjEzCmFkbWluCUlOCUEJNzcuNzcuNzcuMTIKYWRtaW4JSU4JQQk3Ny43Ny43Ny4xMwo= /run/named/public-epl-infra.net.zone
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
#
# 12.77.77.77.in-addr.arpa.	IN	PTR	admin
# 13.77.77.77.in-addr.arpa.	IN	PTR	admin
maybe_update_dns_file JFRUTCAzNjAwCmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS5lcGwtaW5mcmEubmV0LiBlcGwtaW5mcmEubmV0LiAoCiBTRVJJQUxfVE9fUkVQTEFDRSA7IFNlcmlhbAogMzYwMCA7IFJlZnJlc2gKIDE4MDAgOyBSZXRyeQogNjA0ODAwIDsgRXhwaXJlCiAzNjAwIDsgTWluaW11bSBUVEwKKQppbi1hZGRyLmFycGEuCUlOCU5TCW5zMS5lcGwtaW5mcmEubmV0Lgppbi1hZGRyLmFycGEuCUlOCU5TCW5zMi5lcGwtaW5mcmEubmV0LgoKMTIuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4KMTMuNzcuNzcuNzcuaW4tYWRkci5hcnBhLglJTglQVFIJYWRtaW4K /run/named/public-in-addr.arpa.zone



# we could implement some complex mechanism
# to detect if zone files changed later
/run/current-system/sw/bin/systemctl reload bind.service || true

# zone file changed, reload will not reload it
if [ -f /run/restart-bind ]
then
  rm -f /run/restart-bind
  /run/current-system/sw/bin/systemctl restart bind.service || true
fi


cp -pu /run/keys/K10-in-addr-arpa--015-39018-private /run/dnsseckeys/K10.in-addr.arpa.+015+39018.private
cp -pu /run/keys/K10-in-addr-arpa--015-04946-private /run/dnsseckeys/K10.in-addr.arpa.+015+04946.private
cp -pu /run/keys/K10-in-addr-arpa--015-39018-key /run/dnsseckeys/K10.in-addr.arpa.+015+39018.key
cp -pu /run/keys/K10-in-addr-arpa--015-04946-key /run/dnsseckeys/K10.in-addr.arpa.+015+04946.key
cp -pu /run/keys/K17-10-in-addr-arpa--015-18308-private /run/dnsseckeys/K17.10.in-addr.arpa.+015+18308.private
cp -pu /run/keys/K17-10-in-addr-arpa--015-57593-private /run/dnsseckeys/K17.10.in-addr.arpa.+015+57593.private
cp -pu /run/keys/K17-10-in-addr-arpa--015-18308-key /run/dnsseckeys/K17.10.in-addr.arpa.+015+18308.key
cp -pu /run/keys/K17-10-in-addr-arpa--015-57593-key /run/dnsseckeys/K17.10.in-addr.arpa.+015+57593.key
cp -pu /run/keys/Kepl-infra-net--015-56109-private /run/dnsseckeys/Kepl-infra.net.+015+56109.private
cp -pu /run/keys/Kepl-infra-net--015-02768-private /run/dnsseckeys/Kepl-infra.net.+015+02768.private
cp -pu /run/keys/Kepl-infra-net--015-56109-key /run/dnsseckeys/Kepl-infra.net.+015+56109.key
cp -pu /run/keys/Kepl-infra-net--015-02768-key /run/dnsseckeys/Kepl-infra.net.+015+02768.key
cp -pu /run/keys/Kus-west-epl-infra-net--015-13820-private /run/dnsseckeys/Kus-west.epl-infra.net.+015+13820.private
cp -pu /run/keys/Kus-west-epl-infra-net--015-06484-private /run/dnsseckeys/Kus-west.epl-infra.net.+015+06484.private
cp -pu /run/keys/Kus-west-epl-infra-net--015-13820-key /run/dnsseckeys/Kus-west.epl-infra.net.+015+13820.key
cp -pu /run/keys/Kus-west-epl-infra-net--015-06484-key /run/dnsseckeys/Kus-west.epl-infra.net.+015+06484.key

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
epl_l1_provisioning_last_hash{hash=\"f14cf811415db9686c0e8cb284cfe7a4dcd4151d98d8d79e498f242e7a063d7d\",hostname=\"server-d\"} $BOOT_TIME
" > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE

fi
