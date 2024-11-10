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
# loki/main-us-east
SEC_KEY=epl/loki/main-us-east
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
path "epl/data/loki/main-us-east" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-loki-main-us-east" "$VAULT_POLICY"
# create consul kv policy epl-loki-main-us-east-loki
CONSUL_POLICY=$( cat <<EOF
key_prefix "nomad-loki/epl-loki-main-us-east-loki" {
    policy = "write"
}
EOF
)
ensure_consul_token_with_policy_exists $( echo "$MERGED_SECRET" | jq -r '.consul_token' ) 'epl-loki-main-us-east-loki' "$CONSUL_POLICY"
# tempo/r2-tempo
SEC_KEY=epl/tempo/r2-tempo
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
path "epl/data/tempo/r2-tempo" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-tempo-r2-tempo" "$VAULT_POLICY"
# minio/main-us-east
SEC_KEY=epl/minio/main-us-east
CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )
CURRENT_SECRET=${CURRENT_SECRET:-\{\}}
SEC_1=$(aws_secret_key)
SEC_2=$( vault kv get -format=json epl/docker-registry | jq -Srj '.data.data.minio_bucket_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
SEC_3=$( vault kv get -format=json epl/loki/main-us-east | jq -Srj '.data.data.minio_bucket_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
SEC_4=$( vault kv get -format=json epl/tempo/r2-tempo | jq -Srj '.data.data.minio_bucket_password' | sed 's/$/\\n/g' | tr -d '\n' | sed 's/..$//' )
MERGED_SECRET=$( echo "{\"admin_password\":\"$SEC_1\",\"minio_user_docker_registry_password\":\"$SEC_2\",\"minio_user_loki_main_us_east_password\":\"$SEC_3\",\"minio_user_tempo_r2_tempo_password\":\"$SEC_4\"} $CURRENT_SECRET" | jq -S -s add )
if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
# create vault kv policy for job
VAULT_POLICY=$( cat <<EOF
path "epl/data/minio/main-us-east" {
  capabilities = ["read"]
}
EOF
)
ensure_vault_policy_for_kv_exists "epl-minio-main-us-east" "$VAULT_POLICY"
