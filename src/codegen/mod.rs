pub mod infra;
pub mod nixplan;
pub mod l1_provisioning;
pub mod rust;
pub mod secrets;
pub mod makefile;
pub mod integration_tests;
pub mod terraform;
pub mod preconditions;

use std::{
    collections::{HashSet, BTreeSet, BTreeMap},
    io::Write,
    os::unix::prelude::PermissionsExt,
    path::Path,
};

use regex::Regex;
use sodiumoxide::crypto::hash::sha256;

use crate::{database::{TableRowPointerRegion, TableRowPointerServer}, static_analysis::{CheckedDB, networking::first_three_octets, get_global_settings}, codegen::{infra::configure_server_ip_interfaces, nixplan::generate_l1_provisioning_script, l1_provisioning::utils::epl_arch_to_linux_arch, preconditions::generate_server_preconditions}, prom_metrics_dump::metrics_db_schema};

use self::{secrets::{SecretsStorage, SecretValue, sec_files, SecretKind}, terraform::generate_terraform_outputs, makefile::vms_exist, nixplan::NixAllServerPlans, l1_provisioning::wireguard::{generate_admin_vpn_config, WgSecrets, generate_wg_secrets}};

pub struct File {
    contents: Vec<u8>,
    name: String,
    if_not_exists: bool,
    always_overwrite: bool,
    permissions: &'static str,
    condition: Option<SpecialFileCreationCondition>,
}

pub struct Directory {
    directories: Vec<(String, Directory)>,
    files: Vec<File>,
    name_registry: HashSet<String>,
}

pub struct CodegenPlan {
    root_dir: Directory,
}

impl CodegenPlan {
    fn root_dir(&mut self) -> &mut Directory {
        &mut self.root_dir
    }

    fn new() -> CodegenPlan {
        CodegenPlan {
            root_dir: Directory::new(),
        }
    }
}

lazy_static! {
    static ref FILE_DIR_REGEX: Regex = Regex::new("^[A-Za-z0-9_\\.-]+$").unwrap();
}

pub enum SpecialFileCreationCondition {
    DontCreateIfDirectoryExists(String)
}

impl Directory {
    fn create_file(&mut self, name: &str, contents: String) {
        self.create_file_custom(name, contents.into_bytes(), false, "644", false, None);
    }

    /// Yeah function name is ugly but its explicit in what it does
    #[allow(dead_code)]
    fn create_file_binary_always_overwrite(&mut self, name: &str, contents: Vec<u8>) {
        self.create_file_custom(name, contents, false, "644", true, None);
    }

    fn create_executable_file(&mut self, name: &str, contents: String) {
        self.create_file_custom(name, contents.into_bytes(), false, "755", false, None);
    }

    fn create_file_if_not_exists(&mut self, name: &str, contents: String) {
        self.create_file_custom(name, contents.into_bytes(), true, "644", false, None);
    }

    fn create_file_if_not_exists_condition(&mut self, name: &str, contents: String, cond: SpecialFileCreationCondition) {
        self.create_file_custom(name, contents.into_bytes(), true, "644", false, Some(cond));
    }

    fn create_file_custom(
        &mut self,
        name: &str,
        contents: Vec<u8>,
        if_not_exists: bool,
        permissions: &'static str,
        always_overwrite: bool,
        condition: Option<SpecialFileCreationCondition>,
    ) {
        assert!(
            FILE_DIR_REGEX.is_match(name),
            "File name regex doesn't match"
        );
        assert!(!self.name_registry.contains(name), "Name taken");

        let _ = self.name_registry.insert(name.to_string());

        self.files.push(File {
            name: name.to_string(),
            contents,
            if_not_exists,
            always_overwrite,
            permissions,
            condition,
        });
    }

    fn create_directory(&mut self, name: &str) -> &mut Directory {
        assert!(
            FILE_DIR_REGEX.is_match(name),
            "Dir name regex doesn't match"
        );
        assert!(!self.name_registry.contains(name), "Name taken");

        let _ = self.name_registry.insert(name.to_string());

        self.directories.push((
            name.to_string(),
            Directory {
                directories: vec![],
                files: vec![],
                name_registry: HashSet::new(),
            },
        ));

        &mut self.directories.last_mut().unwrap().1
    }

    fn new() -> Directory {
        Directory {
            directories: vec![],
            files: vec![],
            name_registry: HashSet::new(),
        }
    }
}

