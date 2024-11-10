use crate::{
    database::Database,
    static_analysis::{
        server_runtime::{AdminService, ServerRuntime},
        PlatformValidationError,
    },
};

pub fn lock_system_resources(
    db: &Database,
    runtime: &mut ServerRuntime,
) -> Result<(), PlatformValidationError> {
    let _ = runtime.lock_port_all_servers(22, "SSH".to_string())?;

    let _ = runtime.lock_port_all_servers(53, "DNS".to_string())?;
    let _ = runtime.lock_port_all_servers(68, "dhcpd".to_string())?;
    let _ = runtime.lock_port_all_servers(179, "BGP".to_string())?;
    let _ = runtime.lock_port_all_servers(546, "dhcpd".to_string())?;

    let _ = runtime.lock_port_all_servers(3784, "BFD UDP".to_string())?;
    let _ = runtime.lock_port_all_servers(4784, "BFD TCP".to_string())?;

    let _ = runtime.lock_port_all_servers(8500, "Consul local HTTP".to_string())?;
    let p = runtime.lock_port_all_servers(8501, "Consul HTTPS".to_string())?;
    let consul_https_port = p;
    let _ = runtime.lock_port_all_servers(8300, "Consul Server RPC".to_string())?;
    let _ = runtime.lock_port_all_servers(8301, "Consul Serf LAN".to_string())?;
    let _ = runtime.lock_port_all_servers(8302, "Consul Serf WAN".to_string())?;
    let _ = runtime.lock_port_all_servers(8600, "Consul DNS".to_string())?;

    let p = runtime.lock_port_all_servers(4646, "Nomad HTTPS".to_string())?;
    let nomad_https_port = p;
    let _ = runtime.lock_port_all_servers(4647, "Nomad RPC".to_string())?;
    let _ = runtime.lock_port_all_servers(4648, "Nomad Serf".to_string())?;

    let p = runtime.lock_port_all_servers(8200, "Vault HTTPS address".to_string())?;
    let vault_https_port = p;
    let _ = runtime.lock_port_all_servers(8201, "Vault cluster HTTPS address".to_string())?;

    let _ = runtime.lock_port_all_servers(9100, "Prometheus Node Exporter".to_string())?;
    let _ = runtime.lock_port_all_servers(9134, "Prometheus ZFS Exporter".to_string())?;
    let _ = runtime.lock_port_all_servers(9280, "Prometheus cAdvisor Exporter".to_string())?;
    let _ = runtime.lock_port_all_servers(9281, "Prometheus Vector Exporter".to_string())?;

    let _ = runtime.lock_port_all_servers(51820, "Wireguard VPN".to_string())?;
    let _ = runtime.lock_port_all_servers(51821, "Wireguard VPN".to_string())?;

    for region in db.region().rows_iter() {
        let region_name = db.region().c_region_name(region);
        let nomad_service = runtime.instantiate_and_seal_consul_service(region, "nomad-servers");
        runtime.expose_admin_service(
            db,
            AdminService {
                service_title: "Nomad".to_string(),
                service_kind: "nomad".to_string(),
                service_instance: region_name.clone(),
                service_internal_upstream: nomad_service,
                service_internal_port: nomad_https_port.value(),
                is_https: true,
            },
        )?;

        let vault_service = runtime.instantiate_and_seal_consul_service(region, "vault");
        runtime.expose_admin_service(
            db,
            AdminService {
                service_title: "Vault".to_string(),
                service_kind: "vault".to_string(),
                service_instance: region_name.clone(),
                service_internal_upstream: vault_service,
                service_internal_port: vault_https_port.value(),
                is_https: true,
            },
        )?;

        let consul_service = runtime.instantiate_and_seal_consul_service(region, "consul");
        runtime.expose_admin_service(
            db,
            AdminService {
                service_title: "Consul".to_string(),
                service_kind: "consul".to_string(),
                service_instance: region_name.clone(),
                service_internal_upstream: consul_service,
                service_internal_port: consul_https_port.value(),
                is_https: true,
            },
        )?;
    }

    Ok(())
}

pub fn provision_acme_cert_secrets(
    db: &Database,
    runtime: &mut ServerRuntime,
) -> Result<(), PlatformValidationError> {
    for region in db.region().rows_iter() {
        for tld in db.tld().rows_iter() {
            if db.tld().c_automatic_certificates(tld) {
                let domain = db.tld().c_domain(tld);
                let domain_kebab = domain.replace(".", "-");
                let sec_key = format!("certs/{domain_kebab}");
                let mut builder = runtime.declare_vault_secret(region, &sec_key);
                builder.request_secret(region, "full_chain", crate::static_analysis::server_runtime::VaultSecretRequest::Pem);
                builder.request_secret(region, "key", crate::static_analysis::server_runtime::VaultSecretRequest::Pem);
                builder.request_secret(region, "cert", crate::static_analysis::server_runtime::VaultSecretRequest::Pem);
                let _ = builder.finalize();
            }
        }
    }

    Ok(())
}
