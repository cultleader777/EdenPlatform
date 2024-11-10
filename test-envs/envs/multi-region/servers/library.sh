
function start_server() {
  THE_HOST=$1
  MEMORY_MB=$2
  SERVER_CORES=$3
  SERVER_ARCH=$4
  VIRT_TYPE="--virt-type qemu"
  if [ "$(uname -m)" == "$SERVER_ARCH" ];
  then
    VIRT_TYPE="--virt-type kvm"
  fi
  FEATURES=""
  if [ "$SERVER_ARCH" == "aarch64" ];
  then
    FEATURES="--features acpi=off"
  fi
  shift
  shift
  shift
  shift
  virsh list --name | grep ${THE_HOST} && return
  echo Starting server ${THE_HOST}
  virt-install --name ${THE_HOST} \
    --import \
    ${VIRT_TYPE} ${FEATURES} --memory ${MEMORY_MB} \
    --arch ${SERVER_ARCH} \
    --vcpus ${SERVER_CORES} \
    --boot hd,menu=on \
    --graphics none \
    --os-variant nixos-unstable \
    --console pty,target_type=serial \
    --check path_in_use=off \
    --check mac_in_use=off \
    "$@" \
    --noautoconsole
}

function maybe_bootstrap_nomad() {
  SERVER=$1 # example: 10.17.0.11
  REGION=$2 # example: us-west
  EPL_EXECUTABLE=$3
  # make sure we don't lose initial token with tmux remote session
  ssh admin@$SERVER -i aux/root_ssh_key \
    -o UserKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=false \
    -o ConnectTimeout=2 "tmux new-session -d 'epl-nomad-acl-bootstrap'"

  # wait until bootstrap script finishes and cat output
  NOMAD_ACL_BOOTSTRAP_OUTPUT=$( ssh admin@$SERVER -i aux/root_ssh_key \
    -o UserKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=false \
    -o ConnectTimeout=2 " \
      ps aux | grep epl-nomad-acl-bootstrap | grep -v grep | awk '{print \$2}' | xargs -I{} tail --pid={} -f /dev/null &> /dev/null; \
      cat /run/secdir/nomad-bootstrap-output.txt
    " )
  SECRET_TOKEN=$( echo "$NOMAD_ACL_BOOTSTRAP_OUTPUT" | grep 'Secret ID' | sed -E 's/^.*= //g' )

  if [ -n "$SECRET_TOKEN" ]
  then
    # guard secrets.yml with mutex
    lockfile secrets.yml.lock
    $EPL_EXECUTABLE override-secret --output-directory .. --key nomad_region_${REGION}_bootstrap_acl_token --value $SECRET_TOKEN --kind Guid
    $EPL_EXECUTABLE override-secret --output-directory .. --key nomad_region_${REGION}_bootstrap_acl_output --value "$NOMAD_ACL_BOOTSTRAP_OUTPUT" --kind Misc
    rm -f secrets.yml.lock

    # delete the token remotely
    ssh admin@$SERVER -i aux/root_ssh_key \
      -o UserKnownHostsFile=/dev/null \
      -o StrictHostKeyChecking=false \
      -o ConnectTimeout=2 'rm -f /run/secdir/nomad-bootstrap-output.txt'
  fi
}

function maybe_init_vault() {
  SERVER=$1    # example: 10.17.0.11
  FQDN=$2      # example: https://server-b.us-west.epl-infra.net:8200
  REGION=$3    # example: us-west
  EPL_EXECUTABLE=$4

  # make sure we don't lose initial tokens with tmux remote session
  ssh admin@$SERVER -i aux/root_ssh_key \
      -o UserKnownHostsFile=/dev/null \
      -o StrictHostKeyChecking=false \
      -o ConnectTimeout=2 " epl-vault-operator-init $FQDN"

  # wait until bootstrap script finishes and cat output
  VAULT_OPERATOR_INIT_OUTPUT=$( ssh admin@$SERVER -i aux/root_ssh_key \
    -o UserKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=false \
    -o ConnectTimeout=2 " \
      ps aux | grep epl-vault-operator-init | grep -v grep | awk '{print \$2}' | xargs -I{} tail --pid={} -f /dev/null &> /dev/null; \
      cat /run/secdir/vault-init-output.txt
    " )
  if echo "$VAULT_OPERATOR_INIT_OUTPUT" | grep 'Initial Root Token:' &> /dev/null
  then
      # guard secrets.yml with mutex
      lockfile secrets.yml.lock
      $EPL_EXECUTABLE override-secret --output-directory .. --key vault_region_${REGION}_initial_keys --value "$VAULT_OPERATOR_INIT_OUTPUT" --kind Misc
      rm -f secrets.yml.lock

      # delete the secret remotely
      ssh admin@$SERVER -i aux/root_ssh_key \
        -o UserKnownHostsFile=/dev/null \
        -o StrictHostKeyChecking=false \
        -o ConnectTimeout=2 'rm -f /run/secdir/vault-init-output.txt'

      echo Waiting 10 seconds after vault init...
      sleep 10
      echo Done
  fi
}

function maybe_unseal_vault() {
  SERVER=$1    # example: 10.17.0.11
  FQDN=$2      # example: https://server-b.us-west.epl-infra.net:8200
  REGION=$3    # example: us-west
  EPL_EXECUTABLE=$4
  METRICS_CLUSTER=$5 # skip unseal if metrics db tells us vault is now unsealed

  if [ -n "$METRICS_CLUSTER" ]
  then
    METRICS_RESULTS=$( echo "
      SELECT value
      FROM metrics
      WHERE metric_name = 'vault_sealed_clusters'
      AND cluster = '$METRICS_CLUSTER'
    " | sqlite3 ../prometheus/scraped_metrics.sqlite )

    if [ -n "$METRICS_RESULTS" ]
    then
      IS_SEALED=$( echo "$METRICS_RESULTS" | jq '.data.result | length' )
      if [ "$IS_SEALED" == "0" ]
      then
        # vault isn't sealed, return
        return 0
      else
        echo "Some vault instances in $REGION are sealed, unsealing"
      fi
    else
      echo "------------------------------------ WARNING ------------------------------------"
      echo "vault_sealed_clusters metric is undefined in monitoring_cluster_scraped_metric for cluster $METRICS_CLUSTER"
      echo "!!!!!!! AUTOMATIC CI VAULT UNSEALING WILL NOT WORK FOR REGION $REGION !!!!!!!"
    fi
  fi

  VAULT_BOOTSTRAP_KEYS=$( $EPL_EXECUTABLE get-secret --output-directory .. --key vault_region_${REGION}_initial_keys | base64 -w 0 )
  ssh admin@$SERVER -i aux/root_ssh_key \
      -o UserKnownHostsFile=/dev/null \
      -o StrictHostKeyChecking=false \
      -o ConnectTimeout=2 \
      " consul members && epl-vault-operator-unseal $FQDN $VAULT_BOOTSTRAP_KEYS || echo 'Before unsealing vault perform L1 provisioning first on node $FQDN'"
}

function get_vault_root_token() {
  REGION=$1
  EPL_EXECUTABLE=$2
  $EPL_EXECUTABLE get-secret --output-directory .. \
    --key vault_region_${REGION}_initial_keys | \
    grep "Initial Root Token:" | sed -E 's/^.*: //'
}

function extract_grafana_admin_keys_from_vault() {
  REGION=$1
  shift
  EPL_EXECUTABLE=$1
  shift
  SERVER=$1
  shift
  VAULT_ROOT_TOKEN=$( get_vault_root_token $REGION $EPL_EXECUTABLE )
  QUERY_FOR_SERVERS=""
  for CLUSTER in $@;
  do
    QUERY_FOR_SERVERS="$QUERY_FOR_SERVERS echo $CLUSTER \$( vault kv get -field=admin_password epl/grafana/$CLUSTER );"
  done

  timeout 10s ssh admin@$SERVER -i aux/root_ssh_key \
    -o UserKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=false \
    -o ServerAliveCountMax=2 \
    -o ServerAliveInterval=5 \
    -o ConnectTimeout=2 " export VAULT_TOKEN=$VAULT_ROOT_TOKEN; $QUERY_FOR_SERVERS"
}

function nomad_policies_provision() {
  SERVER=$1 # example: 10.17.0.11
  REGION=$2 # example: us-west
  EPL_EXECUTABLE=$3
  NOMAD_BOOTSTRAP_ACL_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key nomad_region_${REGION}_bootstrap_acl_token )

  ssh admin@$SERVER -i aux/root_ssh_key \
	  -o UserKnownHostsFile=/dev/null \
	  -o StrictHostKeyChecking=false \
	  -o ConnectTimeout=2 " export NOMAD_TOKEN=$NOMAD_BOOTSTRAP_ACL_TOKEN; epl-nomad-acl-policies"
}

function vault_nomad_policies_provision() {
  SERVER=$1 # example: 10.17.0.11
  REGION=$2 # example: us-west
  EPL_EXECUTABLE=$3

  INFRA_ROOT_VAULT_TOKEN=$( $EPL_EXECUTABLE get-secret \
    --output-directory .. \
    --key vault_region_${REGION}_initial_keys \
    | grep 'Initial Root Token:' | sed -E 's/.*: hvs./hvs./' )
  NOMAD_VAULT_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key nomad_region_${REGION}_vault_token )
  NOMAD_VAULT_TOKEN=$( ssh admin@$SERVER -i aux/root_ssh_key \
      -o UserKnownHostsFile=/dev/null \
      -o StrictHostKeyChecking=false \
      -o ConnectTimeout=2 " export VAULT_TOKEN=$INFRA_ROOT_VAULT_TOKEN; epl-nomad-vault-policies $NOMAD_VAULT_TOKEN" | grep NOMAD_VAULT_TOKEN | sed -E 's/^NOMAD_VAULT_TOKEN //' )
  if [ -n "$NOMAD_VAULT_TOKEN" ];
  then
      $EPL_EXECUTABLE override-secret --output-directory .. --key nomad_region_${REGION}_vault_token --value $NOMAD_VAULT_TOKEN --kind VaultToken
  fi
}

function vault_acme_policies_provision() {
  SERVER=$1 # example: 10.17.0.11
  REGION=$2 # example: us-west
  EPL_EXECUTABLE=$3

  INFRA_ROOT_VAULT_TOKEN=$( $EPL_EXECUTABLE get-secret \
    --output-directory .. \
    --key vault_region_${REGION}_initial_keys \
    | grep 'Initial Root Token:' | sed -E 's/.*: hvs./hvs./' )
  ACME_VAULT_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key acme_region_${REGION}_vault_token )
  ACME_VAULT_TOKEN=$( ssh admin@$SERVER -i aux/root_ssh_key \
      -o UserKnownHostsFile=/dev/null \
      -o StrictHostKeyChecking=false \
      -o ConnectTimeout=2 " export VAULT_TOKEN=$INFRA_ROOT_VAULT_TOKEN; epl-acme-vault-policies $ACME_VAULT_TOKEN" | grep ACME_CERTS_VAULT_TOKEN | sed -E 's/^ACME_CERTS_VAULT_TOKEN //' )
  if [ -n "$ACME_VAULT_TOKEN" ];
  then
      $EPL_EXECUTABLE override-secret --output-directory .. --key acme_region_${REGION}_vault_token --value $ACME_VAULT_TOKEN --kind VaultToken
  fi
}

