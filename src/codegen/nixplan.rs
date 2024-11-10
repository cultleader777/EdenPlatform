use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::database::TableRowPointerServer;

use super::secrets::SecretValue;

pub struct NixSecretKey {
    key_name: String,
    contents: SecretValue,
    user: String,
    group: String,
    mode: &'static str,
}

pub struct NixSecretConfig {
    key_name: String,
    contents: String,
    user: String,
    group: String,
    mode: &'static str,
}

impl NixSecretKey {
    pub fn absolute_path(&self) -> String {
        format!("/run/keys/{}", self.key_name)
    }
}

impl NixSecretConfig {
    pub fn absolute_path(&self) -> String {
        format!("/run/keys/{}", self.key_name)
    }
}

pub fn root_secret_key(key_name: String, contents: SecretValue) -> NixSecretKey {
    NixSecretKey {
        key_name,
        contents,
        user: "root".to_string(),
        group: "root".to_string(),
        mode: "0600",
    }
}

pub fn custom_user_secret_key(
    user_name: String,
    key_name: String,
    contents: SecretValue,
) -> NixSecretKey {
    NixSecretKey {
        key_name,
        contents,
        user: user_name.clone(),
        group: user_name,
        mode: "0600",
    }
}

pub fn all_users_readable_key(key_name: String, contents: SecretValue) -> NixSecretKey {
    NixSecretKey {
        key_name,
        contents,
        user: "root".to_string(),
        group: "root".to_string(),
        mode: "0644",
    }
}

pub fn root_secret_config(key_name: String, contents: String) -> NixSecretConfig {
    NixSecretConfig {
        key_name,
        contents,
        user: "root".to_string(),
        group: "root".to_string(),
        mode: "0600",
    }
}

pub fn custom_user_secret_config(
    user_name: String,
    key_name: String,
    contents: String,
) -> NixSecretConfig {
    NixSecretConfig {
        key_name,
        contents,
        user: user_name.clone(),
        group: user_name,
        mode: "0600",
    }
}

pub struct NomadHostVolume {
    pub mountpoint: String,
    pub name: String,
    pub read_only: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NixServerFeatures {
    Nftables,
}

pub struct NixServerPlan {
    secrets: Vec<NixSecretKey>,
    secret_configs: Vec<NixSecretConfig>,
    custom_nix_blocks: Vec<String>,
    variables: BTreeMap<String, String>,
    environment_variables: BTreeMap<String, String>,
    ca_certificates: BTreeSet<String>,
    shell_packages: BTreeMap<String, String>,
    // for values like ssl certs
    nomad_host_volumes: BTreeMap<String, NomadHostVolume>,
    nix_packages: BTreeSet<String>,
    server_features: BTreeSet<NixServerFeatures>,
    interface_routes: BTreeMap<String, Vec<String>>,
    post_second_round_secrets_shell_hooks: Vec<String>,
    pre_l1_provisioning_shell_hooks: Vec<String>,
    zfs_datasets: BTreeMap<String, ZfsDataset>,
}

pub struct ZfsDataset {
    pub zpool: String,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub mountpoint: String,
    pub expose_to_containers: bool,
    pub record_size: String,
}

impl NixServerPlan {
    fn new() -> Self {
        NixServerPlan {
            secrets: Default::default(),
            secret_configs: Default::default(),
            custom_nix_blocks: Default::default(),
            variables: Default::default(),
            environment_variables: Default::default(),
            ca_certificates: Default::default(),
            shell_packages: Default::default(),
            nomad_host_volumes: Default::default(),
            nix_packages: Default::default(),
            server_features: Default::default(),
            interface_routes: Default::default(),
            post_second_round_secrets_shell_hooks: Default::default(),
            pre_l1_provisioning_shell_hooks: Default::default(),
            zfs_datasets: Default::default(),
        }
    }

    pub fn add_zfs_dataset(&mut self, dataset_name: String, params: ZfsDataset) {
        assert!(self.zfs_datasets.insert(dataset_name, params).is_none(), "Found duplicate zfs dataset name");
    }

    pub fn zfs_datasets(&self) -> &BTreeMap<String, ZfsDataset> {
        &self.zfs_datasets
    }

