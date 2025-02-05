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


/run/current-system/sw/bin/ip route del 10.17.0.1/32 || true
/run/current-system/sw/bin/ip route add 10.17.0.1/32 via 10.18.0.1
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
      "agent": "4ac38cc1-1169-4087-b00b-8f36601aecd8",
      "default": "47802b8c-9b8e-44ba-9882-3ef57592486e"
    }
  },
  "addresses": {
    "dns": "127.0.0.1",
    "grpc": "127.0.0.1",
    "http": "127.0.0.1",
    "https": "10.18.0.13"
  },
  "advertise_addr": "10.18.0.13",
  "advertise_addr_wan": "10.18.0.13",
  "auto_encrypt": {
    "tls": true
  },
  "bind_addr": "10.18.0.13",
  "client_addr": "127.0.0.1",
  "data_dir": "/var/lib/consul",
  "datacenter": "us-east",
  "disable_update_check": false,
  "domain": "consul",
  "enable_local_script_checks": false,
  "enable_script_checks": false,
  "encrypt": "n4HGSAOf4DINpyrQguuPBGQetvDxeQ+RRQO3pVQZq0g=",
  "encrypt_verify_incoming": true,
  "encrypt_verify_outgoing": true,
  "limits": {
    "rpc_max_conns_per_client": 1000
  },
  "log_level": "INFO",
  "log_rotate_bytes": 0,
  "log_rotate_duration": "24h",
  "log_rotate_max_files": 0,
  "node_name": "server-h",
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
    "10.18.0.10",
    "10.18.0.11",
    "10.18.0.12"
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


name = "server-h"
region = "us-east"
datacenter = "dc2"

enable_debug = false
disable_update_check = false