function stop_server() {
  SERVER=$1
  virsh list --name | grep $SERVER && \
    virsh destroy $SERVER || true; \
  virsh list --all --name | grep $SERVER && \
    virsh undefine --managed-save --snapshots-metadata --checkpoints-metadata $SERVER || true
}

function prepare_disk_img() {
  OUT_PATH=$1
  DISK_CAPACITY_BYTES=$2
  VM_TEMPLATE=$3
  EXTRA_PREP_DISK_ARGS=''
  if [ -n "$VM_TEMPLATE" ]
  then
    EXTRA_PREP_DISK_ARGS="-b $(cat $VM_TEMPLATE) -F qcow2"
  fi
  mkdir -p disks
  echo qemu-img create -f qcow2 $EXTRA_PREP_DISK_ARGS $OUT_PATH $DISK_CAPACITY_BYTES
  qemu-img create -f qcow2 $EXTRA_PREP_DISK_ARGS $OUT_PATH $DISK_CAPACITY_BYTES
}

function maybe_run_nix_serve() {
  NIX_SERVE_FOUND=$( netstat -tulpn | grep 0.0.0.0:12777 )

  if [ -z "$NIX_SERVE_FOUND" ];
  then
    echo nix-serve not running, starting...

    # serve for every gateway ip
    tmux new-session -d 'export NIX_SECRET_KEY_FILE=aux/cache-priv-key.pem; nix-serve --host 0.0.0.0 --port 12777'
  fi
}

