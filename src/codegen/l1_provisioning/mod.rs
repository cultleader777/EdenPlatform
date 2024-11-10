use self::cadvisor::provision_cadvisor;
use self::cloud_specific::provision_cloud_specific;
use self::hardware::provision_spec_hardware;
use self::log_forwarding::provision_log_forwarding;
use self::node_exporter::provision_node_exporter;
use self::nomad::provision_nomad;
use self::routing::provision_routing;
use self::tls_certs::provision_tls_certificates;
use self::dns::provision_dns;
use self::utils::provision_generic_every_server;
use self::wireguard::provision_wireguard;
use self::disks::provision_disks;
use self::{consul::provision_consul, vault::provision_vault};
use crate::static_analysis::{CheckedDB, get_global_settings};

use super::CodegenSecrets;
use super::makefile::vms_exist;
use super::{nixplan::NixAllServerPlans, secrets::SecretsStorage};

mod cadvisor;
pub mod consul;
mod log_forwarding;
pub mod node_exporter;
pub mod nomad;
mod tls_certs;
pub mod utils;
pub mod vault;
pub mod dns;
pub mod wireguard;
pub mod routing;
pub mod hardware;
pub mod disks;
mod cloud_specific;
pub mod fast_l1;

pub fn provision_servers(
    db: &CheckedDB,
    plans: &mut NixAllServerPlans,
    secrets: &mut SecretsStorage,
    cgen_secrets: &CodegenSecrets,
) {
    provision_generic_every_server(db, plans, cgen_secrets);
    provision_spec_hardware(db, plans);
    provision_tls_certificates(db, plans, secrets);
    provision_dns(db, plans, secrets);
    provision_docker_registry(db, plans);
    provision_epl_prov_group(db, plans);
    provision_consul(db, plans, secrets);
    provision_nomad(db, plans, secrets);
    provision_vault(db, plans, secrets);
    provision_node_exporter(db, plans);
    provision_cadvisor(db, plans);
    provision_log_forwarding(db, plans);
    provision_wireguard(db, plans, secrets);
    provision_routing(db, plans, secrets);
    provision_cloud_specific(db, plans);
    provision_disks(db, plans, secrets);
}

fn provision_docker_registry(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    let vms_exist = vms_exist(db);
    let settings = get_global_settings(&db.db);
    for server in db.db.server().rows_iter() {
        for iface in db.db.server().c_children_network_interface(server) {
            if db
                .db
                .network()
                .c_network_name(db.db.network_interface().c_if_network(*iface))
                == "lan"
            {
                let mut mirrors = Vec::with_capacity(2);
                if vms_exist {
                    if let Some(gw_ip) = &db.sync_res.network.test_docker_registry_gw_address {
                        mirrors.push(format!("http://{gw_ip}:12778"));
                    }
                } else {
                    mirrors.push("https://registry-1.docker.io".to_string());
                }
                let mut extra_options = String::new();
                let port = settings.docker_registry_port;
                let service_slug = &settings.docker_registry_service_name;
                let registry_url = format!("http://{service_slug}.service.consul:{port}");
                extra_options += "--insecure-registry ";
                extra_options += &registry_url;
                mirrors.push(registry_url);

                let mirr_contents = mirrors
                    .iter()
                    .map(|m| format!("\"{m}\""))
                    .collect::<Vec<_>>()
                    .join(" ");

                let plan = plans.fetch_plan(server);
                plan.add_custom_nix_block(format!(
                    r#"
    virtualisation.docker.daemon.settings = {{ "registry-mirrors" = [ {mirr_contents} ]; }};
    virtualisation.docker.extraOptions = "{extra_options}";
"#
                ));
            }
        }
    }
}

/// Provision group for accessing provisioning logs
fn provision_epl_prov_group(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    for server in db.db.server().rows_iter() {
        let plan = plans.fetch_plan(server);
        plan.add_custom_nix_block(
            r#"
    users.groups.epl-prov = {};
"#
            .to_string(),
        );
    }
}