pub fn generate_outputs(checked: &CheckedDB, secrets: &mut SecretsStorage) -> CodegenPlan {
    let mut plan = CodegenPlan::new();

    let project_settings = get_global_settings(&checked.db);
    let root_ssh_keys = generate_ssh_root_key_secrets(secrets);
    let nix_cache_keys = generate_nix_cache_keys(secrets);
    let wg_secrets = generate_wg_secrets(checked, secrets);
    let admin_panel_htpasswd_file = generate_admin_panel_htpasswd_file(secrets);
    let fast_prov_secrets = l1_provisioning::fast_l1::generate_fast_prov_secrets(checked, secrets);

    let cgen_secrets = CodegenSecrets {
        root_ssh_keys,
        nix_cache_keys,
        wg_secrets,
        admin_panel_htpasswd_file,
        fast_prov_secrets,
    };

    let mut plans = nixplan::NixAllServerPlans::new();
    l1_provisioning::provision_servers(checked, &mut plans, secrets, &cgen_secrets);

    generate_applications(checked, &mut plan);
    generate_prometheus_tests(checked, &mut plan);
    let l1_outputs = generate_machines(checked, &mut plan, &plans, &cgen_secrets);
    makefile::generate_makefile(checked, &mut plan, &l1_outputs);
    if checked.projections.cloud_topologies.cloud_needed() {
        generate_terraform_outputs(checked, &cgen_secrets.root_ssh_keys, &mut plan);
    }

    // for every release docker cache is separate from env
    if !cfg!(debug_assertions) {
        generate_docker_cache_dir(plan.root_dir(), project_settings.local_docker_cache_port);
    }

    plan
}

fn generate_docker_cache_dir(root_dir: &mut Directory, port: i64) {
    let dir = root_dir.create_directory("docker-cache");
    dir.create_file("config.yml", format!(r#"
version: 0.1
storage:
  filesystem:
    rootdirectory: /var/lib/registry
proxy:
  remoteurl: https://registry-1.docker.io
http:
  addr: 0.0.0.0:{port}
"#))
}

fn generate_admin_panel_htpasswd_file(secrets: &mut SecretsStorage) -> SecretValue {
    let admin_panel_password = secrets.fetch_secret("admin_panel_password".to_string(), SecretKind::StrongPassword42Symbols);
    let admin_panel_htpasswd_secret_key = "admin_panel_htpasswd_file".to_string();
    if let Some(file) = secrets.get_secret(&admin_panel_htpasswd_secret_key) {
        file
    } else {
        let mut htpasswd_file = passivized_htpasswd::Htpasswd::new();
        htpasswd_file.set("admin", admin_panel_password.to_string()).expect("Should work");
        secrets.put_secret(
            admin_panel_htpasswd_secret_key.to_string(),
            SecretKind::HtpasswdFile,
            htpasswd_file.to_string()
        );
        secrets.get_secret(&admin_panel_htpasswd_secret_key).unwrap()
    }
}

#[cfg(test)]
pub fn find_server_config_region<'a>(server: &str, plan: &'a CodegenPlan) -> String {
    for (dname, files) in &plan.root_dir.directories {
        if dname == "l1-provisioning" {
            for (dname, files) in &files.directories {
                if dname == server {
                    for file in &files.files {
                        if file.name == "provision.sh" {
                            return String::from_utf8(file.contents.clone()).expect("Can't convert to string");
                        }
                    }
                }
            }
        }
    }

    panic!("Can't find server {server} nix configuration");
}

pub fn write_outputs_to_disk(target_dir: &str, plan: &CodegenPlan) {
    let path = std::path::Path::new(target_dir);
    mk_dir_no_error(path);
    write_outputs_to_disk_recur(path, &plan.root_dir);
}

fn write_outputs_to_disk_recur(path: &Path, dir: &Directory) {
    for file in &dir.files {
        let file_path = std::path::Path::join(path, &file.name);
        if let Some(cond) = &file.condition {
            match cond {
                SpecialFileCreationCondition::DontCreateIfDirectoryExists(dir_name) => {
                    let check_path = std::path::Path::join(path, dir_name.as_str());
                    match std::fs::metadata(check_path) {
                        Ok(data) => {
                            if data.is_dir() {
                                // skip, condition satisfied
                                break;
                            }
                        }
                        Err(e) => {
                            if e.kind() != std::io::ErrorKind::NotFound {
                                panic!("error checking if dir exists: {}", e);
                            }
                        }
                    }
                }
            }
        }

        write_file_if_not_changed(
            file_path.as_path(),
            &file.contents,
            file.if_not_exists,
            file.permissions,
            file.always_overwrite,
        );
    }

    for (dir_path, dir_contents) in &dir.directories {
        let path = std::path::Path::join(path, dir_path);
        mk_dir_no_error(path.as_path());
        write_outputs_to_disk_recur(path.as_path(), dir_contents);
    }
}

fn mk_dir_no_error(path: &Path) {
    match std::fs::create_dir(path) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => {}
            _ => {
                panic!("Cannot create directory: {}", e);
            }
        },
    }
}