function ensure_server_ready() {
  THE_IP=$1

  while ! timeout 10s ssh admin@$THE_IP -i aux/root_ssh_key \
            -o UserKnownHostsFile=/dev/null \
            -o StrictHostKeyChecking=false \
            -o ConnectTimeout=2 'ls' &>/dev/null
  do
    sleep 1
  done

  echo $THE_IP server is ready
}

function aws_bootstrap_private_node_internet() {
  TARGET_HOSTNAME=$1
  PRIVATE_IP=$2
  NEW_GW_IP=$3
  PUBLIC_IP_TO_PING=$4
  DC_NETWORK=$5
  DC_GW=$6
  ENABLE_IP_FORWARD=$7

  if [ -f ../markers/l1-bootstrapped/$TARGET_HOSTNAME ];
  then
    # node already bootstrapped
    return 0
  fi

  FWD_CMD=""
  if [ -n "$ENABLE_IP_FORWARD" ];
  then
    FWD_CMD="sudo sysctl -w net.ipv4.ip_forward=1;"
  fi

  if ! ssh admin@$PRIVATE_IP -i aux/root_ssh_key \
            -o UserKnownHostsFile=/dev/null \
            -o StrictHostKeyChecking=false \
            -o ConnectTimeout=2 "ping -W 3 -c 1 $PUBLIC_IP_TO_PING" &>/dev/null
  then
    echo "Can't ping public ip addrsss $PUBLIC_IP_TO_PING, manually adjusting routes for an hour"
    ssh admin@$PRIVATE_IP -n -f -i aux/root_ssh_key \
            -o UserKnownHostsFile=/dev/null \
            -o StrictHostKeyChecking=false \
            -o ConnectTimeout=2 "{ $FWD_CMD sudo ip route add $DC_NETWORK via $DC_GW; for I in \$(seq 1 3600) ; do sudo ip route del default; sudo ip route add default via $NEW_GW_IP; sleep 1; done } > /dev/null 2>&1  &"
  fi
}