    pub fn secret_configs(&self) -> &[NixSecretConfig] {
        &self.secret_configs
    }

    pub fn add_interface_route(&mut self, interface: &str, route_nix_expression: String) {
        let routes = self.interface_routes.entry(interface.to_string()).or_default();
        routes.push(route_nix_expression)
    }

    pub fn add_env_variable(&mut self, var_name: String, var_value: String) {
        assert!(self
            .environment_variables
            .insert(var_name, var_value)
            .is_none());
    }

    pub fn add_nix_package(&mut self, package_name: &'static str) {
        assert!(self.nix_packages.insert(package_name.to_string()));
    }

    pub fn add_ca_cert_file(&mut self, cert_contents: String) {
        assert!(self.ca_certificates.insert(cert_contents));
    }

    pub fn add_shell_package(&mut self, package_name: &str, package_source: &str) {
        assert!(self
            .shell_packages
            .insert(package_name.to_string(), package_source.to_string())
            .is_none());
    }

    pub fn add_secret(&mut self, secret: NixSecretKey) -> &NixSecretKey {
        assert!(
            !secret.contents.value().contains("''"),
            "secret contains nix string boundary characters ''"
        );

        for i in &self.secrets {
            assert_ne!(
                i.key_name, secret.key_name,
                "Duplicate secrets in server plan detected"
            )
        }

        let key = format!("secret_{}", secret.key_name);
        assert!(
            !self.variables.contains_key(&key),
            "Found duplicate variables for server"
        );

        self.variables.insert(key, secret.contents.value().clone());
        self.secrets.push(secret);

        self.secrets.last().unwrap()
    }

    pub fn add_secret_config(&mut self, secret: NixSecretConfig) -> &NixSecretConfig {
        assert!(
            !secret.contents.contains("''"),
            "secret contains nix string boundary characters ''"
        );

        for i in &self.secret_configs {
            assert_ne!(
                i.key_name, secret.key_name,
                "Duplicate secret configs in server plan detected"
            )
        }

        let key = format!("seccfg_{}", secret.key_name);
        assert!(
            !self.variables.contains_key(&key),
            "Found duplicate variables for server"
        );

        self.variables.insert(key, secret.contents.clone());
        self.secret_configs.push(secret);

        self.secret_configs.last().unwrap()
    }

    /// Script to be run after initial secrets ran, nixos-rebuild-switch ran
    /// and another round of secrets with created users ran
    ///
    /// Say, bind is installed a first time. It has owned folder named:named
    /// First secrets load doesn't work because named:named users don't exist yet
    /// After NixOS rebuild switch then second secret load runs which is executed
    /// successfully, but bind is not reloaded is master zone is therefore empty.
    /// In this hook we add to check if zone files are loaded and if not restart
    /// bind service
    pub fn add_post_second_round_secrets_shell_hook(&mut self, script: String) {
        self.post_second_round_secrets_shell_hooks.push(script);
    }

    /// Provision zfs datasets and make mountpoints ready
    pub fn add_pre_l1_provisioning_shell_hook(&mut self, script: String) {
        self.pre_l1_provisioning_shell_hooks.push(script);
    }

    fn ensure_valid_nomad_volume(input: &str) {
        lazy_static! {
            static ref VALID_NOMAD_VOLUME: regex::Regex =
                regex::Regex::new(r"^[a-z0-9_-]+$").unwrap();
        }

        assert!(
            VALID_NOMAD_VOLUME.is_match(input),
            "Invalid nomad host volume name {input}"
        );
    }

    /// Creates a volume out of existing secrets and exposes it to Nomad
    pub fn create_secrets_nomad_volume(&mut self, vol_name: &str, conf: &[String]) {
        Self::ensure_valid_nomad_volume(vol_name);

        let abs_path = format!("/run/sec_volumes/{vol_name}");
        let mut b = String::new();
        b += &format!(
            r#"
mkdir -m 700 -p {abs_path}
"#
        );

        for c in conf {
            b += &format!("cp {} {}/\n", c, abs_path);
        }

        let res = self.nomad_host_volumes.insert(
            vol_name.to_string(),
            NomadHostVolume {
                mountpoint: abs_path,
                name: vol_name.to_string(),
                read_only: true,
            },
        );
        assert!(
            res.is_none(),
            "System volumes must not conflict with each other"
        );

        self.add_pre_l1_provisioning_shell_hook(b.clone());
        self.add_post_second_round_secrets_shell_hook(b);
    }

