use crate::{database::{Database, TableRowPointerRegion}, codegen::l1_provisioning::nomad::provisioning_nomad_namespaces_script};

use super::{
    server_runtime::{NomadJobStage, ProvisioningScriptTag, ServerRuntime},
    L1Projections, PlatformValidationError,
};

pub mod build_epl_apps;
pub mod docker_registry;
pub mod epl_app_ingress;
pub mod epl_jobs;
pub mod external_lb;
pub mod grafana;
pub mod grafana_loki;
pub mod grafana_tempo;
pub mod minio;
pub mod monitoring;
pub mod nats;
pub mod postgres;
pub mod clickhouse;
pub mod schedule_nomad_jobs;
pub mod system;
pub mod vault_secrets;
pub mod consul_resources;
pub mod blackbox_deployments;

pub fn deploy_all_components(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
) -> Result<(), PlatformValidationError> {
    system::lock_system_resources(db, runtime)?;
    system::provision_acme_cert_secrets(db, runtime)?;
    docker_registry::deploy_docker_registry(db, runtime, l1proj)?;
    postgres::deploy_postgres_instances(db, runtime, l1proj)?;
    clickhouse::deploy_clickhouse_instances(db, runtime, l1proj)?;
    nats::deploy_nats_instances(db, runtime, l1proj)?;
    monitoring::deploy_monitoring_instances(db, runtime, l1proj)?;
    grafana::deploy_grafana(db, runtime, l1proj)?;
    grafana_loki::deploy_loki(db, runtime, l1proj)?;
    grafana_tempo::deploy_tempo(db, runtime, l1proj)?;
    epl_jobs::deploy_epl_applications(db, runtime, l1proj)?;
    minio::deploy_minio_instances(db, runtime, l1proj)?;
    blackbox_deployments::deploy_blackbox_deployments(db, runtime, l1proj)?;
    epl_app_ingress::deploy_epl_backend_applications(
        db,
        runtime,
        l1proj.checked_http_endpoints,
        l1proj.backend_ingress_endpoints,
    )?;
    epl_app_ingress::deploy_epl_frontend_applications(
        db,
        runtime,
        l1proj.checked_frontend_pages
    )?;

    for region in db.region().rows_iter() {
        deploy_regional_components(db, runtime, l1proj, region)?;
    }

    Ok(())
}


/// Components in this function ought
/// to be deployed once per region
pub fn deploy_regional_components(
    db: &Database,
    runtime: &mut ServerRuntime,
    l1proj: &L1Projections,
    region: TableRowPointerRegion,
) -> Result<(), PlatformValidationError> {
    // after all jobs provisioned, generate secret provision script
    external_lb::deploy_external_lb(db, region, runtime, l1proj)?;
    vault_secrets::generate_nomad_job_secrets(runtime, region);
    consul_resources::generate_consul_resources(runtime, region);
    let system_jobs =
        schedule_nomad_jobs::schedule_nomad_jobs(db, region, runtime, NomadJobStage::SystemJob);
    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::L1Resources, // technicaly not l1 resource but must be earlier than jobs
        "provision-nomad-namespaces.sh",
        provisioning_nomad_namespaces_script(db),
    );
    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::RunNomadSystemJob,
        "schedule-nomad-system-jobs.sh",
        system_jobs,
    );
    build_epl_apps::build_epl_jobs_script(db, l1proj, region, runtime);
    build_epl_apps::push_epl_apps_to_registry(db, l1proj, region, runtime);

    let app_jobs =
        schedule_nomad_jobs::schedule_nomad_jobs(db, region, runtime, NomadJobStage::Application);
    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::RunNomadAppJob,
        "schedule-nomad-app-jobs.sh",
        app_jobs,
    );

    Ok(())
}