function nomad_server_join() {
  TO_IP=$1
  FROM_IP=$2
  FROM_IP_REGION=$3
  NOMAD_BOOTSTRAP_ACL_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key nomad_region_${FROM_IP_REGION}_bootstrap_acl_token )
  ssh admin@$FROM_IP -i aux/root_ssh_key \
    -o UserKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=false \
    -o ConnectTimeout=2 " NOMAD_TOKEN=$NOMAD_BOOTSTRAP_ACL_TOKEN nomad server join $TO_IP:4648"
}

function wait_until_ping_succeeds() {
  SSH_IP=$1
  PINGER_IP=$2
  PINGEE_IP=$3
  while ! ssh admin@$SSH_IP -i aux/root_ssh_key \
            -o UserKnownHostsFile=/dev/null \
            -o StrictHostKeyChecking=false \
            -o ConnectTimeout=2 "ping -W 3 -c 1 $PINGEE_IP" &>/dev/null
  do
    sleep 1
  done

  echo ping from $PINGER_IP to $PINGEE_IP succeeded
}

function wait_l1_provisioning_finished() {
  PROVISIONING_ID=$1
  SERVER_IP=$2
  SERVER_HOST=$3
  SERVER_REGION=$4

  SQL_QUERY="select exit_code, is_finished from l1_provisionings where provisioning_id=$PROVISIONING_ID"

  for I in $(seq 1 77);
  do
    SQL_OUTPUT=$( echo "$SQL_QUERY" | \
          timeout 10s ssh admin@$SERVER_IP -i aux/root_ssh_key \
            -o UserKnownHostsFile=/dev/null \
            -o StrictHostKeyChecking=false \
            -o ConnectTimeout=5 "sudo sqlite3 -csv /var/lib/epl-l1-prov/provisionings.sqlite" 2>&1 | \
          grep -v 'Warning:' )

    if echo "$SQL_OUTPUT" | grep -i -e 'Connection timed out' -e 'No route to host'
    then
      echo Timed out trying to reach server $SERVER_HOST, trying again after few seconds
      sleep 17
      continue
    fi

    if echo "$SQL_OUTPUT" | grep -i 'ssh: connect to host'
    then
      echo Unknown ssh error for server $SERVER_HOST, exiting
      exit 7
    fi

    if [ -z "$SQL_OUTPUT" ];
    then
      echo Provisioning id $PROVISIONING_ID not found in l1 provisioning database
      exit 7
    fi
    EXIT_CODE=$( echo "$SQL_OUTPUT" | cut -d, -f1 )
    IS_FINISHED=$( echo "$SQL_OUTPUT" | cut -d, -f2 )
    if [ "1" == "$IS_FINISHED" ];
    then
      if [ "0" != "$EXIT_CODE" ];
      then
        tail -n 20 /var/log/epl-l1-prov/$PROVISIONING_ID.log
        echo L1 provisioning $PROVISIONING_ID failed for $SERVER_HOST $SERVER_IP, logs above
      fi
      echo L1 provisioning $PROVISIONING_ID successful for $SERVER_HOST $SERVER_IP
      echo "
        INSERT OR IGNORE INTO bootstrapped_servers(hostname, region)
        VALUES('$SERVER_HOST', '$SERVER_REGION')
      " | sqlite3 ../infra-state.sqlite || true
      exit $EXIT_CODE
    fi
    sleep 17
  done

  echo L1 provisioning $PROVISIONING_ID timed out for $SERVER_IP
  exit 17
}

