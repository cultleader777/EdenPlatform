use std::{sync::Arc, collections::{BTreeMap, HashMap, BTreeSet, HashSet}};

use crate::{database::{Database, TableRowPointerBackendApplication, TableRowPointerPgSchema, TableRowPointerVersionedType, TableRowPointerBackendHttpEndpoint, TableRowPointerHttpMethods, TableRowPointerHttpEndpointDataType, TableRowPointerPgTransaction, TableRowPointerServer, TableRowPointerNetworkInterface, TableRowPointerBackendApplicationPgShard, TableRowPointerBackendApplicationDeployment, TableRowPointerPgDeploymentSchemas, TableRowPointerBackendApplicationNatsStream, TableRowPointerNatsJetstreamStream, TableRowPointerAlertTriggerTest, TableRowPointerNixpkgsVersion, TableRowPointerFrontendPage, TableRowPointerFrontendApplication, TableRowPointerFrontendApplicationUsedEndpoint, TableRowPointerFrontendApplicationDeployment, TableRowPointerBackendApplicationDeploymentIngress, TableRowPointerFrontendApplicationExternalPage, TableRowPointerFrontendApplicationExternalLink, TableRowPointerFrontendApplicationDeploymentIngress, TableRowPointerTld, TableRowPointerLokiCluster, TableRowPointerMonitoringCluster, TableRowPointerRegion, TableRowPointerDatacenter, TableRowGlobalSettings, TableRowPointerServerKind, TableRowPointerServerDisk, TableRowPointerTempoCluster, TableRowPointerBackendApplicationS3Bucket, TableRowPointerMinioBucket, TableRowPointerBackendApplicationConfig, TableRowPointerChSchema, TableRowPointerChDeploymentSchemas, TableRowPointerBackendApplicationChShard, TableRowPointerNomadNamespace}, codegen::rust::{RustVersionedTypeSnippets, GeneratedRustSourceForHttpEndpoint}, prom_metrics_dump::AllClusterSeriesDatabase};
use self::{databases::{postgres::{EnrichedPgDbData, TransactionStep}, clickhouse::EnrichedChDbData}, bw_compat_types::ComputedType, projections::{Projection, Index, VersionedTypeUsageFlags}, http_endpoints::{CheckedHttpEndpoint, HttpPathTree, PathArgs}, networking::{NetworkAnalysisOutput, ClusterPicker, DcVpnGateways, VpnGateway}, server_runtime::ServerRuntime, applications::{ApplicationPgQueries, ApplicationChQueries}, alerts::{PromtoolTestSuite, AlertTestCompiled}, dns::DnsChecks, dc_impl::{aws::AwsTopology, gcloud::GcloudTopology, bm_simple::BmSimpleDatacenterArguments}, docker_images::run_docker_image_checks, server_labels::LabelDatabase, l2_provisioning::{epl_app_ingress::IpsCollection, blackbox_deployments::BlackBoxDeploymentResource}};

pub mod networking;
pub mod databases;
pub mod bw_compat_types;
pub mod http_endpoints;
pub mod projections;
pub mod errors;
pub mod server_runtime;
pub mod l2_provisioning;
pub mod applications;
pub mod alerts;
pub mod dns;
pub mod dc_impl;
pub mod docker_images;
pub mod server_labels;
pub mod server_disks;

pub use crate::static_analysis::errors::PlatformValidationError;
pub use crate::static_analysis::alerts::AlertTestSeries;

impl std::fmt::Display for PlatformValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "platform validation error: {:?}", self)
    }
}

