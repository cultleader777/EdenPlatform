use std::collections::{HashMap, BTreeMap, btree_map::Entry};
use std::fmt::Write;

use crate::codegen::nixplan::{custom_user_secret_config, self};
use crate::static_analysis::get_global_settings;
use crate::{static_analysis::CheckedDB, codegen::{nixplan::{NixAllServerPlans, custom_user_secret_key, NixServerPlan}, secrets::{SecretsStorage, SecretValue, SecretKind, sec_files}}, database::{TableRowPointerTld, TableRowPointerRegion}};

pub fn provision_dns(db: &CheckedDB, plans: &mut NixAllServerPlans, secrets: &mut SecretsStorage) {
    let keys = generate_all_dns_sec_keys(db, secrets);
    provision_bind(db, plans, &keys);

    let keys = generate_acme_dns_keys(db, secrets);
    provision_acme_certs(db, plans, &keys);
}

struct AllDnsSecKeys {
    tlds: HashMap<TableRowPointerTld, DnsSecKey>,
    regions: HashMap<TableRowPointerRegion, DnsSecKey>,
    rev_zones: BTreeMap<String, DnsSecKey>,
}

struct DnsSecKey {
    fqdn: String,
    zsk_private_key_filename: String,
    zsk_private_key: SecretValue,
    zsk_public_key_filename: String,
    zsk_public_key: SecretValue,
    ksk_private_key_filename: String,
    ksk_private_key: SecretValue,
    ksk_public_key_filename: String,
    ksk_public_key: SecretValue,
    ksk_trust_anchor_key: String,
    ksk_ds_record: String,
}

fn generate_all_dns_sec_keys(db: &CheckedDB, secrets: &mut SecretsStorage) -> AllDnsSecKeys {
    let mut tlds = HashMap::new();
    let mut regions = HashMap::new();
    // btree for deterministic sorting
    let mut rev_zones: BTreeMap<String, DnsSecKey> = BTreeMap::new();

    // private dc PTR records
    assert!(rev_zones.insert(
        "10.in-addr.arpa.".to_string(),
        generate_dns_sec_key("10.in-addr.arpa", secrets)).is_none()
    );

    // public internet PTR records
    assert!(rev_zones.insert(
        "in-addr.arpa.".to_string(),
        generate_dns_sec_key("in-addr.arpa", secrets)).is_none()
    );

    for tld in db.db.tld().rows_iter() {
        let fqdn = db.db.tld().c_domain(tld);
        assert!(tlds.insert(tld, generate_dns_sec_key(fqdn, secrets)).is_none());
        for region in db.db.tld().c_referrers_region__tld(tld) {
            let fqdn = format!("{}.{}", db.db.region().c_region_name(*region), fqdn);
            regions.insert(*region, generate_dns_sec_key(&fqdn, secrets));

            for dc in db.db.region().c_referrers_datacenter__region(*region) {
                let second_octet = cidr_second_octet(db.db.datacenter().c_network_cidr(*dc));
                let rev_fqdn = format!("{second_octet}.10.in-addr.arpa");
                assert!(rev_zones.insert(format!("{rev_fqdn}."), generate_dns_sec_key(&rev_fqdn, secrets)).is_none());
            }
        }
    }

    AllDnsSecKeys { tlds, regions, rev_zones }
}