function ensure_script_running_from_its_directory() {
  SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
  if [ "$PWD" != "$SCRIPT_DIR" ]
  then
    echo 'Script can only be run in its own directory'
    exit
  fi
}

function fast_l1_provisioning_for_region() {
  SERVER=$1 # example: 10.17.0.11
  REGION=$2 # example: us-west
  PAYLOAD_FILE=$3 # example: us-west.tar.gz
  L1_PROV_ID=$4
  EPL_EXECUTABLE=$5
  CONSUL_ACL_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key consul_region_${REGION}_acl_fast_l1_token )
  # make sure we don't lose initial token with tmux remote session
  rsync -av \
    -e 'ssh -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectTimeout=2' \
    $PAYLOAD_FILE admin@$SERVER:/tmp/

  ssh admin@$SERVER -i aux/root_ssh_key \
    -o UserKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=false \
    -o ConnectTimeout=2 \
    "rm -rf /tmp/$REGION; cd /tmp/; tar xvf $REGION.tar.gz; tmux new-session -d ' cd /tmp/$REGION; export CONSUL_HTTP_TOKEN=${CONSUL_ACL_TOKEN}; ./consul-push.sh &> /var/log/epl-l1-upload/${L1_PROV_ID}.log'"
}

function init_infra_state_db() {
    echo '
        CREATE TABLE IF NOT EXISTS servers(
          hostname TEXT PRIMARY KEY,
          region TEXT,
          expected_l1_hash TEXT
        );

        CREATE TABLE IF NOT EXISTS current_server_state(
          hostname TEXT PRIMARY KEY,
          hash TEXT,
          last_boot_time INT
        );

        -- if hashes dont match and server is ready it goes here
        CREATE TABLE IF NOT EXISTS servers_for_fast_l1_provision(
          hostname TEXT PRIMARY KEY,
          region TEXT
        );

        -- if server is not inside fast_l1 and hash mismatches or server state unknown it goes here
        CREATE TABLE IF NOT EXISTS servers_for_slow_l1_provision(
          hostname TEXT PRIMARY KEY
        );

        CREATE TABLE IF NOT EXISTS bootstrapped_servers(
          hostname TEXT PRIMARY KEY,
          region TEXT
        );

        CREATE INDEX IF NOT EXISTS bootstrapped_by_region ON bootstrapped_servers(region);
    ' | sqlite3 ../infra-state.sqlite
}