impl std::error::Error for PlatformValidationError {}
pub struct Projections {
    pub versioned_types: Projection<TableRowPointerVersionedType, Vec<ComputedType>>,
    pub rust_versioned_type_snippets: Projection<TableRowPointerVersionedType, RustVersionedTypeSnippets>,
    pub application_used_bw_types: Projection<TableRowPointerBackendApplication, BTreeMap<TableRowPointerVersionedType, VersionedTypeUsageFlags>>,
    pub checked_http_endpoints: Projection<TableRowPointerBackendHttpEndpoint, CheckedHttpEndpoint>,
    pub checked_app_http_paths: Projection<TableRowPointerBackendApplication, HttpPathTree<TableRowPointerBackendHttpEndpoint>>,
    pub checked_frontend_pages: Projection<TableRowPointerFrontendPage, PathArgs>,
    pub checked_frontend_http_paths: Projection<TableRowPointerFrontendApplication, HttpPathTree<TableRowPointerFrontendPage>>,
    pub backend_ingress_endpoints: Projection<TableRowPointerBackendApplicationDeploymentIngress, BTreeSet<TableRowPointerBackendHttpEndpoint>>,
    pub rust_sources_for_http_endpoints: Projection<TableRowPointerBackendHttpEndpoint, GeneratedRustSourceForHttpEndpoint>,
    pub bw_type_by_name: Index<String, TableRowPointerVersionedType>,
    pub valid_http_method_by_name: Index<String, TableRowPointerHttpMethods>,
    pub valid_http_data_type: Index<String, TableRowPointerHttpEndpointDataType>,
    pub transaction_steps: Projection<TableRowPointerPgTransaction, Vec<TransactionStep>>,
    pub consul_network_iface: Projection<TableRowPointerServer, TableRowPointerNetworkInterface>,
    // server may or may not have public network interface
    pub internet_network_iface: HashMap<TableRowPointerServer, TableRowPointerNetworkInterface>,
    pub vpn_network_iface: HashMap<TableRowPointerServer, TableRowPointerNetworkInterface>,
    pub application_pg_shard_queries: Projection<TableRowPointerBackendApplicationPgShard, ApplicationPgQueries>,
    pub application_ch_shard_queries: Projection<TableRowPointerBackendApplicationChShard, ApplicationChQueries>,
    pub application_deployment_pg_wirings: HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationPgShard, TableRowPointerPgDeploymentSchemas>>,
    pub application_deployment_ch_wirings: HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationChShard, TableRowPointerChDeploymentSchemas>>,
    pub application_deployment_stream_wirings: HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationNatsStream, TableRowPointerNatsJetstreamStream>>,
    pub application_deployment_bucket_wirings: HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationS3Bucket, TableRowPointerMinioBucket>>,
    pub application_deployment_configs: HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationConfig, String>>,
    pub frontend_deployment_endpoint_wirings: HashMap<TableRowPointerFrontendApplicationDeployment, HashMap<TableRowPointerFrontendApplicationUsedEndpoint, TableRowPointerBackendApplicationDeploymentIngress>>,
    pub frontend_deployment_page_wirings: HashMap<TableRowPointerFrontendApplicationDeployment, HashMap<TableRowPointerFrontendApplicationExternalPage, TableRowPointerFrontendApplicationDeploymentIngress>>,
    pub frontend_deployment_link_wirings: HashMap<TableRowPointerFrontendApplicationDeployment, HashMap<TableRowPointerFrontendApplicationExternalLink, TableRowPointerBackendApplicationDeploymentIngress>>,
    pub server_runtime: ServerRuntime,
    pub parsed_alert_trigger_test_series: BTreeMap<TableRowPointerAlertTriggerTest, AlertTestCompiled>,
    pub server_fqdns: Projection<TableRowPointerServer, String>,
    pub series_database: Option<AllClusterSeriesDatabase>,
    pub promtool_test_suite: PromtoolTestSuite,
    pub default_used_nixpkgs_version: TableRowPointerNixpkgsVersion,
    pub default_used_nixpkgs_checksum: String,
    pub default_used_nixpkgs_tarball_checksum: String,
    pub dns_checks: DnsChecks,
    pub ingress_dns_entries: BTreeMap<TableRowPointerTld, BTreeMap<String, IpsCollection>>,
    pub region_ingresses: HashMap<TableRowPointerRegion, IpsCollection>,
    pub loki_clusters: ClusterPicker<TableRowPointerLokiCluster>,
    pub monitoring_clusters: ClusterPicker<TableRowPointerMonitoringCluster>,
    pub tempo_clusters: ClusterPicker<TableRowPointerTempoCluster>,
    pub backend_apps_in_region: Projection<TableRowPointerRegion, HashSet<TableRowPointerBackendApplication>>,
    pub frontend_apps_in_region: Projection<TableRowPointerRegion, HashSet<TableRowPointerFrontendApplication>>,
    pub pg_schemas_in_region: Projection<TableRowPointerRegion, HashSet<TableRowPointerPgSchema>>,
    pub ch_schemas_in_region: Projection<TableRowPointerRegion, HashSet<TableRowPointerChSchema>>,
    pub provisioning_server_in_region: Projection<TableRowPointerRegion, Option<TableRowPointerServer>>,
    pub bb_depl_resources_per_region: Projection<TableRowPointerRegion, BTreeMap<String, BlackBoxDeploymentResource>>,
    pub vpn_gateways: BTreeMap<TableRowPointerDatacenter, DcVpnGateways>,
    pub vpn_p2p_links: BTreeMap<TableRowPointerServer, Vec<VpnGateway>>,
    pub server_kinds: Projection<TableRowPointerServer, TableRowPointerServerKind>,
    pub cloud_topologies: CloudTopologies,
    pub bm_topologies: BmTopologies,
    pub used_architectures: BTreeSet<String>,
    pub used_architectures_per_region: BTreeMap<TableRowPointerRegion, BTreeSet<String>>,
    pub label_database: LabelDatabase,
    pub server_disk_sizes: BTreeMap<TableRowPointerServerDisk, i64>,
}