fn generate_dns_sec_key(fqdn: &str, secrets: &mut SecretsStorage) -> DnsSecKey {
    assert!(!fqdn.ends_with('.'));
    let private_zsk_key_path = format!("dnssec_{fqdn}_zsk_private_key");
    let public_zsk_key_path = format!("dnssec_{fqdn}_zsk_public_key");
    let private_ksk_key_path = format!("dnssec_{fqdn}_ksk_private_key");
    let public_ksk_key_path = format!("dnssec_{fqdn}_ksk_public_key");
    let public_ds_record_path = format!("dnssec_{fqdn}_ds_record");
    let dns_key_files = sec_files(&[
        (
            SecretKind::DnsSecZSKPrivateKey,
            &private_zsk_key_path,
            "zsk-output.private",
        ),
        (
            SecretKind::DnsSecZSKPublicKey,
            &public_zsk_key_path,
            "zsk-output.key",
        ),
        (
            SecretKind::DnsSecKSKPrivateKey,
            &private_ksk_key_path,
            "ksk-output.private",
        ),
        (
            SecretKind::DnsSecKSKPublicKey,
            &public_ksk_key_path,
            "ksk-output.key",
        ),
    ]);
    let ds_record_inputs = sec_files(&[
        (
            SecretKind::DnsSecKSKPublicKey,
            &public_ksk_key_path,
            "ksk-output.key",
        ),
    ]);
    let derived_ds_record = sec_files(&[
        (
            SecretKind::DnsSecDSRecord,
            &public_ds_record_path,
            "ds-record.txt",
        ),
    ]);

    let mut res = secrets.multi_secret_derive(
        &[], Vec::new(), dns_key_files,
        &format!(r#"
            dnssec-keygen -a ED25519 -n ZONE {fqdn}
            mv K*.key zsk-output.key
            mv K*.private zsk-output.private
            dnssec-keygen -a ED25519 -n ZONE -f KSK {fqdn}
            mv K*.key ksk-output.key
            mv K*.private ksk-output.private
        "#)
    );
    let ds_res = secrets.multi_secret_derive(
        &[],
        ds_record_inputs,
        derived_ds_record,
        r#"
           dnssec-dsfromkey ksk-output.key > ds-record.txt
        "#
    );

    let ksk_public_key = res.pop().unwrap();
    let ksk_private_key = res.pop().unwrap();
    let zsk_public_key = res.pop().unwrap();
    let zsk_private_key = res.pop().unwrap();
    let ksk_ds_record = ds_res[0].value().clone();

    let ksk_key_id = get_dnssec_key_id(ksk_public_key.value().as_str());
    let zsk_key_id = get_dnssec_key_id(zsk_public_key.value().as_str());
    let ksk_trust_anchor_key = get_dnssec_trust_anchor(ksk_public_key.value().as_str());

    DnsSecKey {
        fqdn: fqdn.to_string(),
        zsk_private_key,
        zsk_public_key,
        ksk_private_key,
        ksk_public_key,
        zsk_private_key_filename: format!("K{}.+015+{:0>5}.private", fqdn, zsk_key_id),
        zsk_public_key_filename: format!("K{}.+015+{:0>5}.key", fqdn, zsk_key_id),
        ksk_private_key_filename: format!("K{}.+015+{:0>5}.private", fqdn, ksk_key_id),
        ksk_public_key_filename: format!("K{}.+015+{:0>5}.key", fqdn, ksk_key_id),
        ksk_trust_anchor_key,
        ksk_ds_record,
    }
}

fn get_dnssec_key_id(input: &str) -> u32 {
    lazy_static! {
        static ref REGEX: regex::Regex = regex::Regex::new("keyid\\s+([0-9]+)").expect("Should work");
    }

    let res = REGEX.captures(input).unwrap();
    res.get(1).unwrap().as_str().parse::<u32>().unwrap()
}

struct AcmeKeys {
    tlds: BTreeMap<TableRowPointerTld, SecretValue>,
    region_vault_tokens: BTreeMap<TableRowPointerRegion, SecretValue>,
}

fn generate_acme_dns_keys(db: &CheckedDB, secrets: &mut SecretsStorage) -> AcmeKeys {
    let mut res = AcmeKeys {
        tlds: Default::default(),
        region_vault_tokens: Default::default(),
    };

    if db.projections.dns_checks.certs_needed {
        for reg in db.db.region().rows_iter() {
            let region_name = db.db.region().c_region_name(reg);
            let skey = format!("acme_region_{region_name}_vault_token");
            if secrets.contains_secret(&skey) {
                let sec = secrets.fetch_secret(skey, SecretKind::VaultToken);
                assert!(res.region_vault_tokens.insert(reg, sec).is_none());
            }
        }
    }

    for tld in db.db.tld().rows_iter() {
        if !db.db.tld().c_automatic_certificates(tld) {
            continue;
        }

        let tld_name = db.db.tld().c_domain(tld);
        let tld_snake = sanitize_dns_name(&tld_name);
        let secret = secrets.fetch_secret(format!("dns_update_rfc2136_key_{tld_snake}"), SecretKind::DnsRfc2136Key);
        assert!(res.tlds.insert(tld, secret).is_none());
    }

    res
}

pub fn sanitize_dns_name(input: &str) -> String {
    input.replace(".", "_").replace("-", "_")
}

#[test]
fn test_get_dnssec_key_id() {
    let input = r#"
; This is a zone-signing key, keyid 18630, for epl-infra.net.
; Created: 20230714153245 (Fri Jul 14 15:32:45 2023)
; Publish: 20230714153245 (Fri Jul 14 15:32:45 2023)
; Activate: 20230714153245 (Fri Jul 14 15:32:45 2023)
epl-infra.net. IN DNSKEY 256 3 15 IgTyZxRb9t3VVD6A1fO9Lnq17op2Jl+sMR6XxzoxO/A=
"#;
    assert_eq!(get_dnssec_key_id(input), 18630);
}

fn get_dnssec_trust_anchor(input: &str) -> String {
    lazy_static! {
        static ref KEY_REGEX: regex::Regex =
            regex::Regex::new(r#"\s+IN\s+DNSKEY\s+(\d+)\s+(\d+)\s+(\d+)\s+(.+)"#).expect("Should work");
        static ref DOMAIN_REGEX: regex::Regex =
            regex::Regex::new(r#"This is a key-signing key,.+, for\s+(.+)"#).expect("Should work");
    }

    let domain = DOMAIN_REGEX.captures(input).unwrap();
    let domain = domain.get(1).unwrap();

    let res = KEY_REGEX.captures(input).unwrap();
    let mut numbers = Vec::with_capacity(3);
    for i in 1..=3 {
        numbers.push(res.get(i).unwrap().as_str().parse::<u32>().unwrap());
    }
    let key = res.get(4).unwrap().as_str().trim().to_string();

    format!("{} initial-key {} {} {} \"{}\";", domain.as_str(), numbers[0], numbers[1], numbers[2], key)
}

#[test]
fn test_get_dnssec_trust_anchor() {
    let input = r#"
; This is a key-signing key, keyid 36162, for epl-infra.net.
; Created: 20230714153245 (Fri Jul 14 15:32:45 2023)
; Publish: 20230714153245 (Fri Jul 14 15:32:45 2023)
; Activate: 20230714153245 (Fri Jul 14 15:32:45 2023)
; SyncPublish: 20230721183245 (Fri Jul 21 18:32:45 2023)
epl-infra.net. IN DNSKEY 257 3 15 S24/J2jOQQLNHSScBgWd38PUHXGkwBY7PGwjlSpemIw=
"#;
    assert_eq!(
        get_dnssec_trust_anchor(input),
        "epl-infra.net. initial-key 257 3 15 \"S24/J2jOQQLNHSScBgWd38PUHXGkwBY7PGwjlSpemIw=\";".to_string()
    );
}

struct DnsSecNeededKeys<'a> {
    key: &'a DnsSecKey,
    need_public: bool,
    need_private: bool,
}

fn provision_bind(db: &CheckedDB, plans: &mut NixAllServerPlans, keys: &AllDnsSecKeys) {
    let dns = &db.projections.dns_checks;

    let mut zone_files: HashMap<(TableRowPointerTld, TableRowPointerRegion), ZoneFiles> = HashMap::new();
    for region in db.db.region().rows_iter() {
        for tld in db.db.tld().rows_iter() {
            assert!(zone_files.insert((tld, region), dns_internal_zone_files_gen(db, tld, region)).is_none());
        }
    }

    let mut private_root_zone_files: HashMap<TableRowPointerTld, ZoneFiles> = HashMap::new();
    for tld in db.db.tld().rows_iter() {
        assert!(private_root_zone_files.insert(tld, dns_private_root_zone_file(db, tld, keys)).is_none());
    }

    let mut public_root_zone_files: HashMap<TableRowPointerTld, ZoneFiles> = HashMap::new();
    for tld in db.db.tld().rows_iter() {
        assert!(public_root_zone_files.insert(tld, dns_public_root_zone_file(db, tld)).is_none());
    }

    let mut zone_configs: HashMap<TableRowPointerRegion, String> = HashMap::new();
    for region in db.db.region().rows_iter() {
        let fwd_zone_configs = generate_forwarding_zone_configs(db, region);
        zone_configs.insert(region, fwd_zone_configs);
    }
    let root_fwd_zone_configs = generate_root_forwarding_zone_configs(db);

    let is_multi_region_setup = db.db.region().len() >= 2;
    let is_single_region_setup = !is_multi_region_setup;

    for server in db.db.server().rows_iter() {
        let dc = db.db.server().c_dc(server);
        let region = db.db.datacenter().c_region(dc);
        let dns = dns.regions.get(&region).unwrap();
        let dns_master = dns.master_server.as_ref().unwrap();
        let plan = plans.fetch_plan(server);
        let is_dns_master = server == *dns_master;
        let is_dns_slave = dns.slave_servers.contains(&server);
        let fqdn = db.projections.server_fqdns.value(server);

        let mut dnssec_keys_for_server: BTreeMap<String, DnsSecNeededKeys> = BTreeMap::new();

        let mut ctx = BindContext {
            private_root_zone_files: &private_root_zone_files,
            public_root_zone_files: &public_root_zone_files,
            zone_files: &zone_files,
            keys,
            region,
            is_single_region_setup,
            is_multi_region_setup,
            dns_sec_keys_for_server: &mut dnssec_keys_for_server,
            zone_vars: ZoneVars { linkings: String::new() },
        };

        let forward_configs =
            if !dns.contains(server) {
                zone_configs.get(&region).unwrap()
            } else { "" };

        let root_forward_configs =
            if !db.projections.dns_checks.all_regions.contains(server) {
                &root_fwd_zone_configs
            } else { "" };

        let mut rfc2136_keys = String::new();
        let mut no_keys_acl = String::new();
        let mut with_keys_acl = String::new();
        if Some(server) == db.projections.dns_checks.all_regions.master_server {
            for tld in db.db.tld().rows_iter() {
                if !db.db.tld().c_automatic_certificates(tld) {
                    continue;
                }

                let tld = db.db.tld().c_domain(tld);
                writeln!(&mut rfc2136_keys, "include \"/run/keys/dns_bind_rfc2136_key_{tld}.config\";").unwrap();
                write!(&mut no_keys_acl, "!key rfc2136key.{tld}.; ").unwrap();
                write!(&mut with_keys_acl, "key rfc2136key.{tld}.; ").unwrap();
            }
        }

        let mut nix_block = format!(
            r#"
    users.users.named.extraGroups = ["keys"];
    services.bind =
    {{
        enable = true;
        extraOptions = ''
          recursion yes;
          dnssec-validation auto;
          validate-except {{ consul.; }};
          key-directory "/run/dnsseckeys";
        '';
        forwarders = [ "1.1.1.1" ];
        cacheNetworks = [
          # bind can be internet
          # facing depending on DC
          "0.0.0.0/0"
        ];
        extraConfig = ''
          BIND_TRUST_ANCHORS

{rfc2136_keys}

          dnssec-policy epl {{
            keys {{
              ksk key-directory lifetime unlimited algorithm ED25519;
              zsk key-directory lifetime unlimited algorithm ED25519;
            }};
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
          }};

view lan {{
          # add VPN address so local user integration tests pass
          match-clients {{ {no_keys_acl}10.0.0.0/8; 172.21.0.0/16; localhost; }};
          zone "consul." IN {{
              type forward;
              forward only;
              forwarders {{ 127.0.0.1 port 8600; }};
          }};
{forward_configs}
{root_forward_configs}
"#
        );

        if is_dns_master {
            // this is master
            write_dc_master_internal_bind_configs(db, &mut nix_block, &mut ctx);
        } else if is_dns_slave {
            // this is slave
            write_dc_slave_internal_bind_configs(db, &mut nix_block, &mut ctx);
        }

        let mkdirs_script = r#"
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
"#;

        let key_copy = generate_key_links(&ctx, plan);
        let mut extra_by_role = String::new();
        if is_dns_master {
            // in first provisioning bind is running, but zone files are not loaded
            // because users named:named are not created yet which should own those files
            // post second round of secrets load in l1 provisioning we check
            // if there's error in bind about not finding zone files and if there is we restart
            // the service and everything should work.
            extra_by_role += &key_copy;
            extra_by_role += r#"
journalctl _SYSTEMD_INVOCATION_ID=$( systemctl show --value -p InvocationID bind.service ) \
  | grep -e 'zone_rekey:dns_zone_getdnsseckeys failed: permission denied' -e "key-directory: '/run/dnsseckeys' does not exist" \
  && systemctl restart bind.service || true
"#;
        } else {
            write!(&mut extra_by_role, r#"
if [[ ! $( timeout 0.5s dig @127.0.0.1 {fqdn} +dnssec +short ) ]]
then
    systemctl restart bind.service
fi
"#).unwrap();
        }

        nix_block += "
};
";

        if db.projections.dns_checks.all_regions.contains(server) {
            write!(&mut nix_block, r#"
view internet {{
          match-clients {{ {with_keys_acl}any; }};
          recursion no;
"#).unwrap();
            if db.projections.dns_checks.all_regions.master_server == Some(server) {
                write_dc_master_public_bind_configs(db, &mut nix_block, &mut ctx);
            } else {
                write_dc_slave_public_bind_configs(db, &mut nix_block, &mut ctx);
            }

            nix_block += "
};
";
        }

        nix_block += "
        '';
    };
";

        let bind_trust_anchors = bind_trust_anchors(db, keys);
        // hacky but works
        let nix_block = nix_block.replace("BIND_TRUST_ANCHORS", &bind_trust_anchors);
        let server_private_ip = db.db.network_interface().c_if_ip(
            *db.projections.consul_network_iface.value(server)
        );
        let nix_block = nix_block.replace("TRANSFER_SOURCE_LAN", &server_private_ip);

        plan.add_custom_nix_block(nix_block);

        let linkings = &ctx.zone_vars.linkings;
        let dns_update_functions = r#"
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
"#;
        let maybe_reload_bind = if !ctx.zone_vars.linkings.is_empty() {
            r#"
# we could implement some complex mechanism
# to detect if zone files changed later
/run/current-system/sw/bin/systemctl reload bind.service || true
"#
        } else { "" };
        let maybe_linkings = if is_dns_master { linkings.as_str() } else { "" };
        let maybe_dns_update_functions = if is_dns_master { dns_update_functions } else { "" };
        if is_dns_master {
            // in case DNS is down (shouldn't be, but it was) write zone files for bind
            plan.add_pre_l1_provisioning_shell_hook(format!(r#"
# ------------- DNS START ---------------
{maybe_dns_update_functions}

# first installation bind not installed yet
if id named
then

{mkdirs_script}

{maybe_linkings}

fi
# ------------- DNS END   ---------------
"#));
        }
        plan.add_post_second_round_secrets_shell_hook(format!(r#"
# ------------- DNS START ---------------
{maybe_dns_update_functions}

{mkdirs_script}

{maybe_linkings}

{maybe_reload_bind}

{extra_by_role}
# ------------- DNS END   ---------------
"#));
    }
}

fn provision_acme_certs(db: &CheckedDB, plans: &mut NixAllServerPlans, keys: &AcmeKeys) {
    let global_setings = get_global_settings(&db.db);
    let admin_email = &global_setings.admin_email;

    let shell_updater_name = "epl-upload-acme-cert-to-vault";

    if let Some(ms) = &db.projections.dns_checks.all_regions.master_server {
        let plan = plans.fetch_plan(*ms);
        let net_if = db.projections.internet_network_iface.get(ms).unwrap();
        let internet_ip = db.db.network_interface().c_if_ip(*net_if);

        if db.projections.dns_checks.certs_needed {
            let mut shell_script = r#"
TARGET_TLD=$1
TARGET_TLD_SANITIZED=$( echo $TARGET_TLD | tr '.-' '_' )

SECRET_VALUE=$( echo "{\"full_chain\":\"$( cat fullchain.pem | sed 's/$/\\n/g' | tr -d '\n' )\",\"key\":\"$( cat key.pem | sed 's/$/\\n/g' | tr -d '\n' )\",\"cert\":\"$( cat cert.pem | sed 's/$/\\n/g' | tr -d '\n' )\"}" | jq -S )
EXTLB_VALUE=$( echo "{\"full_chain_$TARGET_TLD_SANITIZED\":\"$( cat fullchain.pem | sed 's/$/\\n/g' | tr -d '\n' )\",\"key_$TARGET_TLD_SANITIZED\":\"$( cat key.pem | sed 's/$/\\n/g' | tr -d '\n' )\"}" | jq -S )
"#.to_string();
            let mut token_renewal_script = String::new();

            // renew all regions at once
            for (region, vt) in &keys.region_vault_tokens {
                let region_name = db.db.region().c_region_name(*region);
                let abs_token = plan.add_secret(nixplan::custom_user_secret_key(
                    "acme".to_string(),
                    format!("acme-vault-update-token-{region_name}"),
                    vt.clone(),
                ));
                let abs_token_path = abs_token.absolute_path();

                write!(&mut shell_script, r#"
export VAULT_TOKEN=$( ${{pkgs.coreutils}}/bin/cat {abs_token_path} )
export VAULT_ADDR="https://vault.service.{region_name}.consul:8200"

# always renew token first
${{pkgs.vault}}/bin/vault token renew | ${{pkgs.gnugrep}}/bin/grep -v token

VAULT_PATH=epl/certs/$TARGET_TLD
CURRENT_VALUE=$( ${{pkgs.vault}}/bin/vault kv get -format=json $VAULT_PATH | ${{pkgs.jq}}/bin/jq -S '.data.data' )
if [ "$CURRENT_VALUE" != "$SECRET_VALUE" ]
then
    echo Cert changed, updating in vault
    echo "$SECRET_VALUE" | ${{pkgs.vault}}/bin/vault kv put $VAULT_PATH -
fi

EXTLB_VAULT_PATH=epl/ext-lb
CURRENT_EXT_LB=$( ${{pkgs.vault}}/bin/vault kv get -format=json $EXTLB_VAULT_PATH | ${{pkgs.jq}}/bin/jq -S '.data.data' )
UPDATED_EXT_LB=$( echo "$CURRENT_EXT_LB $EXTLB_VALUE" | ${{pkgs.jq}}/bin/jq -S -s add )
if [ "$UPDATED_EXT_LB" != "$CURRENT_EXT_LB" ]
then
    echo Ext lb changed, updating in vault
    echo "$UPDATED_EXT_LB" | ${{pkgs.vault}}/bin/vault kv put $EXTLB_VAULT_PATH -
fi

"#).unwrap();
                write!(&mut token_renewal_script, r#"
      if [ -f {abs_token_path} ];
      then
        export VAULT_TOKEN=$( cat {abs_token_path} )
        export VAULT_ADDR="https://vault.service.{region_name}.consul:8200"
        # don't emit secret in logs
        vault token renew | grep -v token
        echo "Vault token at {abs_token_path} renewed"
      else
        echo "Vault token doesn't exist at path {abs_token_path}"
      fi
"#).unwrap();
            }

            // service to renew vault token periodically if it exists
            plan.add_custom_nix_block(format!(r#"
  systemd.timers.acme-refresh-vault-tokens = {{
    description = "Refresh vault tokens timer";
    wantedBy = [ "timers.target" ];
    partOf = [ "acme-refresh-vault-tokens.service" ];
    timerConfig.OnCalendar = "hourly";
    timerConfig.Persistent = "true";
  }};

  systemd.services.acme-refresh-vault-tokens = {{
    description = "refresh vault token that allows putting into cert secrets";
    serviceConfig.Type = "simple";
    path = with pkgs; [ vault gnugrep coreutils ];
    script = ''
{token_renewal_script}
    '';
  }};
"#));

            plan.add_shell_package(shell_updater_name, shell_script.as_str());
        }

        plan.add_zfs_dataset(
            "acme".to_string(),
            crate::codegen::nixplan::ZfsDataset {
                zpool: "rpool".to_string(),
                encryption_enabled: true,
                compression_enabled: true,
                mountpoint: "/var/lib/acme".to_string(),
                expose_to_containers: false,
                record_size: "128k".to_string(),
            },
        );

        for tld in db.db.tld().rows_iter() {
            if !db.db.tld().c_automatic_certificates(tld) {
                continue;
            }

            let key = keys.tlds.get(&tld).unwrap();
            let tld_name = db.db.tld().c_domain(tld);
            let tld_name_kebab = db.db.tld().c_domain(tld).replace(".", "-");
            let key = key.value();

            plan.add_secret_config(custom_user_secret_config(
                "named".to_string(),
                format!("dns_bind_rfc2136_key_{tld_name}.config"),
                format!(r#"
key "rfc2136key.{tld_name}" {{
        algorithm hmac-sha256;
        secret "{key}";
}};
"#)
            ));

            let acme_creds = plan.add_secret_config(custom_user_secret_config(
                "acme".to_string(),
                format!("dns_acme_rfc2136_key_{tld_name}.env"),
                format!(r#"
RFC2136_NAMESERVER='{internet_ip}:53'
RFC2136_TSIG_ALGORITHM='hmac-sha256.'
RFC2136_TSIG_KEY='rfc2136key.{tld_name}'
RFC2136_TSIG_SECRET='{key}'
"#)
            ));
            let acme_creds_path = acme_creds.absolute_path();
            let maybe_update = if !keys.region_vault_tokens.is_empty() {
                format!(r#"
        postRun = "/run/current-system/sw/bin/{shell_updater_name} {tld_name_kebab}";"#)
            } else { "".to_string() };

            plan.add_custom_nix_block(format!(r#"
    users.users.acme.extraGroups = ["keys"];
    security.acme.acceptTerms = true;
    security.acme.defaults.email = "{admin_email}";
    security.acme.certs."{tld_name}" = {{
        domain = "{tld_name}";
        extraDomainNames = [ "*.{tld_name}" ];
        dnsProvider = "rfc2136";{maybe_update}
        environmentFile = "{acme_creds_path}";
        # We don't need to wait for propagation since this is a local DNS server
        dnsPropagationCheck = false;
    }};
"#));
        }
    }

    // generate vault policy for cert update + token
    for region in db.db.region().rows_iter() {
        let prov_server = db.projections.provisioning_server_in_region.value(region);

        // provision dns cert policies for vault
        if let Some(prov_server) = prov_server {
            let plan = plans.fetch_plan(*prov_server);
            plan.add_shell_package("epl-acme-vault-policies", r#"
                if [ -z "$VAULT_TOKEN" ]
                then
                    echo Must set VAULT_TOKEN for this script
                    exit 7
                fi

                while ! curl -f -s https://vault.service.consul:8200 &>/dev/null
                do
                    sleep 1
                done

                cat > /tmp/epl-acme-vault-token-policy.json<<EOL
                {
                    "token_explicit_max_ttl": 0,
                    "name": "epl-acme-certs",
                    "orphan": true,
                    "token_period": 259200,
                    "renewable": true
                }
                EOL

                cat > /tmp/epl-acme-vault-policy.hcl<<EOL
                path "epl/data/certs/*" {
                    capabilities = ["read", "list", "create", "patch", "update", "delete"]
                }

                # allow updating external load balancer as well
                path "epl/data/ext-lb" {
                    capabilities = ["read", "list", "create", "patch", "update", "delete"]
                }

                # Allow looking up the token passed to validate the token has the
                # proper capabilities. This is provided by the "default" policy.
                path "auth/token/lookup-self" {
                    capabilities = ["read"]
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

                vault policy write epl-acme-certs /tmp/epl-acme-vault-policy.hcl
                vault write /auth/token/roles/epl-acme-certs @/tmp/epl-acme-vault-token-policy.json

                ORIGINAL_TOKEN=$VAULT_TOKEN
                export VAULT_TOKEN=$1
                if ! vault token lookup
                then
                    # token invalid, needs to be created
                    export VAULT_TOKEN=$ORIGINAL_TOKEN
                    NEW_TOKEN=$( vault token create -policy epl-acme-certs -period 168h -orphan | grep 'hvs.' | sed -E 's/^.* hvs/hvs/' )
                    echo "ACME_CERTS_VAULT_TOKEN $NEW_TOKEN"
                fi
"#);
        }
    }
}

fn bind_trust_anchors(db: &CheckedDB, keys: &AllDnsSecKeys) -> String {
    let mut res = String::new();

    res += "trust-anchors {\n";

    for tld in db.db.tld().rows_iter() {
        let key = keys.tlds.get(&tld).unwrap();
        res += "  ";
        res += &key.ksk_trust_anchor_key;
        res += "\n";
    }

    for region in db.db.region().rows_iter() {
        let key = keys.regions.get(&region).unwrap();
        res += "  ";
        res += &key.ksk_trust_anchor_key;
        res += "\n";
    }

    for rev_key in keys.rev_zones.values() {
        res += "  ";
        res += &rev_key.ksk_trust_anchor_key;
        res += "\n";
    }

    res += "};\n";


    res
}

fn generate_key_links(ctx: &BindContext, plan: &mut NixServerPlan) -> String {
    let mut key_links = String::new();

    let mut write_key_link = |filename: &str, source: &SecretValue| {
        let sanitized_name = sanitize_key_name(filename);
        let sec_key = plan.add_secret(custom_user_secret_key(
            "named".to_string(),
            sanitized_name,
            source.clone())
        );
        key_links += "cp -pu ";
        key_links += &sec_key.absolute_path();
        key_links += " /run/dnsseckeys/";
        key_links += filename;
        key_links += "\n";
    };

    for dns_key in ctx.dns_sec_keys_for_server.values() {
        if dns_key.need_private {
            write_key_link(&dns_key.key.zsk_private_key_filename, &dns_key.key.zsk_private_key);
            write_key_link(&dns_key.key.ksk_private_key_filename, &dns_key.key.ksk_private_key);
        }

        if dns_key.need_public {
            write_key_link(&dns_key.key.zsk_public_key_filename, &dns_key.key.zsk_public_key);
            write_key_link(&dns_key.key.ksk_public_key_filename, &dns_key.key.ksk_public_key);
        }
    }

    key_links
}

fn sanitize_key_name(input: &str) -> String {
    lazy_static! {
        static ref REGEX: regex::Regex = regex::Regex::new("[^A-Za-z0-9-]").expect("Should work");
    }

    REGEX.replace_all(input, "-").to_string()
}

#[test]
fn test_sanitize_key_name() {
    assert_eq!(sanitize_key_name("Kdc1.epl-infra.net+015+00000.key-key"), "Kdc1-epl-infra-net-015-00000-key-key")
}

struct ZoneVars {
    linkings: String,
}

struct BindContext<'a> {
    private_root_zone_files: &'a HashMap<TableRowPointerTld, ZoneFiles>,
    public_root_zone_files: &'a HashMap<TableRowPointerTld, ZoneFiles>,
    zone_files: &'a HashMap<(TableRowPointerTld, TableRowPointerRegion), ZoneFiles>,
    is_single_region_setup: bool,
    is_multi_region_setup: bool,
    region: TableRowPointerRegion,
    keys: &'a AllDnsSecKeys,
    dns_sec_keys_for_server: &'a mut BTreeMap<String, DnsSecNeededKeys<'a>>,
    zone_vars: ZoneVars,
}

impl<'a> BindContext<'a> {
    fn dns_sec_tag_key(&mut self, key: &'a DnsSecKey) {
        let e = self.dns_sec_keys_for_server.entry(key.fqdn.clone()).or_insert_with(|| {
            DnsSecNeededKeys { key, need_public: false, need_private: false }
        });
        e.need_private = true;
        e.need_public = true;
    }
}

fn write_dc_master_public_bind_configs(db: &CheckedDB, res: &mut String, ctx: &mut BindContext) {
    let is_master = true;

    let first_tld = db.db.tld().rows_iter().next().unwrap();
    let first_tld_prefix = format!("{}.", db.db.tld().c_domain(first_tld));
    let rev_fqdn = "in-addr.arpa.";
    let prefix = "public";
    // reverse zone
    let mut rev = String::new();
    rev += "$TTL 3600\n";
    rev += &rev_fqdn;
    rev += "\tIN\tSOA\tns1.";
    rev += &first_tld_prefix;
    rev += " ";
    rev += &first_tld_prefix;
    rev += " (\n";
    rev += " SERIAL_TO_REPLACE ; Serial\n";
    rev += " 3600 ; Refresh\n";
    rev += " 1800 ; Retry\n";
    rev += " 604800 ; Expire\n";
    rev += " 3600 ; Minimum TTL\n";
    rev += ")\n";

    for i in 0..db.projections.dns_checks.all_regions.total_dns_servers() {
        let i = i + 1;
        rev += &format!("in-addr.arpa.\tIN\tNS\tns{i}.");
        rev += &first_tld_prefix;
        rev += "\n";
    }

    rev += "\n";

    for tld in db.db.tld().rows_iter() {
        let fqdn = format!("{}.", db.db.tld().c_domain(tld));
        let key_for_domain = ctx.keys.tlds.get(&tld).unwrap();
        ctx.dns_sec_tag_key(key_for_domain);
        ctx.dns_sec_tag_key(ctx.keys.rev_zones.get(rev_fqdn).unwrap());
        // Main zone
        let zone_file = ctx.public_root_zone_files.get(&tld).unwrap();
        let allow_update = if db.db.tld().c_automatic_certificates(tld) {
            format!(r#"
  allow-update {{ key rfc2136key.{}.; }};
  allow-query {{ any; }};
"#, db.db.tld().c_domain(tld))
        } else { "".to_string() };
        write_nix_zone(
            res,
            prefix,
            &mut ctx.zone_vars,
            &fqdn,
            is_master,
            &db.projections.dns_checks.all_regions.master_internet_ip(db),
            &db.projections.dns_checks.all_regions.slaves_internet_ips(db),
            &zone_file.tld_zone,
            &allow_update,
        );

        for (ip, record) in &zone_file.reverse_zone_mappings {
            rev += ip;
            // ipv6 hack
            if !ip.ends_with("arpa.") {
                rev += ".in-addr.arpa.";
            }
            rev += "\tIN\tPTR\t";
            rev += record;
            rev += "\n";
        }
    }

    // one rev zone file for all tlds
    write_nix_zone(
        res,
        prefix,
        &mut ctx.zone_vars,
        &rev_fqdn,
        is_master,
        &db.projections.dns_checks.all_regions.master_internet_ip(db),
        &db.projections.dns_checks.all_regions.slaves_internet_ips(db),
        &rev,
        "",
    );
}

fn write_dc_slave_public_bind_configs(db: &CheckedDB, res: &mut String, ctx: &mut BindContext) {
    let is_master = false;
    let rev_fqdn = "in-addr.arpa.";
    let prefix = "public";

    for tld in db.db.tld().rows_iter() {
        let fqdn = format!("{}.", db.db.tld().c_domain(tld));
        let key_for_domain = ctx.keys.tlds.get(&tld).unwrap();
        ctx.dns_sec_tag_key(key_for_domain);
        ctx.dns_sec_tag_key(ctx.keys.rev_zones.get(rev_fqdn).unwrap());
        // Main zone
        let zone_file = ctx.public_root_zone_files.get(&tld).unwrap();
        write_nix_zone(
            res,
            prefix,
            &mut ctx.zone_vars,
            &fqdn,
            is_master,
            &db.projections.dns_checks.all_regions.master_internet_ip(db),
            &db.projections.dns_checks.all_regions.slaves_internet_ips(db),
            &zone_file.tld_zone,
            "
  allow-query { any; };
",
        );
    }

    // one rev zone file for all tlds
    write_nix_zone(
        res,
        prefix,
        &mut ctx.zone_vars,
        &rev_fqdn,
        is_master,
        &db.projections.dns_checks.all_regions.master_internet_ip(db),
        &db.projections.dns_checks.all_regions.slaves_internet_ips(db),
        "",
        "",
    );
}

fn write_dc_master_internal_bind_configs(db: &CheckedDB, res: &mut String, ctx: &mut BindContext) {
    let dns = db.projections.dns_checks.regions.get(&ctx.region).unwrap();
    let prefix = "private";
    if db.db.region().c_is_dns_master(ctx.region) {
        for tld in db.db.tld().rows_iter() {
            let fqdn = format!("{}.", db.db.tld().c_domain(tld));
            let is_master = true;
            let key_for_domain = ctx.keys.tlds.get(&tld).unwrap();
            ctx.dns_sec_tag_key(key_for_domain);
            // Main zone
            let zone_file = ctx.private_root_zone_files.get(&tld).unwrap();
            write_nix_zone(
                res,
                prefix,
                &mut ctx.zone_vars,
                &fqdn,
                is_master,
                &db.projections.dns_checks.all_regions.master_lan_ip(db),
                &db.projections.dns_checks.all_regions.slaves_lan_ips(db),
                &zone_file.tld_zone,
                "",
            );

            // Reverse zone
            for (rev_fqdn, zfile) in &zone_file.reverse_zones {
                ctx.dns_sec_tag_key(ctx.keys.rev_zones.get(rev_fqdn).unwrap());
                write_nix_zone(
                    res,
                    prefix,
                    &mut ctx.zone_vars,
                    rev_fqdn,
                    is_master,
                    &db.projections.dns_checks.all_regions.master_lan_ip(db),
                    &db.projections.dns_checks.all_regions.slaves_lan_ips(db),
                    zfile,
                    "",
                );
            }
        }
    }

    if ctx.is_multi_region_setup {
        // master zone file
        // TODO: use public ip here to transfer across DCs
        if db.db.region().c_is_dns_slave(ctx.region) {
            for tld in db.db.tld().rows_iter() {
                let fqdn = format!("{}.", db.db.tld().c_domain(tld));
                let is_master = false;
                let key_for_domain = ctx.keys.tlds.get(&tld).unwrap();
                ctx.dns_sec_tag_key(key_for_domain);
                // Main zone
                let zone_file = ctx.private_root_zone_files.get(&tld).unwrap();
                write_nix_zone(
                    res,
                    prefix,
                    &mut ctx.zone_vars,
                    &fqdn,
                    is_master,
                    &db.projections.dns_checks.all_regions.master_lan_ip(db),
                    &db.projections.dns_checks.all_regions.slaves_lan_ips(db),
                    &zone_file.tld_zone,
                    "",
                );

                // Reverse zone
                for (rev_fqdn, zfile) in &zone_file.reverse_zones {
                    ctx.dns_sec_tag_key(ctx.keys.rev_zones.get(rev_fqdn).unwrap());
                    write_nix_zone(
                        res,
                        prefix,
                        &mut ctx.zone_vars,
                        rev_fqdn,
                        is_master,
                        &db.projections.dns_checks.all_regions.master_lan_ip(db),
                        &db.projections.dns_checks.all_regions.slaves_lan_ips(db),
                        zfile,
                        "",
                    );
                }
            }
        }
    }

    for tld in db.db.tld().rows_iter() {
        let fqdn = format!("{}.{}.", db.db.region().c_region_name(ctx.region), db.db.tld().c_domain(tld));
        let key_for_domain = ctx.keys.regions.get(&ctx.region).unwrap();
        ctx.dns_sec_tag_key(key_for_domain);
        let is_master = true;
        // Main zone
        let zone_file = ctx.zone_files.get(&(tld, ctx.region)).unwrap();
        write_nix_zone(
            res,
            prefix,
            &mut ctx.zone_vars,
            &fqdn,
            is_master,
            &dns.master_lan_ip(db),
            &dns.slaves_lan_ips(db),
            &zone_file.tld_zone,
            "",
        );

        // Reverse zone
        for (rev_fqdn, zfile) in &zone_file.reverse_zones {
            ctx.dns_sec_tag_key(ctx.keys.rev_zones.get(rev_fqdn).unwrap());
            write_nix_zone(
                res,
                prefix,
                &mut ctx.zone_vars,
                rev_fqdn,
                is_master,
                &dns.master_lan_ip(db),
                &dns.slaves_lan_ips(db),
                &zfile,
                "",
            );
        }
    }
}

fn write_dc_slave_internal_bind_configs(db: &CheckedDB, res: &mut String, ctx: &mut BindContext) {
    let dns = db.projections.dns_checks.regions.get(&ctx.region).unwrap();
    let is_master = false;
    let prefix = "private";
    if ctx.is_single_region_setup {
        for tld in db.db.tld().rows_iter() {
            let fqdn = format!("{}.", db.db.tld().c_domain(tld));
            // Main zone
            let zone_file = ctx.private_root_zone_files.get(&tld).unwrap();
            let key_for_domain = ctx.keys.tlds.get(&tld).unwrap();
            ctx.dns_sec_tag_key(key_for_domain);
            write_nix_zone(
                res,
                prefix,
                &mut ctx.zone_vars,
                &fqdn,
                is_master,
                &db.projections.dns_checks.all_regions.master_lan_ip(db),
                &db.projections.dns_checks.all_regions.slaves_lan_ips(db),
                &zone_file.tld_zone,
                "",
            );

            // Reverse zone
            for (rev_fqdn, zfile) in &zone_file.reverse_zones {
                ctx.dns_sec_tag_key(ctx.keys.rev_zones.get(rev_fqdn).unwrap());
                write_nix_zone(
                    res,
                    prefix,
                    &mut ctx.zone_vars,
                    rev_fqdn,
                    is_master,
                    &db.projections.dns_checks.all_regions.master_lan_ip(db),
                    &db.projections.dns_checks.all_regions.slaves_lan_ips(db),
                    zfile,
                    "",
                );
            }
        }
    }

    for tld in db.db.tld().rows_iter() {
        let fqdn = format!("{}.{}.", db.db.region().c_region_name(ctx.region), db.db.tld().c_domain(tld));
        // Main zone
        let zone_file = ctx.zone_files.get(&(tld, ctx.region)).unwrap();
        let key_for_domain = ctx.keys.regions.get(&ctx.region).unwrap();
        ctx.dns_sec_tag_key(key_for_domain);
        write_nix_zone(
            res,
            prefix,
            &mut ctx.zone_vars,
            &fqdn,
            is_master,
            &dns.master_lan_ip(db),
            &dns.slaves_lan_ips(db),
            &zone_file.tld_zone,
            "",
        );

        // Reverse zone
        for (rev_fqdn, zfile) in &zone_file.reverse_zones {
            ctx.dns_sec_tag_key(ctx.keys.rev_zones.get(rev_fqdn).unwrap());
            write_nix_zone(
                res,
                prefix,
                &mut ctx.zone_vars,
                &rev_fqdn,
                is_master,
                &dns.master_lan_ip(db),
                &dns.slaves_lan_ips(db),
                zfile,
                "",
            );
        }
    }
}

// compute all keys that will be needed for the server?
fn write_nix_zone(res: &mut String, prefix: &'static str, zone_vars: &mut ZoneVars, fqdn: &str, is_master: bool, master_ip: &str, slave_ips: &[String], zone_file: &str, extra_cfg: &str) {
    assert!(!zone_file.contains("''"));
    assert!(fqdn.ends_with('.'));
    *res += "zone \"";
    *res += fqdn;
    *res += "\" {\n";
    if is_master {
        *res += "  type master;\n";
    } else {
        *res += "  type slave;\n";
    }

    let mut filename = format!("{prefix}-{fqdn}zone");
    if !is_master {
        filename += ".signed"
    }
    let target_path = format!("/run/named/{filename}");
    *res += "  file \"";
    *res += &target_path;
    *res += "\";\n";

    if is_master {
        *res += "  dnssec-policy epl;\n";
        *res += "  inline-signing yes;\n";
    }

    if is_master {
        *res += "  allow-transfer {\n";
        for slave in slave_ips {
            *res += &format!("    {slave};\n");
        }
        *res += "  };\n";
    } else {
        *res += &format!("  masters {{ {master_ip}; }};\n");
        if master_ip.starts_with("10.") {
            write!(res, "  transfer-source TRANSFER_SOURCE_LAN;\n").unwrap();
        }
    }

    *res += extra_cfg;

    *res += "};\n";

    if is_master {
        let zone_file_b64 = base64::encode(zone_file);

        zone_vars.linkings += &comment_shell_lines(zone_file);
        zone_vars.linkings += "maybe_update_dns_file ";
        zone_vars.linkings += &zone_file_b64;
        zone_vars.linkings += " ";
        zone_vars.linkings += &target_path;
        zone_vars.linkings += "\n";
    }
}

fn comment_shell_lines(input: &str) -> String {
    let mut res = String::new();
    for line in input.lines() {
        res += "#";
        if !line.is_empty() {
            res += " ";
            res += line;
        }
        res += "\n";
    }
    res
}

fn generate_root_forwarding_zone_configs(db: &CheckedDB) -> String {
    let mut forwarders = "              forwarders {\n".to_string();
    if let Some(master) = &db.projections.dns_checks.all_regions.master_server {
        forwarders += "                ";
        let iface = db.projections.consul_network_iface.value(*master);
        forwarders += db.db.network_interface().c_if_ip(*iface);
        forwarders += " port 53;\n";
    }

    for slave in &db.projections.dns_checks.all_regions.slave_servers {
        forwarders += "                ";
        let iface = db.projections.consul_network_iface.value(*slave);
        forwarders += db.db.network_interface().c_if_ip(*iface);
        forwarders += " port 53;\n";
    }

    forwarders += "              };\n";

    let mut res = String::new();
    for tld in db.db.tld().rows_iter() {
        let zone = db.db.tld().c_domain(tld);
        res += &format!(r#"
          zone "{zone}." IN {{
              type forward;
              forward only;
{forwarders}
          }};
"#);
        res += &format!(r#"
          zone "10.in-addr.arpa." IN {{
              type forward;
              forward only;
{forwarders}
          }};
"#);
    }

    res
}

fn generate_forwarding_zone_configs(db: &CheckedDB, region: TableRowPointerRegion) -> String {
    let mut res = String::new();
    let region_name = db.db.region().c_region_name(region);
    let mut forwarders = "              forwarders {\n".to_string();
    if let Some(master) = &db.projections.dns_checks.regions.get(&region).and_then(|i| i.master_server) {
        forwarders += "                ";
        let iface = db.projections.consul_network_iface.value(*master);
        forwarders += db.db.network_interface().c_if_ip(*iface);
        forwarders += " port 53;\n";
    }

    if let Some(dc_info) = db.projections.dns_checks.regions.get(&region) {
        for slave in &dc_info.slave_servers {
            forwarders += "                ";
            let iface = db.projections.consul_network_iface.value(*slave);
            forwarders += db.db.network_interface().c_if_ip(*iface);
            forwarders += " port 53;\n";
        }
    }

    forwarders += "              };\n";
    for tld in db.db.tld().rows_iter() {
        let zone = db.db.tld().c_domain(tld);
        res += &format!(r#"
          zone "{region_name}.{zone}" IN {{
              type forward;
              forward only;
{forwarders}
          }};
"#);

        // every dc has different 10. ip range
        for dc in db.db.region().c_referrers_datacenter__region(region) {
            let zone_third_octet = cidr_second_octet(db.db.datacenter().c_network_cidr(*dc));
            res += &format!(r#"
          zone "{zone_third_octet}.10.in-addr.arpa." IN {{
              type forward;
              forward only;
{forwarders}
          }};
"#);
        }
    }

    res
}

struct ZoneFiles {
    tld_zone: String,
    reverse_zones: BTreeMap<String, String>,
    // reversed ip fqdn to domain name
    reverse_zone_mappings: BTreeMap<String, String>,
}

fn dns_public_root_zone_file(db: &CheckedDB, tld: TableRowPointerTld) -> ZoneFiles {
    let mut res = String::new();

    let settings = get_global_settings(&db.db);
    let prefix = format!("{}.", db.db.tld().c_domain(tld));
    let mut ip_to_dns_map: BTreeMap<String, String> = BTreeMap::new();

    res += "$TTL 3600\n";
    res += &prefix;
    res += "\tIN\tSOA\tns1.";
    res += &prefix;
    res += " ";

    res += &prefix;
    res += " (\n";
    res += " SERIAL_TO_REPLACE ; Serial\n";
    res += " 3600 ; Refresh\n";
    res += " 1800 ; Retry\n";
    res += " 604800 ; Expire\n";
    res += " 3600 ; Minimum TTL\n";
    res += ")\n";

    let dns_servers = &db.projections.dns_checks;
    // 1. NS records for our servers
    // 2. A records for our servers
    for i in 0..dns_servers.all_regions.total_dns_servers() {
        let i = i + 1;
        res += &prefix;
        res += &format!("\tIN\tNS\tns{i}.");
        res += &prefix;
        res += "\n";
    }

    if let Some(master) = &dns_servers.all_regions.master_server {
        let fqdn = format!("ns1.{prefix}");
        let iface = db.projections.internet_network_iface.get(master)
            .expect("We assume now every root master DNS server has public ip");
        let ip = db.db.network_interface().c_if_ip(*iface);
        // Last domain wins, we can only have one PTR record
        let _ = ip_to_dns_map.insert(reverse_ip(ip), fqdn.clone());
        res += &fqdn;
        res += "\tIN\tA\t";
        res += ip;
        res += "\n";

        if settings.enable_ipv6 {
            if db.sync_res.network.node_public_ipv6_addrs.contains_key(master) {
                let ipv6 = db.db.server().c_public_ipv6_address(*master);
                res += &fqdn;
                res += "\tIN\tAAAA\t";
                res += ipv6;
                res += "\n";
            }
        }
    }

    for (idx, slave) in dns_servers.all_regions.slave_servers.iter().enumerate() {
        let idx = idx + 2;
        let fqdn = format!("ns{idx}.{prefix}");
        let iface = db.projections.internet_network_iface.get(slave)
            .expect("We assume now every root slave DNS server has public ip");
        let ip = db.db.network_interface().c_if_ip(*iface);

        // ns names override simple host ptr names
        // because ns is more important record than a hostname
        let _ = ip_to_dns_map.insert(reverse_ip(ip), fqdn.clone());
        res += &fqdn;
        res += "\tIN\tA\t";
        res += ip;
        res += "\n";

        if settings.enable_ipv6 {
            if db.sync_res.network.node_public_ipv6_addrs.contains_key(slave) {
                let ipv6 = db.db.server().c_public_ipv6_address(*slave);
                res += &fqdn;
                res += "\tIN\tAAAA\t";
                res += ipv6;
                res += "\n";
            }
        }
    }

    if let Some(ent) = db.projections.ingress_dns_entries.get(&tld) {
        for (fqdn, ips) in ent {
            for ip in &ips.ipv4 {
                let ip = ip.to_string();
                res += &fqdn;
                res += "\tIN\tA\t";
                res += &ip;
                res += "\n";
                // some ingress service PTR record is more important
                // than NS server dns records, for instance, for mail service
                let _ = ip_to_dns_map.insert(reverse_ip(&ip), fqdn.clone());
            }

            if settings.enable_ipv6 {
                for ip in &ips.ipv6 {
                    let ptr = ipv6_ptr_record(ip);
                    let ip = ip.to_string();
                    res += &fqdn;
                    res += "\tIN\tAAAA\t";
                    res += &ip;
                    res += "\n";
                    // some ingress service PTR record is more important
                    // than NS server dns records, for instance, for mail service
                    let _ = ip_to_dns_map.insert(ptr, fqdn.clone());
                }
            }
        }
    }

    ZoneFiles {
        tld_zone: res,
        // we use mappings instead of reverse zone file
        reverse_zones: BTreeMap::new(),
        reverse_zone_mappings: ip_to_dns_map,
    }
}

fn dns_private_root_zone_file(db: &CheckedDB, tld: TableRowPointerTld, keys: &AllDnsSecKeys) -> ZoneFiles {
    let mut res = String::new();

    let prefix = format!("{}.", db.db.tld().c_domain(tld));
    let mut ip_to_dns_map: BTreeMap<String, String> = BTreeMap::new();

    res += "$TTL 3600\n";
    res += &prefix;
    res += "\tIN\tSOA\tns1.";
    res += &prefix;
    res += " ";

    res += &prefix;
    res += " (\n";
    res += " SERIAL_TO_REPLACE ; Serial\n";
    res += " 3600 ; Refresh\n";
    res += " 1800 ; Retry\n";
    res += " 604800 ; Expire\n";
    res += " 3600 ; Minimum TTL\n";
    res += ")\n";

    let dns_servers = &db.projections.dns_checks;
    // 1. NS records for our servers
    // 2. A records for our servers
    for i in 0..dns_servers.all_regions.total_dns_servers() {
        let i = i + 1;
        res += &prefix;
        res += &format!("\tIN\tNS\tns{i}.");
        res += &prefix;
        res += "\n";
    }

    if let Some(master) = &dns_servers.all_regions.master_server {
        let fqdn = format!("ns1.{prefix}");
        let iface = db.projections.consul_network_iface.value(*master);
        let ip = db.db.network_interface().c_if_ip(*iface);
        // Last domain wins, we can only have one PTR record
        let _ = ip_to_dns_map.insert(reverse_ip(ip), fqdn.clone());
        res += &fqdn;
        res += "\tIN\tA\t";
        res += ip;
        res += "\n";
    }

    for (idx, slave) in dns_servers.all_regions.slave_servers.iter().enumerate() {
        let idx = idx + 2;
        let fqdn = format!("ns{idx}.{prefix}");
        let iface = db.projections.consul_network_iface.value(*slave);
        let ip = db.db.network_interface().c_if_ip(*iface);

        // ns names override simple host ptr names
        // because ns is more important record than a hostname
        let _ = ip_to_dns_map.insert(reverse_ip(ip), fqdn.clone());
        res += &fqdn;
        res += "\tIN\tA\t";
        res += ip;
        res += "\n";
    }

    for region in db.db.region().rows_iter() {
        let prefix = format!("{}.{}.", db.db.region().c_region_name(region), db.db.tld().c_domain(tld));
        let nameservers = db.projections.dns_checks.regions.get(&region).unwrap();
        let keys = keys.regions.get(&region).unwrap();
        for idx in 0..nameservers.total_dns_servers() {
            let idx = idx + 1;
            res += &prefix;
            res += &format!("\tIN\tNS\tns{idx}.");
            res += &prefix;
            res += "\n";
        }

        if let Some(master) = &nameservers.master_server {
            res += "ns1.";
            res += &prefix;
            res += "\tIN\tA\t";
            let master_if = db.projections.consul_network_iface.value(*master);
            res += db.db.network_interface().c_if_ip(*master_if);
            res += "\n";
        }

        for (idx, slave) in nameservers.slave_servers.iter().enumerate() {
            let idx = idx + 2;
            let slave_if = db.projections.consul_network_iface.value(*slave);
            let slave_ip = db.db.network_interface().c_if_ip(*slave_if);
            res += &format!("ns{idx}.{prefix}\tIN\tA\t{slave_ip}\n");
        }

        res += &keys.ksk_ds_record.replace(" IN DS ", "\tIN\tDS\t");
        res += "\n";
    }

    let mut rev = String::new();
    rev += "$TTL 3600\n";
    rev += "10.in-addr.arpa.";
    rev += "\tIN\tSOA\tns1.";
    rev += &prefix;
    rev += " ";
    rev += &prefix;
    rev += " (\n";
    rev += " SERIAL_TO_REPLACE ; Serial\n";
    rev += " 3600 ; Refresh\n";
    rev += " 1800 ; Retry\n";
    rev += " 604800 ; Expire\n";
    rev += " 3600 ; Minimum TTL\n";
    rev += ")\n";

    for i in 0..db.projections.dns_checks.all_regions.total_dns_servers() {
        let i = i + 1;
        rev += &format!("10.in-addr.arpa.\tIN\tNS\tns{i}.");
        rev += db.db.tld().c_domain(tld);
        rev += ".\n";
    }

    rev += "\n";
    // generate PTR records
    for (ip, record) in &ip_to_dns_map {
        rev += ip;
        rev += ".in-addr.arpa.\tIN\tPTR\t";
        rev += record;
        rev += "\n";
    }

    for region in db.db.region().rows_iter() {
        let dc_ns = db.projections.dns_checks.regions.get(&region).unwrap();
        let region_name = db.db.region().c_region_name(region);
        for dc in db.db.region().c_referrers_datacenter__region(region) {
            let second_octet = cidr_second_octet(db.db.datacenter().c_network_cidr(*dc));
            for idx in 0..dc_ns.total_dns_servers() {
                let idx = idx + 1;
                rev += &format!("{second_octet}.10.in-addr.arpa.\tIN\tNS\tns{idx}.{region_name}.{prefix}\n");
            }
        }
    }

    let mut reverse_zones = BTreeMap::new();
    assert!(reverse_zones.insert("10.in-addr.arpa.".to_string(), rev).is_none());


    ZoneFiles {
        tld_zone: res,
        reverse_zones,
        reverse_zone_mappings: ip_to_dns_map,
    }
}

fn dns_internal_zone_files_gen(db: &CheckedDB, tld: TableRowPointerTld, region: TableRowPointerRegion) -> ZoneFiles {
    let mut res = String::new();

    let prefix = format!("{}.{}.", db.db.region().c_region_name(region), db.db.tld().c_domain(tld));
    let mut ip_to_dns_map: BTreeMap<String, String> = BTreeMap::new();

    res += "$TTL 3600\n";
    res += &prefix;
    res += "\tIN\tSOA\tns1.";
    res += &prefix;
    res += " ";

    res += &prefix;
    res += " (\n";
    res += " SERIAL_TO_REPLACE ; Serial\n";
    res += " 3600 ; Refresh\n";
    res += " 1800 ; Retry\n";
    res += " 604800 ; Expire\n";
    res += " 3600 ; Minimum TTL\n";
    res += ")\n";

    if let Some(dns_servers) = db.projections.dns_checks.regions.get(&region) {
        for i in 0..dns_servers.total_dns_servers() {
            let i = i + 1;
            res += &prefix;
            res += &format!("\tIN\tNS\tns{i}.");
            res += &prefix;
            res += "\n";
        }

        if let Some(master) = &dns_servers.master_server {
            let fqdn = format!("ns1.{prefix}");
            let iface = db.projections.consul_network_iface.value(*master);
            let ip = db.db.network_interface().c_if_ip(*iface);
            // Last domain wins, we can only have one PTR record
            let _ = ip_to_dns_map.insert(reverse_ip(ip), fqdn.clone());
            res += &fqdn;
            res += "\tIN\tA\t";
            res += ip;
            res += "\n";
        }

        for (idx, slave) in dns_servers.slave_servers.iter().enumerate() {
            let idx = idx + 2;
            let fqdn = format!("ns{idx}.{prefix}");
            let iface = db.projections.consul_network_iface.value(*slave);
            let ip = db.db.network_interface().c_if_ip(*iface);

            // ns names override simple host ptr names
            // because ns is more important record than a hostname
            let _ = ip_to_dns_map.insert(reverse_ip(ip), fqdn.clone());
            res += &fqdn;
            res += "\tIN\tA\t";
            res += ip;
            res += "\n";
        }

        for server in db.db.server().rows_iter() {
            let dc = db.db.server().c_dc(server);
            let region = db.db.datacenter().c_region(dc);
            let this_tld = db.db.region().c_tld(region);
            if this_tld == tld {
                let iface = db.projections.consul_network_iface.value(server);
                let fqdn = format!("{}.{}", db.db.server().c_hostname(server), prefix);
                let ip = db.db.network_interface().c_if_ip(*iface);
                let rev_ip = reverse_ip(ip);
                if let Entry::Vacant(e) = ip_to_dns_map.entry(rev_ip) {
                    // Only if not more authoritative dns record
                    // exists generate PTR record
                    let _ = e.insert(fqdn.clone());
                }
                res += &fqdn;
                res += "\tIN\tA\t";
                res += ip;
                res += "\n";
            }
        }
    }

    let mut reverse_zones = BTreeMap::new();

    for dc in db.db.region().c_referrers_datacenter__region(region) {
        let second_octet = cidr_second_octet(db.db.datacenter().c_network_cidr(*dc));
        let fqdn = format!("{second_octet}.10.in-addr.arpa.");
        let mut rev = String::new();
        rev += "$TTL 3600\n";
        rev += &fqdn;
        rev += "\tIN\tSOA\tns1.";
        rev += &prefix;
        rev += " ";
        rev += &prefix;
        rev += " (\n";
        rev += " SERIAL_TO_REPLACE ; Serial\n";
        rev += " 3600 ; Refresh\n";
        rev += " 1800 ; Retry\n";
        rev += " 604800 ; Expire\n";
        rev += " 3600 ; Minimum TTL\n";
        rev += ")\n";

        for i in 0..db.projections.dns_checks.regions.get(&region).unwrap().total_dns_servers() {
            let i = i + 1;
            rev += &format!("{fqdn}\tIN\tNS\tns{i}.");
            rev += db.db.region().c_region_name(region);
            rev += ".";
            rev += db.db.tld().c_domain(tld);
            rev += ".\n";
        }

        rev += "\n";
        // generate PTR records
        let ends_with = format!("{second_octet}.10");
        for (ip, record) in &ip_to_dns_map {
            if ip.ends_with(&ends_with) {
                rev += ip;
                rev += ".in-addr.arpa.\tIN\tPTR\t";
                rev += record;
                rev += "\n";
            }
        }

        assert!(reverse_zones.insert(fqdn, rev).is_none());
    }

    ZoneFiles {
        tld_zone: res,
        reverse_zones,
        reverse_zone_mappings: ip_to_dns_map,
    }
}

pub fn ipv6_ptr_record(addr: &std::net::Ipv6Addr) -> String {
    let mut symbs: Vec<char> = Vec::with_capacity(32);
    for oct in addr.octets() {
        let left = format!("{:#x}", oct>>4);
        let right = format!("{:#x}", oct & 0x0fu8);
        symbs.push('.');
        // skip 0x in front
        symbs.push(left.as_bytes()[2] as char);
        symbs.push('.');
        symbs.push(right.as_bytes()[2] as char);
    }
    symbs.reverse();

    let prefix: String = symbs.iter().collect();
    format!("{prefix}ip6.arpa.")
}

#[test]
fn test_ipv6_arpa_record_generation() {
    use std::str::FromStr;

    assert_eq!(
        ipv6_ptr_record(&std::net::Ipv6Addr::from_str("2a03:2880:f32e:3:face:b00c:0:25de").unwrap()),
        "e.d.5.2.0.0.0.0.c.0.0.b.e.c.a.f.3.0.0.0.e.2.3.f.0.8.8.2.3.0.a.2.ip6.arpa."
    );
}

pub fn reverse_ip(input: &str) -> String {
    let mut res = input.split('.').collect::<Vec<_>>();
    assert_eq!(res.len(), 4);
    res.reverse();
    res.join(".")
}

#[test]
fn test_reverse_ip() {
    assert_eq!(reverse_ip("123.4.56.78"), "78.56.4.123".to_string());
}

// 10.11.12.13/16 -> 11
// we assume only valid ips can end up in this function
// as all the ip checks passed before calling this
fn cidr_second_octet(input: &str) -> String {
    let spl = input.split('/').collect::<Vec<_>>();
    let spl = spl[0].split('.').collect::<Vec<_>>();
    spl[1].to_string()
}

#[test]
fn test_cidr_second_octet() {
    assert_eq!(cidr_second_octet("123.4.56.78/32"), "4");
}