    pub fn add_nomad_host_volume(&mut self, vol_name: &str, absolute_path: &str, read_only: bool) {
        Self::ensure_valid_nomad_volume(vol_name);

        let res = self.nomad_host_volumes.insert(
            vol_name.to_string(),
            NomadHostVolume {
                mountpoint: absolute_path.to_string(),
                name: vol_name.to_string(),
                read_only,
            },
        );
        // TODO: nice bubbling up error if user using disallowed volumes
        assert!(
            res.is_none(),
            "Duplicate nomad host volume name {vol_name} with path {absolute_path}"
        );
    }

    pub fn nomad_host_volumes(&self) -> &BTreeMap<String, NomadHostVolume> {
        &self.nomad_host_volumes
    }

    pub fn add_custom_nix_block(&mut self, input: String) {
        self.custom_nix_blocks.push(input);
    }

    pub fn add_server_feature(&mut self, feature: NixServerFeatures) {
        if self.server_features.insert(feature) {
            let block = match feature {
                NixServerFeatures::Nftables => {
                    r#"
    networking.nftables.enable = true;
"#
                }
            };
            self.add_custom_nix_block(block.to_owned());
        }
    }

    fn dump_custom_nix_blocks(&self, output: &mut String) {
        for block in &self.custom_nix_blocks {
            *output += block;
        }
    }