pub struct CloudTopologies {
    pub aws: AwsTopology,
    pub gcloud: GcloudTopology,
}

pub struct BmTopologies {
    pub bm_simple: BTreeMap<TableRowPointerDatacenter, BmSimpleDatacenterArguments>,
}

impl CloudTopologies {
    pub fn cloud_needed(&self) -> bool {
        !self.aws.is_empty() || !self.gcloud.is_empty()
    }
}

// Projections needed for server runtime
// but not yet in codegen
pub struct L1Projections<'a> {
    pub application_deployment_pg_wirings: &'a HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationPgShard, TableRowPointerPgDeploymentSchemas>>,
    pub application_deployment_ch_wirings: &'a HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationChShard, TableRowPointerChDeploymentSchemas>>,
    pub application_ch_shard_queries: &'a Projection<TableRowPointerBackendApplicationChShard, ApplicationChQueries>,
    pub application_deployment_stream_wirings: &'a HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationNatsStream, TableRowPointerNatsJetstreamStream>>,
    pub application_deployment_bucket_wirings: &'a HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationS3Bucket, TableRowPointerMinioBucket>>,
    pub application_deployment_configs: &'a HashMap<TableRowPointerBackendApplicationDeployment, HashMap<TableRowPointerBackendApplicationConfig, String>>,
    pub checked_http_endpoints: &'a Projection<TableRowPointerBackendHttpEndpoint, CheckedHttpEndpoint>,
    pub checked_frontend_pages: &'a Projection<TableRowPointerFrontendPage, PathArgs>,
    pub backend_ingress_endpoints: &'a Projection<TableRowPointerBackendApplicationDeploymentIngress, BTreeSet<TableRowPointerBackendHttpEndpoint>>,
    pub frontend_deployment_endpoint_wirings: &'a HashMap<TableRowPointerFrontendApplicationDeployment, HashMap<TableRowPointerFrontendApplicationUsedEndpoint, TableRowPointerBackendApplicationDeploymentIngress>>,
    pub frontend_deployment_page_wirings: &'a HashMap<TableRowPointerFrontendApplicationDeployment, HashMap<TableRowPointerFrontendApplicationExternalPage, TableRowPointerFrontendApplicationDeploymentIngress>>,
    pub frontend_deployment_link_wirings: &'a HashMap<TableRowPointerFrontendApplicationDeployment, HashMap<TableRowPointerFrontendApplicationExternalLink, TableRowPointerBackendApplicationDeploymentIngress>>,
    pub loki_clusters: &'a ClusterPicker<TableRowPointerLokiCluster>,
    pub monitoring_clusters: &'a ClusterPicker<TableRowPointerMonitoringCluster>,
    pub tempo_clusters: &'a ClusterPicker<TableRowPointerTempoCluster>,
    pub backend_apps_in_region: &'a Projection<TableRowPointerRegion, HashSet<TableRowPointerBackendApplication>>,
    pub frontend_apps_in_region: &'a Projection<TableRowPointerRegion, HashSet<TableRowPointerFrontendApplication>>,
    pub pg_schemas_in_region: &'a Projection<TableRowPointerRegion, HashSet<TableRowPointerPgSchema>>,
    pub ch_schemas_in_region: &'a Projection<TableRowPointerRegion, HashSet<TableRowPointerChSchema>>,
    pub dns_checks: &'a DnsChecks,
    pub consul_network_iface: &'a Projection<TableRowPointerServer, TableRowPointerNetworkInterface>,
    pub internet_network_iface: &'a HashMap<TableRowPointerServer, TableRowPointerNetworkInterface>,
    pub server_fqdns: &'a Projection<TableRowPointerServer, String>,
    pub region_ingresses: &'a HashMap<TableRowPointerRegion, IpsCollection>,
    pub server_kinds: &'a Projection<TableRowPointerServer, TableRowPointerServerKind>,
    pub networking: &'a NetworkAnalysisOutput,
    pub used_architectures_per_region: &'a BTreeMap<TableRowPointerRegion, BTreeSet<String>>,
    pub label_database: &'a LabelDatabase,
    pub versioned_types: &'a Projection<TableRowPointerVersionedType, Vec<ComputedType>>,
    pub epl_nomad_namespace: TableRowPointerNomadNamespace,
    pub bb_depl_resources_per_region: &'a Projection<TableRowPointerRegion, BTreeMap<String, BlackBoxDeploymentResource>>,
}

