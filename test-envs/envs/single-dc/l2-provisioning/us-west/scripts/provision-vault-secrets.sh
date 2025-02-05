#!/bin/sh

function aws_secret_key() {
  < /dev/urandom tr -dc A-Za-z0-9 | head -c${1:-40}
}

function random_password_42() {
  < /dev/urandom tr -dc A-Za-z0-9 | head -c${1:-42}
}

function generate_consul_acl_token() {
  uuidgen -r
}

function ensure_consul_token_with_policy_exists() {
  THE_TOKEN=$1
  POLICY_NAME=$2
  DESIRED_POLICY=$3
  POLICY_PATH=/tmp/consul-policy-to-provision.hcl

  CURRENT_POLICY=$( consul acl policy read -name "$POLICY_NAME" | grep -A 9999999 'Rules:' | tail -n +2 )
  if [ "$CURRENT_POLICY" != "$DESIRED_POLICY" ];
  then
    echo "Creating/updating policy $POLICY_NAME"
    echo -n "$DESIRED_POLICY" | consul acl policy create -name "$POLICY_NAME" -rules=-
    echo -n "$DESIRED_POLICY" | consul acl policy update -no-merge -name "$POLICY_NAME" -rules=-
  fi

  consul acl token list | grep 'Policies:' --no-group-separator -A 1 | grep -vE -e '^Policies:' | awk '{print $3}' | grep -E "^$POLICY_NAME\$" > /dev/null
  TOKEN_EXISTS=$?
  if [ "$TOKEN_EXISTS" -ne 0 ];
  then
    echo "Creating acl token with policy $POLICY_NAME"
    # leaks token to stdout if not redirected to /dev/null
    consul acl token create -policy-name=$POLICY_NAME -secret=$THE_TOKEN &> /dev/null
  fi
}

function ensure_vault_policy_for_kv_exists() {
  POLICY_NAME=$1
  DESIRED_POLICY=$2

  CURRENT_POLICY=$( vault policy read "$POLICY_NAME" )
  if [ "$CURRENT_POLICY" != "$DESIRED_POLICY" ];
  then
    echo "Updating policy $POLICY_NAME"
    echo -n "$DESIRED_POLICY" | vault policy write "$POLICY_NAME" -
  fi

  TARGET_TOKEN_DIR="/run/secdir/epl-job-tokens"
  TARGET_TOKEN_FILE="$TARGET_TOKEN_DIR/$POLICY_NAME"
  if [ ! -f "$TARGET_TOKEN_FILE" ];
  then
    mkdir -m 0700 -p $TARGET_TOKEN_DIR
    # token doesn't exist, create
    vault token create -policy="$POLICY_NAME" -format=json | jq -r '.auth.client_token' > $TARGET_TOKEN_FILE
  else
    CURRENT_TOKEN=$( cat $TARGET_TOKEN_FILE )
    vault token renew $CURRENT_TOKEN &> /dev/null || vault token create -policy="$POLICY_NAME" -format=json | jq -r '.auth.client_token' > $TARGET_TOKEN_FILE
  fi
}