bind_addr = "10.18.0.13"
advertise {
    http = "10.18.0.13:4646"
    rpc = "10.18.0.13:4647"
    serf = "10.18.0.13:4648"
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
    token = "bf67b77a-d352-4861-8ef2-4e7236255289"
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
    token = "hvs.CAESIJ7NaquMzvVZpvhSXj65x7QHqH12_Kh3DjcMfGWRYRyTGh4KHGh2cy5JdElOY3ppUWwyR0tiWlBNZG9uNENIbHI"

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
    "private_ip" = "10.18.0.13"
    "run_unassigned_workloads" = "1"
    "lock_epl-ingress-us-east" = "1"
    "lock_epl-minio-server-h-main-us-east" = "1"
    "lock_epl-mon-server-h-am-main-us-east" = "1"
    "lock_epl-mon-server-h-main-us-east" = "1"
  }

  host_volume "minio-docker" {
    path = "/srv/xfs-jbods/vdb/minio-docker"
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

  host_volume "mon-secondary" {
    path = "/srv/volumes/mon-secondary"
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

    encrypt = "o5i/YatCMlIP5sbnRsserUOXPRJ1dcZ3YaLLfHEKftA="

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

cluster_name = "us-east"
max_lease_ttl = "768h"
default_lease_ttl = "768h"

disable_clustering = "False"
cluster_addr = "https://server-h.us-east.epl-infra.net:8201"
api_addr = "https://server-h.us-east.epl-infra.net:8200"

plugin_directory = "/usr/local/lib/vault/plugins"

listener "tcp" {
  address = "10.18.0.13:8200"
  cluster_address = "10.18.0.13:8201"
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
  node_id = "server-h.us-east.epl-infra.net"

  retry_join {
    leader_api_addr = "https://server-f.us-east.epl-infra.net:8200"
    leader_ca_cert_file = "/run/keys/vault-ca.crt"
    leader_client_cert_file = "/run/keys/vault-instance.crt"
    leader_client_key_file = "/run/keys/vault-instance.key"
  }

  retry_join {
    leader_api_addr = "https://server-g.us-east.epl-infra.net:8200"
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
  service_address = "10.18.0.13"
  token = "76a82c06-cff8-4f17-8421-bb8a009743d0"
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
  tags = ["epl-mon-main-us-east"]

  meta = {
    metrics_path = "/metrics"
  }

  tagged_addresses = {
    lan = {
      address = "10.18.0.13"
      port    = 9100
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.18.0.13:9100/"
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
  tags = ["epl-mon-main-us-east"]

  tagged_addresses = {
    lan = {
      address = "10.18.0.13"
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
        http     = "http://10.18.0.13:9280/"
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
  tags = ["epl-mon-main-us-east"]

  tagged_addresses = {
    lan = {
      address = "10.18.0.13"
      port    = 9281
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "home"
        tcp      = "10.18.0.13:9281"
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
address = "10.18.0.13:9281"

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
endpoint = "http://epl-loki-main-us-east-loki-writer.service.consul:3020"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_journald.buffer]
type = "disk"
max_size = 268435488
when_full = "block"

[sinks.loki_journald.labels]
source_type = "journald"
host = "server-h.us-east.epl-infra.net"
systemd_unit = "{{ _SYSTEMD_UNIT }}"

# ----------------------------------
# loki l1 provisioning sink
# ----------------------------------
[sinks.loki_l1_provisioning]
type = "loki"
inputs = [ "l1_provisioning_logs_extra" ]
endpoint = "http://epl-loki-main-us-east-loki-writer.service.consul:3020"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l1_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l1_provisioning.labels]
source_type = "l1_provisioning"
host = "server-h.us-east.epl-infra.net"
file = "{{ filename }}"
provisioning_id = "{{ provisioning_id }}"

# ----------------------------------
# loki l2 provisioning sink
# ----------------------------------
[sinks.loki_l2_provisioning]
type = "loki"
inputs = [ "l2_provisioning_logs_extra" ]
endpoint = "http://epl-loki-main-us-east-loki-writer.service.consul:3020"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l2_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l2_provisioning.labels]
source_type = "l2_provisioning"
host = "server-h.us-east.epl-infra.net"
file = "{{ filename }}"
provisioning_id = "{{ provisioning_id }}"

# ----------------------------------
# loki nomad docker sink for main-us-east
# ----------------------------------
[transforms.loki_nomad_docker_router]
type = "route"
inputs = [ "docker" ]
[transforms.loki_nomad_docker_router.route]
main-us-east = '.label.epl_loki_cluster == "main-us-east"'


# ----------------------------------
# loki nomad docker sink for main-us-east
# ----------------------------------
[sinks.loki_nomad_docker_main-us-east]
type = "loki"
inputs = [ "loki_nomad_docker_router.main-us-east", "loki_nomad_docker_router._unmatched" ]
endpoint = "http://epl-loki-main-us-east-loki-writer.service.consul:3020"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_nomad_docker_main-us-east.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_nomad_docker_main-us-east.labels]
source_type = "nomad_docker"
host = "server-h.us-east.epl-infra.net"
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
  unicast_src_ip 10.18.0.13
  unicast_peer {
    10.18.0.12
  }
  virtual_ipaddress {
    10.18.0.2
  }

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
  tags = ["epl-mon-main-us-east"]

  meta = {
    metrics_path = "/metrics"
  }

  tagged_addresses = {
    lan = {
      address = "10.18.0.13"
      port    = 9134
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.18.0.13:9134/"
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
Xxnnu3Lzn844FStSU+XoBAsc6pPI7K6yW4q1QxmuRxk=
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
CgErcWy8j/cvI7ogCkTmzjGAMvnfCo/OdInd+hi/MHw=
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
MHcCAQEEINyga2A7n3EMUeKjwWLdpO0CZ/hC0warzaaUFd/dw69yoAoGCCqGSM49
AwEHoUQDQgAEPE70RJaGH2qyOJ0yE1Tj6qOvrlZyrE86tKCoXCnpuJ9TBiDsCO3T
ZMkUeDyJjw3rzRdCDH3XL5DqLXUX1AfZ6g==
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
MIIB1DCCAXmgAwIBAgIUd7erzd27ISQoaK9W/+varzigcDQwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MDMwNjAzMTcwMFoXDTQxMDMwMjAzMTcw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABDxO9ESWhh9qsjidMhNU4+qj
r65WcqxPOrSgqFwp6bifUwYg7Ajt02TJFHg8iY8N680XQgx91y+Q6i11F9QH2eqj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSKXn0KCwYZVsMCXDOJP5FNci9c
uDAfBgNVHSMEGDAWgBTPuqyRyh1VGZvEnL8MGN62Pca2XjA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNJADBGAiEA+Ul7J6RcwFcbNJFIVibLPd5Xn+uwSH3CF++I
oBujjP4CIQCBwtYU0t72IMLGA85R8BfjEmqNBMKGKkGnQ79ONhIMtg==
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

# NIX REGION secret_value_K10-in-addr-arpa--015-20386-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: OYKo9b0ENemfbDa3Js37Z/mlreq2AK9iFkLa5T0fTAU=
Created: 20240306032225
Publish: 20240306032225
Activate: 20240306032225
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-20386-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-20386-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-20386-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-20386-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-01203-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: 2PKAaB8NyMorKbHgZZ4tOnYQ6kdu6tCxF/I+OoE7ZRw=
Created: 20240306032225
Publish: 20240306032225
Activate: 20240306032225
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-01203-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-01203-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-01203-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-01203-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-20386-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 20386, for 10.in-addr.arpa.
; Created: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Publish: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Activate: 20240306032225 (Wed Mar  6 05:22:25 2024)
10.in-addr.arpa. IN DNSKEY 256 3 15 NgribxUefs+s3YJR/Pe71IaSOLpmD86d4JBPD+Gqseo=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-20386-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-20386-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-20386-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-20386-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K10-in-addr-arpa--015-01203-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 1203, for 10.in-addr.arpa.
; Created: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Publish: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Activate: 20240306032225 (Wed Mar  6 05:22:25 2024)
10.in-addr.arpa. IN DNSKEY 257 3 15 xziBKJb8w4Iu+ITuIHJu72RUx4/BvW5vg6QmkQmyCoE=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K10-in-addr-arpa--015-01203-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-01203-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K10-in-addr-arpa--015-01203-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K10-in-addr-arpa--015-01203-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-63576-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: rgkm7PhHQT3MkcO51QXJA4Xt0oVVb0/Og0/7OocYt98=
Created: 20240306032226
Publish: 20240306032226
Activate: 20240306032226
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-63576-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-63576-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-63576-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-63576-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-42662-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: n7JoRRFVxQ1hu7eCb5giBVBpwyuUg8WT8dlcbyOGUpY=
Created: 20240306032226
Publish: 20240306032226
Activate: 20240306032226
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-42662-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-42662-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-42662-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-42662-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-63576-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 63576, for 18.10.in-addr.arpa.
; Created: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Publish: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Activate: 20240306032226 (Wed Mar  6 05:22:26 2024)
18.10.in-addr.arpa. IN DNSKEY 256 3 15 75onwyq0il4nz/HkDcu3aD/UgaeT5iDZD0uOTn3qtzE=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-63576-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-63576-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-63576-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-63576-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_K18-10-in-addr-arpa--015-42662-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 42662, for 18.10.in-addr.arpa.
; Created: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Publish: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Activate: 20240306032226 (Wed Mar  6 05:22:26 2024)
18.10.in-addr.arpa. IN DNSKEY 257 3 15 v0qufAGopxffmXTs/kiXUfC64Q7DFrn23PbuzPDglnI=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_K18-10-in-addr-arpa--015-42662-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-42662-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/K18-10-in-addr-arpa--015-42662-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/K18-10-in-addr-arpa--015-42662-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-56589-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: PXlMxE3x50fZxJMQFHCN0fOEMMJnco9uTq04ZQE+IgI=
Created: 20240306032225
Publish: 20240306032225
Activate: 20240306032225
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-56589-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56589-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-56589-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56589-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-00239-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: IPySjznuWzieD4L18zpf711WlAOQE0pFUGVncAoS1M4=
Created: 20240306032225
Publish: 20240306032225
Activate: 20240306032225
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-00239-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-00239-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-00239-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-00239-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-56589-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 56589, for epl-infra.net.
; Created: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Publish: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Activate: 20240306032225 (Wed Mar  6 05:22:25 2024)
epl-infra.net. IN DNSKEY 256 3 15 joaF2KQBXjam8tvQJajs87Aj6EwMpu01zYoQieiH1B8=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-56589-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56589-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-56589-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-56589-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kepl-infra-net--015-00239-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 239, for epl-infra.net.
; Created: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Publish: 20240306032225 (Wed Mar  6 05:22:25 2024)
; Activate: 20240306032225 (Wed Mar  6 05:22:25 2024)
epl-infra.net. IN DNSKEY 257 3 15 baEW4FrpIp5KsX6jmedjTmZ2eHhnLg9XVCInj8IeoQY=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kepl-infra-net--015-00239-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-00239-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kepl-infra-net--015-00239-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kepl-infra-net--015-00239-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-east-epl-infra-net--015-47408-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: PmpnbIkVGMyjyeW1YqTNFnCybCLso3OCN+b469sKntw=
Created: 20240306032226
Publish: 20240306032226
Activate: 20240306032226
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-east-epl-infra-net--015-47408-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-47408-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-east-epl-infra-net--015-47408-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-47408-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-east-epl-infra-net--015-29833-private START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
Private-key-format: v1.3
Algorithm: 15 (ED25519)
PrivateKey: fiGVX4o1mUcvEEj+6d/uiuHHypYPiDMW9gzcjkiSsuk=
Created: 20240306032226
Publish: 20240306032226
Activate: 20240306032226
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-east-epl-infra-net--015-29833-private END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-29833-private || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-east-epl-infra-net--015-29833-private')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-29833-private
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-east-epl-infra-net--015-47408-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a zone-signing key, keyid 47408, for us-east.epl-infra.net.
; Created: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Publish: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Activate: 20240306032226 (Wed Mar  6 05:22:26 2024)
us-east.epl-infra.net. IN DNSKEY 256 3 15 HU1DLJERxQ7LPhFB0dupHnQF1d6h/5LFHwQyndtW+2s=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-east-epl-infra-net--015-47408-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-47408-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-east-epl-infra-net--015-47408-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-47408-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_Kus-east-epl-infra-net--015-29833-key START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
; This is a key-signing key, keyid 29833, for us-east.epl-infra.net.
; Created: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Publish: 20240306032226 (Wed Mar  6 05:22:26 2024)
; Activate: 20240306032226 (Wed Mar  6 05:22:26 2024)
us-east.epl-infra.net. IN DNSKEY 257 3 15 VA1DbcY1t1ChE+XY4HEubIbftzk6aQuoK9CW0knRNQ8=
LilBoiPeepLikesBenzTruck

# NIX REGION secret_value_Kus-east-epl-infra-net--015-29833-key END
if id -u named &>/dev/null && id -g named &>/dev/null; then
  chown named $TMP_SECRET_PATH
  chgrp named $TMP_SECRET_PATH
  unset NEEDS_MOVE
  cmp --silent $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-29833-key || NEEDS_MOVE=true
  [ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/Kus-east-epl-infra-net--015-29833-key')" ] || NEEDS_MOVE=true
  [ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/Kus-east-epl-infra-net--015-29833-key
fi
rm -f $TMP_SECRET_PATH || true
TMP_SECRET_PATH=/run/tmpsec-$RANDOM

# NIX REGION secret_value_consul-tls-ca-cert.pem START
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIC7jCCApSgAwIBAgIRAPQF0h0YxU1hnMTU/3HF1vcwCgYIKoZIzj0EAwIwgbkx
CzAJBgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNj
bzEaMBgGA1UECRMRMTAxIFNlY29uZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcw
FQYDVQQKEw5IYXNoaUNvcnAgSW5jLjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENB
IDMyNDM2MTg1NDA2NzExMTc5MDkzNjE1MjUwMTk5Mzg1NzIxMDEwMzAeFw0yNDAz
MDYwMzIyMjZaFw00MTAzMDIwMzIyMjZaMIG5MQswCQYDVQQGEwJVUzELMAkGA1UE
CBMCQ0ExFjAUBgNVBAcTDVNhbiBGcmFuY2lzY28xGjAYBgNVBAkTETEwMSBTZWNv
bmQgU3RyZWV0MQ4wDAYDVQQREwU5NDEwNTEXMBUGA1UEChMOSGFzaGlDb3JwIElu
Yy4xQDA+BgNVBAMTN0NvbnN1bCBBZ2VudCBDQSAzMjQzNjE4NTQwNjcxMTE3OTA5
MzYxNTI1MDE5OTM4NTcyMTAxMDMwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASC
w5OsEpnPe+U9+Y2G0zMmJD0Ee75GQnZITsxux+HMzy9o3uuS/N7qvRaXIX06+Gpn
6YUH22O8eGblgW+PXJsMo3sweTAOBgNVHQ8BAf8EBAMCAYYwDwYDVR0TAQH/BAUw
AwEB/zApBgNVHQ4EIgQgAH0QHslLUbAWNGV9sf5sqLRGFdwzt7wIMngWfBmIYnMw
KwYDVR0jBCQwIoAgAH0QHslLUbAWNGV9sf5sqLRGFdwzt7wIMngWfBmIYnMwCgYI
KoZIzj0EAwIDSAAwRQIgAR2vUqNBdxMXy3GxQjLFwUMsbVigbIhraXu1XpnWOHMC
IQCnPWE/4lSzZ9Hw0FKiy7z51IpthmYUKp4kObk2OQgU/w==
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
4ac38cc1-1169-4087-b00b-8f36601aecd8
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
MIIBZDCCAQqgAwIBAgIUc4FvA3RvBBPXWvSWIXczcEydUKswCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwMzA2MDMxNzAwWhcNNDEwMzAyMDMxNzAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABPPV
4+/0y5d27H63VDgtzIQsEmb+egE6eJeG2b/ZCMxmd/6P0MbBKyMpCAbCt53nx9tG
oUAa5TCX0L2lEGxzDvejQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSnSZoE5Lm8WxUSHdkXt2xVslRu9jAKBggqhkjOPQQDAgNI
ADBFAiA6pM4Uf2lWZtKEdC6m3k1NscZhqn43MWzKutZjG79nqQIhAIgVxMbGKeTR
tuGFfsrGBs00dlAuK3eOqUrTlp/nYMd0
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
MIIB5jCCAYygAwIBAgIUEu5QLcAJwlaTrQB86fpI24i2+GowCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwMzA2MDMxNzAwWhcNMjUwMzA2MDMxNzAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEw+EWjJkRlRsTQ44ggSE6Cj7l
Jv8kTjrvjtVxh/juLYZ0R3JeMAByJaEgXf8foRGRItpH/2+j/m2iugOgub8mbKOB
0zCB0DAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFHzA/rTo3rBJ1aRD8/RfBnefL4Fr
MB8GA1UdIwQYMBaAFKdJmgTkubxbFRId2Re3bFWyVG72MFEGA1UdEQEB/wRHMEWC
FHNlcnZlci51cy1lYXN0Lm5vbWFkghxub21hZC1zZXJ2ZXJzLnNlcnZpY2UuY29u
c3Vsgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIgbCynpJWB1ZwS
8DZuAIuOdSaeQ4ODB815JoPmU4J5NTMCIQClz80GHdF/5Jb9hm8wNThi8Z6Ungoz
uLRFrRj23Ov3Ug==
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
MHcCAQEEIKhYnRKvLXarTR4krS5HdQHo0+MdqmmqeJxEMd5F/UMHoAoGCCqGSM49
AwEHoUQDQgAEw+EWjJkRlRsTQ44ggSE6Cj7lJv8kTjrvjtVxh/juLYZ0R3JeMABy
JaEgXf8foRGRItpH/2+j/m2iugOgub8mbA==
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
MIIBkTCCATegAwIBAgIUDzNum8dRQiAvc+Xn0ruJGr2THw0wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwMzA2MDMxNzAwWhcNMjUwMzA2MDMxNzAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEzbdejVNP4RTGnC2o1EsE3CnA
AHqq6JKypAxWbav5lDJO4CPCnlW4xhkL1rvoJ3SaMSM+/6V6k/IznWu0+SzdM6N/
MH0wDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcD
AjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBQXNmo0y8MyMw5Tr3rHLYjvuciRGzAf
BgNVHSMEGDAWgBSnSZoE5Lm8WxUSHdkXt2xVslRu9jAKBggqhkjOPQQDAgNIADBF
AiEA9uULb2qJfMHr9LG0JzpBYMjDoZcKYxg0/0IvLYcDZXACIBVobNOvyG5tnklo
1oqMI7z1La4PcxO9zwPKYcLUlpZe
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
MHcCAQEEIP6NJyuMm2nRGPvN485KT7RZ42lMYd6r/LmpWuBkd8bJoAoGCCqGSM49
AwEHoUQDQgAEzbdejVNP4RTGnC2o1EsE3CnAAHqq6JKypAxWbav5lDJO4CPCnlW4
xhkL1rvoJ3SaMSM+/6V6k/IznWu0+SzdMw==
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
MIIBZDCCAQqgAwIBAgIUGBv/2biRVzUT/D0zakHmje+rjMUwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwMzA2MDMxNzAwWhcNNDEwMzAyMDMxNzAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABFFC
caW7VHrx7X/bJFCw9TSnpNt5cbsQ9meJs0x6lVZseOx0JPZCRxdC7DLQ7Z/yMnYM
YfnPYfCgT59yltMyC4ujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQ5Lk4e3Ph+VQWTssa0zkKOQaPiWzAKBggqhkjOPQQDAgNI
ADBFAiAgHuf7jj7X70GK9uSeaNxGC10HHbBJsGWq9WiGMTpEvgIhAMrsKjt7XnZp
M6rwEAfjvpRXN4je7ZR3HFz7wfYoAvvp
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
MIICQjCCAemgAwIBAgIUUQ5qVEgPb4fKGnd8aL+k3dV0pgkwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwMzA2MDMxNzAwWhcNMjUwMzA2MDMxNzAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEq99EEpesIRrDGo6K/odgFp9A
CRlEufB6lGRwnoOQPxp4x4RxS/V2oWULy7kZ5GhsrA2LvtW4cxtjTdXTYqlLsqOC
AS8wggErMA4GA1UdDwEB/wQEAwIFoDAdBgNVHSUEFjAUBggrBgEFBQcDAQYIKwYB
BQUHAwIwDAYDVR0TAQH/BAIwADAdBgNVHQ4EFgQU7mgkANOOjtYtAnfArpTxPZiV
edswHwYDVR0jBBgwFoAUOS5OHtz4flUFk7LGtM5CjkGj4lswgasGA1UdEQEB/wSB
oDCBnYIec2VydmVyLWgudXMtZWFzdC5lcGwtaW5mcmEubmV0ghR2YXVsdC5zZXJ2
aWNlLmNvbnN1bIIWKi52YXVsdC5zZXJ2aWNlLmNvbnN1bIIcdmF1bHQuc2Vydmlj
ZS51cy1lYXN0LmNvbnN1bIIeKi52YXVsdC5zZXJ2aWNlLnVzLWVhc3QuY29uc3Vs
gglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0EAwIDRwAwRAIgWEUKmea21yVpxB65
rGv0cVk61JLfbFCplN5fXTWmT70CIA58BQDw0+go4+290TyOZP0ijf1p/QyHHSFD
Wtuft7uX
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
MHcCAQEEIM8ux/n6ECgBrzjTtXSjnd004wJgAc/6j0PIrW73PLKvoAoGCCqGSM49
AwEHoUQDQgAEq99EEpesIRrDGo6K/odgFp9ACRlEufB6lGRwnoOQPxp4x4RxS/V2
oWULy7kZ5GhsrA2LvtW4cxtjTdXTYqlLsg==
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
iHoXGIx1yRR0XA8b0Ry024KZwBkd+rFZYFkk6e3aR0k=
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
  # bind journalfile will clash with zone file, we have the source
  # so journalfile is irrelevant for us
  if [ -f "$TARGET_FILE.jnl" ]
  then
    if [ "$TARGET_FILE" -nt "$TARGET_FILE.jnl" ]
    then
      echo "Deleting older journalfile $TARGET_FILE.jnl"
      rm -f $TARGET_FILE.jnl
    fi
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
# us-east.epl-infra.net.	IN	SOA	ns1.us-east.epl-infra.net. us-east.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# us-east.epl-infra.net.	IN	NS	ns1.us-east.epl-infra.net.
# us-east.epl-infra.net.	IN	NS	ns2.us-east.epl-infra.net.
# ns1.us-east.epl-infra.net.	IN	A	10.18.0.13
# ns2.us-east.epl-infra.net.	IN	A	10.18.0.12
# server-a.us-east.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-east.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-east.epl-infra.net.	IN	A	10.17.0.12
# server-d.us-east.epl-infra.net.	IN	A	10.17.0.13
# server-e.us-east.epl-infra.net.	IN	A	10.18.0.10
# server-f.us-east.epl-infra.net.	IN	A	10.18.0.11
# server-g.us-east.epl-infra.net.	IN	A	10.18.0.12
# server-h.us-east.epl-infra.net.	IN	A	10.18.0.13
maybe_update_dns_file JFRUTCAzNjAwCnVzLWVhc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy1lYXN0LmVwbC1pbmZyYS5uZXQuIHVzLWVhc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtZWFzdC5lcGwtaW5mcmEubmV0Lgp1cy1lYXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCm5zMS51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTguMC4xMwpuczIudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTIKc2VydmVyLWEudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTIKc2VydmVyLWQudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTMKc2VydmVyLWUudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWYudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTEKc2VydmVyLWcudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTIKc2VydmVyLWgudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTMK /run/named/private-us-east.epl-infra.net.zone
# $TTL 3600
# 18.10.in-addr.arpa.	IN	SOA	ns1.us-east.epl-infra.net. us-east.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 18.10.in-addr.arpa.	IN	NS	ns1.us-east.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-east.epl-infra.net.
#
# 10.0.18.10.in-addr.arpa.	IN	PTR	server-e.us-east.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	server-f.us-east.epl-infra.net.
# 12.0.18.10.in-addr.arpa.	IN	PTR	ns2.us-east.epl-infra.net.
# 13.0.18.10.in-addr.arpa.	IN	PTR	ns1.us-east.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy1lYXN0LmVwbC1pbmZyYS5uZXQuIHVzLWVhc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtZWFzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCgoxMC4wLjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1lLnVzLWVhc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMyLnVzLWVhc3QuZXBsLWluZnJhLm5ldC4KMTMuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEudXMtZWFzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-18.10.in-addr.arpa.zone


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
provision_zfs_dataset rpool docker /var/lib/docker 128k on no
provision_zfs_dataset rpool mon-am /srv/volumes/mon-am 4k on yes js6DIlseXyXN0F4nafC6PIDrtIDODYCO1QkwvrK7JB
provision_zfs_dataset rpool mon-default /srv/volumes/mon-default 4k on yes GJAVCHpPrmPpatNX3nhzLe8T7nSqUalkHj6b0imrSK
provision_zfs_dataset rpool mon-secondary /srv/volumes/mon-secondary 4k on yes 7B9IzWJiII4t0Z1TyKKYDBtW2RmANtIIbhmunvHam3
provision_zfs_dataset rpool nats1 /srv/volumes/nats1 4k on yes akLDaSmsHJLENCAw3OBLieDZFbDS5m3FuLGOSEJDZv
provision_zfs_dataset rpool nomad /var/lib/nomad 4k on no oI0Qiy5cCsRfudZMxPMY4xyXpJyqNIOXzW5Z7xueG5
provision_zfs_dataset rpool vault /var/lib/vault 4k on no Xw0I9hFetqa3lqEnJOCZF6RV7h6oocd5LR1CPQDIYg


mkdir -p /srv/xfs-jbods/vdb
chmod 700 /srv/xfs-jbods/vdb

# we don't bother with nixos config or fstab
# because we must run l1 provisioning post boot anyway
CURR_FS_TYPE=$( lsblk -n -o FSTYPE /dev/vdb )
if [ -z "$CURR_FS_TYPE" ]
then
    # if disk is empty provision xfs filesystem
    nix-shell -p xfsprogs --command 'mkfs.xfs /dev/vdb'
fi

if ! mount | grep '/dev/vdb'
then
    mount -o noatime /dev/vdb /srv/xfs-jbods/vdb
fi

CURR_FS_SIZE=$( df --output=size -B1 /dev/vdb | tail -n-1 )
CURR_DEVICE_SIZE=$( lsblk -n -o SIZE -b /dev/vdb )
AT_LEAST_SIZE=$(( CURR_FS_SIZE + 536870912 ))
if [ "$AT_LEAST_SIZE" -le "$CURR_DEVICE_SIZE" ]
then
    echo "Block device /dev/vdb expanded, expanding xfs filesystem..."
    nix-shell -p xfsprogs --command 'xfs_growfs /dev/vdb'
fi

mkdir -p /srv/xfs-jbods/vdb/minio-docker
chmod 777 /srv/xfs-jbods/vdb/minio-docker


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
        "epl-nix-cache:J1ra9/yUStIqnlqFf4FjS/pd9zwBW4bo+WP0FoBq72M="
      ];

    };

    networking.hostId = "a53dba54";


    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJfLsQZrFopCBQ0+md0XHgVAhz3i7tRJ+RL7ILH29IaB epl-root-ssh-key"

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
MIIB1DCCAXmgAwIBAgIUd7erzd27ISQoaK9W/+varzigcDQwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MDMwNjAzMTcwMFoXDTQxMDMwMjAzMTcw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABDxO9ESWhh9qsjidMhNU4+qj
r65WcqxPOrSgqFwp6bifUwYg7Ajt02TJFHg8iY8N680XQgx91y+Q6i11F9QH2eqj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSKXn0KCwYZVsMCXDOJP5FNci9c
uDAfBgNVHSMEGDAWgBTPuqyRyh1VGZvEnL8MGN62Pca2XjA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNJADBGAiEA+Ul7J6RcwFcbNJFIVibLPd5Xn+uwSH3CF++I
oBujjP4CIQCBwtYU0t72IMLGA85R8BfjEmqNBMKGKkGnQ79ONhIMtg==
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUGBv/2biRVzUT/D0zakHmje+rjMUwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwMzA2MDMxNzAwWhcNNDEwMzAyMDMxNzAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABFFC
caW7VHrx7X/bJFCw9TSnpNt5cbsQ9meJs0x6lVZseOx0JPZCRxdC7DLQ7Z/yMnYM
YfnPYfCgT59yltMyC4ujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQ5Lk4e3Ph+VQWTssa0zkKOQaPiWzAKBggqhkjOPQQDAgNI
ADBFAiAgHuf7jj7X70GK9uSeaNxGC10HHbBJsGWq9WiGMTpEvgIhAMrsKjt7XnZp
M6rwEAfjvpRXN4je7ZR3HFz7wfYoAvvp
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUc4FvA3RvBBPXWvSWIXczcEydUKswCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwMzA2MDMxNzAwWhcNNDEwMzAyMDMxNzAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABPPV
4+/0y5d27H63VDgtzIQsEmb+egE6eJeG2b/ZCMxmd/6P0MbBKyMpCAbCt53nx9tG
oUAa5TCX0L2lEGxzDvejQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSnSZoE5Lm8WxUSHdkXt2xVslRu9jAKBggqhkjOPQQDAgNI
ADBFAiA6pM4Uf2lWZtKEdC6m3k1NscZhqn43MWzKutZjG79nqQIhAIgVxMbGKeTR
tuGFfsrGBs00dlAuK3eOqUrTlp/nYMd0
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
        epl-vault-operator-init
        epl-vault-operator-unseal
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "10.17.0.1"; prefixLength = 32; via = "10.18.0.1"; }

    ];

    networking.interfaces."eth1".ipv4.routes = [

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
           iifname eth1 ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 77.77.77.13/32 } drop comment "Disallow traffic from internet to internal networks";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-internet-fw END

# NIX REGION firewall START

  networking.hostName = "server-h";
  networking.firewall.allowPing = true;
  networking.firewall.enable = true;
  networking.firewall.checkReversePath = false;
  networking.firewall.trustedInterfaces = [

    "eth0"

    "wg0"

  ];

  networking.firewall.interfaces."eth1".allowedTCPPorts = [ 22 80 443 53 ];

  networking.firewall.interfaces."eth1".allowedUDPPorts = [ 53 51820 ];

# NIX REGION firewall END

   programs.bash.promptInit = ''
     # Provide a nice prompt if the terminal supports it.
     if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
       PROMPT_COLOR="1;31m"
       ((UID)) && PROMPT_COLOR="1;32m"
       if [ -n "$INSIDE_EMACS" ]; then
         # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-h.dc2.us-east.multi-region:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-h.dc2.us-east.multi-region: \w\a\]\u@server-h.dc2.us-east.multi-region:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-h.dc2.us-east.multi-region:\u:\w\007\]$PS1"
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
         while ! ${pkgs.consul}/bin/consul kv get epl-l1-plans/server-h
         do
           sleep 7
         done

         ${pkgs.consul}/bin/consul watch \
           -type=key -key=epl-l1-plans/server-h \
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
    boot.extraModulePackages = [ ];

    boot.kernelParams = [ "console=ttyS0,115200n8" ];
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
  epl-infra.net. initial-key 257 3 15 "baEW4FrpIp5KsX6jmedjTmZ2eHhnLg9XVCInj8IeoQY=";
  us-west.epl-infra.net. initial-key 257 3 15 "ftP7Fcgh5mOf4RFi8EJxgeqabrMRh73p84Bje168Pcg=";
  us-east.epl-infra.net. initial-key 257 3 15 "VA1DbcY1t1ChE+XY4HEubIbftzk6aQuoK9CW0knRNQ8=";
  10.in-addr.arpa. initial-key 257 3 15 "xziBKJb8w4Iu+ITuIHJu72RUx4/BvW5vg6QmkQmyCoE=";
  17.10.in-addr.arpa. initial-key 257 3 15 "hnuAeTi9gNoGSSOXCsnZyTozZse/+h8Z7tUNHGUFNIg=";
  18.10.in-addr.arpa. initial-key 257 3 15 "v0qufAGopxffmXTs/kiXUfC64Q7DFrn23PbuzPDglnI=";
  in-addr.arpa. initial-key 257 3 15 "codMviPjtLXnRS0QPnXgYq2GWvONqkUev8H1B/PNh6A=";
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
  masters { 10.17.0.13; };
  transfer-source 10.18.0.13;
};
zone "10.in-addr.arpa." {
  type slave;
  file "/run/named/private-10.in-addr.arpa.zone.signed";
  masters { 10.17.0.13; };
  transfer-source 10.18.0.13;
};
zone "us-east.epl-infra.net." {
  type master;
  file "/run/named/private-us-east.epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.18.0.12;
  };
};
zone "18.10.in-addr.arpa." {
  type master;
  file "/run/named/private-18.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.18.0.12;
  };
};

};

view internet {
          match-clients { any; };
          recursion no;
zone "epl-infra.net." {
  type slave;
  file "/run/named/public-epl-infra.net.zone.signed";
  masters { 77.77.77.11; };

  allow-query { any; };
};
zone "in-addr.arpa." {
  type slave;
  file "/run/named/public-in-addr.arpa.zone.signed";
  masters { 77.77.77.11; };
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
          " --web.listen-address=10.18.0.13:9100" +
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
          " --listen_ip=10.18.0.13" +
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
wg set wg0 peer "z+zb364ARWHKLUsY23JaI+Tpqr20ML4XOAQ7ic71B1Y=" allowed-ips "172.21.7.254/32"

wg set wg0 peer "BmdFeQXEro8/tXGO2/jEYvhWyV6VBMPJNzj0KwU1Ph8=" allowed-ips "172.21.7.11/32,10.17.0.0/16" endpoint "77.77.77.11:51820"

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
               ip saddr 10.18.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
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
          ospf router-id 10.18.0.13
          redistribute bgp
          network 10.18.0.0/16 area 10.18.0.0
          area 10.18.0.0 range 10.18.0.0/16 advertise
          area 10.18.0.0 range 0.0.0.0/0 not-advertise
          area 10.18.0.0 authentication message-digest
          default-information originate always
          neighbor 10.18.0.12
        !
        interface eth0
          ip ospf cost 500
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 O8a4E38DgzyjFRaA
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
          peer 10.18.0.13
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
          bgp router-id 10.18.0.13
          address-family ipv4 unicast
            network 10.18.0.0/16
          exit-address-family
          neighbor 10.18.0.12 remote-as 64530
          neighbor 10.18.0.12 password RlCOZakQrxqa0GuVTs3kzFWzxYk5Pfpn8GwxaSbUTX
          neighbor 10.18.0.12 bfd
          neighbor 172.21.7.11 remote-as 64529
          neighbor 172.21.7.11 password A36W4XhjkRsz7CcjV7bRGhOj3SX9SxlSRv2tEO8YGT
          neighbor 172.21.7.11 bfd
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
          set src 10.18.0.13
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol ospf route-map LANRM
        !
        ip protocol bgp route-map LANRM
        !
        interface eth0
          ip address 10.18.0.13/24
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
        ip route 10.17.0.1/32 10.18.0.1

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
      { address = "10.18.0.13"; prefixLength = 24; }

    ];

    networking.interfaces.eth1.ipv4.addresses = [
      { address = "77.77.77.13"; prefixLength = 24; }

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
  # bind journalfile will clash with zone file, we have the source
  # so journalfile is irrelevant for us
  if [ -f "$TARGET_FILE.jnl" ]
  then
    if [ "$TARGET_FILE" -nt "$TARGET_FILE.jnl" ]
    then
      echo "Deleting older journalfile $TARGET_FILE.jnl"
      rm -f $TARGET_FILE.jnl
    fi
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
# us-east.epl-infra.net.	IN	SOA	ns1.us-east.epl-infra.net. us-east.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# us-east.epl-infra.net.	IN	NS	ns1.us-east.epl-infra.net.
# us-east.epl-infra.net.	IN	NS	ns2.us-east.epl-infra.net.
# ns1.us-east.epl-infra.net.	IN	A	10.18.0.13
# ns2.us-east.epl-infra.net.	IN	A	10.18.0.12
# server-a.us-east.epl-infra.net.	IN	A	10.17.0.10
# server-b.us-east.epl-infra.net.	IN	A	10.17.0.11
# server-c.us-east.epl-infra.net.	IN	A	10.17.0.12
# server-d.us-east.epl-infra.net.	IN	A	10.17.0.13
# server-e.us-east.epl-infra.net.	IN	A	10.18.0.10
# server-f.us-east.epl-infra.net.	IN	A	10.18.0.11
# server-g.us-east.epl-infra.net.	IN	A	10.18.0.12
# server-h.us-east.epl-infra.net.	IN	A	10.18.0.13
maybe_update_dns_file JFRUTCAzNjAwCnVzLWVhc3QuZXBsLWluZnJhLm5ldC4JSU4JU09BCW5zMS51cy1lYXN0LmVwbC1pbmZyYS5uZXQuIHVzLWVhc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKdXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglOUwluczEudXMtZWFzdC5lcGwtaW5mcmEubmV0Lgp1cy1lYXN0LmVwbC1pbmZyYS5uZXQuCUlOCU5TCW5zMi51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCm5zMS51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCUlOCUEJMTAuMTguMC4xMwpuczIudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTIKc2VydmVyLWEudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTAKc2VydmVyLWIudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTEKc2VydmVyLWMudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTIKc2VydmVyLWQudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE3LjAuMTMKc2VydmVyLWUudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTAKc2VydmVyLWYudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTEKc2VydmVyLWcudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTIKc2VydmVyLWgudXMtZWFzdC5lcGwtaW5mcmEubmV0LglJTglBCTEwLjE4LjAuMTMK /run/named/private-us-east.epl-infra.net.zone
# $TTL 3600
# 18.10.in-addr.arpa.	IN	SOA	ns1.us-east.epl-infra.net. us-east.epl-infra.net. (
#  SERIAL_TO_REPLACE ; Serial
#  3600 ; Refresh
#  1800 ; Retry
#  604800 ; Expire
#  3600 ; Minimum TTL
# )
# 18.10.in-addr.arpa.	IN	NS	ns1.us-east.epl-infra.net.
# 18.10.in-addr.arpa.	IN	NS	ns2.us-east.epl-infra.net.
#
# 10.0.18.10.in-addr.arpa.	IN	PTR	server-e.us-east.epl-infra.net.
# 11.0.18.10.in-addr.arpa.	IN	PTR	server-f.us-east.epl-infra.net.
# 12.0.18.10.in-addr.arpa.	IN	PTR	ns2.us-east.epl-infra.net.
# 13.0.18.10.in-addr.arpa.	IN	PTR	ns1.us-east.epl-infra.net.
maybe_update_dns_file JFRUTCAzNjAwCjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JU09BCW5zMS51cy1lYXN0LmVwbC1pbmZyYS5uZXQuIHVzLWVhc3QuZXBsLWluZnJhLm5ldC4gKAogU0VSSUFMX1RPX1JFUExBQ0UgOyBTZXJpYWwKIDM2MDAgOyBSZWZyZXNoCiAxODAwIDsgUmV0cnkKIDYwNDgwMCA7IEV4cGlyZQogMzYwMCA7IE1pbmltdW0gVFRMCikKMTguMTAuaW4tYWRkci5hcnBhLglJTglOUwluczEudXMtZWFzdC5lcGwtaW5mcmEubmV0LgoxOC4xMC5pbi1hZGRyLmFycGEuCUlOCU5TCW5zMi51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCgoxMC4wLjE4LjEwLmluLWFkZHIuYXJwYS4JSU4JUFRSCXNlcnZlci1lLnVzLWVhc3QuZXBsLWluZnJhLm5ldC4KMTEuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUglzZXJ2ZXItZi51cy1lYXN0LmVwbC1pbmZyYS5uZXQuCjEyLjAuMTguMTAuaW4tYWRkci5hcnBhLglJTglQVFIJbnMyLnVzLWVhc3QuZXBsLWluZnJhLm5ldC4KMTMuMC4xOC4xMC5pbi1hZGRyLmFycGEuCUlOCVBUUgluczEudXMtZWFzdC5lcGwtaW5mcmEubmV0Lgo= /run/named/private-18.10.in-addr.arpa.zone



# we could implement some complex mechanism
# to detect if zone files changed later
/run/current-system/sw/bin/systemctl reload bind.service || true


cp -pu /run/keys/K10-in-addr-arpa--015-20386-private /run/dnsseckeys/K10.in-addr.arpa.+015+20386.private
cp -pu /run/keys/K10-in-addr-arpa--015-01203-private /run/dnsseckeys/K10.in-addr.arpa.+015+01203.private
cp -pu /run/keys/K10-in-addr-arpa--015-20386-key /run/dnsseckeys/K10.in-addr.arpa.+015+20386.key
cp -pu /run/keys/K10-in-addr-arpa--015-01203-key /run/dnsseckeys/K10.in-addr.arpa.+015+01203.key
cp -pu /run/keys/K18-10-in-addr-arpa--015-63576-private /run/dnsseckeys/K18.10.in-addr.arpa.+015+63576.private
cp -pu /run/keys/K18-10-in-addr-arpa--015-42662-private /run/dnsseckeys/K18.10.in-addr.arpa.+015+42662.private
cp -pu /run/keys/K18-10-in-addr-arpa--015-63576-key /run/dnsseckeys/K18.10.in-addr.arpa.+015+63576.key
cp -pu /run/keys/K18-10-in-addr-arpa--015-42662-key /run/dnsseckeys/K18.10.in-addr.arpa.+015+42662.key
cp -pu /run/keys/Kepl-infra-net--015-56589-private /run/dnsseckeys/Kepl-infra.net.+015+56589.private
cp -pu /run/keys/Kepl-infra-net--015-00239-private /run/dnsseckeys/Kepl-infra.net.+015+00239.private
cp -pu /run/keys/Kepl-infra-net--015-56589-key /run/dnsseckeys/Kepl-infra.net.+015+56589.key
cp -pu /run/keys/Kepl-infra-net--015-00239-key /run/dnsseckeys/Kepl-infra.net.+015+00239.key
cp -pu /run/keys/Kus-east-epl-infra-net--015-47408-private /run/dnsseckeys/Kus-east.epl-infra.net.+015+47408.private
cp -pu /run/keys/Kus-east-epl-infra-net--015-29833-private /run/dnsseckeys/Kus-east.epl-infra.net.+015+29833.private
cp -pu /run/keys/Kus-east-epl-infra-net--015-47408-key /run/dnsseckeys/Kus-east.epl-infra.net.+015+47408.key
cp -pu /run/keys/Kus-east-epl-infra-net--015-29833-key /run/dnsseckeys/Kus-east.epl-infra.net.+015+29833.key

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
epl_l1_provisioning_last_hash{hash=\"a2c8cc8ae0eafc4c8327f6b691fdd30570e767093ed2f8e299ce89281ac9d71e\",hostname=\"server-h\"} $BOOT_TIME
" > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE

fi