pub struct CheckedDB {
    pub db: Arc<Database>,
    pub async_res: AsyncChecksOutputs,
    pub projections: Projections,
    pub sync_res: SyncChecksOutputs,
}

pub struct SyncChecksOutputs {
    pub network: NetworkAnalysisOutput,
}

pub fn run_static_checks(db: Arc<Database>) -> Result<CheckedDB, PlatformValidationError> {
    let bencher = bench_start("Synchronous checks");
    let sync_res = run_sync_checks(&db)?;
    let projections = projections::create_projections(&db, &sync_res)?;
    bencher.end();
    let bencher = bench_start("Asynchronous checks");
    let async_res = run_async_checks(&db, &projections)?;
    bencher.end();

    Ok(CheckedDB { db, async_res, projections, sync_res })
}

pub struct Bencher {
    message: &'static str,
    start_time: std::time::Instant,
}

impl Bencher {
    pub fn end(self) {
        let duration = self.start_time.elapsed();
        eprintln!("{}: {:.2}ms", self.message, duration.as_secs_f64() * 1000.0);
    }
}

pub fn bench_start(message: &'static str) -> Bencher {
    Bencher { message, start_time: std::time::Instant::now() }
}

fn run_sync_checks(db: &Database) -> Result<SyncChecksOutputs, PlatformValidationError> {
    run_metadata_checks(db)?;
    let network = networking::validations(db)?;
    run_docker_image_checks(db)?;
    bw_compat_types::compute_types(db)?;
    databases::postgres::sync_checks(db)?;
    databases::clickhouse::sync_checks(db)?;

    Ok(SyncChecksOutputs { network })
}