# clickhouse/testch
SEC_KEY=epl/clickhouse/testch
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(random_password_42)
SEC_2=$(random_password_42)
SEC_3=$(random_password_42)
SEC_4=$(random_password_42)
SEC_5=$(random_password_42)
MERGED_SECRET=$( echo "{\"admin_password\":\"$SEC_1\",\"db_chdb_a_admin\":\"$SEC_2\",\"db_chdb_a_ro\":\"$SEC_3\",\"db_chdb_a_rw\":\"$SEC_4\",\"interserver_password\":\"$SEC_5\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/clickhouse/testch" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-clickhouse-testch" "$VAULT_POLICY"
# docker-registry
SEC_KEY=epl/docker-registry
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(aws_secret_key)
SEC_2=$(random_password_42)
MERGED_SECRET=$( echo "{\"minio_bucket_password\":\"$SEC_1\",\"registry_http_secret\":\"$SEC_2\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/docker-registry" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-docker-registry" "$VAULT_POLICY"
# loki/main
SEC_KEY=epl/loki/main
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(generate_consul_acl_token)
SEC_2=$(aws_secret_key)
MERGED_SECRET=$( echo "{\"consul_token\":\"$SEC_1\",\"minio_bucket_password\":\"$SEC_2\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/loki/main" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-loki-main" "$VAULT_POLICY"
# create consul kv policy epl-loki-main-loki
CONSUL_POLICY=$( cat <<EOF
key_prefix "nomad-loki/epl-loki-main-loki" {
    policy = "write"
}
EOF
)
ensure_consul_token_with_policy_exists $( echo "$MERGED_SECRET" | jq -r '.consul_token' ) 'epl-loki-main-loki' "$CONSUL_POLICY"
# pg/testdb
SEC_KEY=epl/pg/testdb
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(generate_consul_acl_token)
SEC_2=$(random_password_42)
SEC_3=$(random_password_42)
SEC_4=$(random_password_42)
SEC_5=$(random_password_42)
SEC_6=$(random_password_42)
SEC_7=$(random_password_42)
SEC_8=$(random_password_42)
SEC_9=$(random_password_42)
MERGED_SECRET=$( echo "{\"consul_token\":\"$SEC_1\",\"pg_admin_password\":\"$SEC_2\",\"pg_db_bbtest_password\":\"$SEC_3\",\"pg_db_grafana_password\":\"$SEC_4\",\"pg_db_testdb_a_password\":\"$SEC_5\",\"pg_exporter_password\":\"$SEC_6\",\"pg_replicator_password\":\"$SEC_7\",\"pg_rewind_password\":\"$SEC_8\",\"pg_superuser_password\":\"$SEC_9\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/pg/testdb" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-pg-testdb" "$VAULT_POLICY"
# create consul kv policy epl-pg-testdb
CONSUL_POLICY=$( cat <<EOF
key_prefix "epl-patroni/epl-pg-testdb" {
    policy = "write"
}

service_prefix "epl-pg-testdb" {
    policy = "write"
}

session_prefix "" {
    policy = "write"
}
EOF
)
ensure_consul_token_with_policy_exists $( echo "$MERGED_SECRET" | jq -r '.consul_token' ) 'epl-pg-testdb' "$CONSUL_POLICY"
# tempo/us-west
SEC_KEY=epl/tempo/us-west
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(aws_secret_key)
MERGED_SECRET=$( echo "{\"minio_bucket_password\":\"$SEC_1\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/tempo/us-west" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-tempo-us-west" "$VAULT_POLICY"
# app/test-hello-world
SEC_KEY=epl/app/test-hello-world
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$( vault kv get -format=json epl/clickhouse/testch | jq -Srj '.data.data.db_chdb_a_rw' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
SEC_2=$(aws_secret_key)
SEC_3=$( vault kv get -format=json epl/pg/testdb | jq -Srj '.data.data.pg_db_testdb_a_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' | xargs -0 printf 'postgresql://testdb_a:%s@epl-pg-testdb.service.consul:5433/testdb_a' )
MERGED_SECRET=$( echo "{\"ch_shard_chshard_password\":\"$SEC_1\",\"minio_bucket_storage_password\":\"$SEC_2\",\"pg_shard_default\":\"$SEC_3\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/app/test-hello-world" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-app-test-hello-world" "$VAULT_POLICY"
# bb-depl/moonbeam-dev
SEC_KEY=epl/bb-depl/moonbeam-dev
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(aws_secret_key)
SEC_2=$( vault kv get -format=json epl/pg/testdb | jq -Srj '.data.data.pg_db_bbtest_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
MERGED_SECRET=$( echo "{\"env_var_test_minio\":\"$SEC_1\",\"env_var_test_postgresql\":\"$SEC_2\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/bb-depl/moonbeam-dev" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-bb-depl-moonbeam-dev" "$VAULT_POLICY"
# grafana/main
SEC_KEY=epl/grafana/main
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(random_password_42)
SEC_2=$( vault kv get -format=json epl/pg/testdb | jq -Srj '.data.data.pg_db_grafana_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
MERGED_SECRET=$( echo "{\"admin_password\":\"$SEC_1\",\"postgres_password\":\"$SEC_2\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/grafana/main" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-grafana-main" "$VAULT_POLICY"
# minio/global
SEC_KEY=epl/minio/global
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(aws_secret_key)
SEC_2=$( vault kv get -format=json epl/bb-depl/moonbeam-dev | jq -Srj '.data.data.env_var_test_minio' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
SEC_3=$( vault kv get -format=json epl/docker-registry | jq -Srj '.data.data.minio_bucket_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
SEC_4=$( vault kv get -format=json epl/app/test-hello-world | jq -Srj '.data.data.minio_bucket_storage_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
SEC_5=$( vault kv get -format=json epl/loki/main | jq -Srj '.data.data.minio_bucket_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
SEC_6=$( vault kv get -format=json epl/tempo/us-west | jq -Srj '.data.data.minio_bucket_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
MERGED_SECRET=$( echo "{\"admin_password\":\"$SEC_1\",\"minio_user_bb-depl-moonbeam-dev_password\":\"$SEC_2\",\"minio_user_docker_registry_password\":\"$SEC_3\",\"minio_user_epl_app_test_hello_world_password\":\"$SEC_4\",\"minio_user_loki_main_password\":\"$SEC_5\",\"minio_user_tempo_us_west_password\":\"$SEC_6\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/minio/global" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-minio-global" "$VAULT_POLICY"