fn write_file_if_not_changed(
    path: &Path,
    contents: &[u8],
    skip_if_exists: bool,
    permissions: &'static str,
    always_overwrite: bool,
) {
    if always_overwrite {
        write_file_impl(path, contents, permissions);
        return;
    }

    match std::fs::read(path) {
        Ok(vec) => {
            if skip_if_exists {
                return;
            }

            if vec.as_slice() != contents {
                write_file_impl(path, contents, permissions);
            }
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                write_file_impl(path, contents, permissions);
            }
            _ => panic!("Cannot read file status: {e}"),
        },
    }
}

fn write_file_impl(path: &Path, contents: &[u8], permissions: &'static str) {
    match permissions {
        "644" => {
            std::fs::write(path, contents).expect("Cannot write file");
        }
        "755" | "600" => {
            let mut fl = std::fs::File::create(path).unwrap();
            let mut perms = fl.metadata().unwrap().permissions();
            match permissions {
                "755" => {
                    perms.set_mode(0o755);
                }
                "600" => {
                    perms.set_mode(0o600);
                }
                _ => panic!("This is ugly but I'm tired today"),
            }
            fl.set_permissions(perms)
                .expect("Cannot change permissions to a file");
            fl.write_all(contents)
                .expect("Cannot write executable file to disk");
        }
        other => {
            panic!("These ain't permissions I've heard of boi: {}", other)
        }
    }
}