lazy_static! {
    static ref VALID_PROJECT_NAME_REGEX: regex::Regex = regex::Regex::new(r#"^[a-z0-9-]+$"#).unwrap();
}

fn run_metadata_checks(db: &Database) -> Result<(), PlatformValidationError> {
    if db.global_settings().len() != 1 {
        return Err(PlatformValidationError::EnvironmentMustHaveExactlyOneGlobalSettingsRow {
            table_name: "global_settings".to_string(),
            expected_row_count: 1,
            actual_row_count: db.global_settings().len(),
        });
    }

    for gs in db.global_settings().rows_iter() {
        let pname = db.global_settings().c_project_name(gs);
        if pname.is_empty() {
            return Err(PlatformValidationError::EnvironmentProjectNameCannotBeEmpty {
                table_name: "global_settings".to_string(),
                column_name: "project_name".to_string(),
                value: pname.clone(),
            });
        }

        let project_name_max_length = 32;
        if pname.len() > project_name_max_length {
            return Err(PlatformValidationError::EnvironmentProjectNameMustBeNotTooLong {
                table_name: "global_settings".to_string(),
                column_name: "project_name".to_string(),
                value: pname.clone(),
                length: pname.len(),
                max_length: project_name_max_length,
            });
        }

        if !VALID_PROJECT_NAME_REGEX.is_match(pname.as_str()) {
            return Err(PlatformValidationError::EnvironmentProjectNameMustBeKebabCase {
                table_name: "global_settings".to_string(),
                column_name: "project_name".to_string(),
                value: pname.clone(),
            });
        }
    }

    Ok(())
}

pub fn get_global_settings(db: &Database) -> &TableRowGlobalSettings {
    let gs_ptr = db.global_settings().rows_iter().next().unwrap();
    db.global_settings().row(gs_ptr)
}

#[derive(Clone)]
pub struct AsyncCheckContext {
    wait_for: tokio::sync::mpsc::UnboundedSender<tokio::task::JoinHandle<()>>,
}

pub struct AsyncChecksOutputs {
    pub checked_pg_dbs: HashMap<TableRowPointerPgSchema, EnrichedPgDbData>,
    pub checked_ch_dbs: HashMap<TableRowPointerChSchema, EnrichedChDbData>,
}

fn run_async_checks(db: &Database, proj: &Projections) -> Result<AsyncChecksOutputs, PlatformValidationError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all().build().unwrap();

    let res = rt.block_on(async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let ctx = AsyncCheckContext { wait_for: tx };
        let pg_validations = databases::postgres::validations(ctx.clone(), db);
        let ch_validations = databases::clickhouse::validations(ctx.clone(), db, proj);
        let pg_validations = pg_validations.await?;
        let ch_validations = ch_validations.await?;

        drop(ctx);
        while let Some(routine) = rx.recv().await {
            let _ = routine.await;
        }

        Ok(AsyncChecksOutputs {
            checked_pg_dbs: pg_validations,
            checked_ch_dbs: ch_validations,
        })
    })?;

    rt.shutdown_timeout(tokio::time::Duration::from_millis(3000));

    Ok(res)
}

pub async fn join_validation_errors<T>(iter: impl IntoIterator<Item = impl futures::Future<Output = Result<T, PlatformValidationError>>>) -> Result<Vec<T>, PlatformValidationError>
{
    let mut results = Vec::new();
    let res: Vec<PlatformValidationError> =
        futures::future::join_all(iter)
            .await
            .into_iter()
            .filter_map(|i| {
                match i {
                    Err(e) => {
                        Some(e)
                    }
                    Ok(r) => {
                        results.push(r);
                        None
                    }
                }
            }).collect::<Vec<_>>();

    let res = flatten_validation_errors(res);

    if res.is_empty() {
        Ok(results)
    } else if res.len() == 1 {
        if let Some(e) = res.into_iter().next() {
            return Err(e);
        }
        panic!("Should never be reached")
    } else {
        Err(PlatformValidationError::MultiplePlatformValidationErrorsFound { errors: res })
    }
}

fn flatten_validation_errors(v: Vec<PlatformValidationError>) -> Vec<PlatformValidationError> {
    let mut res = Vec::with_capacity(v.len());

    for e in v {
        match e {
            PlatformValidationError::MultiplePlatformValidationErrorsFound { errors } => {
                let flattened = flatten_validation_errors(errors);
                res.extend(flattened);
            },
            e => res.push(e)
        }
    }

    res
}
