use std::collections::HashSet;
use std::fmt::Write;

use crate::{static_analysis::server_runtime::{
    ProvisioningScriptTag, ServerRuntime, VaultSecretHandle,
}, database::TableRowPointerRegion};

pub fn generate_nomad_job_secrets(runtime: &mut ServerRuntime, region: TableRowPointerRegion) {
    let mut res = String::new();

    res += r#"#!/bin/sh

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

"#;

    // figure out secrets deployment order by dependencies on other secrets
    let mut key_deployed_register: HashSet<&VaultSecretHandle> = HashSet::new();
    let mut vault_secret_deployed_register: HashSet<&String> = HashSet::new();
    let mut secret_deployment_order: Vec<&String> =
        Vec::with_capacity(runtime.vault_secrets(region).len());

    while secret_deployment_order.len() < runtime.vault_secrets(region).len() {
        let current_length = secret_deployment_order.len();
        for (vs, job_secs) in runtime.vault_secrets(region) {
            if vault_secret_deployed_register.contains(vs) {
                continue;
            }

            let mut can_deploy = true;
            for svv in job_secs.keys.values() {
                match svv.request() {
                    crate::static_analysis::server_runtime::VaultSecretRequest::ConsulTokenWithPolicy { .. } => {},
                    crate::static_analysis::server_runtime::VaultSecretRequest::AlphanumericPassword42Symbols => {},
                    crate::static_analysis::server_runtime::VaultSecretRequest::AwsSecretKey => {},
                    crate::static_analysis::server_runtime::VaultSecretRequest::Pem => {},
                    crate::static_analysis::server_runtime::VaultSecretRequest::PasswordSaltOfCurrentSecret { .. } => {},
                    crate::static_analysis::server_runtime::VaultSecretRequest::ExistingVaultSecret { handle, .. } => {
                        if !key_deployed_register.contains(handle.as_ref()) {
                            can_deploy = false;
                        }
                    },
                }
            }

            if can_deploy {
                for svv in job_secs.keys.values() {
                    assert!(key_deployed_register.insert(svv));
                }
                assert!(vault_secret_deployed_register.insert(vs));

                secret_deployment_order.push(vs);
            }
        }
        if current_length == secret_deployment_order.len() {
            panic!(
                "Secret deployment order is stuck and cannot progress: {:?}",
                secret_deployment_order
            );
        }
    }

    assert_eq!(secret_deployment_order.len(), runtime.vault_secrets(region).len());

    for vs in secret_deployment_order {
        let job_secs = runtime.vault_secrets(region).get(vs).unwrap();

        if job_secs.declare_only || job_secs.keys.is_empty() {
            continue;
        }

        res += "# ";
        res += vs;
        res += "\n";
        res += "SEC_KEY=epl/";
        res += vs;
        res += "\n";
        res += "CURRENT_SECRET=$( vault kv get -format=json $SEC_KEY | jq -S '.data.data' )\n";
        res += "CURRENT_SECRET=${CURRENT_SECRET:-\\{\\}}\n";

        let mut sd = String::new();
        let mut sm = String::new();
        // if secret is static, non generated, then we need
        if job_secs.renew_if_source_changed {
            sm += r#"MERGED_SECRET=$( echo "$CURRENT_SECRET {"#;
        } else {
            sm += r#"MERGED_SECRET=$( echo "{"#;
        }

        let mut count = 0;
        for (svk, svv) in &job_secs.keys {
            count += 1;
            assert!(!svk.contains('\"'));
            sm += "\\\"";
            sm += svk;
            sm += "\\\":\\\"";
            // declare secrets earlier so we can refer to them as variable later
            write!(&mut sd, "SEC_{count}=").unwrap();
            write!(&mut sm, "$SEC_{count}").unwrap();
            match svv.request() {
                crate::static_analysis::server_runtime::VaultSecretRequest::ConsulTokenWithPolicy { .. } => {
                    sd += "$(generate_consul_acl_token)";
                },
                crate::static_analysis::server_runtime::VaultSecretRequest::AlphanumericPassword42Symbols => {
                    sd += "$(random_password_42)";
                },
                crate::static_analysis::server_runtime::VaultSecretRequest::AwsSecretKey => {
                    sd += "$(aws_secret_key)";
                },
                crate::static_analysis::server_runtime::VaultSecretRequest::PasswordSaltOfCurrentSecret { key_name } => {
                    let kv = job_secs.keys.keys().enumerate().filter(|(_, orig_sec_key)| {
                        orig_sec_key.as_str() == key_name.as_str()
                    }).next();
                    if let Some((idx, _)) = kv {
                        let idx = idx + 1;
                        assert!(idx < count, "Secret to salt must go before secret generation");
                        write!(&mut sd, "$( echo -n $SEC_{idx} | mkpasswd -s )").unwrap();
                    } else {
                        panic!("Can't find existing secret in current secret with {key_name}");
                    }
                },
                crate::static_analysis::server_runtime::VaultSecretRequest::Pem => {
                    // nothing to generate we get from nginx
                },
                crate::static_analysis::server_runtime::VaultSecretRequest::ExistingVaultSecret { handle, sprintf } => {
                    let maybe_sprintf = if let Some(form) = sprintf {
                        assert!(!form.contains('\''), "Sprintf form cannot contain single quotes");
                        assert!(!form.contains('\"'), "Sprintf form cannot contain double quotes");
                        assert_eq!(form.match_indices("%s").count(), 1, "Vault secret sprintf form must contan one %s, got: {form}");
                        format!(" | xargs -0 printf '{form}'")
                    } else { "".to_string() };
                    // replace new lines in json as \n then remove new line symbols with tr and replace new line at the end with nothing
                    sd += &format!("$( vault kv get -format=json {} | jq -Srj '{}' | sed 's/$/\\\\n/g' | tr -d '\\n' | sed 's/..$//'{} )", handle.secret_kv_path(), handle.secret_key_data_path(), maybe_sprintf);
                },
            }
            sd += "\n";
            sm += "\\\"";
            if count < job_secs.keys.len() {
                sm += ","
            }
        }

        if job_secs.renew_if_source_changed {
            sm += r#"}" | jq -S -s add )"#;
        } else {
            sm += r#"} $CURRENT_SECRET" | jq -S -s add )"#;
        }
        sm += "\n";

        // first secret computations
        res += &sd;
        // then merging of json from the variables
        res += &sm;

        res += r#"if [ "$MERGED_SECRET" != "$CURRENT_SECRET" ];
then
  echo "Secret $SEC_KEY changed"
  echo "$MERGED_SECRET" | vault kv put $SEC_KEY -
fi
"#;

        // All jobs can only access secrets, never modify
        res += "# create vault kv policy for job\n";
        res += "VAULT_POLICY=$( cat <<EOF\n";
        res += &format!(
            r#"path "epl/data/{vs}" {{
  capabilities = ["read"]
}}
"#
        );
        res += "EOF\n";
        res += ")\n";
        let policy_name = vs.replace("/", "-");
        res += &format!("ensure_vault_policy_for_kv_exists \"epl-{policy_name}\" \"$VAULT_POLICY\"\n");
        for (svk, svv) in &job_secs.keys {
            count += 1;
            match svv.request() {
                crate::static_analysis::server_runtime::VaultSecretRequest::ConsulTokenWithPolicy { policy } => {
                    res += "# create consul kv policy "; res += policy.policy_name(); res += "\n";
                    res += "CONSUL_POLICY=$( cat <<EOF\n";
                    res += policy.source();
                    res += "EOF\n";
                    res += ")\n";
                    res += &format!("ensure_consul_token_with_policy_exists $( echo \"$MERGED_SECRET\" | jq -r '.{}' ) '{}' \"$CONSUL_POLICY\"\n",
                        svk, policy.policy_name());
                },
                crate::static_analysis::server_runtime::VaultSecretRequest::AlphanumericPassword42Symbols => {},
                crate::static_analysis::server_runtime::VaultSecretRequest::PasswordSaltOfCurrentSecret { .. } => {},
                crate::static_analysis::server_runtime::VaultSecretRequest::AwsSecretKey => {},
                crate::static_analysis::server_runtime::VaultSecretRequest::Pem => {},
                crate::static_analysis::server_runtime::VaultSecretRequest::ExistingVaultSecret { .. } => {},
            }
        }
    }

    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::L1Resources,
        "provision-vault-secrets.sh",
        res,
    );
}