fn generate_applications(checked: &CheckedDB, plan: &mut CodegenPlan) {
    let rd = plan.root_dir();

    let cenvs_dir = rd.create_directory("comp-envs");
    rust::generate_compile_envs(checked, cenvs_dir);

    let apps_dir = rd.create_directory("apps");

    let db = &checked.db;
    for app in db.backend_application().rows_iter() {
        let this_app_dir =
            apps_dir.create_directory(db.backend_application().c_application_name(app));

        rust::backend::generate_rust_backend_app(checked, app, this_app_dir);
    }
    for app in db.frontend_application().rows_iter() {
        let this_app_dir =
            apps_dir.create_directory(db.frontend_application().c_application_name(app));

        rust::frontend::generate_rust_frontend_app(checked, app, this_app_dir);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct MetricScrapeResult {
    pub name: String,
    pub expression: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MetricScrapePlan {
    pub cluster_name: String,
    pub mirrors: Vec<String>,
    pub scrapes: Vec<MetricScrapeResult>,
}

fn generate_prometheus_tests(checked: &CheckedDB, plan: &mut CodegenPlan) {
    if (checked.db.alert().len() == 0 || checked.projections.promtool_test_suite.test_count == 0)
        && checked.db.monitoring_cluster_scraped_metric().len() == 0 {
        return;
    }

    let rd = plan.root_dir();

    let cenvs_dir = rd.create_directory("prometheus");
    cenvs_dir.create_file(
        "alert_rules.yml",
        checked.projections.promtool_test_suite.rules_file.clone(),
    );
    cenvs_dir.create_file(
        "tests.yml",
        checked.projections.promtool_test_suite.tests_file.clone(),
    );
    cenvs_dir.create_executable_file(
        "test.sh",
        r#"#!/bin/sh
promtool test rules tests.yml
"#
        .to_string(),
    );

    let ms_plan = metrics_scrape_plan(checked);
    cenvs_dir.create_file("metric_scrape_plan.yml", serde_yaml::to_string(&ms_plan).unwrap());
    cenvs_dir.create_file("db_schema.sql", metrics_db_schema().to_string());
}

fn metrics_scrape_plan(checked: &CheckedDB) -> Vec<MetricScrapePlan> {
    let mut res = Vec::with_capacity(checked.db.monitoring_cluster().len());

    for mon_c in checked.db.monitoring_cluster().rows_iter() {
        let region = checked.db.monitoring_cluster().c_region(mon_c);
        let cluster_name = checked.db.monitoring_cluster().c_cluster_name(mon_c).clone();
        let mirrors_arr = checked.db.monitoring_cluster().c_children_monitoring_instance(mon_c);
        let scrapes_arr = checked.db.monitoring_cluster().c_children_monitoring_cluster_scraped_metric(mon_c);
        let reg_default = checked.projections.monitoring_clusters.region_default(region);
        let is_this_region_default = Some(mon_c) == reg_default;
        let mut mirrors: Vec<String> = Vec::with_capacity(mirrors_arr.len());
        let mut scrapes: Vec<MetricScrapeResult> = Vec::with_capacity(scrapes_arr.len());
        let port = checked.db.monitoring_cluster().c_prometheus_port(mon_c);
        for ma in mirrors_arr {
            let mon_disk = checked.db.monitoring_instance().c_monitoring_server(*ma);
            let mon_server = checked.db.server_volume().c_parent(mon_disk);
            let nif = checked.projections.consul_network_iface.value(mon_server);
            let lan_ip = checked.db.network_interface().c_if_ip(*nif);

            mirrors.push(format!("http://{lan_ip}:{port}"));
        }

        if is_this_region_default {
            // builtins for system operations
            scrapes.push(MetricScrapeResult {
                name: "vault_sealed_clusters".to_string(),
                expression: "max_over_time( sum by (cluster) (vault_core_unsealed)[7d:] ) - sum by (cluster) (vault_core_unsealed) > 0".to_string(),
            });
            scrapes.push(MetricScrapeResult {
                name: "epl_l1_provisioning_last_hash".to_string(),
                expression: "max by (hostname, hash) (epl_l1_provisioning_last_hash)".to_string(),
            });
            scrapes.push(MetricScrapeResult {
                name: "node_boot_time_seconds".to_string(),
                expression: "node_boot_time_seconds".to_string(),
            });
        }

        for sm in scrapes_arr {
            scrapes.push(MetricScrapeResult {
                name: checked.db.monitoring_cluster_scraped_metric().c_metric_name(*sm).clone(),
                expression: checked.db.monitoring_cluster_scraped_metric().c_expression(*sm).clone(),
            })
        }

        res.push(MetricScrapePlan {
            cluster_name,
            mirrors,
            scrapes,
        })
    }

    res
}

fn gw_ip(checked: &CheckedDB) -> Option<String> {
    if vms_exist(checked) {
        checked.sync_res.network.test_docker_registry_gw_address.as_ref().cloned()
    } else { None }
}

pub struct FastProvSecrets {
    pub admin_provisioning_encryption_key: SecretValue,
    pub admin_provisioning_public_key: String,
    pub region_provisioning_decryption_seeds: BTreeMap<TableRowPointerRegion, SecretValue>,
}

pub struct CodegenSecrets {
    pub root_ssh_keys: SshKeysSecrets,
    pub nix_cache_keys: NixCacheKeys,
    pub wg_secrets: WgSecrets,
    pub admin_panel_htpasswd_file: SecretValue,
    pub fast_prov_secrets: FastProvSecrets,
}

fn generate_machines(
    checked: &CheckedDB,
    plan: &mut CodegenPlan,
    plans: &NixAllServerPlans,
    secrets: &CodegenSecrets,
) -> L1ProvOutputs {
    let rd = plan.root_dir();

    let machines_dir = rd.create_directory("servers");
    let networks_dir = machines_dir.create_directory("networks");
    infra::generate_vm_network_xml(checked, networks_dir);

    let mut gw_ips = BTreeSet::new();
    for nm in checked.sync_res.network.libvirt_network_topology.networks.values() {
        // there's only one internet gateway ip
        // on testing machine, hence that is the gateway
        if checked.db.network().c_network_name(nm.network) == "internet" {
            gw_ips.insert(nm.gw_ip.clone());
        }
    }

    let ver_name = checked.db.nixpkgs_version().c_version(checked.projections.default_used_nixpkgs_version);

    let maybe_gw_ip = gw_ip(checked);
    if let Some(gw_ip) = &maybe_gw_ip {
        for arch in &checked.projections.used_architectures {
            let out_file = format!("vm-template-{arch}.nix");
            machines_dir.create_file(
                &out_file,
                infra::base_machine_template(
                    checked,
                    gw_ip.as_str(),
                    &checked.projections.default_used_nixpkgs_checksum,
                    &checked.projections.default_used_nixpkgs_tarball_checksum,
                    ver_name.as_str(),
                    &secrets.root_ssh_keys,
                    &secrets.nix_cache_keys,
                ),
            );
        }
    }

    machines_dir.create_file("library.sh", infra::utils_library_script().to_string());
    machines_dir.create_executable_file("provision", generate_vm_provision_script());

    let aux_dir = machines_dir.create_directory("aux");
    aux_dir.create_file(
        "cache-priv-key.pem",
        secrets.nix_cache_keys.private_nix_cache_key.value().clone(),
    );
    aux_dir.create_file(
        "cache-pub-key.pem",
        secrets.nix_cache_keys.public_nix_cache_key.value().clone(),
    );
    aux_dir.create_file("ssh_config", infra::ssh_config_for_colmena().to_string());
    aux_dir.create_file_custom(
        "root_ssh_key",
        secrets.root_ssh_keys.private_root_ssh_key.value().clone().into_bytes(),
        false,
        "600",
        false,
        None,
    );
    aux_dir.create_file(
        "root_ssh_key.pub",
        secrets.root_ssh_keys.public_root_ssh_key.value().clone(),
    );

    let l1_prov_dir = rd.create_directory("l1-provisioning");
    let l1_outputs = generate_l1_provisioning_part(
        l1_prov_dir, checked, &maybe_gw_ip, plans, &secrets.nix_cache_keys, &secrets.root_ssh_keys
    );
    let l1_fast_dir = rd.create_directory("l1-fast");
    l1_provisioning::fast_l1::generate_fast_l1_provisioning_part(l1_fast_dir, checked, &l1_outputs, secrets);

    let l2_prov_dir = rd.create_directory("l2-provisioning");
    for region in checked.db.region().rows_iter() {
        let l2_prov_dir = l2_prov_dir.create_directory(checked.db.region().c_region_name(region));
        generate_l2_provisioning_part(l2_prov_dir, checked, region, secrets);
    }

    let integration_test_dir = rd.create_directory("integration-tests");
    integration_tests::generate_integration_test_dir(checked, integration_test_dir);

    let vpn_conf = generate_admin_vpn_config(checked, &secrets.wg_secrets);
    rd.create_file_custom("admin-wg.conf", vpn_conf.into_bytes(), false, "600", false, None);

    l1_outputs
}

#[allow(dead_code)]
pub struct L1ServerData {
    pub preconditions: String,
    pub provisioning: String,
    pub provisioning_hash: String,
    pub configuration_nix: String,
}

pub struct L1ProvOutputs {
    pub regions: BTreeMap<TableRowPointerRegion, BTreeMap<TableRowPointerServer, L1ServerData>>,
}

fn generate_l1_provisioning_part(prov_dir: &mut Directory, checked: &CheckedDB, gw_ip: &Option<String>, plans: &NixAllServerPlans, nix_cache_keys: &NixCacheKeys, root_ssh_keys: &SshKeysSecrets)
                                 -> L1ProvOutputs
{
    use std::fmt::Write;

    let mut outputs = L1ProvOutputs {
        regions: BTreeMap::new(),
    };

    let maybe_gw = match &gw_ip {
        Some(gw) => {
            format!(r#"
        "http://{gw}:12777/""#)
        }
        None => "".to_string(),
    };

    let default_nixpkgs = &checked.projections.default_used_nixpkgs_checksum;
    let default_nixpkgs_checksum = &checked.projections.default_used_nixpkgs_tarball_checksum;
    let mut all_srvs = format!(r#"
let
  pkgs = import (fetchTarball {{ url = "https://github.com/NixOS/nixpkgs/archive/{default_nixpkgs}.tar.gz"; sha256 = "{default_nixpkgs_checksum}"; }}) {{}};
  evalConfig = import (pkgs.path + "/nixos/lib/eval-config.nix");
  buildServer = args: (evalConfig args).config.system.build.toplevel;
in
{{
"#);

    let all_targets_file = "build-all-servers.nix";
    for server in checked.db.server().rows_iter() {
        let dc = checked.db.server().c_dc(server);
        let region = checked.db.datacenter().c_region(dc);
        let reg_srvs = outputs.regions.entry(region).or_default();
        let hostname = checked.db.server().c_hostname(server);
        assert_ne!(hostname, all_targets_file, "You gotta be kidding me boi... This would be a user error but you're an asswipe to trigger this assert.");
        let mut config_nix = String::new();
        let env = checked.db.server().c_nixpkgs_environment(server);
        let ver = checked.db.nixpkgs_environment().c_version(env);
        let ver_name = checked.db.nixpkgs_version().c_version(ver);
        let checksum = checked.db.nixpkgs_version().c_checksum(ver);
        let tarball_checksum = checked.db.nixpkgs_version().c_tarball_checksum(ver);
        let nixos_version = checked.db.nixpkgs_version().c_version(ver);
        let plan = plans.fetch_plan_ro(server);
        let is_testvm = "testvms" == checked.db.datacenter().c_implementation(checked.db.server().c_dc(server));
        let lan_iface = checked.projections.consul_network_iface.value(server);
        let lan_ip = checked.db.network_interface().c_if_ip(*lan_iface);
        let server_kind = checked.projections.server_kinds.value(server);
        let server_arch = epl_arch_to_linux_arch(&checked.db.server_kind().c_architecture(*server_kind));
        write!(&mut all_srvs, r#"
  {hostname} = buildServer {{
    system = "{server_arch}-linux";
    modules = [
      ./{hostname}/configuration.nix
    ];
  }};
"#).unwrap();
        let maybe_gw_routes =
            if is_testvm {
                if let Some(gw_ip) = &gw_ip {
                    let libvirt_gw_ip = format!("{}.1", first_three_octets(lan_ip));
                    Some(format!(r#"
/run/current-system/sw/bin/ip route del {gw_ip}/32 || true
/run/current-system/sw/bin/ip route add {gw_ip}/32 via {libvirt_gw_ip}
"#))
                } else { None }
            } else { None };
        let maybe_gw_routes = maybe_gw_routes.as_ref().map(|i| i.as_str());

        write!(&mut config_nix, r#"# EDEN PLATFORM GENERATED NIX CONFIG
# changes done to this file will be overwritten by Eden platform
let
  pkgs = import (fetchTarball {{ url = "https://github.com/NixOS/nixpkgs/archive/{}.tar.gz"; sha256 = "{}"; }}) {{}};
  lib = pkgs.lib;
  modulesPath = pkgs.path + "/nixos/modules";
in
"#, checksum, tarball_checksum
        ).unwrap();

        config_nix += r#"
{ ... }:
{

    nix.settings = {
      tarball-ttl = 60 * 60 * 7;
      experimental-features = [ "nix-command" "flakes" ];
      substituters = ["#;

        config_nix += &maybe_gw;
        config_nix += "
        \"https://cache.nixos.org/\"\n";

        write!(&mut config_nix, r#"      ];
      trusted-public-keys = [
        "{}"
      ];
"#, nix_cache_keys.public_nix_cache_key).unwrap();

        config_nix += r#"
    };
"#;

        let host_hash = hmac_sha256::Hash::hash(hostname.as_bytes());
        write!(&mut config_nix, r#"
    networking.hostId = "{}";
"#, hex::encode(&host_hash[0..4])).unwrap();
        config_nix += r#"

    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
"#;

        let mut keys: HashSet<String> = HashSet::new();

        if keys.insert(root_ssh_keys.public_root_ssh_key.value().clone()) {
            config_nix += "      \"";
            config_nix += root_ssh_keys.public_root_ssh_key.value().trim();
            config_nix += "\"\n";
        }

        for key in checked.db.admin_ssh_keys().rows_iter() {
            if keys.insert(checked.db.admin_ssh_keys().c_contents(key).clone()) {
                config_nix += "      \"";
                config_nix += checked.db.admin_ssh_keys().c_contents(key);
                config_nix += "\"\n";
            }
        }

        write!(&mut config_nix, r#"
      ];
    }};
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

    system.stateVersion = "{ver_name}";
"#).unwrap();

        plan.dump_nix_blocks(&mut config_nix);
        configure_server_ip_interfaces(checked, server, &mut config_nix);

        config_nix += r#"
}
"#;

        let l1_prov_script = generate_l1_provisioning_script(maybe_gw_routes, &config_nix, server, plans, nixos_version.as_str());
        let l1_script_hash = sha256::hash(l1_prov_script.as_bytes());
        let provisioning_hash = hex::encode(&l1_script_hash.0);
        let preconditions = generate_server_preconditions(checked, server);

        assert!(reg_srvs.insert(server, L1ServerData {
            preconditions: preconditions.clone(),
            provisioning: l1_prov_script.clone(),
            configuration_nix: config_nix,
            provisioning_hash,
        }).is_none());
    }

    // hashes computed at this point
    for server in checked.db.server().rows_iter() {
        let dc = checked.db.server().c_dc(server);
        let region = checked.db.datacenter().c_region(dc);
        let reg_srvs = outputs.regions.entry(region).or_default();
        let hostname = checked.db.server().c_hostname(server);

        let Some(l1_data) = reg_srvs.get(&server) else {
            continue;
        };

        let l1_hash = &l1_data.provisioning_hash;

        let this_server_dir = prov_dir.create_directory(&hostname);
        let mut l1_script = String::new();

        let prov_server = checked.projections.provisioning_server_in_region.value(region);

        write!(&mut l1_script, r#"
if [ -d /var/lib/node_exporter ]
then
  # l1 last hash
  METRICS_FILE=/var/lib/node_exporter/epl_l1_last_hash.prom
  BOOT_TIME=$( cat /proc/stat | grep btime | awk '{{ print $2 }}' )
  echo "
epl_l1_provisioning_last_hash{{hash=\"{l1_hash}\",hostname=\"{hostname}\"}} $BOOT_TIME
" > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE
"#).unwrap();

        // save all other server expected hashes
        // so we could alert if some server mismatches the hash
        if &Some(server) == prov_server {
            // write all region server expected hashes
            write!(&mut l1_script, r#"
  # l1 expected hash
  METRICS_FILE=/var/lib/node_exporter/epl_l1_expected_hash.prom
  echo '
"#).unwrap();
            let mut region_servers = Vec::new();
            for dc in checked.db.region().c_referrers_datacenter__region(region) {
                for region_srv in checked.db.datacenter().c_referrers_server__dc(*dc) {
                    region_servers.push(*region_srv);
                }
            }
            region_servers.sort_by_key(|i| checked.db.server().c_hostname(*i));

            for region_srv in region_servers {
                if let Some(reg_srv_data) = reg_srvs.get(&region_srv) {
                    let reg_host = checked.db.server().c_hostname(region_srv);
                    let reg_srv_hash = &reg_srv_data.provisioning_hash;
                    writeln!(&mut l1_script, r#"epl_l1_provisioning_expected_hash{{hash="{reg_srv_hash}",hostname="{reg_host}"}} 1"#).unwrap();
                }
            }

            write!(&mut l1_script, r#"
' > $METRICS_FILE.tmp
  chmod 644 $METRICS_FILE.tmp
  mv -f $METRICS_FILE.tmp $METRICS_FILE
"#).unwrap();
        }

        write!(&mut l1_script, r#"
fi
"#).unwrap();

        let l1_data = reg_srvs.get_mut(&server).expect("We got this before as immutable");
        l1_data.provisioning += &l1_script;

        this_server_dir.create_file(
            &"provision.sh",
            l1_data.provisioning.clone(),
        );
        this_server_dir.create_file(
            &"configuration.nix".to_string(),
            l1_data.configuration_nix.clone()
        );
        this_server_dir.create_file(
            &"preconditions.sh",
            l1_data.preconditions.clone(),
        );

        // mock for fast l1 provisioner
        let l1_checker_dir = this_server_dir.create_directory("l1-checker");
        l1_checker_dir.create_file(
            "default.nix",
            r#"{ pkgs }: "mock""#.to_string(),
        );
    }

    write!(&mut all_srvs, r#"
}}
"#).unwrap();

    prov_dir.create_file(all_targets_file, all_srvs);

    outputs
}

fn generate_l2_provisioning_part(prov_dir: &mut Directory, checked: &CheckedDB, region: TableRowPointerRegion, secrets: &CodegenSecrets) {
    let mut root_script = String::new();
    root_script += "#!/bin/sh\n\n";

    root_script += "set -e\n";
    root_script += "\n";
    root_script += "PROVISIONING_TIME=$( date '+%Y-%m-%dT%H:%M:%S.%NZ' )\n";
    root_script += "PROVISIONING_LOG_DIR=/var/log/epl-l2-prov/$PROVISIONING_TIME\n";
    // delete runs older than 7 days
    root_script += "find /var/log/epl-l2-prov/* -type d -ctime +7 -exec rm -rf {} \\; || true\n";
    root_script += "mkdir -p $PROVISIONING_LOG_DIR\n";
    root_script += "\n";

    for (res_kind, files) in checked.projections.server_runtime.provisioning_resources(region) {
        let dir = prov_dir.create_directory(*res_kind);
        for (fname, fcont) in files {
            if fcont.is_executable() {
                dir.create_executable_file(fname.as_str(), fcont.replaced_contents(secrets));
            } else {
                dir.create_file(fname.as_str(), fcont.replaced_contents(secrets));
            }
        }
    }

    let scripts_dir = prov_dir.create_directory("scripts");

    let mut longest_script_name = 0;
    for v in checked
        .projections
        .server_runtime
        .provisioning_scripts(region)
        .values()
    {
        for ps in v {
            if ps.name().len() > longest_script_name {
                longest_script_name = ps.name().len();
            }
        }
    }

    let mut run_log_files = Vec::new();
    // every layer can run in parallel
    for (k, v) in checked.projections.server_runtime.provisioning_scripts(region) {
        root_script += &format!("# level {k}\n");
        root_script += &format!("stage={k}\n");
        root_script += "jobs=( )\n";
        root_script += "stage_pids=( )\n";

        for ps in v {
            assert!(!ps.name().contains('\''), "Should be checked much earlier");
            let log_file = format!("$PROVISIONING_LOG_DIR/{:0>3}_{}.log", k, ps.name());

            let mut padded_ps_name = ps.name().to_string();
            while padded_ps_name.len() < longest_script_name {
                padded_ps_name += " ";
            }

            scripts_dir.create_executable_file(ps.name(), ps.script().to_string());

            root_script += &format!("jobs+=( {} )\n", ps.name());
            root_script += "scripts/";
            root_script += ps.name();
            root_script += &format!(
                " 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] {:0>3} {} |' | tee {} & stage_pids+=( $! )\n",
                k, padded_ps_name, log_file
            );

            run_log_files.push(log_file);
        }

        root_script += r#"
job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

"#;
    }

    // create log summary with all logs
    root_script += "# create log summary\n";
    root_script += "cat ";
    for log_file in &run_log_files {
        root_script += log_file;
        root_script += " ";
    }
    root_script += "| sort > $PROVISIONING_LOG_DIR/_combined.log\n";

    prov_dir.create_executable_file("provision.sh", root_script);
}

pub(crate) fn generate_vm_provision_script() -> String {
    r#"#!/bin/sh
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source $SCRIPT_DIR/library.sh
EPL_EXECUTABLE=${EPL_EXECUTABLE:-../../target/debug/epl}
REGION=$1
SERVER_IP=$2

if ! which $EPL_EXECUTABLE
then
  echo eden platform executable not found
  exit 7
fi

ensure_script_running_from_its_directory

# if repo is cloned permissions are too open for ssh key
chmod 600 aux/root_ssh_key

INFRA_ROOT_NOMAD_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key nomad_region_${REGION}_bootstrap_acl_token )
INFRA_ROOT_VAULT_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key vault_region_${REGION}_initial_keys | grep 'Initial Root Token:' | sed -E 's/.*: hvs./hvs./' )
CONSUL_ROOT_TOKEN=$( $EPL_EXECUTABLE get-secret --output-directory .. --key consul_region_${REGION}_acl_management_token )

rsync -av --mkpath --delete -e 'ssh -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectTimeout=2' \
  ../l2-provisioning/$REGION/ admin@$SERVER_IP:/run/secdir/provisioning
rsync -av --exclude apps/*/result --exclude=apps/*/target --exclude=apps/*/target/** --delete -e 'ssh -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectTimeout=2' \
  ../apps admin@$SERVER_IP:/run/secdir/provisioning/

ssh admin@$SERVER_IP -i aux/root_ssh_key \
    -o UserKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=false \
    -o ConnectTimeout=2 "
    tmux new-session -d '\
        export VAULT_TOKEN=$INFRA_ROOT_VAULT_TOKEN; \
        export NOMAD_TOKEN=$INFRA_ROOT_NOMAD_TOKEN; \
        export CONSUL_HTTP_TOKEN=$CONSUL_ROOT_TOKEN; \
        export EPL_PROVISIONING_DIR=/run/secdir/provisioning; \
      cd /run/secdir/provisioning && /bin/sh /run/secdir/provisioning/provision.sh'"
"#
    .to_string()
}

pub struct SshKeysSecrets {
    pub private_root_ssh_key: SecretValue,
    pub public_root_ssh_key: SecretValue,
}

pub fn generate_ssh_root_key_secrets(secrets: &mut SecretsStorage) -> SshKeysSecrets {
    let ssh_secret_files = sec_files(&[
        (
            SecretKind::SshPublicKey,
            "global_ssh_root_public_key",
            "key.pub",
        ),
        (
            SecretKind::SshPrivateKey,
            "global_ssh_root_private_key",
            "key",
        ),
    ]);

    let mut ssh_secrets = secrets.multi_secret_derive(
        &[],
        sec_files(&[]),
        ssh_secret_files.clone(),
        r#"
            ssh-keygen -f key -C epl-root-ssh-key -t ed25519 -N ''
        "#,
    );

    let private_root_ssh_key = ssh_secrets.pop().unwrap();
    let public_root_ssh_key = ssh_secrets.pop().unwrap();

    SshKeysSecrets {
        private_root_ssh_key,
        public_root_ssh_key,
    }
}

pub struct NixCacheKeys {
    pub private_nix_cache_key: SecretValue,
    pub public_nix_cache_key: SecretValue,
}

pub fn generate_nix_cache_keys(secrets: &mut SecretsStorage) -> NixCacheKeys {
    let nix_secret_files = sec_files(&[
        (
            SecretKind::NixCachePrivateKey,
            "global_nix_cache_private_key",
            "private",
        ),
        (
            SecretKind::NixCachePublicKey,
            "global_nix_cache_public_key",
            "public",
        ),
    ]);

    let mut nix_cache_secrets = secrets.multi_secret_derive(
        &[],
        sec_files(&[]),
        nix_secret_files.clone(),
        r#"
            nix key generate-secret --key-name epl-nix-cache > private
            cat private | nix key convert-secret-to-public > public
        "#,
    );

    let public_nix_cache_key = nix_cache_secrets.pop().unwrap();
    let private_nix_cache_key = nix_cache_secrets.pop().unwrap();

    NixCacheKeys {
        private_nix_cache_key,
        public_nix_cache_key,
    }
}