    fn dump_nix_routes(&self, output: &mut String) {
        if self.interface_routes.is_empty() {
            return;
        }

        let region = "static_node_routes";
        write_nix_region_start(region, output);
        for (ifname, routes) in &self.interface_routes {
            if ifname.contains(":") {
                // subinterface
                continue;
            }

            write!(output, r#"
    networking.interfaces."{ifname}".ipv4.routes = [
"#).unwrap();

            for route in routes {
                write!(output, r#"
      {route}
"#).unwrap();
            }

            for (child_ifname, child_routes) in &self.interface_routes {
                if child_ifname.starts_with(child_ifname) && child_ifname.contains(":") {
                    for route in child_routes {
                        write!(output, r#"
      {route}
"#).unwrap();
                    }
                }
            }

            write!(output, r#"
    ];
"#).unwrap();
        }
        write_nix_region_end(region, output);
    }

    pub fn dump_nix_blocks(&self, output: &mut String) {
        self.dump_nix_env_vars(output);
        self.dump_nix_ca_certs(output);
        self.dump_nix_shell_packages(output);
        self.dump_nix_routes(output);
        self.dump_custom_nix_blocks(output);
    }

    fn dump_nix_shell_packages(&self, output: &mut String) {
        if !self.shell_packages.is_empty() {
            output.push_str(
                r#"
    environment.systemPackages =
      let
"#,
            );

            for (k, v) in &self.shell_packages {
                output.push_str("        ");
                output.push_str(k);
                output.push_str(" = pkgs.writeShellScriptBin \"");
                output.push_str(k.as_str());
                output.push_str("\" ''\n");
                output.push_str(v.as_str());
                output.push_str("\n        '';\n");
            }

            output.push_str(
                r#"
      in
      [
"#,
            );
            for p in &self.nix_packages {
                output.push_str("        pkgs.");
                output.push_str(p);
                output.push('\n');
            }

            for k in self.shell_packages.keys() {
                output.push_str("        ");
                output.push_str(k);
                output.push('\n');
            }

            output.push_str(
                r#"      ];
"#,
            );
        }
    }

    fn dump_nix_ca_certs(&self, output: &mut String) {
        if !self.ca_certificates.is_empty() {
            output.push_str(
                r#"
    security.pki.certificates = [
"#,
            );
            for v in &self.ca_certificates {
                output.push_str(&format!(
                    r#"      ''{v}''
"#
                ));
            }

            output.push_str(
                r#"    ];
"#,
            );
        }
    }

    fn dump_nix_env_vars(&self, output: &mut String) {
        if !self.environment_variables.is_empty() {
            output.push_str(
                r#"
    environment.sessionVariables = {
"#,
            );
            for (k, v) in &self.environment_variables {
                output.push_str(&format!(
                    r#"      {k} = "{v}";
"#
                ));
            }

            output.push_str(
                r#"    };
"#,
            );
        }
    }
}

pub struct NixAllServerPlans {
    plans: BTreeMap<TableRowPointerServer, NixServerPlan>,
}

impl NixAllServerPlans {
    pub fn new() -> Self {
        Self {
            plans: BTreeMap::new(),
        }
    }

    pub fn fetch_plan(
        &mut self,
        server: TableRowPointerServer,
    ) -> &mut NixServerPlan {
        self.plans
            .entry(server)
            .or_insert_with(|| NixServerPlan::new())
    }

    pub fn fetch_plan_ro(
        &self,
        server: TableRowPointerServer,
    ) -> &NixServerPlan {
        self.plans
            .get(&server)
            .unwrap()
    }
}

const DELIM_0: &'static str = "ThisIsEplProvL1Script";
const DELIM: &'static str = "LilBoiPeepLikesBenzTruck";

pub fn generate_l1_provisioning_script(gw_route: Option<&str>, nix_cfg: &str, server: TableRowPointerServer, plans: &NixAllServerPlans, nixos_version: &str) -> String {
    let mut output = String::new();

    let plan = plans.fetch_plan_ro(server);

    let l1_prov_path = "/var/lib/epl-l1-prov";
    let l1_log_path = "/var/log/epl-l1-prov";
    let sqlite_db_path = format!("{l1_prov_path}/provisionings.sqlite");
    let l1_script = generate_l1_background_script(nix_cfg, plan, &sqlite_db_path, nixos_version);

    writeln!(&mut output, "#!/bin/sh").unwrap();
    // make created files only rw by root user,
    // if we need to relax permissions do it after
    // its created with root only
    writeln!(output, "umask 0077").unwrap();

    writeln!(output, "mkdir -p {l1_prov_path}").unwrap();
    writeln!(output, "mkdir -p {l1_log_path}").unwrap();
    // delete provisioning logs older than 7 days
    writeln!(output, "find {l1_log_path}/*.log -type f -ctime +7 -exec rm -rf {{}} \\; || true").unwrap();
    // allow vector to read/write/delete logs
    writeln!(output, "grep -q epl-prov /etc/group && chgrp epl-prov /var/log/epl-l1-prov || true").unwrap();
    writeln!(output, "chmod 750 /var/log/epl-l1-prov").unwrap();
    writeln!(output, "echo '

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

' | sqlite3 {sqlite_db_path}").unwrap();

    writeln!(output, r#"
HIGHER_ID=$( echo "
    SELECT provisioning_id, 'found_higher'
    FROM l1_provisionings
    WHERE provisioning_id > L1_EPL_PROVISIONING_ID
    LIMIT 1
" | sqlite3 -csv {sqlite_db_path} | grep 'found_higher' || true )

if [ -n "$HIGHER_ID" ]
then
    HIGHER_ID_CUT=$( echo "$HIGHER_ID" | awk -F, '{{print $1}}' )
    echo "This provisioning id is L1_EPL_PROVISIONING_ID, found higher l1 provisioning id $HIGHER_ID_CUT in the database, refusing to evaluate"
    exit 7
fi
"#).unwrap();

    if let Some(gw_route) = gw_route {
        output += gw_route;
    }

    writeln!(output, "cat > /run/epl-l1-prov <<'{DELIM_0}'").unwrap();
    assert!(!l1_script.contains(DELIM_0), "Bruh...");
    writeln!(output, "{l1_script}").unwrap();
    writeln!(output, "{DELIM_0}").unwrap();
    writeln!(output, "chmod 700 /run/epl-l1-prov").unwrap();
    // not perfectly secure and race condition exists between select and insert, but will catch 99% of the cases, may do it better in the future
    writeln!(output, "echo \"SELECT 'running provisioning id is unfinished', provisioning_id FROM l1_provisionings WHERE is_finished = 0;\" | sqlite3 {sqlite_db_path} | grep unfinished && exit 27 || true").unwrap();
    writeln!(output, "echo 'INSERT INTO l1_provisionings(provisioning_id) VALUES (L1_EPL_PROVISIONING_ID);' | sqlite3 {sqlite_db_path}").unwrap();
    writeln!(output, "tmux new-session -d '/run/epl-l1-prov |& tee /var/log/epl-l1-prov/L1_EPL_PROVISIONING_ID.log'").unwrap();

    output
}

fn generate_l1_background_script(nix_cfg: &str, plan: &NixServerPlan, sqlite_db_path: &str, nixos_version: &str) -> String {
    let mut output = String::new();
    writeln!(&mut output, "#!/bin/sh").unwrap();
    writeln!(output, "set -e").unwrap();
    writeln!(output, "function epl_l1_track_state {{
    EPL_PROV_EXIT_CODE=$1

    if [ -d /var/lib/node_exporter ]
    then
        echo \"
epl_l1_provisioning_id L1_EPL_PROVISIONING_ID
epl_l1_provisioning_status $EPL_PROV_EXIT_CODE
\"      > /var/lib/node_exporter/epl_l1_provisioning.prom.tmp
        chmod 644 /var/lib/node_exporter/epl_l1_provisioning.prom.tmp
        mv -f /var/lib/node_exporter/epl_l1_provisioning.prom.tmp \
              /var/lib/node_exporter/epl_l1_provisioning.prom
    fi
}}").unwrap();
    writeln!(output, "function trap_exit {{
    EXIT_CODE=$?
    rm -f /run/epl-l1-provisioning.lock

    epl_l1_track_state $EXIT_CODE

    echo \"
      UPDATE l1_provisionings
      SET exit_code = $EXIT_CODE,
          time_ended = CURRENT_TIMESTAMP,
          is_finished = 1
      WHERE provisioning_id = L1_EPL_PROVISIONING_ID
    \" | sqlite3 {sqlite_db_path}
}}").unwrap();
    generate_l1_secrets_function(&mut output, plan);
    writeln!(output, "trap trap_exit ERR").unwrap();
    // running
    writeln!(output, "
# -1 means in progress
epl_l1_track_state -1
").unwrap();
    // l1 provisioning lock, if lockfile program exists
    writeln!(output, "which lockfile && lockfile /run/epl-l1-provisioning.lock || true").unwrap();
    // make created files only rw by root user,
    // if we need to relax permissions do it after
    // its created with root only
    writeln!(output, "umask 0077").unwrap();
    // if server rebooted we can instaniate all secrets because users are already created
    writeln!(output, "generate_l1_secrets").unwrap();
    for shell_hook in &plan.pre_l1_provisioning_shell_hooks {
        writeln!(output, "{shell_hook}").unwrap();
    }
    generate_l1_configuration_nix(&mut output, nix_cfg, nixos_version);
    // here we instantiate secrets again because this might be the very first time to provision
    // and users don't exist yet, so we need to provision once to create users and then create
    // secrets
    writeln!(output, "generate_l1_secrets").unwrap();
    writeln!(output, "L1_RESTART_CONSUL_POST_SECRETS && echo restarting consul after sleeping 10 seconds... && sleep 10 && systemctl restart consul.service || true").unwrap();
    for shell_hook in &plan.post_second_round_secrets_shell_hooks {
        writeln!(output, "{shell_hook}").unwrap();
    }
    writeln!(output, "rm -f /run/epl-l1-prov").unwrap();
    writeln!(output, "rm -f /run/epl-l1-provisioning.lock").unwrap();
    writeln!(output, "
echo \"
    UPDATE l1_provisionings
    SET exit_code = 0,
        time_ended = CURRENT_TIMESTAMP,
        is_finished = 1
    WHERE provisioning_id = L1_EPL_PROVISIONING_ID
\" | sqlite3 {sqlite_db_path}
").unwrap();
    // running
    writeln!(output, "
epl_l1_track_state 0
").unwrap();

    // allow reading log file by vector
    writeln!(output, "chmod 644 /var/log/epl-l1-prov/L1_EPL_PROVISIONING_ID.log").unwrap();

    output
}

fn generate_backup_initial_nix_config(output: &mut String) {
    // backup initial nix config, useful for bare metal setups
    writeln!(output, "if ! cat /etc/nixos/configuration.nix | head -n 2 | grep '# EDEN PLATFORM GENERATED NIX CONFIG'").unwrap();
    writeln!(output, "then").unwrap();
    writeln!(output, "  mv -f /etc/nixos/configuration.nix /etc/nixos/configuration-initial.nix || true").unwrap();
    writeln!(output, "fi").unwrap();
}

fn generate_automatic_nixos_upgrade(output: &mut String, nixos_version: &str) {
    writeln!(output, "if ! nixos-version | grep -E \'^{nixos_version}'").unwrap();
    writeln!(output, "then").unwrap();
    writeln!(output, "  nix-channel --add https://channels.nixos.org/nixos-{nixos_version} nixos").unwrap();
    writeln!(output, "  nix-channel --update nixos").unwrap();
    writeln!(output, "fi").unwrap();
}

fn generate_l1_configuration_nix(output: &mut String, nix_cfg: &str, nixos_version: &str) {
    writeln!(output, "mkdir -p /etc/nixos").unwrap();
    writeln!(output, "pushd /etc/nixos").unwrap();
    // we're not SJW fags here
    writeln!(output, "git config --global init.defaultBranch master").unwrap();
    writeln!(output, "git config --global user.name 'EPL L1 provisioner'").unwrap();
    writeln!(output, "git config --global user.email 'epl@example.com'").unwrap();
    writeln!(output, "git init").unwrap();
    generate_backup_initial_nix_config(output);
    writeln!(output, "cat > /etc/nixos/configuration.nix <<'{DELIM}'").unwrap();
    assert!(!nix_cfg.contains(DELIM), "You gotta be kidding me...");
    *output += nix_cfg;
    writeln!(output, "").unwrap();
    writeln!(output, "{DELIM}").unwrap();
    writeln!(output, "echo L1_EPL_PROVISIONING_ID > /etc/nixos/epl-prov-id").unwrap();
    writeln!(output, "chown root:root /etc/nixos/configuration.nix").unwrap();
    writeln!(output, "chmod 0600 /etc/nixos/configuration.nix").unwrap();
    writeln!(output, "git add .").unwrap();
    writeln!(output, "git commit -am 'Update L1_EPL_PROVISIONING_ID' || true").unwrap();
    writeln!(output, "popd").unwrap();
    generate_automatic_nixos_upgrade(output, nixos_version);
    writeln!(output, "nixos-rebuild switch || L1_PROVISIONING_TOLERATE_REBUILD_FAIL").unwrap();
}

fn generate_l1_secrets_function(output: &mut String, plan: &NixServerPlan) {
    // might be leftovers
    writeln!(output, "function generate_l1_secrets() {{").unwrap();

    writeln!(output, "rm -f /run/tmpsec-*").unwrap();
    writeln!(output, "mkdir -p /run/keys").unwrap();
    writeln!(output, "chmod 755 /run/keys").unwrap();

    for sec_cfg in plan.secret_configs() {
        generate_single_secret_write(output, &sec_cfg.key_name, &sec_cfg.contents, &sec_cfg.user, &sec_cfg.group, sec_cfg.mode);
    }
    for sec in &plan.secrets {
        generate_single_secret_write(output, &sec.key_name, sec.contents.value(), &sec.user, &sec.group, sec.mode);
    }

    writeln!(output, "}}").unwrap();
}

fn generate_single_secret_write(output: &mut String, name: &str, secret: &str, user: &str, group: &str, mode: &str) {
    let region_name = format!("secret_value_{name}");
    assert!(!secret.contains(DELIM), "You gotta be kidding me...");
    writeln!(output, "TMP_SECRET_PATH=/run/tmpsec-$RANDOM").unwrap();
    let sec_path = format!("/run/keys/{name}");
    write_nix_region_start(region_name.as_str(), output);
    writeln!(output, "cat > $TMP_SECRET_PATH <<'{DELIM}'").unwrap();
    *output += secret;
    if !output.ends_with("\n") {
        writeln!(output, "").unwrap();
    }
    writeln!(output, "{DELIM}").unwrap();
    write_nix_region_end(region_name.as_str(), output);
    writeln!(output, "if id -u {user} &>/dev/null && id -g {group} &>/dev/null; then").unwrap();
    if user != "root" {
        writeln!(output, "  chown {user} $TMP_SECRET_PATH").unwrap();
    }
    if group != "root" {
        writeln!(output, "  chgrp {group} $TMP_SECRET_PATH").unwrap();
    }
    if mode != "0600" {
        writeln!(output, "  chmod {mode} $TMP_SECRET_PATH").unwrap();
    }
    writeln!(output, "  unset NEEDS_MOVE").unwrap();
    writeln!(output, "  cmp --silent $TMP_SECRET_PATH {sec_path} || NEEDS_MOVE=true").unwrap();
    writeln!(output, "  [ \"$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)\" == \"$(stat -c '%A:%U:%G' '{sec_path}')\" ] || NEEDS_MOVE=true").unwrap();
    writeln!(output, "  [ -n \"$NEEDS_MOVE\" ] && mv -f $TMP_SECRET_PATH {sec_path}").unwrap();
    writeln!(output, "fi").unwrap();
    writeln!(output, "rm -f $TMP_SECRET_PATH || true").unwrap();
}

pub fn write_nix_region_start(region: &str, output: &mut String) {
    assert_eq!(region.trim(), region);
    write!(output, "
# NIX REGION {region} START
").unwrap();
}

pub fn write_nix_region_end(region: &str, output: &mut String) {
    assert_eq!(region.trim(), region);
    write!(output, "
# NIX REGION {region} END
").unwrap()
}

pub fn mk_nix_region(region: &str, input: String) -> String {
    let mut res = String::new();
    write_nix_region_start(region, &mut res);
    res.push_str(&input);
    write_nix_region_end(region, &mut res);
    res
}

#[cfg(test)]
pub fn nix_strip_for_diff(input: &str) -> String {
    input.lines().filter_map(|i| {
        let trimmed = i.trim();
        if !trimmed.is_empty() && !trimmed.contains(DELIM) {
            Some(trimmed)
        } else { None }
    }).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
pub fn find_nix_region<'a>(region: &str, input: &'a str) -> Option<&'a str> {
    let mut start = String::new();
    write_nix_region_start(region, &mut start);
    let mut end = String::new();
    write_nix_region_end(region, &mut end);
    match (input.find(&start), input.find(&end)) {
        (Some(start_idx), Some(end_idx)) => {
            let start_idx_adjusted = start_idx + start.len();
            Some(&input[start_idx_adjusted..end_idx])
        }
        _ => { None }
    }
}

#[test]
fn test_find_nix_region() {
    let region = "asuka_is_the_best_girl";
    let string_to_wrap = r#"

hey ho

here she goes

"#;

    let wrapped = mk_nix_region("asuka_is_the_best_girl", string_to_wrap.to_string());
    assert!(wrapped.contains("# NIX REGION asuka_is_the_best_girl START"));
    assert!(wrapped.contains("# NIX REGION asuka_is_the_best_girl END"));

    let found = find_nix_region(region, &wrapped);
    assert!(found.is_some());
    assert_eq!(found.unwrap(), string_to_wrap);
}

#[test]
fn test_nix_strip_for_diffing() {
    assert_eq!(nix_strip_for_diff(r#"

hello
  this is some

white space
for x in items {

}
 configs
"#), r#"hello
this is some
white space
for x in items {
}
configs"#);
}

#[test]
fn test_nix_strip_for_diffing_with_delim() {
    assert_eq!(nix_strip_for_diff(r#"

hello
  this is some
LilBoiPeepLikesBenzTruck

white space
for x in items {

}
LilBoiPeepLikesBenzTruck
 configs
"#), r#"hello
this is some
white space
for x in items {
}
configs"#);
}