function refresh_l1_provisioning_state() {
    # servers from metrics db that have at least 30 minute timestamp
    # matching boot time with matching last hash with the expected hash
    echo "
        ATTACH DATABASE '../prometheus/scraped_metrics.sqlite' AS scraped_metrics;
        DELETE FROM main.current_server_state;
        DELETE FROM main.servers_for_fast_l1_provision;
        DELETE FROM main.servers_for_slow_l1_provision;
        WITH last_hash_metric AS (
            SELECT json_extract(value,'$.data.result') AS extracted
            FROM scraped_metrics.metrics
            WHERE metric_name = 'epl_l1_provisioning_last_hash'
            AND timestamp > DATETIME('now', '-30 minute')
        ), extracted_lh AS (
            SELECT
            json_extract(json_each.value, '$.metric.hash') AS hash,
            json_extract(json_each.value, '$.metric.hostname') AS hostname,
            json_extract(json_each.value, '$.value[1]') AS last_boot_time
            FROM last_hash_metric, json_each(last_hash_metric.extracted)
        ), node_boot_time_metric AS (
            SELECT json_extract(value,'$.data.result') AS extracted
            FROM scraped_metrics.metrics
            WHERE metric_name = 'node_boot_time_seconds'
            AND timestamp > DATETIME('now', '-30 minute')
        ), extracted_boot_time AS (
            SELECT
            REPLACE(json_extract(json_each.value, '$.metric.instance'), ':9100', '') AS hostname,
            json_extract(json_each.value, '$.value[1]') AS last_boot_time
            FROM node_boot_time_metric, json_each(node_boot_time_metric.extracted)
        )
        INSERT INTO current_server_state
        SELECT hostname, hash, last_boot_time
        FROM extracted_lh
        NATURAL JOIN extracted_boot_time;

        -- all servers whose boot id matches
        -- with fast l1 boot id and whose metrics
        -- we know from last 30 minutes
        WITH dataset AS (
            SELECT hostname, region
            FROM main.current_server_state
            NATURAL JOIN main.servers
            WHERE NOT hash = expected_l1_hash
        )
        INSERT INTO main.servers_for_fast_l1_provision
        SELECT hostname, region FROM dataset;

        -- all servers in expected, except fast l1 servers
        -- and where hash mismatches, meaning it is null or value mismatch
        WITH dataset AS (
          SELECT s.hostname
          FROM servers s
          LEFT JOIN current_server_state css
          ON css.hostname = s.hostname
          WHERE (css.hash IS NULL OR NOT s.expected_l1_hash = css.hash)
          AND css.hostname NOT IN (
            SELECT hostname FROM main.servers_for_fast_l1_provision
          )
        )
        INSERT INTO main.servers_for_slow_l1_provision
        SELECT hostname FROM dataset;
    " | sqlite3 ../infra-state.sqlite

    # delete markers for all regions
    rm -rf ../markers/fast-l1-needed

    # create markers for all regions
    mkdir -p ../markers/fast-l1-needed
    for REGION in $( echo "SELECT DISTINCT region FROM servers_for_fast_l1_provision" | sqlite3 ../infra-state.sqlite )
    do
       touch ../markers/fast-l1-needed/$REGION
    done
}

