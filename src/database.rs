// Test db content
const DB_BYTES: &[u8] = include_bytes!("edb_data.bin");
lazy_static!{
    pub static ref DB: Database = Database::deserialize_compressed(DB_BYTES).unwrap();
}

// Table row pointer types
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerAdminSshKeys(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerAlert(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerAlertGroup(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerAlertTriggerTest(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerAlertmanagerInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplication(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationBackgroundJob(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationChShard(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationConfig(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationDeployment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationDeploymentIngress(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationNatsStream(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationPgShard(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendApplicationS3Bucket(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBackendHttpEndpoint(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeployment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentGroup(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentLocalFile(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentPort(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentServiceInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentServiceRegistration(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentTask(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentTaskMount(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerBlackboxDeploymentVaultSecret(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChDeployment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChDeploymentInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChDeploymentSchemas(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChKeeperDeployment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChKeeperDeploymentInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChMigration(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChMutator(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChMutatorTest(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChNatsStreamImport(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChQuery(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChQueryTest(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChSchema(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerChTestDataset(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerDatacenter(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerDiskKind(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerDockerImage(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerDockerImagePin(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerDockerImagePinImages(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerDockerImageSet(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerDockerRegistryInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerFrontendApplication(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerFrontendApplicationDeployment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerFrontendApplicationDeploymentIngress(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerFrontendApplicationExternalLink(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerFrontendApplicationExternalPage(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerFrontendApplicationUsedEndpoint(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerFrontendPage(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerGlobalSettings(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerGrafana(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerGrafanaDashboard(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerHttpEndpointDataType(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerHttpMethods(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerLokiCluster(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerMinioBucket(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerMinioCluster(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerMinioInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerMonitoringCluster(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerMonitoringClusterAlertGroup(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerMonitoringClusterScrapedMetric(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerMonitoringInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNatsCluster(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNatsDeploymentInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNatsJetstreamStream(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNetwork(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNetworkInterface(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNixpkgsEnvironment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNixpkgsVersion(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerNomadNamespace(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgDeployment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgDeploymentInstance(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgDeploymentSchemas(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgDeploymentUnmanagedDb(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgMatView(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgMatViewTest(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgMatViewUpdateFrequency(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgMigration(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgMutator(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgMutatorTest(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgQuery(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgQueryTest(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgSchema(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgTestDataset(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerPgTransaction(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerRegion(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerRustCompilationEnvironment(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerRustCrateVersion(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServer(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerDisk(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerKind(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerKindAttribute(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerLabel(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerRootVolume(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerVolume(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerVolumeUsageContract(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerXfsVolume(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerZfsDataset(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerZpool(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerZpoolCache(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerZpoolLog(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerZpoolSpare(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerZpoolVdev(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerServerZpoolVdevDisk(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerSubnetRouterFloatingIp(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerTelegramBot(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerTelegramChannel(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerTempoCluster(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerTld(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerUniqueApplicationNames(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerUniqueDeploymentNames(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerValidServerLabels(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerVersionedType(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerVersionedTypeMigration(usize);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, ::std::hash::Hash, serde::Deserialize)]
pub struct TableRowPointerVersionedTypeSnapshot(usize);


// Table struct types
#[derive(Debug)]
pub struct TableRowAdminSshKeys {
    pub contents: ::std::string::String,
}

#[derive(Debug)]
pub struct TableRowAlert {
    pub alert_name: ::std::string::String,
    pub expr: ::std::string::String,
    pub description: ::std::string::String,
    pub for_time: ::std::string::String,
    pub severity: i64,
    pub parent: TableRowPointerAlertGroup,
    pub children_alert_trigger_test: Vec<TableRowPointerAlertTriggerTest>,
}

#[derive(Debug)]
pub struct TableRowAlertGroup {
    pub alert_group_name: ::std::string::String,
    pub children_alert: Vec<TableRowPointerAlert>,
    pub referrers_monitoring_cluster_alert_group__alert_group_name: Vec<TableRowPointerMonitoringClusterAlertGroup>,
}

#[derive(Debug)]
pub struct TableRowAlertTriggerTest {
    pub expected_message: ::std::string::String,
    pub expected_labels: ::std::string::String,
    pub eval_time: ::std::string::String,
    pub interval: ::std::string::String,
    pub input_series: ::std::string::String,
    pub parent: TableRowPointerAlert,
}

#[derive(Debug)]
pub struct TableRowAlertmanagerInstance {
    pub instance_id: i64,
    pub alertmanager_server: TableRowPointerServerVolume,
    pub parent: TableRowPointerMonitoringCluster,
}

#[derive(Debug)]
pub struct TableRowBackendApplication {
    pub application_name: ::std::string::String,
    pub build_environment: TableRowPointerRustCompilationEnvironment,
    pub children_backend_application_background_job: Vec<TableRowPointerBackendApplicationBackgroundJob>,
    pub children_backend_application_config: Vec<TableRowPointerBackendApplicationConfig>,
    pub children_backend_application_s3_bucket: Vec<TableRowPointerBackendApplicationS3Bucket>,
    pub children_backend_application_pg_shard: Vec<TableRowPointerBackendApplicationPgShard>,
    pub children_backend_application_ch_shard: Vec<TableRowPointerBackendApplicationChShard>,
    pub children_backend_application_nats_stream: Vec<TableRowPointerBackendApplicationNatsStream>,
    pub children_backend_http_endpoint: Vec<TableRowPointerBackendHttpEndpoint>,
    pub referrers_backend_application_deployment__application_name: Vec<TableRowPointerBackendApplicationDeployment>,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationBackgroundJob {
    pub job_name: ::std::string::String,
    pub parent: TableRowPointerBackendApplication,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationChShard {
    pub shard_name: ::std::string::String,
    pub ch_schema: TableRowPointerChSchema,
    pub used_queries: ::std::string::String,
    pub used_inserters: ::std::string::String,
    pub used_mutators: ::std::string::String,
    pub parent: TableRowPointerBackendApplication,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationConfig {
    pub config_name: ::std::string::String,
    pub config_type: ::std::string::String,
    pub default_value: ::std::string::String,
    pub min_value: ::std::string::String,
    pub max_value: ::std::string::String,
    pub regex_check: ::std::string::String,
    pub parent: TableRowPointerBackendApplication,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationDeployment {
    pub deployment_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub application_name: TableRowPointerBackendApplication,
    pub workload_architecture: ::std::string::String,
    pub count: i64,
    pub placement: ::std::string::String,
    pub pg_shard_wiring: ::std::string::String,
    pub ch_shard_wiring: ::std::string::String,
    pub nats_stream_wiring: ::std::string::String,
    pub s3_bucket_wiring: ::std::string::String,
    pub config: ::std::string::String,
    pub http_port: i64,
    pub memory_mb: i64,
    pub region: TableRowPointerRegion,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub tracing_cluster: ::std::string::String,
    pub referrers_backend_application_deployment_ingress__deployment: Vec<TableRowPointerBackendApplicationDeploymentIngress>,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationDeploymentIngress {
    pub deployment: TableRowPointerBackendApplicationDeployment,
    pub mountpoint: ::std::string::String,
    pub subdomain: ::std::string::String,
    pub tld: TableRowPointerTld,
    pub endpoint_list: ::std::string::String,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationNatsStream {
    pub stream_name: ::std::string::String,
    pub stream_type: TableRowPointerVersionedType,
    pub enable_consumer: bool,
    pub enable_producer: bool,
    pub is_batch_consumer: bool,
    pub enable_subjects: bool,
    pub parent: TableRowPointerBackendApplication,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationPgShard {
    pub shard_name: ::std::string::String,
    pub pg_schema: TableRowPointerPgSchema,
    pub used_queries: ::std::string::String,
    pub used_mutators: ::std::string::String,
    pub used_transactions: ::std::string::String,
    pub parent: TableRowPointerBackendApplication,
}

#[derive(Debug)]
pub struct TableRowBackendApplicationS3Bucket {
    pub bucket_name: ::std::string::String,
    pub parent: TableRowPointerBackendApplication,
}

#[derive(Debug)]
pub struct TableRowBackendHttpEndpoint {
    pub http_endpoint_name: ::std::string::String,
    pub path: ::std::string::String,
    pub http_method: TableRowPointerHttpMethods,
    pub input_body_type: ::std::string::String,
    pub output_body_type: ::std::string::String,
    pub data_type: TableRowPointerHttpEndpointDataType,
    pub max_input_body_size_bytes: i64,
    pub needs_headers: bool,
    pub receive_body_as_stream: bool,
    pub parent: TableRowPointerBackendApplication,
    pub referrers_frontend_application_used_endpoint__backend_endpoint: Vec<TableRowPointerFrontendApplicationUsedEndpoint>,
    pub referrers_frontend_application_external_link__backend_endpoint: Vec<TableRowPointerFrontendApplicationExternalLink>,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeployment {
    pub deployment_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub children_blackbox_deployment_group: Vec<TableRowPointerBlackboxDeploymentGroup>,
    pub children_blackbox_deployment_service_registration: Vec<TableRowPointerBlackboxDeploymentServiceRegistration>,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentGroup {
    pub group_name: ::std::string::String,
    pub count: i64,
    pub workload_architecture: ::std::string::String,
    pub placement: ::std::string::String,
    pub parent: TableRowPointerBlackboxDeployment,
    pub children_blackbox_deployment_port: Vec<TableRowPointerBlackboxDeploymentPort>,
    pub children_blackbox_deployment_task: Vec<TableRowPointerBlackboxDeploymentTask>,
    pub children_blackbox_deployment_service_instance: Vec<TableRowPointerBlackboxDeploymentServiceInstance>,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentLocalFile {
    pub local_file_name: ::std::string::String,
    pub local_file_contents: ::std::string::String,
    pub mode: ::std::string::String,
    pub parent: TableRowPointerBlackboxDeploymentTask,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentPort {
    pub port: i64,
    pub port_description: ::std::string::String,
    pub protocol: ::std::string::String,
    pub parent: TableRowPointerBlackboxDeploymentGroup,
    pub referrers_blackbox_deployment_service_instance__port: Vec<TableRowPointerBlackboxDeploymentServiceInstance>,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentServiceInstance {
    pub service_registration: TableRowPointerBlackboxDeploymentServiceRegistration,
    pub port: TableRowPointerBlackboxDeploymentPort,
    pub parent: TableRowPointerBlackboxDeploymentGroup,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentServiceRegistration {
    pub service_name: ::std::string::String,
    pub scrape_prometheus_metrics: bool,
    pub prometheus_metrics_path: ::std::string::String,
    pub min_instances: i64,
    pub parent: TableRowPointerBlackboxDeployment,
    pub referrers_blackbox_deployment_service_instance__service_registration: Vec<TableRowPointerBlackboxDeploymentServiceInstance>,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentTask {
    pub task_name: ::std::string::String,
    pub docker_image: TableRowPointerDockerImagePin,
    pub docker_image_set: TableRowPointerDockerImageSet,
    pub memory_mb: i64,
    pub memory_oversubscription_mb: i64,
    pub entrypoint: ::std::string::String,
    pub args: ::std::string::String,
    pub parent: TableRowPointerBlackboxDeploymentGroup,
    pub children_blackbox_deployment_task_mount: Vec<TableRowPointerBlackboxDeploymentTaskMount>,
    pub children_blackbox_deployment_vault_secret: Vec<TableRowPointerBlackboxDeploymentVaultSecret>,
    pub children_blackbox_deployment_local_file: Vec<TableRowPointerBlackboxDeploymentLocalFile>,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentTaskMount {
    pub target_path: ::std::string::String,
    pub server_volume: TableRowPointerServerVolume,
    pub parent: TableRowPointerBlackboxDeploymentTask,
}

#[derive(Debug)]
pub struct TableRowBlackboxDeploymentVaultSecret {
    pub secret_name: ::std::string::String,
    pub target_file_name: ::std::string::String,
    pub target_env_var_name: ::std::string::String,
    pub parent: TableRowPointerBlackboxDeploymentTask,
}

#[derive(Debug)]
pub struct TableRowChDeployment {
    pub deployment_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub docker_image: TableRowPointerDockerImagePin,
    pub workload_architecture: ::std::string::String,
    pub keeper: TableRowPointerChKeeperDeployment,
    pub extra_memory_mb: i64,
    pub mark_cache_size_mb: i64,
    pub index_mark_cache_size_mb: i64,
    pub uncompressed_cache_size_mb: i64,
    pub compiled_expression_cache_size_mb: i64,
    pub query_cache_size_mb: i64,
    pub max_thread_pool_size: i64,
    pub max_concurrent_queries: i64,
    pub merge_max_block_size: i64,
    pub max_bytes_to_merge_at_max_space_in_pool_mb: i64,
    pub max_query_execution_time_seconds: i64,
    pub queue_max_wait_ms: i64,
    pub distribute_over_dcs: bool,
    pub native_port: i64,
    pub http_port: i64,
    pub replication_port: i64,
    pub prometheus_port: i64,
    pub children_ch_deployment_instance: Vec<TableRowPointerChDeploymentInstance>,
    pub children_ch_deployment_schemas: Vec<TableRowPointerChDeploymentSchemas>,
}

#[derive(Debug)]
pub struct TableRowChDeploymentInstance {
    pub instance_id: i64,
    pub ch_server: TableRowPointerServerVolume,
    pub parent: TableRowPointerChDeployment,
}

#[derive(Debug)]
pub struct TableRowChDeploymentSchemas {
    pub db_name: ::std::string::String,
    pub ch_schema: TableRowPointerChSchema,
    pub parent: TableRowPointerChDeployment,
    pub children_ch_nats_stream_import: Vec<TableRowPointerChNatsStreamImport>,
}

#[derive(Debug)]
pub struct TableRowChKeeperDeployment {
    pub deployment_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub docker_image: TableRowPointerDockerImagePin,
    pub workload_architecture: ::std::string::String,
    pub distribute_over_dcs: bool,
    pub memory_mb: i64,
    pub keeper_port: i64,
    pub raft_port: i64,
    pub prometheus_port: i64,
    pub children_ch_keeper_deployment_instance: Vec<TableRowPointerChKeeperDeploymentInstance>,
    pub referrers_ch_deployment__keeper: Vec<TableRowPointerChDeployment>,
}

#[derive(Debug)]
pub struct TableRowChKeeperDeploymentInstance {
    pub instance_id: i64,
    pub keeper_server: TableRowPointerServerVolume,
    pub parent: TableRowPointerChKeeperDeployment,
}

#[derive(Debug)]
pub struct TableRowChMigration {
    pub time: i64,
    pub upgrade: ::std::string::String,
    pub downgrade: ::std::string::String,
    pub needs_admin: bool,
    pub parent: TableRowPointerChSchema,
}

#[derive(Debug)]
pub struct TableRowChMutator {
    pub mutator_name: ::std::string::String,
    pub mutator_expression: ::std::string::String,
    pub parent: TableRowPointerChSchema,
    pub children_ch_mutator_test: Vec<TableRowPointerChMutatorTest>,
}

#[derive(Debug)]
pub struct TableRowChMutatorTest {
    pub arguments: ::std::string::String,
    pub test_dataset: TableRowPointerChTestDataset,
    pub resulting_data: ::std::string::String,
    pub parent: TableRowPointerChMutator,
}

#[derive(Debug)]
pub struct TableRowChNatsStreamImport {
    pub consumer_name: ::std::string::String,
    pub into_table: ::std::string::String,
    pub stream: TableRowPointerNatsJetstreamStream,
    pub parent: TableRowPointerChDeploymentSchemas,
}

#[derive(Debug)]
pub struct TableRowChQuery {
    pub query_name: ::std::string::String,
    pub query_expression: ::std::string::String,
    pub opt_fields: ::std::string::String,
    pub parent: TableRowPointerChSchema,
    pub children_ch_query_test: Vec<TableRowPointerChQueryTest>,
}

#[derive(Debug)]
pub struct TableRowChQueryTest {
    pub arguments: ::std::string::String,
    pub outputs: ::std::string::String,
    pub test_dataset: TableRowPointerChTestDataset,
    pub parent: TableRowPointerChQuery,
}

#[derive(Debug)]
pub struct TableRowChSchema {
    pub schema_name: ::std::string::String,
    pub children_ch_migration: Vec<TableRowPointerChMigration>,
    pub children_ch_query: Vec<TableRowPointerChQuery>,
    pub children_ch_mutator: Vec<TableRowPointerChMutator>,
    pub children_ch_test_dataset: Vec<TableRowPointerChTestDataset>,
    pub referrers_ch_deployment_schemas__ch_schema: Vec<TableRowPointerChDeploymentSchemas>,
    pub referrers_backend_application_ch_shard__ch_schema: Vec<TableRowPointerBackendApplicationChShard>,
}

#[derive(Debug)]
pub struct TableRowChTestDataset {
    pub dataset_name: ::std::string::String,
    pub dataset_contents: ::std::string::String,
    pub min_time: i64,
    pub parent: TableRowPointerChSchema,
    pub referrers_ch_query_test__test_dataset: Vec<TableRowPointerChQueryTest>,
    pub referrers_ch_mutator_test__test_dataset: Vec<TableRowPointerChMutatorTest>,
}

#[derive(Debug)]
pub struct TableRowDatacenter {
    pub dc_name: ::std::string::String,
    pub region: TableRowPointerRegion,
    pub network_cidr: ::std::string::String,
    pub allow_small_subnets: bool,
    pub implementation: ::std::string::String,
    pub implementation_settings: ::std::string::String,
    pub default_server_kind: TableRowPointerServerKind,
    pub disk_ids_policy: ::std::string::String,
    pub router_subnet_vlan_id: i64,
    pub referrers_server__dc: Vec<TableRowPointerServer>,
}

#[derive(Debug)]
pub struct TableRowDiskKind {
    pub kind: ::std::string::String,
    pub medium: ::std::string::String,
    pub is_elastic: bool,
    pub min_capacity_bytes: i64,
    pub max_capacity_bytes: i64,
    pub capacity_bytes: i64,
    pub has_extra_config: bool,
    pub non_eligible_reason: ::std::string::String,
    pub referrers_server_disk__disk_kind: Vec<TableRowPointerServerDisk>,
}

#[derive(Debug)]
pub struct TableRowDockerImage {
    pub checksum: ::std::string::String,
    pub image_set: TableRowPointerDockerImageSet,
    pub repository: ::std::string::String,
    pub architecture: ::std::string::String,
    pub tag: ::std::string::String,
    pub referrers_docker_image_pin_images__checksum: Vec<TableRowPointerDockerImagePinImages>,
}

#[derive(Debug)]
pub struct TableRowDockerImagePin {
    pub pin_name: ::std::string::String,
    pub children_docker_image_pin_images: Vec<TableRowPointerDockerImagePinImages>,
    pub referrers_region__docker_image_external_lb: Vec<TableRowPointerRegion>,
    pub referrers_docker_registry_instance__docker_image: Vec<TableRowPointerDockerRegistryInstance>,
    pub referrers_pg_deployment__docker_image_pg: Vec<TableRowPointerPgDeployment>,
    pub referrers_pg_deployment__docker_image_haproxy: Vec<TableRowPointerPgDeployment>,
    pub referrers_pg_deployment__docker_image_pg_exporter: Vec<TableRowPointerPgDeployment>,
    pub referrers_ch_deployment__docker_image: Vec<TableRowPointerChDeployment>,
    pub referrers_ch_keeper_deployment__docker_image: Vec<TableRowPointerChKeeperDeployment>,
    pub referrers_nats_cluster__docker_image_nats: Vec<TableRowPointerNatsCluster>,
    pub referrers_nats_cluster__docker_image_nats_exporter: Vec<TableRowPointerNatsCluster>,
    pub referrers_minio_cluster__docker_image_minio: Vec<TableRowPointerMinioCluster>,
    pub referrers_minio_cluster__docker_image_minio_mc: Vec<TableRowPointerMinioCluster>,
    pub referrers_minio_cluster__docker_image_nginx: Vec<TableRowPointerMinioCluster>,
    pub referrers_monitoring_cluster__docker_image_prometheus: Vec<TableRowPointerMonitoringCluster>,
    pub referrers_monitoring_cluster__docker_image_alertmanager: Vec<TableRowPointerMonitoringCluster>,
    pub referrers_monitoring_cluster__docker_image_victoriametrics: Vec<TableRowPointerMonitoringCluster>,
    pub referrers_grafana__docker_image_grafana: Vec<TableRowPointerGrafana>,
    pub referrers_grafana__docker_image_promxy: Vec<TableRowPointerGrafana>,
    pub referrers_loki_cluster__docker_image_loki: Vec<TableRowPointerLokiCluster>,
    pub referrers_tempo_cluster__docker_image: Vec<TableRowPointerTempoCluster>,
    pub referrers_blackbox_deployment_task__docker_image: Vec<TableRowPointerBlackboxDeploymentTask>,
}

#[derive(Debug)]
pub struct TableRowDockerImagePinImages {
    pub checksum: TableRowPointerDockerImage,
    pub parent: TableRowPointerDockerImagePin,
}

#[derive(Debug)]
pub struct TableRowDockerImageSet {
    pub set_name: ::std::string::String,
    pub referrers_docker_image__image_set: Vec<TableRowPointerDockerImage>,
    pub referrers_blackbox_deployment_task__docker_image_set: Vec<TableRowPointerBlackboxDeploymentTask>,
}

#[derive(Debug)]
pub struct TableRowDockerRegistryInstance {
    pub region: TableRowPointerRegion,
    pub minio_bucket: TableRowPointerMinioBucket,
    pub memory_mb: i64,
    pub docker_image: TableRowPointerDockerImagePin,
}

#[derive(Debug)]
pub struct TableRowFrontendApplication {
    pub application_name: ::std::string::String,
    pub build_environment: TableRowPointerRustCompilationEnvironment,
    pub index_page_title: ::std::string::String,
    pub children_frontend_page: Vec<TableRowPointerFrontendPage>,
    pub children_frontend_application_used_endpoint: Vec<TableRowPointerFrontendApplicationUsedEndpoint>,
    pub children_frontend_application_external_link: Vec<TableRowPointerFrontendApplicationExternalLink>,
    pub children_frontend_application_external_page: Vec<TableRowPointerFrontendApplicationExternalPage>,
    pub referrers_frontend_application_deployment__application_name: Vec<TableRowPointerFrontendApplicationDeployment>,
}

#[derive(Debug)]
pub struct TableRowFrontendApplicationDeployment {
    pub deployment_name: ::std::string::String,
    pub application_name: TableRowPointerFrontendApplication,
    pub namespace: TableRowPointerNomadNamespace,
    pub explicit_endpoint_wiring: ::std::string::String,
    pub workload_backend_architecture: ::std::string::String,
    pub placement: ::std::string::String,
    pub link_wiring: ::std::string::String,
    pub page_wiring: ::std::string::String,
    pub count: i64,
    pub http_port: i64,
    pub memory_mb: i64,
    pub region: TableRowPointerRegion,
    pub loki_cluster: ::std::string::String,
    pub referrers_frontend_application_deployment_ingress__deployment: Vec<TableRowPointerFrontendApplicationDeploymentIngress>,
}

#[derive(Debug)]
pub struct TableRowFrontendApplicationDeploymentIngress {
    pub deployment: TableRowPointerFrontendApplicationDeployment,
    pub mountpoint: ::std::string::String,
    pub subdomain: ::std::string::String,
    pub tld: TableRowPointerTld,
}

#[derive(Debug)]
pub struct TableRowFrontendApplicationExternalLink {
    pub link_name: ::std::string::String,
    pub backend_endpoint: TableRowPointerBackendHttpEndpoint,
    pub parent: TableRowPointerFrontendApplication,
}

#[derive(Debug)]
pub struct TableRowFrontendApplicationExternalPage {
    pub link_name: ::std::string::String,
    pub frontend_page: TableRowPointerFrontendPage,
    pub parent: TableRowPointerFrontendApplication,
}

#[derive(Debug)]
pub struct TableRowFrontendApplicationUsedEndpoint {
    pub endpoint_name: ::std::string::String,
    pub backend_endpoint: TableRowPointerBackendHttpEndpoint,
    pub parent: TableRowPointerFrontendApplication,
}

#[derive(Debug)]
pub struct TableRowFrontendPage {
    pub page_name: ::std::string::String,
    pub path: ::std::string::String,
    pub parent: TableRowPointerFrontendApplication,
    pub referrers_frontend_application_external_page__frontend_page: Vec<TableRowPointerFrontendApplicationExternalPage>,
}

#[derive(Debug)]
pub struct TableRowGlobalSettings {
    pub project_name: ::std::string::String,
    pub docker_registry_port: i64,
    pub docker_registry_service_name: ::std::string::String,
    pub aws_artefacts_s3_bucket_name: ::std::string::String,
    pub local_docker_cache_port: i64,
    pub admin_email: ::std::string::String,
    pub google_cloud_project_id: ::std::string::String,
    pub google_cloud_artefacts_bucket_name: ::std::string::String,
    pub disable_consul_quorum_tests: bool,
    pub disable_nomad_quorum_tests: bool,
    pub disable_vault_quorum_tests: bool,
    pub disable_dns_quorum_tests: bool,
    pub disable_deployment_min_server_tests: bool,
    pub disable_deployment_min_ingress_tests: bool,
    pub disable_region_docker_registry_tests: bool,
    pub disable_region_monitoring_tests: bool,
    pub disable_region_tracing_tests: bool,
    pub disable_region_logging_tests: bool,
    pub disable_vpn_gateway_tests: bool,
    pub hetzner_inter_dc_vlan_id: i64,
    pub experimental_enable_arm64_support: bool,
    pub update_edl_public_ips_from_terraform: bool,
    pub enable_ipv6: bool,
    pub force_ipv6: bool,
}

#[derive(Debug)]
pub struct TableRowGrafana {
    pub deployment_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub placement: ::std::string::String,
    pub workload_architecture: ::std::string::String,
    pub docker_image_grafana: TableRowPointerDockerImagePin,
    pub docker_image_promxy: TableRowPointerDockerImagePin,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub port: i64,
    pub promxy_port: i64,
    pub instance_count: i64,
    pub database: TableRowPointerPgDeploymentUnmanagedDb,
    pub memory_mb: i64,
    pub promxy_memory_mb: i64,
}

#[derive(Debug)]
pub struct TableRowGrafanaDashboard {
    pub filename: ::std::string::String,
    pub contents: ::std::string::String,
}

#[derive(Debug)]
pub struct TableRowHttpEndpointDataType {
    pub http_endpoint_data_type: ::std::string::String,
    pub referrers_backend_http_endpoint__data_type: Vec<TableRowPointerBackendHttpEndpoint>,
}

#[derive(Debug)]
pub struct TableRowHttpMethods {
    pub http_method_name: ::std::string::String,
    pub referrers_backend_http_endpoint__http_method: Vec<TableRowPointerBackendHttpEndpoint>,
}

#[derive(Debug)]
pub struct TableRowLokiCluster {
    pub cluster_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub workload_architecture: ::std::string::String,
    pub docker_image_loki: TableRowPointerDockerImagePin,
    pub is_region_default: bool,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub storage_bucket: TableRowPointerMinioBucket,
    pub retention_period_days: i64,
    pub loki_writer_http_port: i64,
    pub loki_writer_grpc_port: i64,
    pub loki_reader_http_port: i64,
    pub loki_reader_grpc_port: i64,
    pub loki_backend_http_port: i64,
    pub loki_backend_grpc_port: i64,
    pub loki_writers: i64,
    pub loki_readers: i64,
    pub writer_placement: ::std::string::String,
    pub reader_placement: ::std::string::String,
    pub backend_placement: ::std::string::String,
    pub loki_reader_memory_mb: i64,
    pub loki_writer_memory_mb: i64,
    pub loki_backend_memory_mb: i64,
}

#[derive(Debug)]
pub struct TableRowMinioBucket {
    pub bucket_name: ::std::string::String,
    pub locking_enabled: bool,
    pub parent: TableRowPointerMinioCluster,
    pub referrers_docker_registry_instance__minio_bucket: Vec<TableRowPointerDockerRegistryInstance>,
    pub referrers_loki_cluster__storage_bucket: Vec<TableRowPointerLokiCluster>,
    pub referrers_tempo_cluster__storage_bucket: Vec<TableRowPointerTempoCluster>,
}

#[derive(Debug)]
pub struct TableRowMinioCluster {
    pub cluster_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub workload_architecture: ::std::string::String,
    pub docker_image_minio: TableRowPointerDockerImagePin,
    pub docker_image_minio_mc: TableRowPointerDockerImagePin,
    pub docker_image_nginx: TableRowPointerDockerImagePin,
    pub api_port: i64,
    pub console_port: i64,
    pub lb_port: i64,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub expected_zfs_recordsize: ::std::string::String,
    pub distribute_over_dcs: bool,
    pub instance_memory_mb: i64,
    pub lb_memory_mb: i64,
    pub consul_service_name: ::std::string::String,
    pub children_minio_instance: Vec<TableRowPointerMinioInstance>,
    pub children_minio_bucket: Vec<TableRowPointerMinioBucket>,
}

#[derive(Debug)]
pub struct TableRowMinioInstance {
    pub instance_id: i64,
    pub instance_volume: TableRowPointerServerVolume,
    pub parent: TableRowPointerMinioCluster,
}

#[derive(Debug)]
pub struct TableRowMonitoringCluster {
    pub cluster_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub is_region_default: bool,
    pub workload_architecture: ::std::string::String,
    pub docker_image_prometheus: TableRowPointerDockerImagePin,
    pub docker_image_alertmanager: TableRowPointerDockerImagePin,
    pub docker_image_victoriametrics: TableRowPointerDockerImagePin,
    pub prometheus_memory_mb: i64,
    pub victoriametrics_memory_mb: i64,
    pub alertmanager_memory_mb: i64,
    pub prometheus_port: i64,
    pub victoriametrics_port: i64,
    pub alertmanager_port: i64,
    pub alertmanager_p2p_port: i64,
    pub victoriametrics_retention_months: i64,
    pub children_monitoring_cluster_scraped_metric: Vec<TableRowPointerMonitoringClusterScrapedMetric>,
    pub children_monitoring_cluster_alert_group: Vec<TableRowPointerMonitoringClusterAlertGroup>,
    pub children_monitoring_instance: Vec<TableRowPointerMonitoringInstance>,
    pub children_alertmanager_instance: Vec<TableRowPointerAlertmanagerInstance>,
}

#[derive(Debug)]
pub struct TableRowMonitoringClusterAlertGroup {
    pub alert_group_name: TableRowPointerAlertGroup,
    pub telegram_channel: TableRowPointerTelegramChannel,
    pub telegram_bot: TableRowPointerTelegramBot,
    pub parent: TableRowPointerMonitoringCluster,
}

#[derive(Debug)]
pub struct TableRowMonitoringClusterScrapedMetric {
    pub metric_name: ::std::string::String,
    pub expression: ::std::string::String,
    pub parent: TableRowPointerMonitoringCluster,
}

#[derive(Debug)]
pub struct TableRowMonitoringInstance {
    pub instance_id: i64,
    pub monitoring_server: TableRowPointerServerVolume,
    pub parent: TableRowPointerMonitoringCluster,
}

#[derive(Debug)]
pub struct TableRowNatsCluster {
    pub cluster_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub distribute_over_dcs: bool,
    pub workload_architecture: ::std::string::String,
    pub docker_image_nats: TableRowPointerDockerImagePin,
    pub docker_image_nats_exporter: TableRowPointerDockerImagePin,
    pub nats_clients_port: i64,
    pub nats_cluster_port: i64,
    pub nats_http_mon_port: i64,
    pub nats_prometheus_port: i64,
    pub instance_memory_mb: i64,
    pub children_nats_jetstream_stream: Vec<TableRowPointerNatsJetstreamStream>,
    pub children_nats_deployment_instance: Vec<TableRowPointerNatsDeploymentInstance>,
}

#[derive(Debug)]
pub struct TableRowNatsDeploymentInstance {
    pub instance_id: i64,
    pub nats_server: TableRowPointerServerVolume,
    pub parent: TableRowPointerNatsCluster,
}

#[derive(Debug)]
pub struct TableRowNatsJetstreamStream {
    pub stream_name: ::std::string::String,
    pub stream_type: TableRowPointerVersionedType,
    pub max_bytes: i64,
    pub max_msg_size: i64,
    pub enable_subjects: bool,
    pub parent: TableRowPointerNatsCluster,
    pub referrers_ch_nats_stream_import__stream: Vec<TableRowPointerChNatsStreamImport>,
}

#[derive(Debug)]
pub struct TableRowNetwork {
    pub network_name: ::std::string::String,
    pub cidr: ::std::string::String,
    pub referrers_network_interface__if_network: Vec<TableRowPointerNetworkInterface>,
}

#[derive(Debug)]
pub struct TableRowNetworkInterface {
    pub if_name: ::std::string::String,
    pub if_network: TableRowPointerNetwork,
    pub if_ip: ::std::string::String,
    pub if_prefix: i64,
    pub if_vlan: i64,
    pub parent: TableRowPointerServer,
    pub referrers_server__ssh_interface: Vec<TableRowPointerServer>,
}

#[derive(Debug)]
pub struct TableRowNixpkgsEnvironment {
    pub name: ::std::string::String,
    pub version: TableRowPointerNixpkgsVersion,
    pub referrers_server__nixpkgs_environment: Vec<TableRowPointerServer>,
    pub referrers_rust_compilation_environment__nixpkgs_environment: Vec<TableRowPointerRustCompilationEnvironment>,
}

#[derive(Debug)]
pub struct TableRowNixpkgsVersion {
    pub version: ::std::string::String,
    pub checksum: ::std::string::String,
    pub tarball_checksum: ::std::string::String,
    pub referrers_nixpkgs_environment__version: Vec<TableRowPointerNixpkgsEnvironment>,
}

#[derive(Debug)]
pub struct TableRowNomadNamespace {
    pub namespace: ::std::string::String,
    pub description: ::std::string::String,
    pub referrers_pg_deployment__namespace: Vec<TableRowPointerPgDeployment>,
    pub referrers_ch_deployment__namespace: Vec<TableRowPointerChDeployment>,
    pub referrers_ch_keeper_deployment__namespace: Vec<TableRowPointerChKeeperDeployment>,
    pub referrers_nats_cluster__namespace: Vec<TableRowPointerNatsCluster>,
    pub referrers_backend_application_deployment__namespace: Vec<TableRowPointerBackendApplicationDeployment>,
    pub referrers_frontend_application_deployment__namespace: Vec<TableRowPointerFrontendApplicationDeployment>,
    pub referrers_minio_cluster__namespace: Vec<TableRowPointerMinioCluster>,
    pub referrers_monitoring_cluster__namespace: Vec<TableRowPointerMonitoringCluster>,
    pub referrers_grafana__namespace: Vec<TableRowPointerGrafana>,
    pub referrers_loki_cluster__namespace: Vec<TableRowPointerLokiCluster>,
    pub referrers_tempo_cluster__namespace: Vec<TableRowPointerTempoCluster>,
    pub referrers_blackbox_deployment__namespace: Vec<TableRowPointerBlackboxDeployment>,
}

#[derive(Debug)]
pub struct TableRowPgDeployment {
    pub deployment_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub docker_image_pg: TableRowPointerDockerImagePin,
    pub docker_image_haproxy: TableRowPointerDockerImagePin,
    pub docker_image_pg_exporter: TableRowPointerDockerImagePin,
    pub workload_architecture: ::std::string::String,
    pub distribute_over_dcs: bool,
    pub synchronous_replication: bool,
    pub shared_buffers_mb: i64,
    pub work_mem_mb: i64,
    pub maintenance_work_mem_mb: i64,
    pub overhead_mem_mb: i64,
    pub max_connections: i64,
    pub replica_rolling_update_delay_seconds: i64,
    pub instance_pg_port: i64,
    pub instance_pg_master_port: i64,
    pub instance_pg_slave_port: i64,
    pub instance_patroni_port: i64,
    pub instance_haproxy_metrics_port: i64,
    pub instance_pg_exporter_port: i64,
    pub children_pg_deployment_schemas: Vec<TableRowPointerPgDeploymentSchemas>,
    pub children_pg_deployment_unmanaged_db: Vec<TableRowPointerPgDeploymentUnmanagedDb>,
    pub children_pg_deployment_instance: Vec<TableRowPointerPgDeploymentInstance>,
}

#[derive(Debug)]
pub struct TableRowPgDeploymentInstance {
    pub instance_id: i64,
    pub pg_server: TableRowPointerServerVolume,
    pub parent: TableRowPointerPgDeployment,
}

#[derive(Debug)]
pub struct TableRowPgDeploymentSchemas {
    pub db_name: ::std::string::String,
    pub pg_schema: TableRowPointerPgSchema,
    pub parent: TableRowPointerPgDeployment,
}

#[derive(Debug)]
pub struct TableRowPgDeploymentUnmanagedDb {
    pub db_name: ::std::string::String,
    pub parent: TableRowPointerPgDeployment,
    pub referrers_grafana__database: Vec<TableRowPointerGrafana>,
}

#[derive(Debug)]
pub struct TableRowPgMatView {
    pub mview_name: ::std::string::String,
    pub update_frequency: TableRowPointerPgMatViewUpdateFrequency,
    pub parent: TableRowPointerPgSchema,
    pub children_pg_mat_view_test: Vec<TableRowPointerPgMatViewTest>,
}

#[derive(Debug)]
pub struct TableRowPgMatViewTest {
    pub expected_data: ::std::string::String,
    pub test_dataset: TableRowPointerPgTestDataset,
    pub parent: TableRowPointerPgMatView,
}

#[derive(Debug)]
pub struct TableRowPgMatViewUpdateFrequency {
    pub frequency: ::std::string::String,
    pub referrers_pg_mat_view__update_frequency: Vec<TableRowPointerPgMatView>,
}

#[derive(Debug)]
pub struct TableRowPgMigration {
    pub time: i64,
    pub upgrade: ::std::string::String,
    pub downgrade: ::std::string::String,
    pub needs_admin: bool,
    pub parent: TableRowPointerPgSchema,
}

#[derive(Debug)]
pub struct TableRowPgMutator {
    pub mutator_name: ::std::string::String,
    pub mutator_expression: ::std::string::String,
    pub seqscan_ok: bool,
    pub parent: TableRowPointerPgSchema,
    pub children_pg_mutator_test: Vec<TableRowPointerPgMutatorTest>,
}

#[derive(Debug)]
pub struct TableRowPgMutatorTest {
    pub arguments: ::std::string::String,
    pub test_dataset: TableRowPointerPgTestDataset,
    pub resulting_data: ::std::string::String,
    pub parent: TableRowPointerPgMutator,
}

#[derive(Debug)]
pub struct TableRowPgQuery {
    pub query_name: ::std::string::String,
    pub query_expression: ::std::string::String,
    pub is_mutating: bool,
    pub seqscan_ok: bool,
    pub opt_fields: ::std::string::String,
    pub parent: TableRowPointerPgSchema,
    pub children_pg_query_test: Vec<TableRowPointerPgQueryTest>,
}

#[derive(Debug)]
pub struct TableRowPgQueryTest {
    pub arguments: ::std::string::String,
    pub outputs: ::std::string::String,
    pub test_dataset: TableRowPointerPgTestDataset,
    pub parent: TableRowPointerPgQuery,
}

#[derive(Debug)]
pub struct TableRowPgSchema {
    pub schema_name: ::std::string::String,
    pub children_pg_migration: Vec<TableRowPointerPgMigration>,
    pub children_pg_query: Vec<TableRowPointerPgQuery>,
    pub children_pg_mutator: Vec<TableRowPointerPgMutator>,
    pub children_pg_transaction: Vec<TableRowPointerPgTransaction>,
    pub children_pg_mat_view: Vec<TableRowPointerPgMatView>,
    pub children_pg_test_dataset: Vec<TableRowPointerPgTestDataset>,
    pub referrers_pg_deployment_schemas__pg_schema: Vec<TableRowPointerPgDeploymentSchemas>,
    pub referrers_backend_application_pg_shard__pg_schema: Vec<TableRowPointerBackendApplicationPgShard>,
}

#[derive(Debug)]
pub struct TableRowPgTestDataset {
    pub dataset_name: ::std::string::String,
    pub dataset_contents: ::std::string::String,
    pub min_time: i64,
    pub parent: TableRowPointerPgSchema,
    pub referrers_pg_query_test__test_dataset: Vec<TableRowPointerPgQueryTest>,
    pub referrers_pg_mutator_test__test_dataset: Vec<TableRowPointerPgMutatorTest>,
    pub referrers_pg_mat_view_test__test_dataset: Vec<TableRowPointerPgMatViewTest>,
}

#[derive(Debug)]
pub struct TableRowPgTransaction {
    pub transaction_name: ::std::string::String,
    pub steps: ::std::string::String,
    pub is_read_only: bool,
    pub parent: TableRowPointerPgSchema,
}

#[derive(Debug)]
pub struct TableRowRegion {
    pub region_name: ::std::string::String,
    pub availability_mode: ::std::string::String,
    pub tld: TableRowPointerTld,
    pub is_dns_master: bool,
    pub is_dns_slave: bool,
    pub has_coprocessor_dc: bool,
    pub docker_image_external_lb: TableRowPointerDockerImagePin,
    pub nomad_disable_log_collection: bool,
    pub referrers_datacenter__region: Vec<TableRowPointerDatacenter>,
    pub referrers_docker_registry_instance__region: Vec<TableRowPointerDockerRegistryInstance>,
    pub referrers_pg_deployment__region: Vec<TableRowPointerPgDeployment>,
    pub referrers_ch_deployment__region: Vec<TableRowPointerChDeployment>,
    pub referrers_ch_keeper_deployment__region: Vec<TableRowPointerChKeeperDeployment>,
    pub referrers_nats_cluster__region: Vec<TableRowPointerNatsCluster>,
    pub referrers_backend_application_deployment__region: Vec<TableRowPointerBackendApplicationDeployment>,
    pub referrers_frontend_application_deployment__region: Vec<TableRowPointerFrontendApplicationDeployment>,
    pub referrers_minio_cluster__region: Vec<TableRowPointerMinioCluster>,
    pub referrers_monitoring_cluster__region: Vec<TableRowPointerMonitoringCluster>,
    pub referrers_grafana__region: Vec<TableRowPointerGrafana>,
    pub referrers_loki_cluster__region: Vec<TableRowPointerLokiCluster>,
    pub referrers_tempo_cluster__region: Vec<TableRowPointerTempoCluster>,
    pub referrers_blackbox_deployment__region: Vec<TableRowPointerBlackboxDeployment>,
}

#[derive(Debug)]
pub struct TableRowRustCompilationEnvironment {
    pub env_name: ::std::string::String,
    pub rust_edition: ::std::string::String,
    pub nixpkgs_environment: TableRowPointerNixpkgsEnvironment,
    pub environment_kind: ::std::string::String,
    pub children_rust_crate_version: Vec<TableRowPointerRustCrateVersion>,
    pub referrers_backend_application__build_environment: Vec<TableRowPointerBackendApplication>,
    pub referrers_frontend_application__build_environment: Vec<TableRowPointerFrontendApplication>,
}

#[derive(Debug)]
pub struct TableRowRustCrateVersion {
    pub crate_name: ::std::string::String,
    pub version: ::std::string::String,
    pub features: ::std::string::String,
    pub default_features: bool,
    pub parent: TableRowPointerRustCompilationEnvironment,
}

#[derive(Debug)]
pub struct TableRowServer {
    pub hostname: ::std::string::String,
    pub dc: TableRowPointerDatacenter,
    pub ssh_interface: TableRowPointerNetworkInterface,
    pub root_disk: TableRowPointerServerDisk,
    pub is_consul_master: bool,
    pub is_nomad_master: bool,
    pub is_vault_instance: bool,
    pub is_dns_master: bool,
    pub is_dns_slave: bool,
    pub is_ingress: bool,
    pub is_vpn_gateway: bool,
    pub is_coprocessor_gateway: bool,
    pub is_router: bool,
    pub public_ipv6_address: ::std::string::String,
    pub public_ipv6_address_prefix: i64,
    pub kind: ::std::string::String,
    pub nixpkgs_environment: TableRowPointerNixpkgsEnvironment,
    pub run_unassigned_workloads: bool,
    pub children_server_label: Vec<TableRowPointerServerLabel>,
    pub children_server_disk: Vec<TableRowPointerServerDisk>,
    pub children_server_volume: Vec<TableRowPointerServerVolume>,
    pub children_server_root_volume: Vec<TableRowPointerServerRootVolume>,
    pub children_server_xfs_volume: Vec<TableRowPointerServerXfsVolume>,
    pub children_network_interface: Vec<TableRowPointerNetworkInterface>,
    pub children_server_zpool: Vec<TableRowPointerServerZpool>,
}

#[derive(Debug)]
pub struct TableRowServerDisk {
    pub disk_id: ::std::string::String,
    pub disk_kind: TableRowPointerDiskKind,
    pub xfs_format: bool,
    pub extra_config: ::std::string::String,
    pub capacity_bytes: i64,
    pub parent: TableRowPointerServer,
    pub referrers_server__root_disk: Vec<TableRowPointerServer>,
    pub referrers_server_xfs_volume__xfs_disk: Vec<TableRowPointerServerXfsVolume>,
    pub referrers_server_zpool_spare__disk_id: Vec<TableRowPointerServerZpoolSpare>,
    pub referrers_server_zpool_cache__disk_id: Vec<TableRowPointerServerZpoolCache>,
    pub referrers_server_zpool_log__disk_id: Vec<TableRowPointerServerZpoolLog>,
    pub referrers_server_zpool_vdev_disk__disk_id: Vec<TableRowPointerServerZpoolVdevDisk>,
}

#[derive(Debug)]
pub struct TableRowServerKind {
    pub kind: ::std::string::String,
    pub cores: i64,
    pub memory_bytes: i64,
    pub architecture: ::std::string::String,
    pub bare_metal: bool,
    pub non_eligible_reason: ::std::string::String,
    pub children_server_kind_attribute: Vec<TableRowPointerServerKindAttribute>,
    pub referrers_datacenter__default_server_kind: Vec<TableRowPointerDatacenter>,
}

#[derive(Debug)]
pub struct TableRowServerKindAttribute {
    pub key: ::std::string::String,
    pub value: ::std::string::String,
    pub parent: TableRowPointerServerKind,
}

#[derive(Debug)]
pub struct TableRowServerLabel {
    pub label_name: TableRowPointerValidServerLabels,
    pub label_value: ::std::string::String,
    pub parent: TableRowPointerServer,
}

#[derive(Debug)]
pub struct TableRowServerRootVolume {
    pub volume_name: ::std::string::String,
    pub intended_usage: TableRowPointerServerVolumeUsageContract,
    pub mountpoint: ::std::string::String,
    pub zfs_recordsize: ::std::string::String,
    pub zfs_compression: bool,
    pub zfs_encryption: bool,
    pub parent: TableRowPointerServer,
}

#[derive(Debug)]
pub struct TableRowServerVolume {
    pub volume_name: ::std::string::String,
    pub mountpoint: ::std::string::String,
    pub intended_usage: TableRowPointerServerVolumeUsageContract,
    pub source: ::std::string::String,
    pub parent: TableRowPointerServer,
    pub referrers_pg_deployment_instance__pg_server: Vec<TableRowPointerPgDeploymentInstance>,
    pub referrers_ch_deployment_instance__ch_server: Vec<TableRowPointerChDeploymentInstance>,
    pub referrers_ch_keeper_deployment_instance__keeper_server: Vec<TableRowPointerChKeeperDeploymentInstance>,
    pub referrers_nats_deployment_instance__nats_server: Vec<TableRowPointerNatsDeploymentInstance>,
    pub referrers_minio_instance__instance_volume: Vec<TableRowPointerMinioInstance>,
    pub referrers_monitoring_instance__monitoring_server: Vec<TableRowPointerMonitoringInstance>,
    pub referrers_alertmanager_instance__alertmanager_server: Vec<TableRowPointerAlertmanagerInstance>,
    pub referrers_blackbox_deployment_task_mount__server_volume: Vec<TableRowPointerBlackboxDeploymentTaskMount>,
}

#[derive(Debug)]
pub struct TableRowServerVolumeUsageContract {
    pub usage_contract: ::std::string::String,
    pub referrers_server_volume__intended_usage: Vec<TableRowPointerServerVolume>,
    pub referrers_server_root_volume__intended_usage: Vec<TableRowPointerServerRootVolume>,
    pub referrers_server_xfs_volume__intended_usage: Vec<TableRowPointerServerXfsVolume>,
    pub referrers_server_zfs_dataset__intended_usage: Vec<TableRowPointerServerZfsDataset>,
}

#[derive(Debug)]
pub struct TableRowServerXfsVolume {
    pub volume_name: ::std::string::String,
    pub xfs_disk: TableRowPointerServerDisk,
    pub intended_usage: TableRowPointerServerVolumeUsageContract,
    pub parent: TableRowPointerServer,
}

#[derive(Debug)]
pub struct TableRowServerZfsDataset {
    pub dataset_name: ::std::string::String,
    pub intended_usage: TableRowPointerServerVolumeUsageContract,
    pub zfs_recordsize: ::std::string::String,
    pub zfs_compression: bool,
    pub zfs_encryption: bool,
    pub parent: TableRowPointerServerZpool,
}

#[derive(Debug)]
pub struct TableRowServerZpool {
    pub zpool_name: ::std::string::String,
    pub is_redundant: bool,
    pub parent: TableRowPointerServer,
    pub children_server_zpool_vdev: Vec<TableRowPointerServerZpoolVdev>,
    pub children_server_zpool_spare: Vec<TableRowPointerServerZpoolSpare>,
    pub children_server_zpool_cache: Vec<TableRowPointerServerZpoolCache>,
    pub children_server_zpool_log: Vec<TableRowPointerServerZpoolLog>,
    pub children_server_zfs_dataset: Vec<TableRowPointerServerZfsDataset>,
}

#[derive(Debug)]
pub struct TableRowServerZpoolCache {
    pub disk_id: TableRowPointerServerDisk,
    pub parent: TableRowPointerServerZpool,
}

#[derive(Debug)]
pub struct TableRowServerZpoolLog {
    pub disk_id: TableRowPointerServerDisk,
    pub parent: TableRowPointerServerZpool,
}

#[derive(Debug)]
pub struct TableRowServerZpoolSpare {
    pub disk_id: TableRowPointerServerDisk,
    pub parent: TableRowPointerServerZpool,
}

#[derive(Debug)]
pub struct TableRowServerZpoolVdev {
    pub vdev_number: i64,
    pub vdev_type: ::std::string::String,
    pub parent: TableRowPointerServerZpool,
    pub children_server_zpool_vdev_disk: Vec<TableRowPointerServerZpoolVdevDisk>,
}

#[derive(Debug)]
pub struct TableRowServerZpoolVdevDisk {
    pub disk_id: TableRowPointerServerDisk,
    pub parent: TableRowPointerServerZpoolVdev,
}

#[derive(Debug)]
pub struct TableRowSubnetRouterFloatingIp {
    pub ip_address: ::std::string::String,
}

#[derive(Debug)]
pub struct TableRowTelegramBot {
    pub bot_name: ::std::string::String,
    pub bot_token: ::std::string::String,
    pub referrers_monitoring_cluster_alert_group__telegram_bot: Vec<TableRowPointerMonitoringClusterAlertGroup>,
}

#[derive(Debug)]
pub struct TableRowTelegramChannel {
    pub channel_name: ::std::string::String,
    pub channel_id: i64,
    pub referrers_monitoring_cluster_alert_group__telegram_channel: Vec<TableRowPointerMonitoringClusterAlertGroup>,
}

#[derive(Debug)]
pub struct TableRowTempoCluster {
    pub cluster_name: ::std::string::String,
    pub namespace: TableRowPointerNomadNamespace,
    pub region: TableRowPointerRegion,
    pub workload_architecture: ::std::string::String,
    pub docker_image: TableRowPointerDockerImagePin,
    pub is_region_default: bool,
    pub loki_cluster: ::std::string::String,
    pub monitoring_cluster: ::std::string::String,
    pub storage_bucket: TableRowPointerMinioBucket,
    pub http_port: i64,
    pub grpc_port: i64,
    pub p2p_port: i64,
    pub otlp_http_port: i64,
    pub otlp_grpc_port: i64,
    pub tempo_instances: i64,
    pub placement: ::std::string::String,
    pub trace_retention_days: i64,
    pub memory_mb: i64,
}

#[derive(Debug)]
pub struct TableRowTld {
    pub domain: ::std::string::String,
    pub expose_admin: bool,
    pub automatic_certificates: bool,
    pub referrers_region__tld: Vec<TableRowPointerRegion>,
    pub referrers_backend_application_deployment_ingress__tld: Vec<TableRowPointerBackendApplicationDeploymentIngress>,
    pub referrers_frontend_application_deployment_ingress__tld: Vec<TableRowPointerFrontendApplicationDeploymentIngress>,
}

#[derive(Debug)]
pub struct TableRowUniqueApplicationNames {
    pub application_name: ::std::string::String,
    pub source: ::std::string::String,
}

#[derive(Debug)]
pub struct TableRowUniqueDeploymentNames {
    pub deployment_name: ::std::string::String,
    pub source: ::std::string::String,
}

#[derive(Debug)]
pub struct TableRowValidServerLabels {
    pub label_name: ::std::string::String,
    pub referrers_server_label__label_name: Vec<TableRowPointerServerLabel>,
}

#[derive(Debug)]
pub struct TableRowVersionedType {
    pub type_name: ::std::string::String,
    pub children_versioned_type_snapshot: Vec<TableRowPointerVersionedTypeSnapshot>,
    pub children_versioned_type_migration: Vec<TableRowPointerVersionedTypeMigration>,
    pub referrers_nats_jetstream_stream__stream_type: Vec<TableRowPointerNatsJetstreamStream>,
    pub referrers_backend_application_nats_stream__stream_type: Vec<TableRowPointerBackendApplicationNatsStream>,
}

#[derive(Debug)]
pub struct TableRowVersionedTypeMigration {
    pub version: i64,
    pub migration_source: ::std::string::String,
    pub parent: TableRowPointerVersionedType,
}

#[derive(Debug)]
pub struct TableRowVersionedTypeSnapshot {
    pub version: i64,
    pub snapshot_source: ::std::string::String,
    pub parent: TableRowPointerVersionedType,
}


// Table definitions
pub struct TableDefinitionAdminSshKeys {
    rows: Vec<TableRowAdminSshKeys>,
    c_contents: Vec<::std::string::String>,
}

pub struct TableDefinitionAlert {
    rows: Vec<TableRowAlert>,
    c_alert_name: Vec<::std::string::String>,
    c_expr: Vec<::std::string::String>,
    c_description: Vec<::std::string::String>,
    c_for_time: Vec<::std::string::String>,
    c_severity: Vec<i64>,
    c_parent: Vec<TableRowPointerAlertGroup>,
    c_children_alert_trigger_test: Vec<Vec<TableRowPointerAlertTriggerTest>>,
}

pub struct TableDefinitionAlertGroup {
    rows: Vec<TableRowAlertGroup>,
    c_alert_group_name: Vec<::std::string::String>,
    c_children_alert: Vec<Vec<TableRowPointerAlert>>,
    c_referrers_monitoring_cluster_alert_group__alert_group_name: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>>,
}

pub struct TableDefinitionAlertTriggerTest {
    rows: Vec<TableRowAlertTriggerTest>,
    c_expected_message: Vec<::std::string::String>,
    c_expected_labels: Vec<::std::string::String>,
    c_eval_time: Vec<::std::string::String>,
    c_interval: Vec<::std::string::String>,
    c_input_series: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerAlert>,
}

pub struct TableDefinitionAlertmanagerInstance {
    rows: Vec<TableRowAlertmanagerInstance>,
    c_instance_id: Vec<i64>,
    c_alertmanager_server: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerMonitoringCluster>,
}

pub struct TableDefinitionBackendApplication {
    rows: Vec<TableRowBackendApplication>,
    c_application_name: Vec<::std::string::String>,
    c_build_environment: Vec<TableRowPointerRustCompilationEnvironment>,
    c_children_backend_application_background_job: Vec<Vec<TableRowPointerBackendApplicationBackgroundJob>>,
    c_children_backend_application_config: Vec<Vec<TableRowPointerBackendApplicationConfig>>,
    c_children_backend_application_s3_bucket: Vec<Vec<TableRowPointerBackendApplicationS3Bucket>>,
    c_children_backend_application_pg_shard: Vec<Vec<TableRowPointerBackendApplicationPgShard>>,
    c_children_backend_application_ch_shard: Vec<Vec<TableRowPointerBackendApplicationChShard>>,
    c_children_backend_application_nats_stream: Vec<Vec<TableRowPointerBackendApplicationNatsStream>>,
    c_children_backend_http_endpoint: Vec<Vec<TableRowPointerBackendHttpEndpoint>>,
    c_referrers_backend_application_deployment__application_name: Vec<Vec<TableRowPointerBackendApplicationDeployment>>,
}

pub struct TableDefinitionBackendApplicationBackgroundJob {
    rows: Vec<TableRowBackendApplicationBackgroundJob>,
    c_job_name: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBackendApplication>,
}

pub struct TableDefinitionBackendApplicationChShard {
    rows: Vec<TableRowBackendApplicationChShard>,
    c_shard_name: Vec<::std::string::String>,
    c_ch_schema: Vec<TableRowPointerChSchema>,
    c_used_queries: Vec<::std::string::String>,
    c_used_inserters: Vec<::std::string::String>,
    c_used_mutators: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBackendApplication>,
}

pub struct TableDefinitionBackendApplicationConfig {
    rows: Vec<TableRowBackendApplicationConfig>,
    c_config_name: Vec<::std::string::String>,
    c_config_type: Vec<::std::string::String>,
    c_default_value: Vec<::std::string::String>,
    c_min_value: Vec<::std::string::String>,
    c_max_value: Vec<::std::string::String>,
    c_regex_check: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBackendApplication>,
}

pub struct TableDefinitionBackendApplicationDeployment {
    rows: Vec<TableRowBackendApplicationDeployment>,
    c_deployment_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_application_name: Vec<TableRowPointerBackendApplication>,
    c_workload_architecture: Vec<::std::string::String>,
    c_count: Vec<i64>,
    c_placement: Vec<::std::string::String>,
    c_pg_shard_wiring: Vec<::std::string::String>,
    c_ch_shard_wiring: Vec<::std::string::String>,
    c_nats_stream_wiring: Vec<::std::string::String>,
    c_s3_bucket_wiring: Vec<::std::string::String>,
    c_config: Vec<::std::string::String>,
    c_http_port: Vec<i64>,
    c_memory_mb: Vec<i64>,
    c_region: Vec<TableRowPointerRegion>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_tracing_cluster: Vec<::std::string::String>,
    c_referrers_backend_application_deployment_ingress__deployment: Vec<Vec<TableRowPointerBackendApplicationDeploymentIngress>>,
}

pub struct TableDefinitionBackendApplicationDeploymentIngress {
    rows: Vec<TableRowBackendApplicationDeploymentIngress>,
    c_deployment: Vec<TableRowPointerBackendApplicationDeployment>,
    c_mountpoint: Vec<::std::string::String>,
    c_subdomain: Vec<::std::string::String>,
    c_tld: Vec<TableRowPointerTld>,
    c_endpoint_list: Vec<::std::string::String>,
}

pub struct TableDefinitionBackendApplicationNatsStream {
    rows: Vec<TableRowBackendApplicationNatsStream>,
    c_stream_name: Vec<::std::string::String>,
    c_stream_type: Vec<TableRowPointerVersionedType>,
    c_enable_consumer: Vec<bool>,
    c_enable_producer: Vec<bool>,
    c_is_batch_consumer: Vec<bool>,
    c_enable_subjects: Vec<bool>,
    c_parent: Vec<TableRowPointerBackendApplication>,
}

pub struct TableDefinitionBackendApplicationPgShard {
    rows: Vec<TableRowBackendApplicationPgShard>,
    c_shard_name: Vec<::std::string::String>,
    c_pg_schema: Vec<TableRowPointerPgSchema>,
    c_used_queries: Vec<::std::string::String>,
    c_used_mutators: Vec<::std::string::String>,
    c_used_transactions: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBackendApplication>,
}

pub struct TableDefinitionBackendApplicationS3Bucket {
    rows: Vec<TableRowBackendApplicationS3Bucket>,
    c_bucket_name: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBackendApplication>,
}

pub struct TableDefinitionBackendHttpEndpoint {
    rows: Vec<TableRowBackendHttpEndpoint>,
    c_http_endpoint_name: Vec<::std::string::String>,
    c_path: Vec<::std::string::String>,
    c_http_method: Vec<TableRowPointerHttpMethods>,
    c_input_body_type: Vec<::std::string::String>,
    c_output_body_type: Vec<::std::string::String>,
    c_data_type: Vec<TableRowPointerHttpEndpointDataType>,
    c_max_input_body_size_bytes: Vec<i64>,
    c_needs_headers: Vec<bool>,
    c_receive_body_as_stream: Vec<bool>,
    c_parent: Vec<TableRowPointerBackendApplication>,
    c_referrers_frontend_application_used_endpoint__backend_endpoint: Vec<Vec<TableRowPointerFrontendApplicationUsedEndpoint>>,
    c_referrers_frontend_application_external_link__backend_endpoint: Vec<Vec<TableRowPointerFrontendApplicationExternalLink>>,
}

pub struct TableDefinitionBlackboxDeployment {
    rows: Vec<TableRowBlackboxDeployment>,
    c_deployment_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_children_blackbox_deployment_group: Vec<Vec<TableRowPointerBlackboxDeploymentGroup>>,
    c_children_blackbox_deployment_service_registration: Vec<Vec<TableRowPointerBlackboxDeploymentServiceRegistration>>,
}

pub struct TableDefinitionBlackboxDeploymentGroup {
    rows: Vec<TableRowBlackboxDeploymentGroup>,
    c_group_name: Vec<::std::string::String>,
    c_count: Vec<i64>,
    c_workload_architecture: Vec<::std::string::String>,
    c_placement: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBlackboxDeployment>,
    c_children_blackbox_deployment_port: Vec<Vec<TableRowPointerBlackboxDeploymentPort>>,
    c_children_blackbox_deployment_task: Vec<Vec<TableRowPointerBlackboxDeploymentTask>>,
    c_children_blackbox_deployment_service_instance: Vec<Vec<TableRowPointerBlackboxDeploymentServiceInstance>>,
}

pub struct TableDefinitionBlackboxDeploymentLocalFile {
    rows: Vec<TableRowBlackboxDeploymentLocalFile>,
    c_local_file_name: Vec<::std::string::String>,
    c_local_file_contents: Vec<::std::string::String>,
    c_mode: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBlackboxDeploymentTask>,
}

pub struct TableDefinitionBlackboxDeploymentPort {
    rows: Vec<TableRowBlackboxDeploymentPort>,
    c_port: Vec<i64>,
    c_port_description: Vec<::std::string::String>,
    c_protocol: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBlackboxDeploymentGroup>,
    c_referrers_blackbox_deployment_service_instance__port: Vec<Vec<TableRowPointerBlackboxDeploymentServiceInstance>>,
}

pub struct TableDefinitionBlackboxDeploymentServiceInstance {
    rows: Vec<TableRowBlackboxDeploymentServiceInstance>,
    c_service_registration: Vec<TableRowPointerBlackboxDeploymentServiceRegistration>,
    c_port: Vec<TableRowPointerBlackboxDeploymentPort>,
    c_parent: Vec<TableRowPointerBlackboxDeploymentGroup>,
}

pub struct TableDefinitionBlackboxDeploymentServiceRegistration {
    rows: Vec<TableRowBlackboxDeploymentServiceRegistration>,
    c_service_name: Vec<::std::string::String>,
    c_scrape_prometheus_metrics: Vec<bool>,
    c_prometheus_metrics_path: Vec<::std::string::String>,
    c_min_instances: Vec<i64>,
    c_parent: Vec<TableRowPointerBlackboxDeployment>,
    c_referrers_blackbox_deployment_service_instance__service_registration: Vec<Vec<TableRowPointerBlackboxDeploymentServiceInstance>>,
}

pub struct TableDefinitionBlackboxDeploymentTask {
    rows: Vec<TableRowBlackboxDeploymentTask>,
    c_task_name: Vec<::std::string::String>,
    c_docker_image: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_set: Vec<TableRowPointerDockerImageSet>,
    c_memory_mb: Vec<i64>,
    c_memory_oversubscription_mb: Vec<i64>,
    c_entrypoint: Vec<::std::string::String>,
    c_args: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBlackboxDeploymentGroup>,
    c_children_blackbox_deployment_task_mount: Vec<Vec<TableRowPointerBlackboxDeploymentTaskMount>>,
    c_children_blackbox_deployment_vault_secret: Vec<Vec<TableRowPointerBlackboxDeploymentVaultSecret>>,
    c_children_blackbox_deployment_local_file: Vec<Vec<TableRowPointerBlackboxDeploymentLocalFile>>,
}

pub struct TableDefinitionBlackboxDeploymentTaskMount {
    rows: Vec<TableRowBlackboxDeploymentTaskMount>,
    c_target_path: Vec<::std::string::String>,
    c_server_volume: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerBlackboxDeploymentTask>,
}

pub struct TableDefinitionBlackboxDeploymentVaultSecret {
    rows: Vec<TableRowBlackboxDeploymentVaultSecret>,
    c_secret_name: Vec<::std::string::String>,
    c_target_file_name: Vec<::std::string::String>,
    c_target_env_var_name: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerBlackboxDeploymentTask>,
}

pub struct TableDefinitionChDeployment {
    rows: Vec<TableRowChDeployment>,
    c_deployment_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_docker_image: Vec<TableRowPointerDockerImagePin>,
    c_workload_architecture: Vec<::std::string::String>,
    c_keeper: Vec<TableRowPointerChKeeperDeployment>,
    c_extra_memory_mb: Vec<i64>,
    c_mark_cache_size_mb: Vec<i64>,
    c_index_mark_cache_size_mb: Vec<i64>,
    c_uncompressed_cache_size_mb: Vec<i64>,
    c_compiled_expression_cache_size_mb: Vec<i64>,
    c_query_cache_size_mb: Vec<i64>,
    c_max_thread_pool_size: Vec<i64>,
    c_max_concurrent_queries: Vec<i64>,
    c_merge_max_block_size: Vec<i64>,
    c_max_bytes_to_merge_at_max_space_in_pool_mb: Vec<i64>,
    c_max_query_execution_time_seconds: Vec<i64>,
    c_queue_max_wait_ms: Vec<i64>,
    c_distribute_over_dcs: Vec<bool>,
    c_native_port: Vec<i64>,
    c_http_port: Vec<i64>,
    c_replication_port: Vec<i64>,
    c_prometheus_port: Vec<i64>,
    c_children_ch_deployment_instance: Vec<Vec<TableRowPointerChDeploymentInstance>>,
    c_children_ch_deployment_schemas: Vec<Vec<TableRowPointerChDeploymentSchemas>>,
}

pub struct TableDefinitionChDeploymentInstance {
    rows: Vec<TableRowChDeploymentInstance>,
    c_instance_id: Vec<i64>,
    c_ch_server: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerChDeployment>,
}

pub struct TableDefinitionChDeploymentSchemas {
    rows: Vec<TableRowChDeploymentSchemas>,
    c_db_name: Vec<::std::string::String>,
    c_ch_schema: Vec<TableRowPointerChSchema>,
    c_parent: Vec<TableRowPointerChDeployment>,
    c_children_ch_nats_stream_import: Vec<Vec<TableRowPointerChNatsStreamImport>>,
}

pub struct TableDefinitionChKeeperDeployment {
    rows: Vec<TableRowChKeeperDeployment>,
    c_deployment_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_docker_image: Vec<TableRowPointerDockerImagePin>,
    c_workload_architecture: Vec<::std::string::String>,
    c_distribute_over_dcs: Vec<bool>,
    c_memory_mb: Vec<i64>,
    c_keeper_port: Vec<i64>,
    c_raft_port: Vec<i64>,
    c_prometheus_port: Vec<i64>,
    c_children_ch_keeper_deployment_instance: Vec<Vec<TableRowPointerChKeeperDeploymentInstance>>,
    c_referrers_ch_deployment__keeper: Vec<Vec<TableRowPointerChDeployment>>,
}

pub struct TableDefinitionChKeeperDeploymentInstance {
    rows: Vec<TableRowChKeeperDeploymentInstance>,
    c_instance_id: Vec<i64>,
    c_keeper_server: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerChKeeperDeployment>,
}

pub struct TableDefinitionChMigration {
    rows: Vec<TableRowChMigration>,
    c_time: Vec<i64>,
    c_upgrade: Vec<::std::string::String>,
    c_downgrade: Vec<::std::string::String>,
    c_needs_admin: Vec<bool>,
    c_parent: Vec<TableRowPointerChSchema>,
}

pub struct TableDefinitionChMutator {
    rows: Vec<TableRowChMutator>,
    c_mutator_name: Vec<::std::string::String>,
    c_mutator_expression: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerChSchema>,
    c_children_ch_mutator_test: Vec<Vec<TableRowPointerChMutatorTest>>,
}

pub struct TableDefinitionChMutatorTest {
    rows: Vec<TableRowChMutatorTest>,
    c_arguments: Vec<::std::string::String>,
    c_test_dataset: Vec<TableRowPointerChTestDataset>,
    c_resulting_data: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerChMutator>,
}

pub struct TableDefinitionChNatsStreamImport {
    rows: Vec<TableRowChNatsStreamImport>,
    c_consumer_name: Vec<::std::string::String>,
    c_into_table: Vec<::std::string::String>,
    c_stream: Vec<TableRowPointerNatsJetstreamStream>,
    c_parent: Vec<TableRowPointerChDeploymentSchemas>,
}

pub struct TableDefinitionChQuery {
    rows: Vec<TableRowChQuery>,
    c_query_name: Vec<::std::string::String>,
    c_query_expression: Vec<::std::string::String>,
    c_opt_fields: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerChSchema>,
    c_children_ch_query_test: Vec<Vec<TableRowPointerChQueryTest>>,
}

pub struct TableDefinitionChQueryTest {
    rows: Vec<TableRowChQueryTest>,
    c_arguments: Vec<::std::string::String>,
    c_outputs: Vec<::std::string::String>,
    c_test_dataset: Vec<TableRowPointerChTestDataset>,
    c_parent: Vec<TableRowPointerChQuery>,
}

pub struct TableDefinitionChSchema {
    rows: Vec<TableRowChSchema>,
    c_schema_name: Vec<::std::string::String>,
    c_children_ch_migration: Vec<Vec<TableRowPointerChMigration>>,
    c_children_ch_query: Vec<Vec<TableRowPointerChQuery>>,
    c_children_ch_mutator: Vec<Vec<TableRowPointerChMutator>>,
    c_children_ch_test_dataset: Vec<Vec<TableRowPointerChTestDataset>>,
    c_referrers_ch_deployment_schemas__ch_schema: Vec<Vec<TableRowPointerChDeploymentSchemas>>,
    c_referrers_backend_application_ch_shard__ch_schema: Vec<Vec<TableRowPointerBackendApplicationChShard>>,
}

pub struct TableDefinitionChTestDataset {
    rows: Vec<TableRowChTestDataset>,
    c_dataset_name: Vec<::std::string::String>,
    c_dataset_contents: Vec<::std::string::String>,
    c_min_time: Vec<i64>,
    c_parent: Vec<TableRowPointerChSchema>,
    c_referrers_ch_query_test__test_dataset: Vec<Vec<TableRowPointerChQueryTest>>,
    c_referrers_ch_mutator_test__test_dataset: Vec<Vec<TableRowPointerChMutatorTest>>,
}

pub struct TableDefinitionDatacenter {
    rows: Vec<TableRowDatacenter>,
    c_dc_name: Vec<::std::string::String>,
    c_region: Vec<TableRowPointerRegion>,
    c_network_cidr: Vec<::std::string::String>,
    c_allow_small_subnets: Vec<bool>,
    c_implementation: Vec<::std::string::String>,
    c_implementation_settings: Vec<::std::string::String>,
    c_default_server_kind: Vec<TableRowPointerServerKind>,
    c_disk_ids_policy: Vec<::std::string::String>,
    c_router_subnet_vlan_id: Vec<i64>,
    c_referrers_server__dc: Vec<Vec<TableRowPointerServer>>,
}

pub struct TableDefinitionDiskKind {
    rows: Vec<TableRowDiskKind>,
    c_kind: Vec<::std::string::String>,
    c_medium: Vec<::std::string::String>,
    c_is_elastic: Vec<bool>,
    c_min_capacity_bytes: Vec<i64>,
    c_max_capacity_bytes: Vec<i64>,
    c_capacity_bytes: Vec<i64>,
    c_has_extra_config: Vec<bool>,
    c_non_eligible_reason: Vec<::std::string::String>,
    c_referrers_server_disk__disk_kind: Vec<Vec<TableRowPointerServerDisk>>,
}

pub struct TableDefinitionDockerImage {
    rows: Vec<TableRowDockerImage>,
    c_checksum: Vec<::std::string::String>,
    c_image_set: Vec<TableRowPointerDockerImageSet>,
    c_repository: Vec<::std::string::String>,
    c_architecture: Vec<::std::string::String>,
    c_tag: Vec<::std::string::String>,
    c_referrers_docker_image_pin_images__checksum: Vec<Vec<TableRowPointerDockerImagePinImages>>,
}

pub struct TableDefinitionDockerImagePin {
    rows: Vec<TableRowDockerImagePin>,
    c_pin_name: Vec<::std::string::String>,
    c_children_docker_image_pin_images: Vec<Vec<TableRowPointerDockerImagePinImages>>,
    c_referrers_region__docker_image_external_lb: Vec<Vec<TableRowPointerRegion>>,
    c_referrers_docker_registry_instance__docker_image: Vec<Vec<TableRowPointerDockerRegistryInstance>>,
    c_referrers_pg_deployment__docker_image_pg: Vec<Vec<TableRowPointerPgDeployment>>,
    c_referrers_pg_deployment__docker_image_haproxy: Vec<Vec<TableRowPointerPgDeployment>>,
    c_referrers_pg_deployment__docker_image_pg_exporter: Vec<Vec<TableRowPointerPgDeployment>>,
    c_referrers_ch_deployment__docker_image: Vec<Vec<TableRowPointerChDeployment>>,
    c_referrers_ch_keeper_deployment__docker_image: Vec<Vec<TableRowPointerChKeeperDeployment>>,
    c_referrers_nats_cluster__docker_image_nats: Vec<Vec<TableRowPointerNatsCluster>>,
    c_referrers_nats_cluster__docker_image_nats_exporter: Vec<Vec<TableRowPointerNatsCluster>>,
    c_referrers_minio_cluster__docker_image_minio: Vec<Vec<TableRowPointerMinioCluster>>,
    c_referrers_minio_cluster__docker_image_minio_mc: Vec<Vec<TableRowPointerMinioCluster>>,
    c_referrers_minio_cluster__docker_image_nginx: Vec<Vec<TableRowPointerMinioCluster>>,
    c_referrers_monitoring_cluster__docker_image_prometheus: Vec<Vec<TableRowPointerMonitoringCluster>>,
    c_referrers_monitoring_cluster__docker_image_alertmanager: Vec<Vec<TableRowPointerMonitoringCluster>>,
    c_referrers_monitoring_cluster__docker_image_victoriametrics: Vec<Vec<TableRowPointerMonitoringCluster>>,
    c_referrers_grafana__docker_image_grafana: Vec<Vec<TableRowPointerGrafana>>,
    c_referrers_grafana__docker_image_promxy: Vec<Vec<TableRowPointerGrafana>>,
    c_referrers_loki_cluster__docker_image_loki: Vec<Vec<TableRowPointerLokiCluster>>,
    c_referrers_tempo_cluster__docker_image: Vec<Vec<TableRowPointerTempoCluster>>,
    c_referrers_blackbox_deployment_task__docker_image: Vec<Vec<TableRowPointerBlackboxDeploymentTask>>,
}

pub struct TableDefinitionDockerImagePinImages {
    rows: Vec<TableRowDockerImagePinImages>,
    c_checksum: Vec<TableRowPointerDockerImage>,
    c_parent: Vec<TableRowPointerDockerImagePin>,
}

pub struct TableDefinitionDockerImageSet {
    rows: Vec<TableRowDockerImageSet>,
    c_set_name: Vec<::std::string::String>,
    c_referrers_docker_image__image_set: Vec<Vec<TableRowPointerDockerImage>>,
    c_referrers_blackbox_deployment_task__docker_image_set: Vec<Vec<TableRowPointerBlackboxDeploymentTask>>,
}

pub struct TableDefinitionDockerRegistryInstance {
    rows: Vec<TableRowDockerRegistryInstance>,
    c_region: Vec<TableRowPointerRegion>,
    c_minio_bucket: Vec<TableRowPointerMinioBucket>,
    c_memory_mb: Vec<i64>,
    c_docker_image: Vec<TableRowPointerDockerImagePin>,
}

pub struct TableDefinitionFrontendApplication {
    rows: Vec<TableRowFrontendApplication>,
    c_application_name: Vec<::std::string::String>,
    c_build_environment: Vec<TableRowPointerRustCompilationEnvironment>,
    c_index_page_title: Vec<::std::string::String>,
    c_children_frontend_page: Vec<Vec<TableRowPointerFrontendPage>>,
    c_children_frontend_application_used_endpoint: Vec<Vec<TableRowPointerFrontendApplicationUsedEndpoint>>,
    c_children_frontend_application_external_link: Vec<Vec<TableRowPointerFrontendApplicationExternalLink>>,
    c_children_frontend_application_external_page: Vec<Vec<TableRowPointerFrontendApplicationExternalPage>>,
    c_referrers_frontend_application_deployment__application_name: Vec<Vec<TableRowPointerFrontendApplicationDeployment>>,
}

pub struct TableDefinitionFrontendApplicationDeployment {
    rows: Vec<TableRowFrontendApplicationDeployment>,
    c_deployment_name: Vec<::std::string::String>,
    c_application_name: Vec<TableRowPointerFrontendApplication>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_explicit_endpoint_wiring: Vec<::std::string::String>,
    c_workload_backend_architecture: Vec<::std::string::String>,
    c_placement: Vec<::std::string::String>,
    c_link_wiring: Vec<::std::string::String>,
    c_page_wiring: Vec<::std::string::String>,
    c_count: Vec<i64>,
    c_http_port: Vec<i64>,
    c_memory_mb: Vec<i64>,
    c_region: Vec<TableRowPointerRegion>,
    c_loki_cluster: Vec<::std::string::String>,
    c_referrers_frontend_application_deployment_ingress__deployment: Vec<Vec<TableRowPointerFrontendApplicationDeploymentIngress>>,
}

pub struct TableDefinitionFrontendApplicationDeploymentIngress {
    rows: Vec<TableRowFrontendApplicationDeploymentIngress>,
    c_deployment: Vec<TableRowPointerFrontendApplicationDeployment>,
    c_mountpoint: Vec<::std::string::String>,
    c_subdomain: Vec<::std::string::String>,
    c_tld: Vec<TableRowPointerTld>,
}

pub struct TableDefinitionFrontendApplicationExternalLink {
    rows: Vec<TableRowFrontendApplicationExternalLink>,
    c_link_name: Vec<::std::string::String>,
    c_backend_endpoint: Vec<TableRowPointerBackendHttpEndpoint>,
    c_parent: Vec<TableRowPointerFrontendApplication>,
}

pub struct TableDefinitionFrontendApplicationExternalPage {
    rows: Vec<TableRowFrontendApplicationExternalPage>,
    c_link_name: Vec<::std::string::String>,
    c_frontend_page: Vec<TableRowPointerFrontendPage>,
    c_parent: Vec<TableRowPointerFrontendApplication>,
}

pub struct TableDefinitionFrontendApplicationUsedEndpoint {
    rows: Vec<TableRowFrontendApplicationUsedEndpoint>,
    c_endpoint_name: Vec<::std::string::String>,
    c_backend_endpoint: Vec<TableRowPointerBackendHttpEndpoint>,
    c_parent: Vec<TableRowPointerFrontendApplication>,
}

pub struct TableDefinitionFrontendPage {
    rows: Vec<TableRowFrontendPage>,
    c_page_name: Vec<::std::string::String>,
    c_path: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerFrontendApplication>,
    c_referrers_frontend_application_external_page__frontend_page: Vec<Vec<TableRowPointerFrontendApplicationExternalPage>>,
}

pub struct TableDefinitionGlobalSettings {
    rows: Vec<TableRowGlobalSettings>,
    c_project_name: Vec<::std::string::String>,
    c_docker_registry_port: Vec<i64>,
    c_docker_registry_service_name: Vec<::std::string::String>,
    c_aws_artefacts_s3_bucket_name: Vec<::std::string::String>,
    c_local_docker_cache_port: Vec<i64>,
    c_admin_email: Vec<::std::string::String>,
    c_google_cloud_project_id: Vec<::std::string::String>,
    c_google_cloud_artefacts_bucket_name: Vec<::std::string::String>,
    c_disable_consul_quorum_tests: Vec<bool>,
    c_disable_nomad_quorum_tests: Vec<bool>,
    c_disable_vault_quorum_tests: Vec<bool>,
    c_disable_dns_quorum_tests: Vec<bool>,
    c_disable_deployment_min_server_tests: Vec<bool>,
    c_disable_deployment_min_ingress_tests: Vec<bool>,
    c_disable_region_docker_registry_tests: Vec<bool>,
    c_disable_region_monitoring_tests: Vec<bool>,
    c_disable_region_tracing_tests: Vec<bool>,
    c_disable_region_logging_tests: Vec<bool>,
    c_disable_vpn_gateway_tests: Vec<bool>,
    c_hetzner_inter_dc_vlan_id: Vec<i64>,
    c_experimental_enable_arm64_support: Vec<bool>,
    c_update_edl_public_ips_from_terraform: Vec<bool>,
    c_enable_ipv6: Vec<bool>,
    c_force_ipv6: Vec<bool>,
}

pub struct TableDefinitionGrafana {
    rows: Vec<TableRowGrafana>,
    c_deployment_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_placement: Vec<::std::string::String>,
    c_workload_architecture: Vec<::std::string::String>,
    c_docker_image_grafana: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_promxy: Vec<TableRowPointerDockerImagePin>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_port: Vec<i64>,
    c_promxy_port: Vec<i64>,
    c_instance_count: Vec<i64>,
    c_database: Vec<TableRowPointerPgDeploymentUnmanagedDb>,
    c_memory_mb: Vec<i64>,
    c_promxy_memory_mb: Vec<i64>,
}

pub struct TableDefinitionGrafanaDashboard {
    rows: Vec<TableRowGrafanaDashboard>,
    c_filename: Vec<::std::string::String>,
    c_contents: Vec<::std::string::String>,
}

pub struct TableDefinitionHttpEndpointDataType {
    rows: Vec<TableRowHttpEndpointDataType>,
    c_http_endpoint_data_type: Vec<::std::string::String>,
    c_referrers_backend_http_endpoint__data_type: Vec<Vec<TableRowPointerBackendHttpEndpoint>>,
}

pub struct TableDefinitionHttpMethods {
    rows: Vec<TableRowHttpMethods>,
    c_http_method_name: Vec<::std::string::String>,
    c_referrers_backend_http_endpoint__http_method: Vec<Vec<TableRowPointerBackendHttpEndpoint>>,
}

pub struct TableDefinitionLokiCluster {
    rows: Vec<TableRowLokiCluster>,
    c_cluster_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_workload_architecture: Vec<::std::string::String>,
    c_docker_image_loki: Vec<TableRowPointerDockerImagePin>,
    c_is_region_default: Vec<bool>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_storage_bucket: Vec<TableRowPointerMinioBucket>,
    c_retention_period_days: Vec<i64>,
    c_loki_writer_http_port: Vec<i64>,
    c_loki_writer_grpc_port: Vec<i64>,
    c_loki_reader_http_port: Vec<i64>,
    c_loki_reader_grpc_port: Vec<i64>,
    c_loki_backend_http_port: Vec<i64>,
    c_loki_backend_grpc_port: Vec<i64>,
    c_loki_writers: Vec<i64>,
    c_loki_readers: Vec<i64>,
    c_writer_placement: Vec<::std::string::String>,
    c_reader_placement: Vec<::std::string::String>,
    c_backend_placement: Vec<::std::string::String>,
    c_loki_reader_memory_mb: Vec<i64>,
    c_loki_writer_memory_mb: Vec<i64>,
    c_loki_backend_memory_mb: Vec<i64>,
}

pub struct TableDefinitionMinioBucket {
    rows: Vec<TableRowMinioBucket>,
    c_bucket_name: Vec<::std::string::String>,
    c_locking_enabled: Vec<bool>,
    c_parent: Vec<TableRowPointerMinioCluster>,
    c_referrers_docker_registry_instance__minio_bucket: Vec<Vec<TableRowPointerDockerRegistryInstance>>,
    c_referrers_loki_cluster__storage_bucket: Vec<Vec<TableRowPointerLokiCluster>>,
    c_referrers_tempo_cluster__storage_bucket: Vec<Vec<TableRowPointerTempoCluster>>,
}

pub struct TableDefinitionMinioCluster {
    rows: Vec<TableRowMinioCluster>,
    c_cluster_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_workload_architecture: Vec<::std::string::String>,
    c_docker_image_minio: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_minio_mc: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_nginx: Vec<TableRowPointerDockerImagePin>,
    c_api_port: Vec<i64>,
    c_console_port: Vec<i64>,
    c_lb_port: Vec<i64>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_expected_zfs_recordsize: Vec<::std::string::String>,
    c_distribute_over_dcs: Vec<bool>,
    c_instance_memory_mb: Vec<i64>,
    c_lb_memory_mb: Vec<i64>,
    c_consul_service_name: Vec<::std::string::String>,
    c_children_minio_instance: Vec<Vec<TableRowPointerMinioInstance>>,
    c_children_minio_bucket: Vec<Vec<TableRowPointerMinioBucket>>,
}

pub struct TableDefinitionMinioInstance {
    rows: Vec<TableRowMinioInstance>,
    c_instance_id: Vec<i64>,
    c_instance_volume: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerMinioCluster>,
}

pub struct TableDefinitionMonitoringCluster {
    rows: Vec<TableRowMonitoringCluster>,
    c_cluster_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_is_region_default: Vec<bool>,
    c_workload_architecture: Vec<::std::string::String>,
    c_docker_image_prometheus: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_alertmanager: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_victoriametrics: Vec<TableRowPointerDockerImagePin>,
    c_prometheus_memory_mb: Vec<i64>,
    c_victoriametrics_memory_mb: Vec<i64>,
    c_alertmanager_memory_mb: Vec<i64>,
    c_prometheus_port: Vec<i64>,
    c_victoriametrics_port: Vec<i64>,
    c_alertmanager_port: Vec<i64>,
    c_alertmanager_p2p_port: Vec<i64>,
    c_victoriametrics_retention_months: Vec<i64>,
    c_children_monitoring_cluster_scraped_metric: Vec<Vec<TableRowPointerMonitoringClusterScrapedMetric>>,
    c_children_monitoring_cluster_alert_group: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>>,
    c_children_monitoring_instance: Vec<Vec<TableRowPointerMonitoringInstance>>,
    c_children_alertmanager_instance: Vec<Vec<TableRowPointerAlertmanagerInstance>>,
}

pub struct TableDefinitionMonitoringClusterAlertGroup {
    rows: Vec<TableRowMonitoringClusterAlertGroup>,
    c_alert_group_name: Vec<TableRowPointerAlertGroup>,
    c_telegram_channel: Vec<TableRowPointerTelegramChannel>,
    c_telegram_bot: Vec<TableRowPointerTelegramBot>,
    c_parent: Vec<TableRowPointerMonitoringCluster>,
}

pub struct TableDefinitionMonitoringClusterScrapedMetric {
    rows: Vec<TableRowMonitoringClusterScrapedMetric>,
    c_metric_name: Vec<::std::string::String>,
    c_expression: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerMonitoringCluster>,
}

pub struct TableDefinitionMonitoringInstance {
    rows: Vec<TableRowMonitoringInstance>,
    c_instance_id: Vec<i64>,
    c_monitoring_server: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerMonitoringCluster>,
}

pub struct TableDefinitionNatsCluster {
    rows: Vec<TableRowNatsCluster>,
    c_cluster_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_distribute_over_dcs: Vec<bool>,
    c_workload_architecture: Vec<::std::string::String>,
    c_docker_image_nats: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_nats_exporter: Vec<TableRowPointerDockerImagePin>,
    c_nats_clients_port: Vec<i64>,
    c_nats_cluster_port: Vec<i64>,
    c_nats_http_mon_port: Vec<i64>,
    c_nats_prometheus_port: Vec<i64>,
    c_instance_memory_mb: Vec<i64>,
    c_children_nats_jetstream_stream: Vec<Vec<TableRowPointerNatsJetstreamStream>>,
    c_children_nats_deployment_instance: Vec<Vec<TableRowPointerNatsDeploymentInstance>>,
}

pub struct TableDefinitionNatsDeploymentInstance {
    rows: Vec<TableRowNatsDeploymentInstance>,
    c_instance_id: Vec<i64>,
    c_nats_server: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerNatsCluster>,
}

pub struct TableDefinitionNatsJetstreamStream {
    rows: Vec<TableRowNatsJetstreamStream>,
    c_stream_name: Vec<::std::string::String>,
    c_stream_type: Vec<TableRowPointerVersionedType>,
    c_max_bytes: Vec<i64>,
    c_max_msg_size: Vec<i64>,
    c_enable_subjects: Vec<bool>,
    c_parent: Vec<TableRowPointerNatsCluster>,
    c_referrers_ch_nats_stream_import__stream: Vec<Vec<TableRowPointerChNatsStreamImport>>,
}

pub struct TableDefinitionNetwork {
    rows: Vec<TableRowNetwork>,
    c_network_name: Vec<::std::string::String>,
    c_cidr: Vec<::std::string::String>,
    c_referrers_network_interface__if_network: Vec<Vec<TableRowPointerNetworkInterface>>,
}

pub struct TableDefinitionNetworkInterface {
    rows: Vec<TableRowNetworkInterface>,
    c_if_name: Vec<::std::string::String>,
    c_if_network: Vec<TableRowPointerNetwork>,
    c_if_ip: Vec<::std::string::String>,
    c_if_prefix: Vec<i64>,
    c_if_vlan: Vec<i64>,
    c_parent: Vec<TableRowPointerServer>,
    c_referrers_server__ssh_interface: Vec<Vec<TableRowPointerServer>>,
}

pub struct TableDefinitionNixpkgsEnvironment {
    rows: Vec<TableRowNixpkgsEnvironment>,
    c_name: Vec<::std::string::String>,
    c_version: Vec<TableRowPointerNixpkgsVersion>,
    c_referrers_server__nixpkgs_environment: Vec<Vec<TableRowPointerServer>>,
    c_referrers_rust_compilation_environment__nixpkgs_environment: Vec<Vec<TableRowPointerRustCompilationEnvironment>>,
}

pub struct TableDefinitionNixpkgsVersion {
    rows: Vec<TableRowNixpkgsVersion>,
    c_version: Vec<::std::string::String>,
    c_checksum: Vec<::std::string::String>,
    c_tarball_checksum: Vec<::std::string::String>,
    c_referrers_nixpkgs_environment__version: Vec<Vec<TableRowPointerNixpkgsEnvironment>>,
}

pub struct TableDefinitionNomadNamespace {
    rows: Vec<TableRowNomadNamespace>,
    c_namespace: Vec<::std::string::String>,
    c_description: Vec<::std::string::String>,
    c_referrers_pg_deployment__namespace: Vec<Vec<TableRowPointerPgDeployment>>,
    c_referrers_ch_deployment__namespace: Vec<Vec<TableRowPointerChDeployment>>,
    c_referrers_ch_keeper_deployment__namespace: Vec<Vec<TableRowPointerChKeeperDeployment>>,
    c_referrers_nats_cluster__namespace: Vec<Vec<TableRowPointerNatsCluster>>,
    c_referrers_backend_application_deployment__namespace: Vec<Vec<TableRowPointerBackendApplicationDeployment>>,
    c_referrers_frontend_application_deployment__namespace: Vec<Vec<TableRowPointerFrontendApplicationDeployment>>,
    c_referrers_minio_cluster__namespace: Vec<Vec<TableRowPointerMinioCluster>>,
    c_referrers_monitoring_cluster__namespace: Vec<Vec<TableRowPointerMonitoringCluster>>,
    c_referrers_grafana__namespace: Vec<Vec<TableRowPointerGrafana>>,
    c_referrers_loki_cluster__namespace: Vec<Vec<TableRowPointerLokiCluster>>,
    c_referrers_tempo_cluster__namespace: Vec<Vec<TableRowPointerTempoCluster>>,
    c_referrers_blackbox_deployment__namespace: Vec<Vec<TableRowPointerBlackboxDeployment>>,
}

pub struct TableDefinitionPgDeployment {
    rows: Vec<TableRowPgDeployment>,
    c_deployment_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_docker_image_pg: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_haproxy: Vec<TableRowPointerDockerImagePin>,
    c_docker_image_pg_exporter: Vec<TableRowPointerDockerImagePin>,
    c_workload_architecture: Vec<::std::string::String>,
    c_distribute_over_dcs: Vec<bool>,
    c_synchronous_replication: Vec<bool>,
    c_shared_buffers_mb: Vec<i64>,
    c_work_mem_mb: Vec<i64>,
    c_maintenance_work_mem_mb: Vec<i64>,
    c_overhead_mem_mb: Vec<i64>,
    c_max_connections: Vec<i64>,
    c_replica_rolling_update_delay_seconds: Vec<i64>,
    c_instance_pg_port: Vec<i64>,
    c_instance_pg_master_port: Vec<i64>,
    c_instance_pg_slave_port: Vec<i64>,
    c_instance_patroni_port: Vec<i64>,
    c_instance_haproxy_metrics_port: Vec<i64>,
    c_instance_pg_exporter_port: Vec<i64>,
    c_children_pg_deployment_schemas: Vec<Vec<TableRowPointerPgDeploymentSchemas>>,
    c_children_pg_deployment_unmanaged_db: Vec<Vec<TableRowPointerPgDeploymentUnmanagedDb>>,
    c_children_pg_deployment_instance: Vec<Vec<TableRowPointerPgDeploymentInstance>>,
}

pub struct TableDefinitionPgDeploymentInstance {
    rows: Vec<TableRowPgDeploymentInstance>,
    c_instance_id: Vec<i64>,
    c_pg_server: Vec<TableRowPointerServerVolume>,
    c_parent: Vec<TableRowPointerPgDeployment>,
}

pub struct TableDefinitionPgDeploymentSchemas {
    rows: Vec<TableRowPgDeploymentSchemas>,
    c_db_name: Vec<::std::string::String>,
    c_pg_schema: Vec<TableRowPointerPgSchema>,
    c_parent: Vec<TableRowPointerPgDeployment>,
}

pub struct TableDefinitionPgDeploymentUnmanagedDb {
    rows: Vec<TableRowPgDeploymentUnmanagedDb>,
    c_db_name: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerPgDeployment>,
    c_referrers_grafana__database: Vec<Vec<TableRowPointerGrafana>>,
}

pub struct TableDefinitionPgMatView {
    rows: Vec<TableRowPgMatView>,
    c_mview_name: Vec<::std::string::String>,
    c_update_frequency: Vec<TableRowPointerPgMatViewUpdateFrequency>,
    c_parent: Vec<TableRowPointerPgSchema>,
    c_children_pg_mat_view_test: Vec<Vec<TableRowPointerPgMatViewTest>>,
}

pub struct TableDefinitionPgMatViewTest {
    rows: Vec<TableRowPgMatViewTest>,
    c_expected_data: Vec<::std::string::String>,
    c_test_dataset: Vec<TableRowPointerPgTestDataset>,
    c_parent: Vec<TableRowPointerPgMatView>,
}

pub struct TableDefinitionPgMatViewUpdateFrequency {
    rows: Vec<TableRowPgMatViewUpdateFrequency>,
    c_frequency: Vec<::std::string::String>,
    c_referrers_pg_mat_view__update_frequency: Vec<Vec<TableRowPointerPgMatView>>,
}

pub struct TableDefinitionPgMigration {
    rows: Vec<TableRowPgMigration>,
    c_time: Vec<i64>,
    c_upgrade: Vec<::std::string::String>,
    c_downgrade: Vec<::std::string::String>,
    c_needs_admin: Vec<bool>,
    c_parent: Vec<TableRowPointerPgSchema>,
}

pub struct TableDefinitionPgMutator {
    rows: Vec<TableRowPgMutator>,
    c_mutator_name: Vec<::std::string::String>,
    c_mutator_expression: Vec<::std::string::String>,
    c_seqscan_ok: Vec<bool>,
    c_parent: Vec<TableRowPointerPgSchema>,
    c_children_pg_mutator_test: Vec<Vec<TableRowPointerPgMutatorTest>>,
}

pub struct TableDefinitionPgMutatorTest {
    rows: Vec<TableRowPgMutatorTest>,
    c_arguments: Vec<::std::string::String>,
    c_test_dataset: Vec<TableRowPointerPgTestDataset>,
    c_resulting_data: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerPgMutator>,
}

pub struct TableDefinitionPgQuery {
    rows: Vec<TableRowPgQuery>,
    c_query_name: Vec<::std::string::String>,
    c_query_expression: Vec<::std::string::String>,
    c_is_mutating: Vec<bool>,
    c_seqscan_ok: Vec<bool>,
    c_opt_fields: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerPgSchema>,
    c_children_pg_query_test: Vec<Vec<TableRowPointerPgQueryTest>>,
}

pub struct TableDefinitionPgQueryTest {
    rows: Vec<TableRowPgQueryTest>,
    c_arguments: Vec<::std::string::String>,
    c_outputs: Vec<::std::string::String>,
    c_test_dataset: Vec<TableRowPointerPgTestDataset>,
    c_parent: Vec<TableRowPointerPgQuery>,
}

pub struct TableDefinitionPgSchema {
    rows: Vec<TableRowPgSchema>,
    c_schema_name: Vec<::std::string::String>,
    c_children_pg_migration: Vec<Vec<TableRowPointerPgMigration>>,
    c_children_pg_query: Vec<Vec<TableRowPointerPgQuery>>,
    c_children_pg_mutator: Vec<Vec<TableRowPointerPgMutator>>,
    c_children_pg_transaction: Vec<Vec<TableRowPointerPgTransaction>>,
    c_children_pg_mat_view: Vec<Vec<TableRowPointerPgMatView>>,
    c_children_pg_test_dataset: Vec<Vec<TableRowPointerPgTestDataset>>,
    c_referrers_pg_deployment_schemas__pg_schema: Vec<Vec<TableRowPointerPgDeploymentSchemas>>,
    c_referrers_backend_application_pg_shard__pg_schema: Vec<Vec<TableRowPointerBackendApplicationPgShard>>,
}

pub struct TableDefinitionPgTestDataset {
    rows: Vec<TableRowPgTestDataset>,
    c_dataset_name: Vec<::std::string::String>,
    c_dataset_contents: Vec<::std::string::String>,
    c_min_time: Vec<i64>,
    c_parent: Vec<TableRowPointerPgSchema>,
    c_referrers_pg_query_test__test_dataset: Vec<Vec<TableRowPointerPgQueryTest>>,
    c_referrers_pg_mutator_test__test_dataset: Vec<Vec<TableRowPointerPgMutatorTest>>,
    c_referrers_pg_mat_view_test__test_dataset: Vec<Vec<TableRowPointerPgMatViewTest>>,
}

pub struct TableDefinitionPgTransaction {
    rows: Vec<TableRowPgTransaction>,
    c_transaction_name: Vec<::std::string::String>,
    c_steps: Vec<::std::string::String>,
    c_is_read_only: Vec<bool>,
    c_parent: Vec<TableRowPointerPgSchema>,
}

pub struct TableDefinitionRegion {
    rows: Vec<TableRowRegion>,
    c_region_name: Vec<::std::string::String>,
    c_availability_mode: Vec<::std::string::String>,
    c_tld: Vec<TableRowPointerTld>,
    c_is_dns_master: Vec<bool>,
    c_is_dns_slave: Vec<bool>,
    c_has_coprocessor_dc: Vec<bool>,
    c_docker_image_external_lb: Vec<TableRowPointerDockerImagePin>,
    c_nomad_disable_log_collection: Vec<bool>,
    c_referrers_datacenter__region: Vec<Vec<TableRowPointerDatacenter>>,
    c_referrers_docker_registry_instance__region: Vec<Vec<TableRowPointerDockerRegistryInstance>>,
    c_referrers_pg_deployment__region: Vec<Vec<TableRowPointerPgDeployment>>,
    c_referrers_ch_deployment__region: Vec<Vec<TableRowPointerChDeployment>>,
    c_referrers_ch_keeper_deployment__region: Vec<Vec<TableRowPointerChKeeperDeployment>>,
    c_referrers_nats_cluster__region: Vec<Vec<TableRowPointerNatsCluster>>,
    c_referrers_backend_application_deployment__region: Vec<Vec<TableRowPointerBackendApplicationDeployment>>,
    c_referrers_frontend_application_deployment__region: Vec<Vec<TableRowPointerFrontendApplicationDeployment>>,
    c_referrers_minio_cluster__region: Vec<Vec<TableRowPointerMinioCluster>>,
    c_referrers_monitoring_cluster__region: Vec<Vec<TableRowPointerMonitoringCluster>>,
    c_referrers_grafana__region: Vec<Vec<TableRowPointerGrafana>>,
    c_referrers_loki_cluster__region: Vec<Vec<TableRowPointerLokiCluster>>,
    c_referrers_tempo_cluster__region: Vec<Vec<TableRowPointerTempoCluster>>,
    c_referrers_blackbox_deployment__region: Vec<Vec<TableRowPointerBlackboxDeployment>>,
}

pub struct TableDefinitionRustCompilationEnvironment {
    rows: Vec<TableRowRustCompilationEnvironment>,
    c_env_name: Vec<::std::string::String>,
    c_rust_edition: Vec<::std::string::String>,
    c_nixpkgs_environment: Vec<TableRowPointerNixpkgsEnvironment>,
    c_environment_kind: Vec<::std::string::String>,
    c_children_rust_crate_version: Vec<Vec<TableRowPointerRustCrateVersion>>,
    c_referrers_backend_application__build_environment: Vec<Vec<TableRowPointerBackendApplication>>,
    c_referrers_frontend_application__build_environment: Vec<Vec<TableRowPointerFrontendApplication>>,
}

pub struct TableDefinitionRustCrateVersion {
    rows: Vec<TableRowRustCrateVersion>,
    c_crate_name: Vec<::std::string::String>,
    c_version: Vec<::std::string::String>,
    c_features: Vec<::std::string::String>,
    c_default_features: Vec<bool>,
    c_parent: Vec<TableRowPointerRustCompilationEnvironment>,
}

pub struct TableDefinitionServer {
    rows: Vec<TableRowServer>,
    c_hostname: Vec<::std::string::String>,
    c_dc: Vec<TableRowPointerDatacenter>,
    c_ssh_interface: Vec<TableRowPointerNetworkInterface>,
    c_root_disk: Vec<TableRowPointerServerDisk>,
    c_is_consul_master: Vec<bool>,
    c_is_nomad_master: Vec<bool>,
    c_is_vault_instance: Vec<bool>,
    c_is_dns_master: Vec<bool>,
    c_is_dns_slave: Vec<bool>,
    c_is_ingress: Vec<bool>,
    c_is_vpn_gateway: Vec<bool>,
    c_is_coprocessor_gateway: Vec<bool>,
    c_is_router: Vec<bool>,
    c_public_ipv6_address: Vec<::std::string::String>,
    c_public_ipv6_address_prefix: Vec<i64>,
    c_kind: Vec<::std::string::String>,
    c_nixpkgs_environment: Vec<TableRowPointerNixpkgsEnvironment>,
    c_run_unassigned_workloads: Vec<bool>,
    c_children_server_label: Vec<Vec<TableRowPointerServerLabel>>,
    c_children_server_disk: Vec<Vec<TableRowPointerServerDisk>>,
    c_children_server_volume: Vec<Vec<TableRowPointerServerVolume>>,
    c_children_server_root_volume: Vec<Vec<TableRowPointerServerRootVolume>>,
    c_children_server_xfs_volume: Vec<Vec<TableRowPointerServerXfsVolume>>,
    c_children_network_interface: Vec<Vec<TableRowPointerNetworkInterface>>,
    c_children_server_zpool: Vec<Vec<TableRowPointerServerZpool>>,
}

pub struct TableDefinitionServerDisk {
    rows: Vec<TableRowServerDisk>,
    c_disk_id: Vec<::std::string::String>,
    c_disk_kind: Vec<TableRowPointerDiskKind>,
    c_xfs_format: Vec<bool>,
    c_extra_config: Vec<::std::string::String>,
    c_capacity_bytes: Vec<i64>,
    c_parent: Vec<TableRowPointerServer>,
    c_referrers_server__root_disk: Vec<Vec<TableRowPointerServer>>,
    c_referrers_server_xfs_volume__xfs_disk: Vec<Vec<TableRowPointerServerXfsVolume>>,
    c_referrers_server_zpool_spare__disk_id: Vec<Vec<TableRowPointerServerZpoolSpare>>,
    c_referrers_server_zpool_cache__disk_id: Vec<Vec<TableRowPointerServerZpoolCache>>,
    c_referrers_server_zpool_log__disk_id: Vec<Vec<TableRowPointerServerZpoolLog>>,
    c_referrers_server_zpool_vdev_disk__disk_id: Vec<Vec<TableRowPointerServerZpoolVdevDisk>>,
}

pub struct TableDefinitionServerKind {
    rows: Vec<TableRowServerKind>,
    c_kind: Vec<::std::string::String>,
    c_cores: Vec<i64>,
    c_memory_bytes: Vec<i64>,
    c_architecture: Vec<::std::string::String>,
    c_bare_metal: Vec<bool>,
    c_non_eligible_reason: Vec<::std::string::String>,
    c_children_server_kind_attribute: Vec<Vec<TableRowPointerServerKindAttribute>>,
    c_referrers_datacenter__default_server_kind: Vec<Vec<TableRowPointerDatacenter>>,
}

pub struct TableDefinitionServerKindAttribute {
    rows: Vec<TableRowServerKindAttribute>,
    c_key: Vec<::std::string::String>,
    c_value: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerServerKind>,
}

pub struct TableDefinitionServerLabel {
    rows: Vec<TableRowServerLabel>,
    c_label_name: Vec<TableRowPointerValidServerLabels>,
    c_label_value: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerServer>,
}

pub struct TableDefinitionServerRootVolume {
    rows: Vec<TableRowServerRootVolume>,
    c_volume_name: Vec<::std::string::String>,
    c_intended_usage: Vec<TableRowPointerServerVolumeUsageContract>,
    c_mountpoint: Vec<::std::string::String>,
    c_zfs_recordsize: Vec<::std::string::String>,
    c_zfs_compression: Vec<bool>,
    c_zfs_encryption: Vec<bool>,
    c_parent: Vec<TableRowPointerServer>,
}

pub struct TableDefinitionServerVolume {
    rows: Vec<TableRowServerVolume>,
    c_volume_name: Vec<::std::string::String>,
    c_mountpoint: Vec<::std::string::String>,
    c_intended_usage: Vec<TableRowPointerServerVolumeUsageContract>,
    c_source: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerServer>,
    c_referrers_pg_deployment_instance__pg_server: Vec<Vec<TableRowPointerPgDeploymentInstance>>,
    c_referrers_ch_deployment_instance__ch_server: Vec<Vec<TableRowPointerChDeploymentInstance>>,
    c_referrers_ch_keeper_deployment_instance__keeper_server: Vec<Vec<TableRowPointerChKeeperDeploymentInstance>>,
    c_referrers_nats_deployment_instance__nats_server: Vec<Vec<TableRowPointerNatsDeploymentInstance>>,
    c_referrers_minio_instance__instance_volume: Vec<Vec<TableRowPointerMinioInstance>>,
    c_referrers_monitoring_instance__monitoring_server: Vec<Vec<TableRowPointerMonitoringInstance>>,
    c_referrers_alertmanager_instance__alertmanager_server: Vec<Vec<TableRowPointerAlertmanagerInstance>>,
    c_referrers_blackbox_deployment_task_mount__server_volume: Vec<Vec<TableRowPointerBlackboxDeploymentTaskMount>>,
}

pub struct TableDefinitionServerVolumeUsageContract {
    rows: Vec<TableRowServerVolumeUsageContract>,
    c_usage_contract: Vec<::std::string::String>,
    c_referrers_server_volume__intended_usage: Vec<Vec<TableRowPointerServerVolume>>,
    c_referrers_server_root_volume__intended_usage: Vec<Vec<TableRowPointerServerRootVolume>>,
    c_referrers_server_xfs_volume__intended_usage: Vec<Vec<TableRowPointerServerXfsVolume>>,
    c_referrers_server_zfs_dataset__intended_usage: Vec<Vec<TableRowPointerServerZfsDataset>>,
}

pub struct TableDefinitionServerXfsVolume {
    rows: Vec<TableRowServerXfsVolume>,
    c_volume_name: Vec<::std::string::String>,
    c_xfs_disk: Vec<TableRowPointerServerDisk>,
    c_intended_usage: Vec<TableRowPointerServerVolumeUsageContract>,
    c_parent: Vec<TableRowPointerServer>,
}

pub struct TableDefinitionServerZfsDataset {
    rows: Vec<TableRowServerZfsDataset>,
    c_dataset_name: Vec<::std::string::String>,
    c_intended_usage: Vec<TableRowPointerServerVolumeUsageContract>,
    c_zfs_recordsize: Vec<::std::string::String>,
    c_zfs_compression: Vec<bool>,
    c_zfs_encryption: Vec<bool>,
    c_parent: Vec<TableRowPointerServerZpool>,
}

pub struct TableDefinitionServerZpool {
    rows: Vec<TableRowServerZpool>,
    c_zpool_name: Vec<::std::string::String>,
    c_is_redundant: Vec<bool>,
    c_parent: Vec<TableRowPointerServer>,
    c_children_server_zpool_vdev: Vec<Vec<TableRowPointerServerZpoolVdev>>,
    c_children_server_zpool_spare: Vec<Vec<TableRowPointerServerZpoolSpare>>,
    c_children_server_zpool_cache: Vec<Vec<TableRowPointerServerZpoolCache>>,
    c_children_server_zpool_log: Vec<Vec<TableRowPointerServerZpoolLog>>,
    c_children_server_zfs_dataset: Vec<Vec<TableRowPointerServerZfsDataset>>,
}

pub struct TableDefinitionServerZpoolCache {
    rows: Vec<TableRowServerZpoolCache>,
    c_disk_id: Vec<TableRowPointerServerDisk>,
    c_parent: Vec<TableRowPointerServerZpool>,
}

pub struct TableDefinitionServerZpoolLog {
    rows: Vec<TableRowServerZpoolLog>,
    c_disk_id: Vec<TableRowPointerServerDisk>,
    c_parent: Vec<TableRowPointerServerZpool>,
}

pub struct TableDefinitionServerZpoolSpare {
    rows: Vec<TableRowServerZpoolSpare>,
    c_disk_id: Vec<TableRowPointerServerDisk>,
    c_parent: Vec<TableRowPointerServerZpool>,
}

pub struct TableDefinitionServerZpoolVdev {
    rows: Vec<TableRowServerZpoolVdev>,
    c_vdev_number: Vec<i64>,
    c_vdev_type: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerServerZpool>,
    c_children_server_zpool_vdev_disk: Vec<Vec<TableRowPointerServerZpoolVdevDisk>>,
}

pub struct TableDefinitionServerZpoolVdevDisk {
    rows: Vec<TableRowServerZpoolVdevDisk>,
    c_disk_id: Vec<TableRowPointerServerDisk>,
    c_parent: Vec<TableRowPointerServerZpoolVdev>,
}

pub struct TableDefinitionSubnetRouterFloatingIp {
    rows: Vec<TableRowSubnetRouterFloatingIp>,
    c_ip_address: Vec<::std::string::String>,
}

pub struct TableDefinitionTelegramBot {
    rows: Vec<TableRowTelegramBot>,
    c_bot_name: Vec<::std::string::String>,
    c_bot_token: Vec<::std::string::String>,
    c_referrers_monitoring_cluster_alert_group__telegram_bot: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>>,
}

pub struct TableDefinitionTelegramChannel {
    rows: Vec<TableRowTelegramChannel>,
    c_channel_name: Vec<::std::string::String>,
    c_channel_id: Vec<i64>,
    c_referrers_monitoring_cluster_alert_group__telegram_channel: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>>,
}

pub struct TableDefinitionTempoCluster {
    rows: Vec<TableRowTempoCluster>,
    c_cluster_name: Vec<::std::string::String>,
    c_namespace: Vec<TableRowPointerNomadNamespace>,
    c_region: Vec<TableRowPointerRegion>,
    c_workload_architecture: Vec<::std::string::String>,
    c_docker_image: Vec<TableRowPointerDockerImagePin>,
    c_is_region_default: Vec<bool>,
    c_loki_cluster: Vec<::std::string::String>,
    c_monitoring_cluster: Vec<::std::string::String>,
    c_storage_bucket: Vec<TableRowPointerMinioBucket>,
    c_http_port: Vec<i64>,
    c_grpc_port: Vec<i64>,
    c_p2p_port: Vec<i64>,
    c_otlp_http_port: Vec<i64>,
    c_otlp_grpc_port: Vec<i64>,
    c_tempo_instances: Vec<i64>,
    c_placement: Vec<::std::string::String>,
    c_trace_retention_days: Vec<i64>,
    c_memory_mb: Vec<i64>,
}

pub struct TableDefinitionTld {
    rows: Vec<TableRowTld>,
    c_domain: Vec<::std::string::String>,
    c_expose_admin: Vec<bool>,
    c_automatic_certificates: Vec<bool>,
    c_referrers_region__tld: Vec<Vec<TableRowPointerRegion>>,
    c_referrers_backend_application_deployment_ingress__tld: Vec<Vec<TableRowPointerBackendApplicationDeploymentIngress>>,
    c_referrers_frontend_application_deployment_ingress__tld: Vec<Vec<TableRowPointerFrontendApplicationDeploymentIngress>>,
}

pub struct TableDefinitionUniqueApplicationNames {
    rows: Vec<TableRowUniqueApplicationNames>,
    c_application_name: Vec<::std::string::String>,
    c_source: Vec<::std::string::String>,
}

pub struct TableDefinitionUniqueDeploymentNames {
    rows: Vec<TableRowUniqueDeploymentNames>,
    c_deployment_name: Vec<::std::string::String>,
    c_source: Vec<::std::string::String>,
}

pub struct TableDefinitionValidServerLabels {
    rows: Vec<TableRowValidServerLabels>,
    c_label_name: Vec<::std::string::String>,
    c_referrers_server_label__label_name: Vec<Vec<TableRowPointerServerLabel>>,
}

pub struct TableDefinitionVersionedType {
    rows: Vec<TableRowVersionedType>,
    c_type_name: Vec<::std::string::String>,
    c_children_versioned_type_snapshot: Vec<Vec<TableRowPointerVersionedTypeSnapshot>>,
    c_children_versioned_type_migration: Vec<Vec<TableRowPointerVersionedTypeMigration>>,
    c_referrers_nats_jetstream_stream__stream_type: Vec<Vec<TableRowPointerNatsJetstreamStream>>,
    c_referrers_backend_application_nats_stream__stream_type: Vec<Vec<TableRowPointerBackendApplicationNatsStream>>,
}

pub struct TableDefinitionVersionedTypeMigration {
    rows: Vec<TableRowVersionedTypeMigration>,
    c_version: Vec<i64>,
    c_migration_source: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerVersionedType>,
}

pub struct TableDefinitionVersionedTypeSnapshot {
    rows: Vec<TableRowVersionedTypeSnapshot>,
    c_version: Vec<i64>,
    c_snapshot_source: Vec<::std::string::String>,
    c_parent: Vec<TableRowPointerVersionedType>,
}


// Database definition
pub struct Database {
    admin_ssh_keys: TableDefinitionAdminSshKeys,
    alert: TableDefinitionAlert,
    alert_group: TableDefinitionAlertGroup,
    alert_trigger_test: TableDefinitionAlertTriggerTest,
    alertmanager_instance: TableDefinitionAlertmanagerInstance,
    backend_application: TableDefinitionBackendApplication,
    backend_application_background_job: TableDefinitionBackendApplicationBackgroundJob,
    backend_application_ch_shard: TableDefinitionBackendApplicationChShard,
    backend_application_config: TableDefinitionBackendApplicationConfig,
    backend_application_deployment: TableDefinitionBackendApplicationDeployment,
    backend_application_deployment_ingress: TableDefinitionBackendApplicationDeploymentIngress,
    backend_application_nats_stream: TableDefinitionBackendApplicationNatsStream,
    backend_application_pg_shard: TableDefinitionBackendApplicationPgShard,
    backend_application_s3_bucket: TableDefinitionBackendApplicationS3Bucket,
    backend_http_endpoint: TableDefinitionBackendHttpEndpoint,
    blackbox_deployment: TableDefinitionBlackboxDeployment,
    blackbox_deployment_group: TableDefinitionBlackboxDeploymentGroup,
    blackbox_deployment_local_file: TableDefinitionBlackboxDeploymentLocalFile,
    blackbox_deployment_port: TableDefinitionBlackboxDeploymentPort,
    blackbox_deployment_service_instance: TableDefinitionBlackboxDeploymentServiceInstance,
    blackbox_deployment_service_registration: TableDefinitionBlackboxDeploymentServiceRegistration,
    blackbox_deployment_task: TableDefinitionBlackboxDeploymentTask,
    blackbox_deployment_task_mount: TableDefinitionBlackboxDeploymentTaskMount,
    blackbox_deployment_vault_secret: TableDefinitionBlackboxDeploymentVaultSecret,
    ch_deployment: TableDefinitionChDeployment,
    ch_deployment_instance: TableDefinitionChDeploymentInstance,
    ch_deployment_schemas: TableDefinitionChDeploymentSchemas,
    ch_keeper_deployment: TableDefinitionChKeeperDeployment,
    ch_keeper_deployment_instance: TableDefinitionChKeeperDeploymentInstance,
    ch_migration: TableDefinitionChMigration,
    ch_mutator: TableDefinitionChMutator,
    ch_mutator_test: TableDefinitionChMutatorTest,
    ch_nats_stream_import: TableDefinitionChNatsStreamImport,
    ch_query: TableDefinitionChQuery,
    ch_query_test: TableDefinitionChQueryTest,
    ch_schema: TableDefinitionChSchema,
    ch_test_dataset: TableDefinitionChTestDataset,
    datacenter: TableDefinitionDatacenter,
    disk_kind: TableDefinitionDiskKind,
    docker_image: TableDefinitionDockerImage,
    docker_image_pin: TableDefinitionDockerImagePin,
    docker_image_pin_images: TableDefinitionDockerImagePinImages,
    docker_image_set: TableDefinitionDockerImageSet,
    docker_registry_instance: TableDefinitionDockerRegistryInstance,
    frontend_application: TableDefinitionFrontendApplication,
    frontend_application_deployment: TableDefinitionFrontendApplicationDeployment,
    frontend_application_deployment_ingress: TableDefinitionFrontendApplicationDeploymentIngress,
    frontend_application_external_link: TableDefinitionFrontendApplicationExternalLink,
    frontend_application_external_page: TableDefinitionFrontendApplicationExternalPage,
    frontend_application_used_endpoint: TableDefinitionFrontendApplicationUsedEndpoint,
    frontend_page: TableDefinitionFrontendPage,
    global_settings: TableDefinitionGlobalSettings,
    grafana: TableDefinitionGrafana,
    grafana_dashboard: TableDefinitionGrafanaDashboard,
    http_endpoint_data_type: TableDefinitionHttpEndpointDataType,
    http_methods: TableDefinitionHttpMethods,
    loki_cluster: TableDefinitionLokiCluster,
    minio_bucket: TableDefinitionMinioBucket,
    minio_cluster: TableDefinitionMinioCluster,
    minio_instance: TableDefinitionMinioInstance,
    monitoring_cluster: TableDefinitionMonitoringCluster,
    monitoring_cluster_alert_group: TableDefinitionMonitoringClusterAlertGroup,
    monitoring_cluster_scraped_metric: TableDefinitionMonitoringClusterScrapedMetric,
    monitoring_instance: TableDefinitionMonitoringInstance,
    nats_cluster: TableDefinitionNatsCluster,
    nats_deployment_instance: TableDefinitionNatsDeploymentInstance,
    nats_jetstream_stream: TableDefinitionNatsJetstreamStream,
    network: TableDefinitionNetwork,
    network_interface: TableDefinitionNetworkInterface,
    nixpkgs_environment: TableDefinitionNixpkgsEnvironment,
    nixpkgs_version: TableDefinitionNixpkgsVersion,
    nomad_namespace: TableDefinitionNomadNamespace,
    pg_deployment: TableDefinitionPgDeployment,
    pg_deployment_instance: TableDefinitionPgDeploymentInstance,
    pg_deployment_schemas: TableDefinitionPgDeploymentSchemas,
    pg_deployment_unmanaged_db: TableDefinitionPgDeploymentUnmanagedDb,
    pg_mat_view: TableDefinitionPgMatView,
    pg_mat_view_test: TableDefinitionPgMatViewTest,
    pg_mat_view_update_frequency: TableDefinitionPgMatViewUpdateFrequency,
    pg_migration: TableDefinitionPgMigration,
    pg_mutator: TableDefinitionPgMutator,
    pg_mutator_test: TableDefinitionPgMutatorTest,
    pg_query: TableDefinitionPgQuery,
    pg_query_test: TableDefinitionPgQueryTest,
    pg_schema: TableDefinitionPgSchema,
    pg_test_dataset: TableDefinitionPgTestDataset,
    pg_transaction: TableDefinitionPgTransaction,
    region: TableDefinitionRegion,
    rust_compilation_environment: TableDefinitionRustCompilationEnvironment,
    rust_crate_version: TableDefinitionRustCrateVersion,
    server: TableDefinitionServer,
    server_disk: TableDefinitionServerDisk,
    server_kind: TableDefinitionServerKind,
    server_kind_attribute: TableDefinitionServerKindAttribute,
    server_label: TableDefinitionServerLabel,
    server_root_volume: TableDefinitionServerRootVolume,
    server_volume: TableDefinitionServerVolume,
    server_volume_usage_contract: TableDefinitionServerVolumeUsageContract,
    server_xfs_volume: TableDefinitionServerXfsVolume,
    server_zfs_dataset: TableDefinitionServerZfsDataset,
    server_zpool: TableDefinitionServerZpool,
    server_zpool_cache: TableDefinitionServerZpoolCache,
    server_zpool_log: TableDefinitionServerZpoolLog,
    server_zpool_spare: TableDefinitionServerZpoolSpare,
    server_zpool_vdev: TableDefinitionServerZpoolVdev,
    server_zpool_vdev_disk: TableDefinitionServerZpoolVdevDisk,
    subnet_router_floating_ip: TableDefinitionSubnetRouterFloatingIp,
    telegram_bot: TableDefinitionTelegramBot,
    telegram_channel: TableDefinitionTelegramChannel,
    tempo_cluster: TableDefinitionTempoCluster,
    tld: TableDefinitionTld,
    unique_application_names: TableDefinitionUniqueApplicationNames,
    unique_deployment_names: TableDefinitionUniqueDeploymentNames,
    valid_server_labels: TableDefinitionValidServerLabels,
    versioned_type: TableDefinitionVersionedType,
    versioned_type_migration: TableDefinitionVersionedTypeMigration,
    versioned_type_snapshot: TableDefinitionVersionedTypeSnapshot,
}

// Database implementation
impl Database {
    pub fn admin_ssh_keys(&self) -> &TableDefinitionAdminSshKeys {
        &self.admin_ssh_keys
    }

    pub fn alert(&self) -> &TableDefinitionAlert {
        &self.alert
    }

    pub fn alert_group(&self) -> &TableDefinitionAlertGroup {
        &self.alert_group
    }

    pub fn alert_trigger_test(&self) -> &TableDefinitionAlertTriggerTest {
        &self.alert_trigger_test
    }

    pub fn alertmanager_instance(&self) -> &TableDefinitionAlertmanagerInstance {
        &self.alertmanager_instance
    }

    pub fn backend_application(&self) -> &TableDefinitionBackendApplication {
        &self.backend_application
    }

    pub fn backend_application_background_job(&self) -> &TableDefinitionBackendApplicationBackgroundJob {
        &self.backend_application_background_job
    }

    pub fn backend_application_ch_shard(&self) -> &TableDefinitionBackendApplicationChShard {
        &self.backend_application_ch_shard
    }

    pub fn backend_application_config(&self) -> &TableDefinitionBackendApplicationConfig {
        &self.backend_application_config
    }

    pub fn backend_application_deployment(&self) -> &TableDefinitionBackendApplicationDeployment {
        &self.backend_application_deployment
    }

    pub fn backend_application_deployment_ingress(&self) -> &TableDefinitionBackendApplicationDeploymentIngress {
        &self.backend_application_deployment_ingress
    }

    pub fn backend_application_nats_stream(&self) -> &TableDefinitionBackendApplicationNatsStream {
        &self.backend_application_nats_stream
    }

    pub fn backend_application_pg_shard(&self) -> &TableDefinitionBackendApplicationPgShard {
        &self.backend_application_pg_shard
    }

    pub fn backend_application_s3_bucket(&self) -> &TableDefinitionBackendApplicationS3Bucket {
        &self.backend_application_s3_bucket
    }

    pub fn backend_http_endpoint(&self) -> &TableDefinitionBackendHttpEndpoint {
        &self.backend_http_endpoint
    }

    pub fn blackbox_deployment(&self) -> &TableDefinitionBlackboxDeployment {
        &self.blackbox_deployment
    }

    pub fn blackbox_deployment_group(&self) -> &TableDefinitionBlackboxDeploymentGroup {
        &self.blackbox_deployment_group
    }

    pub fn blackbox_deployment_local_file(&self) -> &TableDefinitionBlackboxDeploymentLocalFile {
        &self.blackbox_deployment_local_file
    }

    pub fn blackbox_deployment_port(&self) -> &TableDefinitionBlackboxDeploymentPort {
        &self.blackbox_deployment_port
    }

    pub fn blackbox_deployment_service_instance(&self) -> &TableDefinitionBlackboxDeploymentServiceInstance {
        &self.blackbox_deployment_service_instance
    }

    pub fn blackbox_deployment_service_registration(&self) -> &TableDefinitionBlackboxDeploymentServiceRegistration {
        &self.blackbox_deployment_service_registration
    }

    pub fn blackbox_deployment_task(&self) -> &TableDefinitionBlackboxDeploymentTask {
        &self.blackbox_deployment_task
    }

    pub fn blackbox_deployment_task_mount(&self) -> &TableDefinitionBlackboxDeploymentTaskMount {
        &self.blackbox_deployment_task_mount
    }

    pub fn blackbox_deployment_vault_secret(&self) -> &TableDefinitionBlackboxDeploymentVaultSecret {
        &self.blackbox_deployment_vault_secret
    }

    pub fn ch_deployment(&self) -> &TableDefinitionChDeployment {
        &self.ch_deployment
    }

    pub fn ch_deployment_instance(&self) -> &TableDefinitionChDeploymentInstance {
        &self.ch_deployment_instance
    }

    pub fn ch_deployment_schemas(&self) -> &TableDefinitionChDeploymentSchemas {
        &self.ch_deployment_schemas
    }

    pub fn ch_keeper_deployment(&self) -> &TableDefinitionChKeeperDeployment {
        &self.ch_keeper_deployment
    }

    pub fn ch_keeper_deployment_instance(&self) -> &TableDefinitionChKeeperDeploymentInstance {
        &self.ch_keeper_deployment_instance
    }

    pub fn ch_migration(&self) -> &TableDefinitionChMigration {
        &self.ch_migration
    }

    pub fn ch_mutator(&self) -> &TableDefinitionChMutator {
        &self.ch_mutator
    }

    pub fn ch_mutator_test(&self) -> &TableDefinitionChMutatorTest {
        &self.ch_mutator_test
    }

    pub fn ch_nats_stream_import(&self) -> &TableDefinitionChNatsStreamImport {
        &self.ch_nats_stream_import
    }

    pub fn ch_query(&self) -> &TableDefinitionChQuery {
        &self.ch_query
    }

    pub fn ch_query_test(&self) -> &TableDefinitionChQueryTest {
        &self.ch_query_test
    }

    pub fn ch_schema(&self) -> &TableDefinitionChSchema {
        &self.ch_schema
    }

    pub fn ch_test_dataset(&self) -> &TableDefinitionChTestDataset {
        &self.ch_test_dataset
    }

    pub fn datacenter(&self) -> &TableDefinitionDatacenter {
        &self.datacenter
    }

    pub fn disk_kind(&self) -> &TableDefinitionDiskKind {
        &self.disk_kind
    }

    pub fn docker_image(&self) -> &TableDefinitionDockerImage {
        &self.docker_image
    }

    pub fn docker_image_pin(&self) -> &TableDefinitionDockerImagePin {
        &self.docker_image_pin
    }

    pub fn docker_image_pin_images(&self) -> &TableDefinitionDockerImagePinImages {
        &self.docker_image_pin_images
    }

    pub fn docker_image_set(&self) -> &TableDefinitionDockerImageSet {
        &self.docker_image_set
    }

    pub fn docker_registry_instance(&self) -> &TableDefinitionDockerRegistryInstance {
        &self.docker_registry_instance
    }

    pub fn frontend_application(&self) -> &TableDefinitionFrontendApplication {
        &self.frontend_application
    }

    pub fn frontend_application_deployment(&self) -> &TableDefinitionFrontendApplicationDeployment {
        &self.frontend_application_deployment
    }

    pub fn frontend_application_deployment_ingress(&self) -> &TableDefinitionFrontendApplicationDeploymentIngress {
        &self.frontend_application_deployment_ingress
    }

    pub fn frontend_application_external_link(&self) -> &TableDefinitionFrontendApplicationExternalLink {
        &self.frontend_application_external_link
    }

    pub fn frontend_application_external_page(&self) -> &TableDefinitionFrontendApplicationExternalPage {
        &self.frontend_application_external_page
    }

    pub fn frontend_application_used_endpoint(&self) -> &TableDefinitionFrontendApplicationUsedEndpoint {
        &self.frontend_application_used_endpoint
    }

    pub fn frontend_page(&self) -> &TableDefinitionFrontendPage {
        &self.frontend_page
    }

    pub fn global_settings(&self) -> &TableDefinitionGlobalSettings {
        &self.global_settings
    }

    pub fn grafana(&self) -> &TableDefinitionGrafana {
        &self.grafana
    }

    pub fn grafana_dashboard(&self) -> &TableDefinitionGrafanaDashboard {
        &self.grafana_dashboard
    }

    pub fn http_endpoint_data_type(&self) -> &TableDefinitionHttpEndpointDataType {
        &self.http_endpoint_data_type
    }

    pub fn http_methods(&self) -> &TableDefinitionHttpMethods {
        &self.http_methods
    }

    pub fn loki_cluster(&self) -> &TableDefinitionLokiCluster {
        &self.loki_cluster
    }

    pub fn minio_bucket(&self) -> &TableDefinitionMinioBucket {
        &self.minio_bucket
    }

    pub fn minio_cluster(&self) -> &TableDefinitionMinioCluster {
        &self.minio_cluster
    }

    pub fn minio_instance(&self) -> &TableDefinitionMinioInstance {
        &self.minio_instance
    }

    pub fn monitoring_cluster(&self) -> &TableDefinitionMonitoringCluster {
        &self.monitoring_cluster
    }

    pub fn monitoring_cluster_alert_group(&self) -> &TableDefinitionMonitoringClusterAlertGroup {
        &self.monitoring_cluster_alert_group
    }

    pub fn monitoring_cluster_scraped_metric(&self) -> &TableDefinitionMonitoringClusterScrapedMetric {
        &self.monitoring_cluster_scraped_metric
    }

    pub fn monitoring_instance(&self) -> &TableDefinitionMonitoringInstance {
        &self.monitoring_instance
    }

    pub fn nats_cluster(&self) -> &TableDefinitionNatsCluster {
        &self.nats_cluster
    }

    pub fn nats_deployment_instance(&self) -> &TableDefinitionNatsDeploymentInstance {
        &self.nats_deployment_instance
    }

    pub fn nats_jetstream_stream(&self) -> &TableDefinitionNatsJetstreamStream {
        &self.nats_jetstream_stream
    }

    pub fn network(&self) -> &TableDefinitionNetwork {
        &self.network
    }

    pub fn network_interface(&self) -> &TableDefinitionNetworkInterface {
        &self.network_interface
    }

    pub fn nixpkgs_environment(&self) -> &TableDefinitionNixpkgsEnvironment {
        &self.nixpkgs_environment
    }

    pub fn nixpkgs_version(&self) -> &TableDefinitionNixpkgsVersion {
        &self.nixpkgs_version
    }

    pub fn nomad_namespace(&self) -> &TableDefinitionNomadNamespace {
        &self.nomad_namespace
    }

    pub fn pg_deployment(&self) -> &TableDefinitionPgDeployment {
        &self.pg_deployment
    }

    pub fn pg_deployment_instance(&self) -> &TableDefinitionPgDeploymentInstance {
        &self.pg_deployment_instance
    }

    pub fn pg_deployment_schemas(&self) -> &TableDefinitionPgDeploymentSchemas {
        &self.pg_deployment_schemas
    }

    pub fn pg_deployment_unmanaged_db(&self) -> &TableDefinitionPgDeploymentUnmanagedDb {
        &self.pg_deployment_unmanaged_db
    }

    pub fn pg_mat_view(&self) -> &TableDefinitionPgMatView {
        &self.pg_mat_view
    }

    pub fn pg_mat_view_test(&self) -> &TableDefinitionPgMatViewTest {
        &self.pg_mat_view_test
    }

    pub fn pg_mat_view_update_frequency(&self) -> &TableDefinitionPgMatViewUpdateFrequency {
        &self.pg_mat_view_update_frequency
    }

    pub fn pg_migration(&self) -> &TableDefinitionPgMigration {
        &self.pg_migration
    }

    pub fn pg_mutator(&self) -> &TableDefinitionPgMutator {
        &self.pg_mutator
    }

    pub fn pg_mutator_test(&self) -> &TableDefinitionPgMutatorTest {
        &self.pg_mutator_test
    }

    pub fn pg_query(&self) -> &TableDefinitionPgQuery {
        &self.pg_query
    }

    pub fn pg_query_test(&self) -> &TableDefinitionPgQueryTest {
        &self.pg_query_test
    }

    pub fn pg_schema(&self) -> &TableDefinitionPgSchema {
        &self.pg_schema
    }

    pub fn pg_test_dataset(&self) -> &TableDefinitionPgTestDataset {
        &self.pg_test_dataset
    }

    pub fn pg_transaction(&self) -> &TableDefinitionPgTransaction {
        &self.pg_transaction
    }

    pub fn region(&self) -> &TableDefinitionRegion {
        &self.region
    }

    pub fn rust_compilation_environment(&self) -> &TableDefinitionRustCompilationEnvironment {
        &self.rust_compilation_environment
    }

    pub fn rust_crate_version(&self) -> &TableDefinitionRustCrateVersion {
        &self.rust_crate_version
    }

    pub fn server(&self) -> &TableDefinitionServer {
        &self.server
    }

    pub fn server_disk(&self) -> &TableDefinitionServerDisk {
        &self.server_disk
    }

    pub fn server_kind(&self) -> &TableDefinitionServerKind {
        &self.server_kind
    }

    pub fn server_kind_attribute(&self) -> &TableDefinitionServerKindAttribute {
        &self.server_kind_attribute
    }

    pub fn server_label(&self) -> &TableDefinitionServerLabel {
        &self.server_label
    }

    pub fn server_root_volume(&self) -> &TableDefinitionServerRootVolume {
        &self.server_root_volume
    }

    pub fn server_volume(&self) -> &TableDefinitionServerVolume {
        &self.server_volume
    }

    pub fn server_volume_usage_contract(&self) -> &TableDefinitionServerVolumeUsageContract {
        &self.server_volume_usage_contract
    }

    pub fn server_xfs_volume(&self) -> &TableDefinitionServerXfsVolume {
        &self.server_xfs_volume
    }

    pub fn server_zfs_dataset(&self) -> &TableDefinitionServerZfsDataset {
        &self.server_zfs_dataset
    }

    pub fn server_zpool(&self) -> &TableDefinitionServerZpool {
        &self.server_zpool
    }

    pub fn server_zpool_cache(&self) -> &TableDefinitionServerZpoolCache {
        &self.server_zpool_cache
    }

    pub fn server_zpool_log(&self) -> &TableDefinitionServerZpoolLog {
        &self.server_zpool_log
    }

    pub fn server_zpool_spare(&self) -> &TableDefinitionServerZpoolSpare {
        &self.server_zpool_spare
    }

    pub fn server_zpool_vdev(&self) -> &TableDefinitionServerZpoolVdev {
        &self.server_zpool_vdev
    }

    pub fn server_zpool_vdev_disk(&self) -> &TableDefinitionServerZpoolVdevDisk {
        &self.server_zpool_vdev_disk
    }

    pub fn subnet_router_floating_ip(&self) -> &TableDefinitionSubnetRouterFloatingIp {
        &self.subnet_router_floating_ip
    }

    pub fn telegram_bot(&self) -> &TableDefinitionTelegramBot {
        &self.telegram_bot
    }

    pub fn telegram_channel(&self) -> &TableDefinitionTelegramChannel {
        &self.telegram_channel
    }

    pub fn tempo_cluster(&self) -> &TableDefinitionTempoCluster {
        &self.tempo_cluster
    }

    pub fn tld(&self) -> &TableDefinitionTld {
        &self.tld
    }

    pub fn unique_application_names(&self) -> &TableDefinitionUniqueApplicationNames {
        &self.unique_application_names
    }

    pub fn unique_deployment_names(&self) -> &TableDefinitionUniqueDeploymentNames {
        &self.unique_deployment_names
    }

    pub fn valid_server_labels(&self) -> &TableDefinitionValidServerLabels {
        &self.valid_server_labels
    }

    pub fn versioned_type(&self) -> &TableDefinitionVersionedType {
        &self.versioned_type
    }

    pub fn versioned_type_migration(&self) -> &TableDefinitionVersionedTypeMigration {
        &self.versioned_type_migration
    }

    pub fn versioned_type_snapshot(&self) -> &TableDefinitionVersionedTypeSnapshot {
        &self.versioned_type_snapshot
    }

    fn deserialize_compressed(compressed: &[u8]) -> Result<Database, Box<dyn ::std::error::Error>> {
        let hash_size = ::std::mem::size_of::<u64>();
        assert!(compressed.len() > hash_size);
        let compressed_end = compressed.len() - hash_size;
        let compressed_slice = &compressed[0..compressed_end];
        let hash_slice = &compressed[compressed_end..];
        let encoded_hash = ::bincode::deserialize::<u64>(hash_slice).unwrap();
        let computed_hash = ::xxhash_rust::xxh3::xxh3_64(compressed_slice);
        if encoded_hash != computed_hash { panic!("EdenDB data is corrupted, checksum mismatch.") }
        let input = ::lz4_flex::decompress_size_prepended(compressed_slice).unwrap();
        Self::deserialize(input.as_slice())
    }

    pub fn deserialize(input: &[u8]) -> Result<Database, Box<dyn ::std::error::Error>> {
        let mut cursor = ::std::io::Cursor::new(input);

        let admin_ssh_keys_contents: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;

        let admin_ssh_keys_len = admin_ssh_keys_contents.len();


        let mut rows_admin_ssh_keys: Vec<TableRowAdminSshKeys> = Vec::with_capacity(admin_ssh_keys_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..admin_ssh_keys_len {
            rows_admin_ssh_keys.push(TableRowAdminSshKeys {
                contents: admin_ssh_keys_contents[row].clone(),
            });
        }

        let alert_alert_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_expr: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_description: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_for_time: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_severity: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_parent: Vec<TableRowPointerAlertGroup> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_children_alert_trigger_test: Vec<Vec<TableRowPointerAlertTriggerTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let alert_len = alert_children_alert_trigger_test.len();

        assert_eq!(alert_len, alert_alert_name.len());
        assert_eq!(alert_len, alert_expr.len());
        assert_eq!(alert_len, alert_description.len());
        assert_eq!(alert_len, alert_for_time.len());
        assert_eq!(alert_len, alert_severity.len());
        assert_eq!(alert_len, alert_parent.len());

        let mut rows_alert: Vec<TableRowAlert> = Vec::with_capacity(alert_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..alert_len {
            rows_alert.push(TableRowAlert {
                alert_name: alert_alert_name[row].clone(),
                expr: alert_expr[row].clone(),
                description: alert_description[row].clone(),
                for_time: alert_for_time[row].clone(),
                severity: alert_severity[row],
                parent: alert_parent[row],
                children_alert_trigger_test: alert_children_alert_trigger_test[row].clone(),
            });
        }

        let alert_group_alert_group_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_group_children_alert: Vec<Vec<TableRowPointerAlert>> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_group_referrers_monitoring_cluster_alert_group__alert_group_name: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>> = ::bincode::deserialize_from(&mut cursor)?;

        let alert_group_len = alert_group_referrers_monitoring_cluster_alert_group__alert_group_name.len();

        assert_eq!(alert_group_len, alert_group_alert_group_name.len());
        assert_eq!(alert_group_len, alert_group_children_alert.len());

        let mut rows_alert_group: Vec<TableRowAlertGroup> = Vec::with_capacity(alert_group_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..alert_group_len {
            rows_alert_group.push(TableRowAlertGroup {
                alert_group_name: alert_group_alert_group_name[row].clone(),
                children_alert: alert_group_children_alert[row].clone(),
                referrers_monitoring_cluster_alert_group__alert_group_name: alert_group_referrers_monitoring_cluster_alert_group__alert_group_name[row].clone(),
            });
        }

        let alert_trigger_test_expected_message: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_trigger_test_expected_labels: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_trigger_test_eval_time: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_trigger_test_interval: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_trigger_test_input_series: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let alert_trigger_test_parent: Vec<TableRowPointerAlert> = ::bincode::deserialize_from(&mut cursor)?;

        let alert_trigger_test_len = alert_trigger_test_parent.len();

        assert_eq!(alert_trigger_test_len, alert_trigger_test_expected_message.len());
        assert_eq!(alert_trigger_test_len, alert_trigger_test_expected_labels.len());
        assert_eq!(alert_trigger_test_len, alert_trigger_test_eval_time.len());
        assert_eq!(alert_trigger_test_len, alert_trigger_test_interval.len());
        assert_eq!(alert_trigger_test_len, alert_trigger_test_input_series.len());

        let mut rows_alert_trigger_test: Vec<TableRowAlertTriggerTest> = Vec::with_capacity(alert_trigger_test_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..alert_trigger_test_len {
            rows_alert_trigger_test.push(TableRowAlertTriggerTest {
                expected_message: alert_trigger_test_expected_message[row].clone(),
                expected_labels: alert_trigger_test_expected_labels[row].clone(),
                eval_time: alert_trigger_test_eval_time[row].clone(),
                interval: alert_trigger_test_interval[row].clone(),
                input_series: alert_trigger_test_input_series[row].clone(),
                parent: alert_trigger_test_parent[row],
            });
        }

        let alertmanager_instance_instance_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let alertmanager_instance_alertmanager_server: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let alertmanager_instance_parent: Vec<TableRowPointerMonitoringCluster> = ::bincode::deserialize_from(&mut cursor)?;

        let alertmanager_instance_len = alertmanager_instance_parent.len();

        assert_eq!(alertmanager_instance_len, alertmanager_instance_instance_id.len());
        assert_eq!(alertmanager_instance_len, alertmanager_instance_alertmanager_server.len());

        let mut rows_alertmanager_instance: Vec<TableRowAlertmanagerInstance> = Vec::with_capacity(alertmanager_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..alertmanager_instance_len {
            rows_alertmanager_instance.push(TableRowAlertmanagerInstance {
                instance_id: alertmanager_instance_instance_id[row],
                alertmanager_server: alertmanager_instance_alertmanager_server[row],
                parent: alertmanager_instance_parent[row],
            });
        }

        let backend_application_application_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_build_environment: Vec<TableRowPointerRustCompilationEnvironment> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_children_backend_application_background_job: Vec<Vec<TableRowPointerBackendApplicationBackgroundJob>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_children_backend_application_config: Vec<Vec<TableRowPointerBackendApplicationConfig>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_children_backend_application_s3_bucket: Vec<Vec<TableRowPointerBackendApplicationS3Bucket>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_children_backend_application_pg_shard: Vec<Vec<TableRowPointerBackendApplicationPgShard>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_children_backend_application_ch_shard: Vec<Vec<TableRowPointerBackendApplicationChShard>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_children_backend_application_nats_stream: Vec<Vec<TableRowPointerBackendApplicationNatsStream>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_children_backend_http_endpoint: Vec<Vec<TableRowPointerBackendHttpEndpoint>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_referrers_backend_application_deployment__application_name: Vec<Vec<TableRowPointerBackendApplicationDeployment>> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_len = backend_application_referrers_backend_application_deployment__application_name.len();

        assert_eq!(backend_application_len, backend_application_application_name.len());
        assert_eq!(backend_application_len, backend_application_build_environment.len());
        assert_eq!(backend_application_len, backend_application_children_backend_application_background_job.len());
        assert_eq!(backend_application_len, backend_application_children_backend_application_config.len());
        assert_eq!(backend_application_len, backend_application_children_backend_application_s3_bucket.len());
        assert_eq!(backend_application_len, backend_application_children_backend_application_pg_shard.len());
        assert_eq!(backend_application_len, backend_application_children_backend_application_ch_shard.len());
        assert_eq!(backend_application_len, backend_application_children_backend_application_nats_stream.len());
        assert_eq!(backend_application_len, backend_application_children_backend_http_endpoint.len());

        let mut rows_backend_application: Vec<TableRowBackendApplication> = Vec::with_capacity(backend_application_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_len {
            rows_backend_application.push(TableRowBackendApplication {
                application_name: backend_application_application_name[row].clone(),
                build_environment: backend_application_build_environment[row],
                children_backend_application_background_job: backend_application_children_backend_application_background_job[row].clone(),
                children_backend_application_config: backend_application_children_backend_application_config[row].clone(),
                children_backend_application_s3_bucket: backend_application_children_backend_application_s3_bucket[row].clone(),
                children_backend_application_pg_shard: backend_application_children_backend_application_pg_shard[row].clone(),
                children_backend_application_ch_shard: backend_application_children_backend_application_ch_shard[row].clone(),
                children_backend_application_nats_stream: backend_application_children_backend_application_nats_stream[row].clone(),
                children_backend_http_endpoint: backend_application_children_backend_http_endpoint[row].clone(),
                referrers_backend_application_deployment__application_name: backend_application_referrers_backend_application_deployment__application_name[row].clone(),
            });
        }

        let backend_application_background_job_job_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_background_job_parent: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_background_job_len = backend_application_background_job_parent.len();

        assert_eq!(backend_application_background_job_len, backend_application_background_job_job_name.len());

        let mut rows_backend_application_background_job: Vec<TableRowBackendApplicationBackgroundJob> = Vec::with_capacity(backend_application_background_job_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_background_job_len {
            rows_backend_application_background_job.push(TableRowBackendApplicationBackgroundJob {
                job_name: backend_application_background_job_job_name[row].clone(),
                parent: backend_application_background_job_parent[row],
            });
        }

        let backend_application_ch_shard_shard_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_ch_shard_ch_schema: Vec<TableRowPointerChSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_ch_shard_used_queries: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_ch_shard_used_inserters: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_ch_shard_used_mutators: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_ch_shard_parent: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_ch_shard_len = backend_application_ch_shard_parent.len();

        assert_eq!(backend_application_ch_shard_len, backend_application_ch_shard_shard_name.len());
        assert_eq!(backend_application_ch_shard_len, backend_application_ch_shard_ch_schema.len());
        assert_eq!(backend_application_ch_shard_len, backend_application_ch_shard_used_queries.len());
        assert_eq!(backend_application_ch_shard_len, backend_application_ch_shard_used_inserters.len());
        assert_eq!(backend_application_ch_shard_len, backend_application_ch_shard_used_mutators.len());

        let mut rows_backend_application_ch_shard: Vec<TableRowBackendApplicationChShard> = Vec::with_capacity(backend_application_ch_shard_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_ch_shard_len {
            rows_backend_application_ch_shard.push(TableRowBackendApplicationChShard {
                shard_name: backend_application_ch_shard_shard_name[row].clone(),
                ch_schema: backend_application_ch_shard_ch_schema[row],
                used_queries: backend_application_ch_shard_used_queries[row].clone(),
                used_inserters: backend_application_ch_shard_used_inserters[row].clone(),
                used_mutators: backend_application_ch_shard_used_mutators[row].clone(),
                parent: backend_application_ch_shard_parent[row],
            });
        }

        let backend_application_config_config_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_config_config_type: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_config_default_value: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_config_min_value: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_config_max_value: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_config_regex_check: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_config_parent: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_config_len = backend_application_config_parent.len();

        assert_eq!(backend_application_config_len, backend_application_config_config_name.len());
        assert_eq!(backend_application_config_len, backend_application_config_config_type.len());
        assert_eq!(backend_application_config_len, backend_application_config_default_value.len());
        assert_eq!(backend_application_config_len, backend_application_config_min_value.len());
        assert_eq!(backend_application_config_len, backend_application_config_max_value.len());
        assert_eq!(backend_application_config_len, backend_application_config_regex_check.len());

        let mut rows_backend_application_config: Vec<TableRowBackendApplicationConfig> = Vec::with_capacity(backend_application_config_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_config_len {
            rows_backend_application_config.push(TableRowBackendApplicationConfig {
                config_name: backend_application_config_config_name[row].clone(),
                config_type: backend_application_config_config_type[row].clone(),
                default_value: backend_application_config_default_value[row].clone(),
                min_value: backend_application_config_min_value[row].clone(),
                max_value: backend_application_config_max_value[row].clone(),
                regex_check: backend_application_config_regex_check[row].clone(),
                parent: backend_application_config_parent[row],
            });
        }

        let backend_application_deployment_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_application_name: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_count: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_pg_shard_wiring: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_ch_shard_wiring: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_nats_stream_wiring: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_s3_bucket_wiring: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_config: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_tracing_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_referrers_backend_application_deployment_ingress__deployment: Vec<Vec<TableRowPointerBackendApplicationDeploymentIngress>> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_deployment_len = backend_application_deployment_referrers_backend_application_deployment_ingress__deployment.len();

        assert_eq!(backend_application_deployment_len, backend_application_deployment_deployment_name.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_namespace.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_application_name.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_workload_architecture.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_count.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_placement.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_pg_shard_wiring.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_ch_shard_wiring.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_nats_stream_wiring.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_s3_bucket_wiring.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_config.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_http_port.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_memory_mb.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_region.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_loki_cluster.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_monitoring_cluster.len());
        assert_eq!(backend_application_deployment_len, backend_application_deployment_tracing_cluster.len());

        let mut rows_backend_application_deployment: Vec<TableRowBackendApplicationDeployment> = Vec::with_capacity(backend_application_deployment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_deployment_len {
            rows_backend_application_deployment.push(TableRowBackendApplicationDeployment {
                deployment_name: backend_application_deployment_deployment_name[row].clone(),
                namespace: backend_application_deployment_namespace[row],
                application_name: backend_application_deployment_application_name[row],
                workload_architecture: backend_application_deployment_workload_architecture[row].clone(),
                count: backend_application_deployment_count[row],
                placement: backend_application_deployment_placement[row].clone(),
                pg_shard_wiring: backend_application_deployment_pg_shard_wiring[row].clone(),
                ch_shard_wiring: backend_application_deployment_ch_shard_wiring[row].clone(),
                nats_stream_wiring: backend_application_deployment_nats_stream_wiring[row].clone(),
                s3_bucket_wiring: backend_application_deployment_s3_bucket_wiring[row].clone(),
                config: backend_application_deployment_config[row].clone(),
                http_port: backend_application_deployment_http_port[row],
                memory_mb: backend_application_deployment_memory_mb[row],
                region: backend_application_deployment_region[row],
                loki_cluster: backend_application_deployment_loki_cluster[row].clone(),
                monitoring_cluster: backend_application_deployment_monitoring_cluster[row].clone(),
                tracing_cluster: backend_application_deployment_tracing_cluster[row].clone(),
                referrers_backend_application_deployment_ingress__deployment: backend_application_deployment_referrers_backend_application_deployment_ingress__deployment[row].clone(),
            });
        }

        let backend_application_deployment_ingress_deployment: Vec<TableRowPointerBackendApplicationDeployment> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_ingress_mountpoint: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_ingress_subdomain: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_ingress_tld: Vec<TableRowPointerTld> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_deployment_ingress_endpoint_list: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_deployment_ingress_len = backend_application_deployment_ingress_endpoint_list.len();

        assert_eq!(backend_application_deployment_ingress_len, backend_application_deployment_ingress_deployment.len());
        assert_eq!(backend_application_deployment_ingress_len, backend_application_deployment_ingress_mountpoint.len());
        assert_eq!(backend_application_deployment_ingress_len, backend_application_deployment_ingress_subdomain.len());
        assert_eq!(backend_application_deployment_ingress_len, backend_application_deployment_ingress_tld.len());

        let mut rows_backend_application_deployment_ingress: Vec<TableRowBackendApplicationDeploymentIngress> = Vec::with_capacity(backend_application_deployment_ingress_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_deployment_ingress_len {
            rows_backend_application_deployment_ingress.push(TableRowBackendApplicationDeploymentIngress {
                deployment: backend_application_deployment_ingress_deployment[row],
                mountpoint: backend_application_deployment_ingress_mountpoint[row].clone(),
                subdomain: backend_application_deployment_ingress_subdomain[row].clone(),
                tld: backend_application_deployment_ingress_tld[row],
                endpoint_list: backend_application_deployment_ingress_endpoint_list[row].clone(),
            });
        }

        let backend_application_nats_stream_stream_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_nats_stream_stream_type: Vec<TableRowPointerVersionedType> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_nats_stream_enable_consumer: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_nats_stream_enable_producer: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_nats_stream_is_batch_consumer: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_nats_stream_enable_subjects: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_nats_stream_parent: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_nats_stream_len = backend_application_nats_stream_parent.len();

        assert_eq!(backend_application_nats_stream_len, backend_application_nats_stream_stream_name.len());
        assert_eq!(backend_application_nats_stream_len, backend_application_nats_stream_stream_type.len());
        assert_eq!(backend_application_nats_stream_len, backend_application_nats_stream_enable_consumer.len());
        assert_eq!(backend_application_nats_stream_len, backend_application_nats_stream_enable_producer.len());
        assert_eq!(backend_application_nats_stream_len, backend_application_nats_stream_is_batch_consumer.len());
        assert_eq!(backend_application_nats_stream_len, backend_application_nats_stream_enable_subjects.len());

        let mut rows_backend_application_nats_stream: Vec<TableRowBackendApplicationNatsStream> = Vec::with_capacity(backend_application_nats_stream_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_nats_stream_len {
            rows_backend_application_nats_stream.push(TableRowBackendApplicationNatsStream {
                stream_name: backend_application_nats_stream_stream_name[row].clone(),
                stream_type: backend_application_nats_stream_stream_type[row],
                enable_consumer: backend_application_nats_stream_enable_consumer[row],
                enable_producer: backend_application_nats_stream_enable_producer[row],
                is_batch_consumer: backend_application_nats_stream_is_batch_consumer[row],
                enable_subjects: backend_application_nats_stream_enable_subjects[row],
                parent: backend_application_nats_stream_parent[row],
            });
        }

        let backend_application_pg_shard_shard_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_pg_shard_pg_schema: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_pg_shard_used_queries: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_pg_shard_used_mutators: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_pg_shard_used_transactions: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_pg_shard_parent: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_pg_shard_len = backend_application_pg_shard_parent.len();

        assert_eq!(backend_application_pg_shard_len, backend_application_pg_shard_shard_name.len());
        assert_eq!(backend_application_pg_shard_len, backend_application_pg_shard_pg_schema.len());
        assert_eq!(backend_application_pg_shard_len, backend_application_pg_shard_used_queries.len());
        assert_eq!(backend_application_pg_shard_len, backend_application_pg_shard_used_mutators.len());
        assert_eq!(backend_application_pg_shard_len, backend_application_pg_shard_used_transactions.len());

        let mut rows_backend_application_pg_shard: Vec<TableRowBackendApplicationPgShard> = Vec::with_capacity(backend_application_pg_shard_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_pg_shard_len {
            rows_backend_application_pg_shard.push(TableRowBackendApplicationPgShard {
                shard_name: backend_application_pg_shard_shard_name[row].clone(),
                pg_schema: backend_application_pg_shard_pg_schema[row],
                used_queries: backend_application_pg_shard_used_queries[row].clone(),
                used_mutators: backend_application_pg_shard_used_mutators[row].clone(),
                used_transactions: backend_application_pg_shard_used_transactions[row].clone(),
                parent: backend_application_pg_shard_parent[row],
            });
        }

        let backend_application_s3_bucket_bucket_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_application_s3_bucket_parent: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_application_s3_bucket_len = backend_application_s3_bucket_parent.len();

        assert_eq!(backend_application_s3_bucket_len, backend_application_s3_bucket_bucket_name.len());

        let mut rows_backend_application_s3_bucket: Vec<TableRowBackendApplicationS3Bucket> = Vec::with_capacity(backend_application_s3_bucket_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_application_s3_bucket_len {
            rows_backend_application_s3_bucket.push(TableRowBackendApplicationS3Bucket {
                bucket_name: backend_application_s3_bucket_bucket_name[row].clone(),
                parent: backend_application_s3_bucket_parent[row],
            });
        }

        let backend_http_endpoint_http_endpoint_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_path: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_http_method: Vec<TableRowPointerHttpMethods> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_input_body_type: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_output_body_type: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_data_type: Vec<TableRowPointerHttpEndpointDataType> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_max_input_body_size_bytes: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_needs_headers: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_receive_body_as_stream: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_parent: Vec<TableRowPointerBackendApplication> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_referrers_frontend_application_used_endpoint__backend_endpoint: Vec<Vec<TableRowPointerFrontendApplicationUsedEndpoint>> = ::bincode::deserialize_from(&mut cursor)?;
        let backend_http_endpoint_referrers_frontend_application_external_link__backend_endpoint: Vec<Vec<TableRowPointerFrontendApplicationExternalLink>> = ::bincode::deserialize_from(&mut cursor)?;

        let backend_http_endpoint_len = backend_http_endpoint_referrers_frontend_application_external_link__backend_endpoint.len();

        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_http_endpoint_name.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_path.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_http_method.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_input_body_type.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_output_body_type.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_data_type.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_max_input_body_size_bytes.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_needs_headers.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_receive_body_as_stream.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_parent.len());
        assert_eq!(backend_http_endpoint_len, backend_http_endpoint_referrers_frontend_application_used_endpoint__backend_endpoint.len());

        let mut rows_backend_http_endpoint: Vec<TableRowBackendHttpEndpoint> = Vec::with_capacity(backend_http_endpoint_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..backend_http_endpoint_len {
            rows_backend_http_endpoint.push(TableRowBackendHttpEndpoint {
                http_endpoint_name: backend_http_endpoint_http_endpoint_name[row].clone(),
                path: backend_http_endpoint_path[row].clone(),
                http_method: backend_http_endpoint_http_method[row],
                input_body_type: backend_http_endpoint_input_body_type[row].clone(),
                output_body_type: backend_http_endpoint_output_body_type[row].clone(),
                data_type: backend_http_endpoint_data_type[row],
                max_input_body_size_bytes: backend_http_endpoint_max_input_body_size_bytes[row],
                needs_headers: backend_http_endpoint_needs_headers[row],
                receive_body_as_stream: backend_http_endpoint_receive_body_as_stream[row],
                parent: backend_http_endpoint_parent[row],
                referrers_frontend_application_used_endpoint__backend_endpoint: backend_http_endpoint_referrers_frontend_application_used_endpoint__backend_endpoint[row].clone(),
                referrers_frontend_application_external_link__backend_endpoint: backend_http_endpoint_referrers_frontend_application_external_link__backend_endpoint[row].clone(),
            });
        }

        let blackbox_deployment_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_children_blackbox_deployment_group: Vec<Vec<TableRowPointerBlackboxDeploymentGroup>> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_children_blackbox_deployment_service_registration: Vec<Vec<TableRowPointerBlackboxDeploymentServiceRegistration>> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_len = blackbox_deployment_children_blackbox_deployment_service_registration.len();

        assert_eq!(blackbox_deployment_len, blackbox_deployment_deployment_name.len());
        assert_eq!(blackbox_deployment_len, blackbox_deployment_namespace.len());
        assert_eq!(blackbox_deployment_len, blackbox_deployment_region.len());
        assert_eq!(blackbox_deployment_len, blackbox_deployment_loki_cluster.len());
        assert_eq!(blackbox_deployment_len, blackbox_deployment_monitoring_cluster.len());
        assert_eq!(blackbox_deployment_len, blackbox_deployment_children_blackbox_deployment_group.len());

        let mut rows_blackbox_deployment: Vec<TableRowBlackboxDeployment> = Vec::with_capacity(blackbox_deployment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_len {
            rows_blackbox_deployment.push(TableRowBlackboxDeployment {
                deployment_name: blackbox_deployment_deployment_name[row].clone(),
                namespace: blackbox_deployment_namespace[row],
                region: blackbox_deployment_region[row],
                loki_cluster: blackbox_deployment_loki_cluster[row].clone(),
                monitoring_cluster: blackbox_deployment_monitoring_cluster[row].clone(),
                children_blackbox_deployment_group: blackbox_deployment_children_blackbox_deployment_group[row].clone(),
                children_blackbox_deployment_service_registration: blackbox_deployment_children_blackbox_deployment_service_registration[row].clone(),
            });
        }

        let blackbox_deployment_group_group_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_group_count: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_group_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_group_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_group_parent: Vec<TableRowPointerBlackboxDeployment> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_group_children_blackbox_deployment_port: Vec<Vec<TableRowPointerBlackboxDeploymentPort>> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_group_children_blackbox_deployment_task: Vec<Vec<TableRowPointerBlackboxDeploymentTask>> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_group_children_blackbox_deployment_service_instance: Vec<Vec<TableRowPointerBlackboxDeploymentServiceInstance>> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_group_len = blackbox_deployment_group_children_blackbox_deployment_service_instance.len();

        assert_eq!(blackbox_deployment_group_len, blackbox_deployment_group_group_name.len());
        assert_eq!(blackbox_deployment_group_len, blackbox_deployment_group_count.len());
        assert_eq!(blackbox_deployment_group_len, blackbox_deployment_group_workload_architecture.len());
        assert_eq!(blackbox_deployment_group_len, blackbox_deployment_group_placement.len());
        assert_eq!(blackbox_deployment_group_len, blackbox_deployment_group_parent.len());
        assert_eq!(blackbox_deployment_group_len, blackbox_deployment_group_children_blackbox_deployment_port.len());
        assert_eq!(blackbox_deployment_group_len, blackbox_deployment_group_children_blackbox_deployment_task.len());

        let mut rows_blackbox_deployment_group: Vec<TableRowBlackboxDeploymentGroup> = Vec::with_capacity(blackbox_deployment_group_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_group_len {
            rows_blackbox_deployment_group.push(TableRowBlackboxDeploymentGroup {
                group_name: blackbox_deployment_group_group_name[row].clone(),
                count: blackbox_deployment_group_count[row],
                workload_architecture: blackbox_deployment_group_workload_architecture[row].clone(),
                placement: blackbox_deployment_group_placement[row].clone(),
                parent: blackbox_deployment_group_parent[row],
                children_blackbox_deployment_port: blackbox_deployment_group_children_blackbox_deployment_port[row].clone(),
                children_blackbox_deployment_task: blackbox_deployment_group_children_blackbox_deployment_task[row].clone(),
                children_blackbox_deployment_service_instance: blackbox_deployment_group_children_blackbox_deployment_service_instance[row].clone(),
            });
        }

        let blackbox_deployment_local_file_local_file_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_local_file_local_file_contents: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_local_file_mode: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_local_file_parent: Vec<TableRowPointerBlackboxDeploymentTask> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_local_file_len = blackbox_deployment_local_file_parent.len();

        assert_eq!(blackbox_deployment_local_file_len, blackbox_deployment_local_file_local_file_name.len());
        assert_eq!(blackbox_deployment_local_file_len, blackbox_deployment_local_file_local_file_contents.len());
        assert_eq!(blackbox_deployment_local_file_len, blackbox_deployment_local_file_mode.len());

        let mut rows_blackbox_deployment_local_file: Vec<TableRowBlackboxDeploymentLocalFile> = Vec::with_capacity(blackbox_deployment_local_file_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_local_file_len {
            rows_blackbox_deployment_local_file.push(TableRowBlackboxDeploymentLocalFile {
                local_file_name: blackbox_deployment_local_file_local_file_name[row].clone(),
                local_file_contents: blackbox_deployment_local_file_local_file_contents[row].clone(),
                mode: blackbox_deployment_local_file_mode[row].clone(),
                parent: blackbox_deployment_local_file_parent[row],
            });
        }

        let blackbox_deployment_port_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_port_port_description: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_port_protocol: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_port_parent: Vec<TableRowPointerBlackboxDeploymentGroup> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_port_referrers_blackbox_deployment_service_instance__port: Vec<Vec<TableRowPointerBlackboxDeploymentServiceInstance>> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_port_len = blackbox_deployment_port_referrers_blackbox_deployment_service_instance__port.len();

        assert_eq!(blackbox_deployment_port_len, blackbox_deployment_port_port.len());
        assert_eq!(blackbox_deployment_port_len, blackbox_deployment_port_port_description.len());
        assert_eq!(blackbox_deployment_port_len, blackbox_deployment_port_protocol.len());
        assert_eq!(blackbox_deployment_port_len, blackbox_deployment_port_parent.len());

        let mut rows_blackbox_deployment_port: Vec<TableRowBlackboxDeploymentPort> = Vec::with_capacity(blackbox_deployment_port_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_port_len {
            rows_blackbox_deployment_port.push(TableRowBlackboxDeploymentPort {
                port: blackbox_deployment_port_port[row],
                port_description: blackbox_deployment_port_port_description[row].clone(),
                protocol: blackbox_deployment_port_protocol[row].clone(),
                parent: blackbox_deployment_port_parent[row],
                referrers_blackbox_deployment_service_instance__port: blackbox_deployment_port_referrers_blackbox_deployment_service_instance__port[row].clone(),
            });
        }

        let blackbox_deployment_service_instance_service_registration: Vec<TableRowPointerBlackboxDeploymentServiceRegistration> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_service_instance_port: Vec<TableRowPointerBlackboxDeploymentPort> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_service_instance_parent: Vec<TableRowPointerBlackboxDeploymentGroup> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_service_instance_len = blackbox_deployment_service_instance_parent.len();

        assert_eq!(blackbox_deployment_service_instance_len, blackbox_deployment_service_instance_service_registration.len());
        assert_eq!(blackbox_deployment_service_instance_len, blackbox_deployment_service_instance_port.len());

        let mut rows_blackbox_deployment_service_instance: Vec<TableRowBlackboxDeploymentServiceInstance> = Vec::with_capacity(blackbox_deployment_service_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_service_instance_len {
            rows_blackbox_deployment_service_instance.push(TableRowBlackboxDeploymentServiceInstance {
                service_registration: blackbox_deployment_service_instance_service_registration[row],
                port: blackbox_deployment_service_instance_port[row],
                parent: blackbox_deployment_service_instance_parent[row],
            });
        }

        let blackbox_deployment_service_registration_service_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_service_registration_scrape_prometheus_metrics: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_service_registration_prometheus_metrics_path: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_service_registration_min_instances: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_service_registration_parent: Vec<TableRowPointerBlackboxDeployment> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_service_registration_referrers_blackbox_deployment_service_instance__service_registration: Vec<Vec<TableRowPointerBlackboxDeploymentServiceInstance>> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_service_registration_len = blackbox_deployment_service_registration_referrers_blackbox_deployment_service_instance__service_registration.len();

        assert_eq!(blackbox_deployment_service_registration_len, blackbox_deployment_service_registration_service_name.len());
        assert_eq!(blackbox_deployment_service_registration_len, blackbox_deployment_service_registration_scrape_prometheus_metrics.len());
        assert_eq!(blackbox_deployment_service_registration_len, blackbox_deployment_service_registration_prometheus_metrics_path.len());
        assert_eq!(blackbox_deployment_service_registration_len, blackbox_deployment_service_registration_min_instances.len());
        assert_eq!(blackbox_deployment_service_registration_len, blackbox_deployment_service_registration_parent.len());

        let mut rows_blackbox_deployment_service_registration: Vec<TableRowBlackboxDeploymentServiceRegistration> = Vec::with_capacity(blackbox_deployment_service_registration_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_service_registration_len {
            rows_blackbox_deployment_service_registration.push(TableRowBlackboxDeploymentServiceRegistration {
                service_name: blackbox_deployment_service_registration_service_name[row].clone(),
                scrape_prometheus_metrics: blackbox_deployment_service_registration_scrape_prometheus_metrics[row],
                prometheus_metrics_path: blackbox_deployment_service_registration_prometheus_metrics_path[row].clone(),
                min_instances: blackbox_deployment_service_registration_min_instances[row],
                parent: blackbox_deployment_service_registration_parent[row],
                referrers_blackbox_deployment_service_instance__service_registration: blackbox_deployment_service_registration_referrers_blackbox_deployment_service_instance__service_registration[row].clone(),
            });
        }

        let blackbox_deployment_task_task_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_docker_image: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_docker_image_set: Vec<TableRowPointerDockerImageSet> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_memory_oversubscription_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_entrypoint: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_args: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_parent: Vec<TableRowPointerBlackboxDeploymentGroup> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_children_blackbox_deployment_task_mount: Vec<Vec<TableRowPointerBlackboxDeploymentTaskMount>> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_children_blackbox_deployment_vault_secret: Vec<Vec<TableRowPointerBlackboxDeploymentVaultSecret>> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_children_blackbox_deployment_local_file: Vec<Vec<TableRowPointerBlackboxDeploymentLocalFile>> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_task_len = blackbox_deployment_task_children_blackbox_deployment_local_file.len();

        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_task_name.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_docker_image.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_docker_image_set.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_memory_mb.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_memory_oversubscription_mb.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_entrypoint.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_args.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_parent.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_children_blackbox_deployment_task_mount.len());
        assert_eq!(blackbox_deployment_task_len, blackbox_deployment_task_children_blackbox_deployment_vault_secret.len());

        let mut rows_blackbox_deployment_task: Vec<TableRowBlackboxDeploymentTask> = Vec::with_capacity(blackbox_deployment_task_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_task_len {
            rows_blackbox_deployment_task.push(TableRowBlackboxDeploymentTask {
                task_name: blackbox_deployment_task_task_name[row].clone(),
                docker_image: blackbox_deployment_task_docker_image[row],
                docker_image_set: blackbox_deployment_task_docker_image_set[row],
                memory_mb: blackbox_deployment_task_memory_mb[row],
                memory_oversubscription_mb: blackbox_deployment_task_memory_oversubscription_mb[row],
                entrypoint: blackbox_deployment_task_entrypoint[row].clone(),
                args: blackbox_deployment_task_args[row].clone(),
                parent: blackbox_deployment_task_parent[row],
                children_blackbox_deployment_task_mount: blackbox_deployment_task_children_blackbox_deployment_task_mount[row].clone(),
                children_blackbox_deployment_vault_secret: blackbox_deployment_task_children_blackbox_deployment_vault_secret[row].clone(),
                children_blackbox_deployment_local_file: blackbox_deployment_task_children_blackbox_deployment_local_file[row].clone(),
            });
        }

        let blackbox_deployment_task_mount_target_path: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_mount_server_volume: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_task_mount_parent: Vec<TableRowPointerBlackboxDeploymentTask> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_task_mount_len = blackbox_deployment_task_mount_parent.len();

        assert_eq!(blackbox_deployment_task_mount_len, blackbox_deployment_task_mount_target_path.len());
        assert_eq!(blackbox_deployment_task_mount_len, blackbox_deployment_task_mount_server_volume.len());

        let mut rows_blackbox_deployment_task_mount: Vec<TableRowBlackboxDeploymentTaskMount> = Vec::with_capacity(blackbox_deployment_task_mount_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_task_mount_len {
            rows_blackbox_deployment_task_mount.push(TableRowBlackboxDeploymentTaskMount {
                target_path: blackbox_deployment_task_mount_target_path[row].clone(),
                server_volume: blackbox_deployment_task_mount_server_volume[row],
                parent: blackbox_deployment_task_mount_parent[row],
            });
        }

        let blackbox_deployment_vault_secret_secret_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_vault_secret_target_file_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_vault_secret_target_env_var_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let blackbox_deployment_vault_secret_parent: Vec<TableRowPointerBlackboxDeploymentTask> = ::bincode::deserialize_from(&mut cursor)?;

        let blackbox_deployment_vault_secret_len = blackbox_deployment_vault_secret_parent.len();

        assert_eq!(blackbox_deployment_vault_secret_len, blackbox_deployment_vault_secret_secret_name.len());
        assert_eq!(blackbox_deployment_vault_secret_len, blackbox_deployment_vault_secret_target_file_name.len());
        assert_eq!(blackbox_deployment_vault_secret_len, blackbox_deployment_vault_secret_target_env_var_name.len());

        let mut rows_blackbox_deployment_vault_secret: Vec<TableRowBlackboxDeploymentVaultSecret> = Vec::with_capacity(blackbox_deployment_vault_secret_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..blackbox_deployment_vault_secret_len {
            rows_blackbox_deployment_vault_secret.push(TableRowBlackboxDeploymentVaultSecret {
                secret_name: blackbox_deployment_vault_secret_secret_name[row].clone(),
                target_file_name: blackbox_deployment_vault_secret_target_file_name[row].clone(),
                target_env_var_name: blackbox_deployment_vault_secret_target_env_var_name[row].clone(),
                parent: blackbox_deployment_vault_secret_parent[row],
            });
        }

        let ch_deployment_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_docker_image: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_keeper: Vec<TableRowPointerChKeeperDeployment> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_extra_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_mark_cache_size_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_index_mark_cache_size_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_uncompressed_cache_size_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_compiled_expression_cache_size_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_query_cache_size_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_max_thread_pool_size: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_max_concurrent_queries: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_merge_max_block_size: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_max_bytes_to_merge_at_max_space_in_pool_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_max_query_execution_time_seconds: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_queue_max_wait_ms: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_distribute_over_dcs: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_native_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_replication_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_prometheus_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_children_ch_deployment_instance: Vec<Vec<TableRowPointerChDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_children_ch_deployment_schemas: Vec<Vec<TableRowPointerChDeploymentSchemas>> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_deployment_len = ch_deployment_children_ch_deployment_schemas.len();

        assert_eq!(ch_deployment_len, ch_deployment_deployment_name.len());
        assert_eq!(ch_deployment_len, ch_deployment_namespace.len());
        assert_eq!(ch_deployment_len, ch_deployment_region.len());
        assert_eq!(ch_deployment_len, ch_deployment_loki_cluster.len());
        assert_eq!(ch_deployment_len, ch_deployment_monitoring_cluster.len());
        assert_eq!(ch_deployment_len, ch_deployment_docker_image.len());
        assert_eq!(ch_deployment_len, ch_deployment_workload_architecture.len());
        assert_eq!(ch_deployment_len, ch_deployment_keeper.len());
        assert_eq!(ch_deployment_len, ch_deployment_extra_memory_mb.len());
        assert_eq!(ch_deployment_len, ch_deployment_mark_cache_size_mb.len());
        assert_eq!(ch_deployment_len, ch_deployment_index_mark_cache_size_mb.len());
        assert_eq!(ch_deployment_len, ch_deployment_uncompressed_cache_size_mb.len());
        assert_eq!(ch_deployment_len, ch_deployment_compiled_expression_cache_size_mb.len());
        assert_eq!(ch_deployment_len, ch_deployment_query_cache_size_mb.len());
        assert_eq!(ch_deployment_len, ch_deployment_max_thread_pool_size.len());
        assert_eq!(ch_deployment_len, ch_deployment_max_concurrent_queries.len());
        assert_eq!(ch_deployment_len, ch_deployment_merge_max_block_size.len());
        assert_eq!(ch_deployment_len, ch_deployment_max_bytes_to_merge_at_max_space_in_pool_mb.len());
        assert_eq!(ch_deployment_len, ch_deployment_max_query_execution_time_seconds.len());
        assert_eq!(ch_deployment_len, ch_deployment_queue_max_wait_ms.len());
        assert_eq!(ch_deployment_len, ch_deployment_distribute_over_dcs.len());
        assert_eq!(ch_deployment_len, ch_deployment_native_port.len());
        assert_eq!(ch_deployment_len, ch_deployment_http_port.len());
        assert_eq!(ch_deployment_len, ch_deployment_replication_port.len());
        assert_eq!(ch_deployment_len, ch_deployment_prometheus_port.len());
        assert_eq!(ch_deployment_len, ch_deployment_children_ch_deployment_instance.len());

        let mut rows_ch_deployment: Vec<TableRowChDeployment> = Vec::with_capacity(ch_deployment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_deployment_len {
            rows_ch_deployment.push(TableRowChDeployment {
                deployment_name: ch_deployment_deployment_name[row].clone(),
                namespace: ch_deployment_namespace[row],
                region: ch_deployment_region[row],
                loki_cluster: ch_deployment_loki_cluster[row].clone(),
                monitoring_cluster: ch_deployment_monitoring_cluster[row].clone(),
                docker_image: ch_deployment_docker_image[row],
                workload_architecture: ch_deployment_workload_architecture[row].clone(),
                keeper: ch_deployment_keeper[row],
                extra_memory_mb: ch_deployment_extra_memory_mb[row],
                mark_cache_size_mb: ch_deployment_mark_cache_size_mb[row],
                index_mark_cache_size_mb: ch_deployment_index_mark_cache_size_mb[row],
                uncompressed_cache_size_mb: ch_deployment_uncompressed_cache_size_mb[row],
                compiled_expression_cache_size_mb: ch_deployment_compiled_expression_cache_size_mb[row],
                query_cache_size_mb: ch_deployment_query_cache_size_mb[row],
                max_thread_pool_size: ch_deployment_max_thread_pool_size[row],
                max_concurrent_queries: ch_deployment_max_concurrent_queries[row],
                merge_max_block_size: ch_deployment_merge_max_block_size[row],
                max_bytes_to_merge_at_max_space_in_pool_mb: ch_deployment_max_bytes_to_merge_at_max_space_in_pool_mb[row],
                max_query_execution_time_seconds: ch_deployment_max_query_execution_time_seconds[row],
                queue_max_wait_ms: ch_deployment_queue_max_wait_ms[row],
                distribute_over_dcs: ch_deployment_distribute_over_dcs[row],
                native_port: ch_deployment_native_port[row],
                http_port: ch_deployment_http_port[row],
                replication_port: ch_deployment_replication_port[row],
                prometheus_port: ch_deployment_prometheus_port[row],
                children_ch_deployment_instance: ch_deployment_children_ch_deployment_instance[row].clone(),
                children_ch_deployment_schemas: ch_deployment_children_ch_deployment_schemas[row].clone(),
            });
        }

        let ch_deployment_instance_instance_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_instance_ch_server: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_instance_parent: Vec<TableRowPointerChDeployment> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_deployment_instance_len = ch_deployment_instance_parent.len();

        assert_eq!(ch_deployment_instance_len, ch_deployment_instance_instance_id.len());
        assert_eq!(ch_deployment_instance_len, ch_deployment_instance_ch_server.len());

        let mut rows_ch_deployment_instance: Vec<TableRowChDeploymentInstance> = Vec::with_capacity(ch_deployment_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_deployment_instance_len {
            rows_ch_deployment_instance.push(TableRowChDeploymentInstance {
                instance_id: ch_deployment_instance_instance_id[row],
                ch_server: ch_deployment_instance_ch_server[row],
                parent: ch_deployment_instance_parent[row],
            });
        }

        let ch_deployment_schemas_db_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_schemas_ch_schema: Vec<TableRowPointerChSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_schemas_parent: Vec<TableRowPointerChDeployment> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_deployment_schemas_children_ch_nats_stream_import: Vec<Vec<TableRowPointerChNatsStreamImport>> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_deployment_schemas_len = ch_deployment_schemas_children_ch_nats_stream_import.len();

        assert_eq!(ch_deployment_schemas_len, ch_deployment_schemas_db_name.len());
        assert_eq!(ch_deployment_schemas_len, ch_deployment_schemas_ch_schema.len());
        assert_eq!(ch_deployment_schemas_len, ch_deployment_schemas_parent.len());

        let mut rows_ch_deployment_schemas: Vec<TableRowChDeploymentSchemas> = Vec::with_capacity(ch_deployment_schemas_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_deployment_schemas_len {
            rows_ch_deployment_schemas.push(TableRowChDeploymentSchemas {
                db_name: ch_deployment_schemas_db_name[row].clone(),
                ch_schema: ch_deployment_schemas_ch_schema[row],
                parent: ch_deployment_schemas_parent[row],
                children_ch_nats_stream_import: ch_deployment_schemas_children_ch_nats_stream_import[row].clone(),
            });
        }

        let ch_keeper_deployment_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_docker_image: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_distribute_over_dcs: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_keeper_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_raft_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_prometheus_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_children_ch_keeper_deployment_instance: Vec<Vec<TableRowPointerChKeeperDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_referrers_ch_deployment__keeper: Vec<Vec<TableRowPointerChDeployment>> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_keeper_deployment_len = ch_keeper_deployment_referrers_ch_deployment__keeper.len();

        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_deployment_name.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_namespace.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_region.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_loki_cluster.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_monitoring_cluster.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_docker_image.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_workload_architecture.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_distribute_over_dcs.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_memory_mb.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_keeper_port.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_raft_port.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_prometheus_port.len());
        assert_eq!(ch_keeper_deployment_len, ch_keeper_deployment_children_ch_keeper_deployment_instance.len());

        let mut rows_ch_keeper_deployment: Vec<TableRowChKeeperDeployment> = Vec::with_capacity(ch_keeper_deployment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_keeper_deployment_len {
            rows_ch_keeper_deployment.push(TableRowChKeeperDeployment {
                deployment_name: ch_keeper_deployment_deployment_name[row].clone(),
                namespace: ch_keeper_deployment_namespace[row],
                region: ch_keeper_deployment_region[row],
                loki_cluster: ch_keeper_deployment_loki_cluster[row].clone(),
                monitoring_cluster: ch_keeper_deployment_monitoring_cluster[row].clone(),
                docker_image: ch_keeper_deployment_docker_image[row],
                workload_architecture: ch_keeper_deployment_workload_architecture[row].clone(),
                distribute_over_dcs: ch_keeper_deployment_distribute_over_dcs[row],
                memory_mb: ch_keeper_deployment_memory_mb[row],
                keeper_port: ch_keeper_deployment_keeper_port[row],
                raft_port: ch_keeper_deployment_raft_port[row],
                prometheus_port: ch_keeper_deployment_prometheus_port[row],
                children_ch_keeper_deployment_instance: ch_keeper_deployment_children_ch_keeper_deployment_instance[row].clone(),
                referrers_ch_deployment__keeper: ch_keeper_deployment_referrers_ch_deployment__keeper[row].clone(),
            });
        }

        let ch_keeper_deployment_instance_instance_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_instance_keeper_server: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_keeper_deployment_instance_parent: Vec<TableRowPointerChKeeperDeployment> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_keeper_deployment_instance_len = ch_keeper_deployment_instance_parent.len();

        assert_eq!(ch_keeper_deployment_instance_len, ch_keeper_deployment_instance_instance_id.len());
        assert_eq!(ch_keeper_deployment_instance_len, ch_keeper_deployment_instance_keeper_server.len());

        let mut rows_ch_keeper_deployment_instance: Vec<TableRowChKeeperDeploymentInstance> = Vec::with_capacity(ch_keeper_deployment_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_keeper_deployment_instance_len {
            rows_ch_keeper_deployment_instance.push(TableRowChKeeperDeploymentInstance {
                instance_id: ch_keeper_deployment_instance_instance_id[row],
                keeper_server: ch_keeper_deployment_instance_keeper_server[row],
                parent: ch_keeper_deployment_instance_parent[row],
            });
        }

        let ch_migration_time: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_migration_upgrade: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_migration_downgrade: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_migration_needs_admin: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_migration_parent: Vec<TableRowPointerChSchema> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_migration_len = ch_migration_parent.len();

        assert_eq!(ch_migration_len, ch_migration_time.len());
        assert_eq!(ch_migration_len, ch_migration_upgrade.len());
        assert_eq!(ch_migration_len, ch_migration_downgrade.len());
        assert_eq!(ch_migration_len, ch_migration_needs_admin.len());

        let mut rows_ch_migration: Vec<TableRowChMigration> = Vec::with_capacity(ch_migration_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_migration_len {
            rows_ch_migration.push(TableRowChMigration {
                time: ch_migration_time[row],
                upgrade: ch_migration_upgrade[row].clone(),
                downgrade: ch_migration_downgrade[row].clone(),
                needs_admin: ch_migration_needs_admin[row],
                parent: ch_migration_parent[row],
            });
        }

        let ch_mutator_mutator_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_mutator_mutator_expression: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_mutator_parent: Vec<TableRowPointerChSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_mutator_children_ch_mutator_test: Vec<Vec<TableRowPointerChMutatorTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_mutator_len = ch_mutator_children_ch_mutator_test.len();

        assert_eq!(ch_mutator_len, ch_mutator_mutator_name.len());
        assert_eq!(ch_mutator_len, ch_mutator_mutator_expression.len());
        assert_eq!(ch_mutator_len, ch_mutator_parent.len());

        let mut rows_ch_mutator: Vec<TableRowChMutator> = Vec::with_capacity(ch_mutator_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_mutator_len {
            rows_ch_mutator.push(TableRowChMutator {
                mutator_name: ch_mutator_mutator_name[row].clone(),
                mutator_expression: ch_mutator_mutator_expression[row].clone(),
                parent: ch_mutator_parent[row],
                children_ch_mutator_test: ch_mutator_children_ch_mutator_test[row].clone(),
            });
        }

        let ch_mutator_test_arguments: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_mutator_test_test_dataset: Vec<TableRowPointerChTestDataset> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_mutator_test_resulting_data: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_mutator_test_parent: Vec<TableRowPointerChMutator> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_mutator_test_len = ch_mutator_test_parent.len();

        assert_eq!(ch_mutator_test_len, ch_mutator_test_arguments.len());
        assert_eq!(ch_mutator_test_len, ch_mutator_test_test_dataset.len());
        assert_eq!(ch_mutator_test_len, ch_mutator_test_resulting_data.len());

        let mut rows_ch_mutator_test: Vec<TableRowChMutatorTest> = Vec::with_capacity(ch_mutator_test_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_mutator_test_len {
            rows_ch_mutator_test.push(TableRowChMutatorTest {
                arguments: ch_mutator_test_arguments[row].clone(),
                test_dataset: ch_mutator_test_test_dataset[row],
                resulting_data: ch_mutator_test_resulting_data[row].clone(),
                parent: ch_mutator_test_parent[row],
            });
        }

        let ch_nats_stream_import_consumer_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_nats_stream_import_into_table: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_nats_stream_import_stream: Vec<TableRowPointerNatsJetstreamStream> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_nats_stream_import_parent: Vec<TableRowPointerChDeploymentSchemas> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_nats_stream_import_len = ch_nats_stream_import_parent.len();

        assert_eq!(ch_nats_stream_import_len, ch_nats_stream_import_consumer_name.len());
        assert_eq!(ch_nats_stream_import_len, ch_nats_stream_import_into_table.len());
        assert_eq!(ch_nats_stream_import_len, ch_nats_stream_import_stream.len());

        let mut rows_ch_nats_stream_import: Vec<TableRowChNatsStreamImport> = Vec::with_capacity(ch_nats_stream_import_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_nats_stream_import_len {
            rows_ch_nats_stream_import.push(TableRowChNatsStreamImport {
                consumer_name: ch_nats_stream_import_consumer_name[row].clone(),
                into_table: ch_nats_stream_import_into_table[row].clone(),
                stream: ch_nats_stream_import_stream[row],
                parent: ch_nats_stream_import_parent[row],
            });
        }

        let ch_query_query_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_query_query_expression: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_query_opt_fields: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_query_parent: Vec<TableRowPointerChSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_query_children_ch_query_test: Vec<Vec<TableRowPointerChQueryTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_query_len = ch_query_children_ch_query_test.len();

        assert_eq!(ch_query_len, ch_query_query_name.len());
        assert_eq!(ch_query_len, ch_query_query_expression.len());
        assert_eq!(ch_query_len, ch_query_opt_fields.len());
        assert_eq!(ch_query_len, ch_query_parent.len());

        let mut rows_ch_query: Vec<TableRowChQuery> = Vec::with_capacity(ch_query_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_query_len {
            rows_ch_query.push(TableRowChQuery {
                query_name: ch_query_query_name[row].clone(),
                query_expression: ch_query_query_expression[row].clone(),
                opt_fields: ch_query_opt_fields[row].clone(),
                parent: ch_query_parent[row],
                children_ch_query_test: ch_query_children_ch_query_test[row].clone(),
            });
        }

        let ch_query_test_arguments: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_query_test_outputs: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_query_test_test_dataset: Vec<TableRowPointerChTestDataset> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_query_test_parent: Vec<TableRowPointerChQuery> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_query_test_len = ch_query_test_parent.len();

        assert_eq!(ch_query_test_len, ch_query_test_arguments.len());
        assert_eq!(ch_query_test_len, ch_query_test_outputs.len());
        assert_eq!(ch_query_test_len, ch_query_test_test_dataset.len());

        let mut rows_ch_query_test: Vec<TableRowChQueryTest> = Vec::with_capacity(ch_query_test_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_query_test_len {
            rows_ch_query_test.push(TableRowChQueryTest {
                arguments: ch_query_test_arguments[row].clone(),
                outputs: ch_query_test_outputs[row].clone(),
                test_dataset: ch_query_test_test_dataset[row],
                parent: ch_query_test_parent[row],
            });
        }

        let ch_schema_schema_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_schema_children_ch_migration: Vec<Vec<TableRowPointerChMigration>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_schema_children_ch_query: Vec<Vec<TableRowPointerChQuery>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_schema_children_ch_mutator: Vec<Vec<TableRowPointerChMutator>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_schema_children_ch_test_dataset: Vec<Vec<TableRowPointerChTestDataset>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_schema_referrers_ch_deployment_schemas__ch_schema: Vec<Vec<TableRowPointerChDeploymentSchemas>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_schema_referrers_backend_application_ch_shard__ch_schema: Vec<Vec<TableRowPointerBackendApplicationChShard>> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_schema_len = ch_schema_referrers_backend_application_ch_shard__ch_schema.len();

        assert_eq!(ch_schema_len, ch_schema_schema_name.len());
        assert_eq!(ch_schema_len, ch_schema_children_ch_migration.len());
        assert_eq!(ch_schema_len, ch_schema_children_ch_query.len());
        assert_eq!(ch_schema_len, ch_schema_children_ch_mutator.len());
        assert_eq!(ch_schema_len, ch_schema_children_ch_test_dataset.len());
        assert_eq!(ch_schema_len, ch_schema_referrers_ch_deployment_schemas__ch_schema.len());

        let mut rows_ch_schema: Vec<TableRowChSchema> = Vec::with_capacity(ch_schema_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_schema_len {
            rows_ch_schema.push(TableRowChSchema {
                schema_name: ch_schema_schema_name[row].clone(),
                children_ch_migration: ch_schema_children_ch_migration[row].clone(),
                children_ch_query: ch_schema_children_ch_query[row].clone(),
                children_ch_mutator: ch_schema_children_ch_mutator[row].clone(),
                children_ch_test_dataset: ch_schema_children_ch_test_dataset[row].clone(),
                referrers_ch_deployment_schemas__ch_schema: ch_schema_referrers_ch_deployment_schemas__ch_schema[row].clone(),
                referrers_backend_application_ch_shard__ch_schema: ch_schema_referrers_backend_application_ch_shard__ch_schema[row].clone(),
            });
        }

        let ch_test_dataset_dataset_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_test_dataset_dataset_contents: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_test_dataset_min_time: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_test_dataset_parent: Vec<TableRowPointerChSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_test_dataset_referrers_ch_query_test__test_dataset: Vec<Vec<TableRowPointerChQueryTest>> = ::bincode::deserialize_from(&mut cursor)?;
        let ch_test_dataset_referrers_ch_mutator_test__test_dataset: Vec<Vec<TableRowPointerChMutatorTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let ch_test_dataset_len = ch_test_dataset_referrers_ch_mutator_test__test_dataset.len();

        assert_eq!(ch_test_dataset_len, ch_test_dataset_dataset_name.len());
        assert_eq!(ch_test_dataset_len, ch_test_dataset_dataset_contents.len());
        assert_eq!(ch_test_dataset_len, ch_test_dataset_min_time.len());
        assert_eq!(ch_test_dataset_len, ch_test_dataset_parent.len());
        assert_eq!(ch_test_dataset_len, ch_test_dataset_referrers_ch_query_test__test_dataset.len());

        let mut rows_ch_test_dataset: Vec<TableRowChTestDataset> = Vec::with_capacity(ch_test_dataset_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..ch_test_dataset_len {
            rows_ch_test_dataset.push(TableRowChTestDataset {
                dataset_name: ch_test_dataset_dataset_name[row].clone(),
                dataset_contents: ch_test_dataset_dataset_contents[row].clone(),
                min_time: ch_test_dataset_min_time[row],
                parent: ch_test_dataset_parent[row],
                referrers_ch_query_test__test_dataset: ch_test_dataset_referrers_ch_query_test__test_dataset[row].clone(),
                referrers_ch_mutator_test__test_dataset: ch_test_dataset_referrers_ch_mutator_test__test_dataset[row].clone(),
            });
        }

        let datacenter_dc_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_network_cidr: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_allow_small_subnets: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_implementation: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_implementation_settings: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_default_server_kind: Vec<TableRowPointerServerKind> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_disk_ids_policy: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_router_subnet_vlan_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let datacenter_referrers_server__dc: Vec<Vec<TableRowPointerServer>> = ::bincode::deserialize_from(&mut cursor)?;

        let datacenter_len = datacenter_referrers_server__dc.len();

        assert_eq!(datacenter_len, datacenter_dc_name.len());
        assert_eq!(datacenter_len, datacenter_region.len());
        assert_eq!(datacenter_len, datacenter_network_cidr.len());
        assert_eq!(datacenter_len, datacenter_allow_small_subnets.len());
        assert_eq!(datacenter_len, datacenter_implementation.len());
        assert_eq!(datacenter_len, datacenter_implementation_settings.len());
        assert_eq!(datacenter_len, datacenter_default_server_kind.len());
        assert_eq!(datacenter_len, datacenter_disk_ids_policy.len());
        assert_eq!(datacenter_len, datacenter_router_subnet_vlan_id.len());

        let mut rows_datacenter: Vec<TableRowDatacenter> = Vec::with_capacity(datacenter_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..datacenter_len {
            rows_datacenter.push(TableRowDatacenter {
                dc_name: datacenter_dc_name[row].clone(),
                region: datacenter_region[row],
                network_cidr: datacenter_network_cidr[row].clone(),
                allow_small_subnets: datacenter_allow_small_subnets[row],
                implementation: datacenter_implementation[row].clone(),
                implementation_settings: datacenter_implementation_settings[row].clone(),
                default_server_kind: datacenter_default_server_kind[row],
                disk_ids_policy: datacenter_disk_ids_policy[row].clone(),
                router_subnet_vlan_id: datacenter_router_subnet_vlan_id[row],
                referrers_server__dc: datacenter_referrers_server__dc[row].clone(),
            });
        }

        let disk_kind_kind: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_medium: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_is_elastic: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_min_capacity_bytes: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_max_capacity_bytes: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_capacity_bytes: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_has_extra_config: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_non_eligible_reason: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let disk_kind_referrers_server_disk__disk_kind: Vec<Vec<TableRowPointerServerDisk>> = ::bincode::deserialize_from(&mut cursor)?;

        let disk_kind_len = disk_kind_referrers_server_disk__disk_kind.len();

        assert_eq!(disk_kind_len, disk_kind_kind.len());
        assert_eq!(disk_kind_len, disk_kind_medium.len());
        assert_eq!(disk_kind_len, disk_kind_is_elastic.len());
        assert_eq!(disk_kind_len, disk_kind_min_capacity_bytes.len());
        assert_eq!(disk_kind_len, disk_kind_max_capacity_bytes.len());
        assert_eq!(disk_kind_len, disk_kind_capacity_bytes.len());
        assert_eq!(disk_kind_len, disk_kind_has_extra_config.len());
        assert_eq!(disk_kind_len, disk_kind_non_eligible_reason.len());

        let mut rows_disk_kind: Vec<TableRowDiskKind> = Vec::with_capacity(disk_kind_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..disk_kind_len {
            rows_disk_kind.push(TableRowDiskKind {
                kind: disk_kind_kind[row].clone(),
                medium: disk_kind_medium[row].clone(),
                is_elastic: disk_kind_is_elastic[row],
                min_capacity_bytes: disk_kind_min_capacity_bytes[row],
                max_capacity_bytes: disk_kind_max_capacity_bytes[row],
                capacity_bytes: disk_kind_capacity_bytes[row],
                has_extra_config: disk_kind_has_extra_config[row],
                non_eligible_reason: disk_kind_non_eligible_reason[row].clone(),
                referrers_server_disk__disk_kind: disk_kind_referrers_server_disk__disk_kind[row].clone(),
            });
        }

        let docker_image_checksum: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_image_set: Vec<TableRowPointerDockerImageSet> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_repository: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_tag: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_referrers_docker_image_pin_images__checksum: Vec<Vec<TableRowPointerDockerImagePinImages>> = ::bincode::deserialize_from(&mut cursor)?;

        let docker_image_len = docker_image_referrers_docker_image_pin_images__checksum.len();

        assert_eq!(docker_image_len, docker_image_checksum.len());
        assert_eq!(docker_image_len, docker_image_image_set.len());
        assert_eq!(docker_image_len, docker_image_repository.len());
        assert_eq!(docker_image_len, docker_image_architecture.len());
        assert_eq!(docker_image_len, docker_image_tag.len());

        let mut rows_docker_image: Vec<TableRowDockerImage> = Vec::with_capacity(docker_image_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..docker_image_len {
            rows_docker_image.push(TableRowDockerImage {
                checksum: docker_image_checksum[row].clone(),
                image_set: docker_image_image_set[row],
                repository: docker_image_repository[row].clone(),
                architecture: docker_image_architecture[row].clone(),
                tag: docker_image_tag[row].clone(),
                referrers_docker_image_pin_images__checksum: docker_image_referrers_docker_image_pin_images__checksum[row].clone(),
            });
        }

        let docker_image_pin_pin_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_children_docker_image_pin_images: Vec<Vec<TableRowPointerDockerImagePinImages>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_region__docker_image_external_lb: Vec<Vec<TableRowPointerRegion>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_docker_registry_instance__docker_image: Vec<Vec<TableRowPointerDockerRegistryInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_pg_deployment__docker_image_pg: Vec<Vec<TableRowPointerPgDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_pg_deployment__docker_image_haproxy: Vec<Vec<TableRowPointerPgDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_pg_deployment__docker_image_pg_exporter: Vec<Vec<TableRowPointerPgDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_ch_deployment__docker_image: Vec<Vec<TableRowPointerChDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_ch_keeper_deployment__docker_image: Vec<Vec<TableRowPointerChKeeperDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_nats_cluster__docker_image_nats: Vec<Vec<TableRowPointerNatsCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_nats_cluster__docker_image_nats_exporter: Vec<Vec<TableRowPointerNatsCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_minio_cluster__docker_image_minio: Vec<Vec<TableRowPointerMinioCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_minio_cluster__docker_image_minio_mc: Vec<Vec<TableRowPointerMinioCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_minio_cluster__docker_image_nginx: Vec<Vec<TableRowPointerMinioCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_monitoring_cluster__docker_image_prometheus: Vec<Vec<TableRowPointerMonitoringCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_monitoring_cluster__docker_image_alertmanager: Vec<Vec<TableRowPointerMonitoringCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_monitoring_cluster__docker_image_victoriametrics: Vec<Vec<TableRowPointerMonitoringCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_grafana__docker_image_grafana: Vec<Vec<TableRowPointerGrafana>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_grafana__docker_image_promxy: Vec<Vec<TableRowPointerGrafana>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_loki_cluster__docker_image_loki: Vec<Vec<TableRowPointerLokiCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_tempo_cluster__docker_image: Vec<Vec<TableRowPointerTempoCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_referrers_blackbox_deployment_task__docker_image: Vec<Vec<TableRowPointerBlackboxDeploymentTask>> = ::bincode::deserialize_from(&mut cursor)?;

        let docker_image_pin_len = docker_image_pin_referrers_blackbox_deployment_task__docker_image.len();

        assert_eq!(docker_image_pin_len, docker_image_pin_pin_name.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_children_docker_image_pin_images.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_region__docker_image_external_lb.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_docker_registry_instance__docker_image.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_pg_deployment__docker_image_pg.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_pg_deployment__docker_image_haproxy.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_pg_deployment__docker_image_pg_exporter.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_ch_deployment__docker_image.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_ch_keeper_deployment__docker_image.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_nats_cluster__docker_image_nats.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_nats_cluster__docker_image_nats_exporter.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_minio_cluster__docker_image_minio.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_minio_cluster__docker_image_minio_mc.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_minio_cluster__docker_image_nginx.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_monitoring_cluster__docker_image_prometheus.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_monitoring_cluster__docker_image_alertmanager.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_monitoring_cluster__docker_image_victoriametrics.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_grafana__docker_image_grafana.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_grafana__docker_image_promxy.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_loki_cluster__docker_image_loki.len());
        assert_eq!(docker_image_pin_len, docker_image_pin_referrers_tempo_cluster__docker_image.len());

        let mut rows_docker_image_pin: Vec<TableRowDockerImagePin> = Vec::with_capacity(docker_image_pin_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..docker_image_pin_len {
            rows_docker_image_pin.push(TableRowDockerImagePin {
                pin_name: docker_image_pin_pin_name[row].clone(),
                children_docker_image_pin_images: docker_image_pin_children_docker_image_pin_images[row].clone(),
                referrers_region__docker_image_external_lb: docker_image_pin_referrers_region__docker_image_external_lb[row].clone(),
                referrers_docker_registry_instance__docker_image: docker_image_pin_referrers_docker_registry_instance__docker_image[row].clone(),
                referrers_pg_deployment__docker_image_pg: docker_image_pin_referrers_pg_deployment__docker_image_pg[row].clone(),
                referrers_pg_deployment__docker_image_haproxy: docker_image_pin_referrers_pg_deployment__docker_image_haproxy[row].clone(),
                referrers_pg_deployment__docker_image_pg_exporter: docker_image_pin_referrers_pg_deployment__docker_image_pg_exporter[row].clone(),
                referrers_ch_deployment__docker_image: docker_image_pin_referrers_ch_deployment__docker_image[row].clone(),
                referrers_ch_keeper_deployment__docker_image: docker_image_pin_referrers_ch_keeper_deployment__docker_image[row].clone(),
                referrers_nats_cluster__docker_image_nats: docker_image_pin_referrers_nats_cluster__docker_image_nats[row].clone(),
                referrers_nats_cluster__docker_image_nats_exporter: docker_image_pin_referrers_nats_cluster__docker_image_nats_exporter[row].clone(),
                referrers_minio_cluster__docker_image_minio: docker_image_pin_referrers_minio_cluster__docker_image_minio[row].clone(),
                referrers_minio_cluster__docker_image_minio_mc: docker_image_pin_referrers_minio_cluster__docker_image_minio_mc[row].clone(),
                referrers_minio_cluster__docker_image_nginx: docker_image_pin_referrers_minio_cluster__docker_image_nginx[row].clone(),
                referrers_monitoring_cluster__docker_image_prometheus: docker_image_pin_referrers_monitoring_cluster__docker_image_prometheus[row].clone(),
                referrers_monitoring_cluster__docker_image_alertmanager: docker_image_pin_referrers_monitoring_cluster__docker_image_alertmanager[row].clone(),
                referrers_monitoring_cluster__docker_image_victoriametrics: docker_image_pin_referrers_monitoring_cluster__docker_image_victoriametrics[row].clone(),
                referrers_grafana__docker_image_grafana: docker_image_pin_referrers_grafana__docker_image_grafana[row].clone(),
                referrers_grafana__docker_image_promxy: docker_image_pin_referrers_grafana__docker_image_promxy[row].clone(),
                referrers_loki_cluster__docker_image_loki: docker_image_pin_referrers_loki_cluster__docker_image_loki[row].clone(),
                referrers_tempo_cluster__docker_image: docker_image_pin_referrers_tempo_cluster__docker_image[row].clone(),
                referrers_blackbox_deployment_task__docker_image: docker_image_pin_referrers_blackbox_deployment_task__docker_image[row].clone(),
            });
        }

        let docker_image_pin_images_checksum: Vec<TableRowPointerDockerImage> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_pin_images_parent: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;

        let docker_image_pin_images_len = docker_image_pin_images_parent.len();

        assert_eq!(docker_image_pin_images_len, docker_image_pin_images_checksum.len());

        let mut rows_docker_image_pin_images: Vec<TableRowDockerImagePinImages> = Vec::with_capacity(docker_image_pin_images_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..docker_image_pin_images_len {
            rows_docker_image_pin_images.push(TableRowDockerImagePinImages {
                checksum: docker_image_pin_images_checksum[row],
                parent: docker_image_pin_images_parent[row],
            });
        }

        let docker_image_set_set_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_set_referrers_docker_image__image_set: Vec<Vec<TableRowPointerDockerImage>> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_image_set_referrers_blackbox_deployment_task__docker_image_set: Vec<Vec<TableRowPointerBlackboxDeploymentTask>> = ::bincode::deserialize_from(&mut cursor)?;

        let docker_image_set_len = docker_image_set_referrers_blackbox_deployment_task__docker_image_set.len();

        assert_eq!(docker_image_set_len, docker_image_set_set_name.len());
        assert_eq!(docker_image_set_len, docker_image_set_referrers_docker_image__image_set.len());

        let mut rows_docker_image_set: Vec<TableRowDockerImageSet> = Vec::with_capacity(docker_image_set_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..docker_image_set_len {
            rows_docker_image_set.push(TableRowDockerImageSet {
                set_name: docker_image_set_set_name[row].clone(),
                referrers_docker_image__image_set: docker_image_set_referrers_docker_image__image_set[row].clone(),
                referrers_blackbox_deployment_task__docker_image_set: docker_image_set_referrers_blackbox_deployment_task__docker_image_set[row].clone(),
            });
        }

        let docker_registry_instance_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_registry_instance_minio_bucket: Vec<TableRowPointerMinioBucket> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_registry_instance_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let docker_registry_instance_docker_image: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;

        let docker_registry_instance_len = docker_registry_instance_docker_image.len();

        assert_eq!(docker_registry_instance_len, docker_registry_instance_region.len());
        assert_eq!(docker_registry_instance_len, docker_registry_instance_minio_bucket.len());
        assert_eq!(docker_registry_instance_len, docker_registry_instance_memory_mb.len());

        let mut rows_docker_registry_instance: Vec<TableRowDockerRegistryInstance> = Vec::with_capacity(docker_registry_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..docker_registry_instance_len {
            rows_docker_registry_instance.push(TableRowDockerRegistryInstance {
                region: docker_registry_instance_region[row],
                minio_bucket: docker_registry_instance_minio_bucket[row],
                memory_mb: docker_registry_instance_memory_mb[row],
                docker_image: docker_registry_instance_docker_image[row],
            });
        }

        let frontend_application_application_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_build_environment: Vec<TableRowPointerRustCompilationEnvironment> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_index_page_title: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_children_frontend_page: Vec<Vec<TableRowPointerFrontendPage>> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_children_frontend_application_used_endpoint: Vec<Vec<TableRowPointerFrontendApplicationUsedEndpoint>> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_children_frontend_application_external_link: Vec<Vec<TableRowPointerFrontendApplicationExternalLink>> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_children_frontend_application_external_page: Vec<Vec<TableRowPointerFrontendApplicationExternalPage>> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_referrers_frontend_application_deployment__application_name: Vec<Vec<TableRowPointerFrontendApplicationDeployment>> = ::bincode::deserialize_from(&mut cursor)?;

        let frontend_application_len = frontend_application_referrers_frontend_application_deployment__application_name.len();

        assert_eq!(frontend_application_len, frontend_application_application_name.len());
        assert_eq!(frontend_application_len, frontend_application_build_environment.len());
        assert_eq!(frontend_application_len, frontend_application_index_page_title.len());
        assert_eq!(frontend_application_len, frontend_application_children_frontend_page.len());
        assert_eq!(frontend_application_len, frontend_application_children_frontend_application_used_endpoint.len());
        assert_eq!(frontend_application_len, frontend_application_children_frontend_application_external_link.len());
        assert_eq!(frontend_application_len, frontend_application_children_frontend_application_external_page.len());

        let mut rows_frontend_application: Vec<TableRowFrontendApplication> = Vec::with_capacity(frontend_application_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..frontend_application_len {
            rows_frontend_application.push(TableRowFrontendApplication {
                application_name: frontend_application_application_name[row].clone(),
                build_environment: frontend_application_build_environment[row],
                index_page_title: frontend_application_index_page_title[row].clone(),
                children_frontend_page: frontend_application_children_frontend_page[row].clone(),
                children_frontend_application_used_endpoint: frontend_application_children_frontend_application_used_endpoint[row].clone(),
                children_frontend_application_external_link: frontend_application_children_frontend_application_external_link[row].clone(),
                children_frontend_application_external_page: frontend_application_children_frontend_application_external_page[row].clone(),
                referrers_frontend_application_deployment__application_name: frontend_application_referrers_frontend_application_deployment__application_name[row].clone(),
            });
        }

        let frontend_application_deployment_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_application_name: Vec<TableRowPointerFrontendApplication> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_explicit_endpoint_wiring: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_workload_backend_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_link_wiring: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_page_wiring: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_count: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_referrers_frontend_application_deployment_ingress__deployment: Vec<Vec<TableRowPointerFrontendApplicationDeploymentIngress>> = ::bincode::deserialize_from(&mut cursor)?;

        let frontend_application_deployment_len = frontend_application_deployment_referrers_frontend_application_deployment_ingress__deployment.len();

        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_deployment_name.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_application_name.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_namespace.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_explicit_endpoint_wiring.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_workload_backend_architecture.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_placement.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_link_wiring.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_page_wiring.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_count.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_http_port.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_memory_mb.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_region.len());
        assert_eq!(frontend_application_deployment_len, frontend_application_deployment_loki_cluster.len());

        let mut rows_frontend_application_deployment: Vec<TableRowFrontendApplicationDeployment> = Vec::with_capacity(frontend_application_deployment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..frontend_application_deployment_len {
            rows_frontend_application_deployment.push(TableRowFrontendApplicationDeployment {
                deployment_name: frontend_application_deployment_deployment_name[row].clone(),
                application_name: frontend_application_deployment_application_name[row],
                namespace: frontend_application_deployment_namespace[row],
                explicit_endpoint_wiring: frontend_application_deployment_explicit_endpoint_wiring[row].clone(),
                workload_backend_architecture: frontend_application_deployment_workload_backend_architecture[row].clone(),
                placement: frontend_application_deployment_placement[row].clone(),
                link_wiring: frontend_application_deployment_link_wiring[row].clone(),
                page_wiring: frontend_application_deployment_page_wiring[row].clone(),
                count: frontend_application_deployment_count[row],
                http_port: frontend_application_deployment_http_port[row],
                memory_mb: frontend_application_deployment_memory_mb[row],
                region: frontend_application_deployment_region[row],
                loki_cluster: frontend_application_deployment_loki_cluster[row].clone(),
                referrers_frontend_application_deployment_ingress__deployment: frontend_application_deployment_referrers_frontend_application_deployment_ingress__deployment[row].clone(),
            });
        }

        let frontend_application_deployment_ingress_deployment: Vec<TableRowPointerFrontendApplicationDeployment> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_ingress_mountpoint: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_ingress_subdomain: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_deployment_ingress_tld: Vec<TableRowPointerTld> = ::bincode::deserialize_from(&mut cursor)?;

        let frontend_application_deployment_ingress_len = frontend_application_deployment_ingress_tld.len();

        assert_eq!(frontend_application_deployment_ingress_len, frontend_application_deployment_ingress_deployment.len());
        assert_eq!(frontend_application_deployment_ingress_len, frontend_application_deployment_ingress_mountpoint.len());
        assert_eq!(frontend_application_deployment_ingress_len, frontend_application_deployment_ingress_subdomain.len());

        let mut rows_frontend_application_deployment_ingress: Vec<TableRowFrontendApplicationDeploymentIngress> = Vec::with_capacity(frontend_application_deployment_ingress_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..frontend_application_deployment_ingress_len {
            rows_frontend_application_deployment_ingress.push(TableRowFrontendApplicationDeploymentIngress {
                deployment: frontend_application_deployment_ingress_deployment[row],
                mountpoint: frontend_application_deployment_ingress_mountpoint[row].clone(),
                subdomain: frontend_application_deployment_ingress_subdomain[row].clone(),
                tld: frontend_application_deployment_ingress_tld[row],
            });
        }

        let frontend_application_external_link_link_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_external_link_backend_endpoint: Vec<TableRowPointerBackendHttpEndpoint> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_external_link_parent: Vec<TableRowPointerFrontendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let frontend_application_external_link_len = frontend_application_external_link_parent.len();

        assert_eq!(frontend_application_external_link_len, frontend_application_external_link_link_name.len());
        assert_eq!(frontend_application_external_link_len, frontend_application_external_link_backend_endpoint.len());

        let mut rows_frontend_application_external_link: Vec<TableRowFrontendApplicationExternalLink> = Vec::with_capacity(frontend_application_external_link_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..frontend_application_external_link_len {
            rows_frontend_application_external_link.push(TableRowFrontendApplicationExternalLink {
                link_name: frontend_application_external_link_link_name[row].clone(),
                backend_endpoint: frontend_application_external_link_backend_endpoint[row],
                parent: frontend_application_external_link_parent[row],
            });
        }

        let frontend_application_external_page_link_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_external_page_frontend_page: Vec<TableRowPointerFrontendPage> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_external_page_parent: Vec<TableRowPointerFrontendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let frontend_application_external_page_len = frontend_application_external_page_parent.len();

        assert_eq!(frontend_application_external_page_len, frontend_application_external_page_link_name.len());
        assert_eq!(frontend_application_external_page_len, frontend_application_external_page_frontend_page.len());

        let mut rows_frontend_application_external_page: Vec<TableRowFrontendApplicationExternalPage> = Vec::with_capacity(frontend_application_external_page_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..frontend_application_external_page_len {
            rows_frontend_application_external_page.push(TableRowFrontendApplicationExternalPage {
                link_name: frontend_application_external_page_link_name[row].clone(),
                frontend_page: frontend_application_external_page_frontend_page[row],
                parent: frontend_application_external_page_parent[row],
            });
        }

        let frontend_application_used_endpoint_endpoint_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_used_endpoint_backend_endpoint: Vec<TableRowPointerBackendHttpEndpoint> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_application_used_endpoint_parent: Vec<TableRowPointerFrontendApplication> = ::bincode::deserialize_from(&mut cursor)?;

        let frontend_application_used_endpoint_len = frontend_application_used_endpoint_parent.len();

        assert_eq!(frontend_application_used_endpoint_len, frontend_application_used_endpoint_endpoint_name.len());
        assert_eq!(frontend_application_used_endpoint_len, frontend_application_used_endpoint_backend_endpoint.len());

        let mut rows_frontend_application_used_endpoint: Vec<TableRowFrontendApplicationUsedEndpoint> = Vec::with_capacity(frontend_application_used_endpoint_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..frontend_application_used_endpoint_len {
            rows_frontend_application_used_endpoint.push(TableRowFrontendApplicationUsedEndpoint {
                endpoint_name: frontend_application_used_endpoint_endpoint_name[row].clone(),
                backend_endpoint: frontend_application_used_endpoint_backend_endpoint[row],
                parent: frontend_application_used_endpoint_parent[row],
            });
        }

        let frontend_page_page_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_page_path: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_page_parent: Vec<TableRowPointerFrontendApplication> = ::bincode::deserialize_from(&mut cursor)?;
        let frontend_page_referrers_frontend_application_external_page__frontend_page: Vec<Vec<TableRowPointerFrontendApplicationExternalPage>> = ::bincode::deserialize_from(&mut cursor)?;

        let frontend_page_len = frontend_page_referrers_frontend_application_external_page__frontend_page.len();

        assert_eq!(frontend_page_len, frontend_page_page_name.len());
        assert_eq!(frontend_page_len, frontend_page_path.len());
        assert_eq!(frontend_page_len, frontend_page_parent.len());

        let mut rows_frontend_page: Vec<TableRowFrontendPage> = Vec::with_capacity(frontend_page_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..frontend_page_len {
            rows_frontend_page.push(TableRowFrontendPage {
                page_name: frontend_page_page_name[row].clone(),
                path: frontend_page_path[row].clone(),
                parent: frontend_page_parent[row],
                referrers_frontend_application_external_page__frontend_page: frontend_page_referrers_frontend_application_external_page__frontend_page[row].clone(),
            });
        }

        let global_settings_project_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_docker_registry_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_docker_registry_service_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_aws_artefacts_s3_bucket_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_local_docker_cache_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_admin_email: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_google_cloud_project_id: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_google_cloud_artefacts_bucket_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_consul_quorum_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_nomad_quorum_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_vault_quorum_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_dns_quorum_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_deployment_min_server_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_deployment_min_ingress_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_region_docker_registry_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_region_monitoring_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_region_tracing_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_region_logging_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_disable_vpn_gateway_tests: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_hetzner_inter_dc_vlan_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_experimental_enable_arm64_support: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_update_edl_public_ips_from_terraform: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_enable_ipv6: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let global_settings_force_ipv6: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;

        let global_settings_len = global_settings_force_ipv6.len();

        assert_eq!(global_settings_len, global_settings_project_name.len());
        assert_eq!(global_settings_len, global_settings_docker_registry_port.len());
        assert_eq!(global_settings_len, global_settings_docker_registry_service_name.len());
        assert_eq!(global_settings_len, global_settings_aws_artefacts_s3_bucket_name.len());
        assert_eq!(global_settings_len, global_settings_local_docker_cache_port.len());
        assert_eq!(global_settings_len, global_settings_admin_email.len());
        assert_eq!(global_settings_len, global_settings_google_cloud_project_id.len());
        assert_eq!(global_settings_len, global_settings_google_cloud_artefacts_bucket_name.len());
        assert_eq!(global_settings_len, global_settings_disable_consul_quorum_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_nomad_quorum_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_vault_quorum_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_dns_quorum_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_deployment_min_server_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_deployment_min_ingress_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_region_docker_registry_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_region_monitoring_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_region_tracing_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_region_logging_tests.len());
        assert_eq!(global_settings_len, global_settings_disable_vpn_gateway_tests.len());
        assert_eq!(global_settings_len, global_settings_hetzner_inter_dc_vlan_id.len());
        assert_eq!(global_settings_len, global_settings_experimental_enable_arm64_support.len());
        assert_eq!(global_settings_len, global_settings_update_edl_public_ips_from_terraform.len());
        assert_eq!(global_settings_len, global_settings_enable_ipv6.len());

        let mut rows_global_settings: Vec<TableRowGlobalSettings> = Vec::with_capacity(global_settings_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..global_settings_len {
            rows_global_settings.push(TableRowGlobalSettings {
                project_name: global_settings_project_name[row].clone(),
                docker_registry_port: global_settings_docker_registry_port[row],
                docker_registry_service_name: global_settings_docker_registry_service_name[row].clone(),
                aws_artefacts_s3_bucket_name: global_settings_aws_artefacts_s3_bucket_name[row].clone(),
                local_docker_cache_port: global_settings_local_docker_cache_port[row],
                admin_email: global_settings_admin_email[row].clone(),
                google_cloud_project_id: global_settings_google_cloud_project_id[row].clone(),
                google_cloud_artefacts_bucket_name: global_settings_google_cloud_artefacts_bucket_name[row].clone(),
                disable_consul_quorum_tests: global_settings_disable_consul_quorum_tests[row],
                disable_nomad_quorum_tests: global_settings_disable_nomad_quorum_tests[row],
                disable_vault_quorum_tests: global_settings_disable_vault_quorum_tests[row],
                disable_dns_quorum_tests: global_settings_disable_dns_quorum_tests[row],
                disable_deployment_min_server_tests: global_settings_disable_deployment_min_server_tests[row],
                disable_deployment_min_ingress_tests: global_settings_disable_deployment_min_ingress_tests[row],
                disable_region_docker_registry_tests: global_settings_disable_region_docker_registry_tests[row],
                disable_region_monitoring_tests: global_settings_disable_region_monitoring_tests[row],
                disable_region_tracing_tests: global_settings_disable_region_tracing_tests[row],
                disable_region_logging_tests: global_settings_disable_region_logging_tests[row],
                disable_vpn_gateway_tests: global_settings_disable_vpn_gateway_tests[row],
                hetzner_inter_dc_vlan_id: global_settings_hetzner_inter_dc_vlan_id[row],
                experimental_enable_arm64_support: global_settings_experimental_enable_arm64_support[row],
                update_edl_public_ips_from_terraform: global_settings_update_edl_public_ips_from_terraform[row],
                enable_ipv6: global_settings_enable_ipv6[row],
                force_ipv6: global_settings_force_ipv6[row],
            });
        }

        let grafana_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_docker_image_grafana: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_docker_image_promxy: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_promxy_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_instance_count: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_database: Vec<TableRowPointerPgDeploymentUnmanagedDb> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_promxy_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;

        let grafana_len = grafana_promxy_memory_mb.len();

        assert_eq!(grafana_len, grafana_deployment_name.len());
        assert_eq!(grafana_len, grafana_namespace.len());
        assert_eq!(grafana_len, grafana_region.len());
        assert_eq!(grafana_len, grafana_placement.len());
        assert_eq!(grafana_len, grafana_workload_architecture.len());
        assert_eq!(grafana_len, grafana_docker_image_grafana.len());
        assert_eq!(grafana_len, grafana_docker_image_promxy.len());
        assert_eq!(grafana_len, grafana_loki_cluster.len());
        assert_eq!(grafana_len, grafana_monitoring_cluster.len());
        assert_eq!(grafana_len, grafana_port.len());
        assert_eq!(grafana_len, grafana_promxy_port.len());
        assert_eq!(grafana_len, grafana_instance_count.len());
        assert_eq!(grafana_len, grafana_database.len());
        assert_eq!(grafana_len, grafana_memory_mb.len());

        let mut rows_grafana: Vec<TableRowGrafana> = Vec::with_capacity(grafana_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..grafana_len {
            rows_grafana.push(TableRowGrafana {
                deployment_name: grafana_deployment_name[row].clone(),
                namespace: grafana_namespace[row],
                region: grafana_region[row],
                placement: grafana_placement[row].clone(),
                workload_architecture: grafana_workload_architecture[row].clone(),
                docker_image_grafana: grafana_docker_image_grafana[row],
                docker_image_promxy: grafana_docker_image_promxy[row],
                loki_cluster: grafana_loki_cluster[row].clone(),
                monitoring_cluster: grafana_monitoring_cluster[row].clone(),
                port: grafana_port[row],
                promxy_port: grafana_promxy_port[row],
                instance_count: grafana_instance_count[row],
                database: grafana_database[row],
                memory_mb: grafana_memory_mb[row],
                promxy_memory_mb: grafana_promxy_memory_mb[row],
            });
        }

        let grafana_dashboard_filename: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let grafana_dashboard_contents: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;

        let grafana_dashboard_len = grafana_dashboard_contents.len();

        assert_eq!(grafana_dashboard_len, grafana_dashboard_filename.len());

        let mut rows_grafana_dashboard: Vec<TableRowGrafanaDashboard> = Vec::with_capacity(grafana_dashboard_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..grafana_dashboard_len {
            rows_grafana_dashboard.push(TableRowGrafanaDashboard {
                filename: grafana_dashboard_filename[row].clone(),
                contents: grafana_dashboard_contents[row].clone(),
            });
        }

        let http_endpoint_data_type_http_endpoint_data_type: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let http_endpoint_data_type_referrers_backend_http_endpoint__data_type: Vec<Vec<TableRowPointerBackendHttpEndpoint>> = ::bincode::deserialize_from(&mut cursor)?;

        let http_endpoint_data_type_len = http_endpoint_data_type_referrers_backend_http_endpoint__data_type.len();

        assert_eq!(http_endpoint_data_type_len, http_endpoint_data_type_http_endpoint_data_type.len());

        let mut rows_http_endpoint_data_type: Vec<TableRowHttpEndpointDataType> = Vec::with_capacity(http_endpoint_data_type_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..http_endpoint_data_type_len {
            rows_http_endpoint_data_type.push(TableRowHttpEndpointDataType {
                http_endpoint_data_type: http_endpoint_data_type_http_endpoint_data_type[row].clone(),
                referrers_backend_http_endpoint__data_type: http_endpoint_data_type_referrers_backend_http_endpoint__data_type[row].clone(),
            });
        }

        let http_methods_http_method_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let http_methods_referrers_backend_http_endpoint__http_method: Vec<Vec<TableRowPointerBackendHttpEndpoint>> = ::bincode::deserialize_from(&mut cursor)?;

        let http_methods_len = http_methods_referrers_backend_http_endpoint__http_method.len();

        assert_eq!(http_methods_len, http_methods_http_method_name.len());

        let mut rows_http_methods: Vec<TableRowHttpMethods> = Vec::with_capacity(http_methods_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..http_methods_len {
            rows_http_methods.push(TableRowHttpMethods {
                http_method_name: http_methods_http_method_name[row].clone(),
                referrers_backend_http_endpoint__http_method: http_methods_referrers_backend_http_endpoint__http_method[row].clone(),
            });
        }

        let loki_cluster_cluster_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_docker_image_loki: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_is_region_default: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_storage_bucket: Vec<TableRowPointerMinioBucket> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_retention_period_days: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_writer_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_writer_grpc_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_reader_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_reader_grpc_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_backend_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_backend_grpc_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_writers: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_readers: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_writer_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_reader_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_backend_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_reader_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_writer_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let loki_cluster_loki_backend_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;

        let loki_cluster_len = loki_cluster_loki_backend_memory_mb.len();

        assert_eq!(loki_cluster_len, loki_cluster_cluster_name.len());
        assert_eq!(loki_cluster_len, loki_cluster_namespace.len());
        assert_eq!(loki_cluster_len, loki_cluster_region.len());
        assert_eq!(loki_cluster_len, loki_cluster_workload_architecture.len());
        assert_eq!(loki_cluster_len, loki_cluster_docker_image_loki.len());
        assert_eq!(loki_cluster_len, loki_cluster_is_region_default.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_cluster.len());
        assert_eq!(loki_cluster_len, loki_cluster_monitoring_cluster.len());
        assert_eq!(loki_cluster_len, loki_cluster_storage_bucket.len());
        assert_eq!(loki_cluster_len, loki_cluster_retention_period_days.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_writer_http_port.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_writer_grpc_port.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_reader_http_port.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_reader_grpc_port.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_backend_http_port.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_backend_grpc_port.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_writers.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_readers.len());
        assert_eq!(loki_cluster_len, loki_cluster_writer_placement.len());
        assert_eq!(loki_cluster_len, loki_cluster_reader_placement.len());
        assert_eq!(loki_cluster_len, loki_cluster_backend_placement.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_reader_memory_mb.len());
        assert_eq!(loki_cluster_len, loki_cluster_loki_writer_memory_mb.len());

        let mut rows_loki_cluster: Vec<TableRowLokiCluster> = Vec::with_capacity(loki_cluster_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..loki_cluster_len {
            rows_loki_cluster.push(TableRowLokiCluster {
                cluster_name: loki_cluster_cluster_name[row].clone(),
                namespace: loki_cluster_namespace[row],
                region: loki_cluster_region[row],
                workload_architecture: loki_cluster_workload_architecture[row].clone(),
                docker_image_loki: loki_cluster_docker_image_loki[row],
                is_region_default: loki_cluster_is_region_default[row],
                loki_cluster: loki_cluster_loki_cluster[row].clone(),
                monitoring_cluster: loki_cluster_monitoring_cluster[row].clone(),
                storage_bucket: loki_cluster_storage_bucket[row],
                retention_period_days: loki_cluster_retention_period_days[row],
                loki_writer_http_port: loki_cluster_loki_writer_http_port[row],
                loki_writer_grpc_port: loki_cluster_loki_writer_grpc_port[row],
                loki_reader_http_port: loki_cluster_loki_reader_http_port[row],
                loki_reader_grpc_port: loki_cluster_loki_reader_grpc_port[row],
                loki_backend_http_port: loki_cluster_loki_backend_http_port[row],
                loki_backend_grpc_port: loki_cluster_loki_backend_grpc_port[row],
                loki_writers: loki_cluster_loki_writers[row],
                loki_readers: loki_cluster_loki_readers[row],
                writer_placement: loki_cluster_writer_placement[row].clone(),
                reader_placement: loki_cluster_reader_placement[row].clone(),
                backend_placement: loki_cluster_backend_placement[row].clone(),
                loki_reader_memory_mb: loki_cluster_loki_reader_memory_mb[row],
                loki_writer_memory_mb: loki_cluster_loki_writer_memory_mb[row],
                loki_backend_memory_mb: loki_cluster_loki_backend_memory_mb[row],
            });
        }

        let minio_bucket_bucket_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_bucket_locking_enabled: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_bucket_parent: Vec<TableRowPointerMinioCluster> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_bucket_referrers_docker_registry_instance__minio_bucket: Vec<Vec<TableRowPointerDockerRegistryInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_bucket_referrers_loki_cluster__storage_bucket: Vec<Vec<TableRowPointerLokiCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_bucket_referrers_tempo_cluster__storage_bucket: Vec<Vec<TableRowPointerTempoCluster>> = ::bincode::deserialize_from(&mut cursor)?;

        let minio_bucket_len = minio_bucket_referrers_tempo_cluster__storage_bucket.len();

        assert_eq!(minio_bucket_len, minio_bucket_bucket_name.len());
        assert_eq!(minio_bucket_len, minio_bucket_locking_enabled.len());
        assert_eq!(minio_bucket_len, minio_bucket_parent.len());
        assert_eq!(minio_bucket_len, minio_bucket_referrers_docker_registry_instance__minio_bucket.len());
        assert_eq!(minio_bucket_len, minio_bucket_referrers_loki_cluster__storage_bucket.len());

        let mut rows_minio_bucket: Vec<TableRowMinioBucket> = Vec::with_capacity(minio_bucket_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..minio_bucket_len {
            rows_minio_bucket.push(TableRowMinioBucket {
                bucket_name: minio_bucket_bucket_name[row].clone(),
                locking_enabled: minio_bucket_locking_enabled[row],
                parent: minio_bucket_parent[row],
                referrers_docker_registry_instance__minio_bucket: minio_bucket_referrers_docker_registry_instance__minio_bucket[row].clone(),
                referrers_loki_cluster__storage_bucket: minio_bucket_referrers_loki_cluster__storage_bucket[row].clone(),
                referrers_tempo_cluster__storage_bucket: minio_bucket_referrers_tempo_cluster__storage_bucket[row].clone(),
            });
        }

        let minio_cluster_cluster_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_docker_image_minio: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_docker_image_minio_mc: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_docker_image_nginx: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_api_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_console_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_lb_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_expected_zfs_recordsize: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_distribute_over_dcs: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_instance_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_lb_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_consul_service_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_children_minio_instance: Vec<Vec<TableRowPointerMinioInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_cluster_children_minio_bucket: Vec<Vec<TableRowPointerMinioBucket>> = ::bincode::deserialize_from(&mut cursor)?;

        let minio_cluster_len = minio_cluster_children_minio_bucket.len();

        assert_eq!(minio_cluster_len, minio_cluster_cluster_name.len());
        assert_eq!(minio_cluster_len, minio_cluster_namespace.len());
        assert_eq!(minio_cluster_len, minio_cluster_region.len());
        assert_eq!(minio_cluster_len, minio_cluster_workload_architecture.len());
        assert_eq!(minio_cluster_len, minio_cluster_docker_image_minio.len());
        assert_eq!(minio_cluster_len, minio_cluster_docker_image_minio_mc.len());
        assert_eq!(minio_cluster_len, minio_cluster_docker_image_nginx.len());
        assert_eq!(minio_cluster_len, minio_cluster_api_port.len());
        assert_eq!(minio_cluster_len, minio_cluster_console_port.len());
        assert_eq!(minio_cluster_len, minio_cluster_lb_port.len());
        assert_eq!(minio_cluster_len, minio_cluster_loki_cluster.len());
        assert_eq!(minio_cluster_len, minio_cluster_monitoring_cluster.len());
        assert_eq!(minio_cluster_len, minio_cluster_expected_zfs_recordsize.len());
        assert_eq!(minio_cluster_len, minio_cluster_distribute_over_dcs.len());
        assert_eq!(minio_cluster_len, minio_cluster_instance_memory_mb.len());
        assert_eq!(minio_cluster_len, minio_cluster_lb_memory_mb.len());
        assert_eq!(minio_cluster_len, minio_cluster_consul_service_name.len());
        assert_eq!(minio_cluster_len, minio_cluster_children_minio_instance.len());

        let mut rows_minio_cluster: Vec<TableRowMinioCluster> = Vec::with_capacity(minio_cluster_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..minio_cluster_len {
            rows_minio_cluster.push(TableRowMinioCluster {
                cluster_name: minio_cluster_cluster_name[row].clone(),
                namespace: minio_cluster_namespace[row],
                region: minio_cluster_region[row],
                workload_architecture: minio_cluster_workload_architecture[row].clone(),
                docker_image_minio: minio_cluster_docker_image_minio[row],
                docker_image_minio_mc: minio_cluster_docker_image_minio_mc[row],
                docker_image_nginx: minio_cluster_docker_image_nginx[row],
                api_port: minio_cluster_api_port[row],
                console_port: minio_cluster_console_port[row],
                lb_port: minio_cluster_lb_port[row],
                loki_cluster: minio_cluster_loki_cluster[row].clone(),
                monitoring_cluster: minio_cluster_monitoring_cluster[row].clone(),
                expected_zfs_recordsize: minio_cluster_expected_zfs_recordsize[row].clone(),
                distribute_over_dcs: minio_cluster_distribute_over_dcs[row],
                instance_memory_mb: minio_cluster_instance_memory_mb[row],
                lb_memory_mb: minio_cluster_lb_memory_mb[row],
                consul_service_name: minio_cluster_consul_service_name[row].clone(),
                children_minio_instance: minio_cluster_children_minio_instance[row].clone(),
                children_minio_bucket: minio_cluster_children_minio_bucket[row].clone(),
            });
        }

        let minio_instance_instance_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_instance_instance_volume: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let minio_instance_parent: Vec<TableRowPointerMinioCluster> = ::bincode::deserialize_from(&mut cursor)?;

        let minio_instance_len = minio_instance_parent.len();

        assert_eq!(minio_instance_len, minio_instance_instance_id.len());
        assert_eq!(minio_instance_len, minio_instance_instance_volume.len());

        let mut rows_minio_instance: Vec<TableRowMinioInstance> = Vec::with_capacity(minio_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..minio_instance_len {
            rows_minio_instance.push(TableRowMinioInstance {
                instance_id: minio_instance_instance_id[row],
                instance_volume: minio_instance_instance_volume[row],
                parent: minio_instance_parent[row],
            });
        }

        let monitoring_cluster_cluster_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_is_region_default: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_docker_image_prometheus: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_docker_image_alertmanager: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_docker_image_victoriametrics: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_prometheus_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_victoriametrics_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_alertmanager_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_prometheus_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_victoriametrics_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_alertmanager_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_alertmanager_p2p_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_victoriametrics_retention_months: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_children_monitoring_cluster_scraped_metric: Vec<Vec<TableRowPointerMonitoringClusterScrapedMetric>> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_children_monitoring_cluster_alert_group: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_children_monitoring_instance: Vec<Vec<TableRowPointerMonitoringInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_children_alertmanager_instance: Vec<Vec<TableRowPointerAlertmanagerInstance>> = ::bincode::deserialize_from(&mut cursor)?;

        let monitoring_cluster_len = monitoring_cluster_children_alertmanager_instance.len();

        assert_eq!(monitoring_cluster_len, monitoring_cluster_cluster_name.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_namespace.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_region.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_is_region_default.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_workload_architecture.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_docker_image_prometheus.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_docker_image_alertmanager.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_docker_image_victoriametrics.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_prometheus_memory_mb.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_victoriametrics_memory_mb.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_alertmanager_memory_mb.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_prometheus_port.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_victoriametrics_port.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_alertmanager_port.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_alertmanager_p2p_port.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_victoriametrics_retention_months.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_children_monitoring_cluster_scraped_metric.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_children_monitoring_cluster_alert_group.len());
        assert_eq!(monitoring_cluster_len, monitoring_cluster_children_monitoring_instance.len());

        let mut rows_monitoring_cluster: Vec<TableRowMonitoringCluster> = Vec::with_capacity(monitoring_cluster_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..monitoring_cluster_len {
            rows_monitoring_cluster.push(TableRowMonitoringCluster {
                cluster_name: monitoring_cluster_cluster_name[row].clone(),
                namespace: monitoring_cluster_namespace[row],
                region: monitoring_cluster_region[row],
                is_region_default: monitoring_cluster_is_region_default[row],
                workload_architecture: monitoring_cluster_workload_architecture[row].clone(),
                docker_image_prometheus: monitoring_cluster_docker_image_prometheus[row],
                docker_image_alertmanager: monitoring_cluster_docker_image_alertmanager[row],
                docker_image_victoriametrics: monitoring_cluster_docker_image_victoriametrics[row],
                prometheus_memory_mb: monitoring_cluster_prometheus_memory_mb[row],
                victoriametrics_memory_mb: monitoring_cluster_victoriametrics_memory_mb[row],
                alertmanager_memory_mb: monitoring_cluster_alertmanager_memory_mb[row],
                prometheus_port: monitoring_cluster_prometheus_port[row],
                victoriametrics_port: monitoring_cluster_victoriametrics_port[row],
                alertmanager_port: monitoring_cluster_alertmanager_port[row],
                alertmanager_p2p_port: monitoring_cluster_alertmanager_p2p_port[row],
                victoriametrics_retention_months: monitoring_cluster_victoriametrics_retention_months[row],
                children_monitoring_cluster_scraped_metric: monitoring_cluster_children_monitoring_cluster_scraped_metric[row].clone(),
                children_monitoring_cluster_alert_group: monitoring_cluster_children_monitoring_cluster_alert_group[row].clone(),
                children_monitoring_instance: monitoring_cluster_children_monitoring_instance[row].clone(),
                children_alertmanager_instance: monitoring_cluster_children_alertmanager_instance[row].clone(),
            });
        }

        let monitoring_cluster_alert_group_alert_group_name: Vec<TableRowPointerAlertGroup> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_alert_group_telegram_channel: Vec<TableRowPointerTelegramChannel> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_alert_group_telegram_bot: Vec<TableRowPointerTelegramBot> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_alert_group_parent: Vec<TableRowPointerMonitoringCluster> = ::bincode::deserialize_from(&mut cursor)?;

        let monitoring_cluster_alert_group_len = monitoring_cluster_alert_group_parent.len();

        assert_eq!(monitoring_cluster_alert_group_len, monitoring_cluster_alert_group_alert_group_name.len());
        assert_eq!(monitoring_cluster_alert_group_len, monitoring_cluster_alert_group_telegram_channel.len());
        assert_eq!(monitoring_cluster_alert_group_len, monitoring_cluster_alert_group_telegram_bot.len());

        let mut rows_monitoring_cluster_alert_group: Vec<TableRowMonitoringClusterAlertGroup> = Vec::with_capacity(monitoring_cluster_alert_group_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..monitoring_cluster_alert_group_len {
            rows_monitoring_cluster_alert_group.push(TableRowMonitoringClusterAlertGroup {
                alert_group_name: monitoring_cluster_alert_group_alert_group_name[row],
                telegram_channel: monitoring_cluster_alert_group_telegram_channel[row],
                telegram_bot: monitoring_cluster_alert_group_telegram_bot[row],
                parent: monitoring_cluster_alert_group_parent[row],
            });
        }

        let monitoring_cluster_scraped_metric_metric_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_scraped_metric_expression: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_cluster_scraped_metric_parent: Vec<TableRowPointerMonitoringCluster> = ::bincode::deserialize_from(&mut cursor)?;

        let monitoring_cluster_scraped_metric_len = monitoring_cluster_scraped_metric_parent.len();

        assert_eq!(monitoring_cluster_scraped_metric_len, monitoring_cluster_scraped_metric_metric_name.len());
        assert_eq!(monitoring_cluster_scraped_metric_len, monitoring_cluster_scraped_metric_expression.len());

        let mut rows_monitoring_cluster_scraped_metric: Vec<TableRowMonitoringClusterScrapedMetric> = Vec::with_capacity(monitoring_cluster_scraped_metric_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..monitoring_cluster_scraped_metric_len {
            rows_monitoring_cluster_scraped_metric.push(TableRowMonitoringClusterScrapedMetric {
                metric_name: monitoring_cluster_scraped_metric_metric_name[row].clone(),
                expression: monitoring_cluster_scraped_metric_expression[row].clone(),
                parent: monitoring_cluster_scraped_metric_parent[row],
            });
        }

        let monitoring_instance_instance_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_instance_monitoring_server: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let monitoring_instance_parent: Vec<TableRowPointerMonitoringCluster> = ::bincode::deserialize_from(&mut cursor)?;

        let monitoring_instance_len = monitoring_instance_parent.len();

        assert_eq!(monitoring_instance_len, monitoring_instance_instance_id.len());
        assert_eq!(monitoring_instance_len, monitoring_instance_monitoring_server.len());

        let mut rows_monitoring_instance: Vec<TableRowMonitoringInstance> = Vec::with_capacity(monitoring_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..monitoring_instance_len {
            rows_monitoring_instance.push(TableRowMonitoringInstance {
                instance_id: monitoring_instance_instance_id[row],
                monitoring_server: monitoring_instance_monitoring_server[row],
                parent: monitoring_instance_parent[row],
            });
        }

        let nats_cluster_cluster_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_distribute_over_dcs: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_docker_image_nats: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_docker_image_nats_exporter: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_nats_clients_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_nats_cluster_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_nats_http_mon_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_nats_prometheus_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_instance_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_children_nats_jetstream_stream: Vec<Vec<TableRowPointerNatsJetstreamStream>> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_cluster_children_nats_deployment_instance: Vec<Vec<TableRowPointerNatsDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;

        let nats_cluster_len = nats_cluster_children_nats_deployment_instance.len();

        assert_eq!(nats_cluster_len, nats_cluster_cluster_name.len());
        assert_eq!(nats_cluster_len, nats_cluster_namespace.len());
        assert_eq!(nats_cluster_len, nats_cluster_region.len());
        assert_eq!(nats_cluster_len, nats_cluster_loki_cluster.len());
        assert_eq!(nats_cluster_len, nats_cluster_monitoring_cluster.len());
        assert_eq!(nats_cluster_len, nats_cluster_distribute_over_dcs.len());
        assert_eq!(nats_cluster_len, nats_cluster_workload_architecture.len());
        assert_eq!(nats_cluster_len, nats_cluster_docker_image_nats.len());
        assert_eq!(nats_cluster_len, nats_cluster_docker_image_nats_exporter.len());
        assert_eq!(nats_cluster_len, nats_cluster_nats_clients_port.len());
        assert_eq!(nats_cluster_len, nats_cluster_nats_cluster_port.len());
        assert_eq!(nats_cluster_len, nats_cluster_nats_http_mon_port.len());
        assert_eq!(nats_cluster_len, nats_cluster_nats_prometheus_port.len());
        assert_eq!(nats_cluster_len, nats_cluster_instance_memory_mb.len());
        assert_eq!(nats_cluster_len, nats_cluster_children_nats_jetstream_stream.len());

        let mut rows_nats_cluster: Vec<TableRowNatsCluster> = Vec::with_capacity(nats_cluster_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..nats_cluster_len {
            rows_nats_cluster.push(TableRowNatsCluster {
                cluster_name: nats_cluster_cluster_name[row].clone(),
                namespace: nats_cluster_namespace[row],
                region: nats_cluster_region[row],
                loki_cluster: nats_cluster_loki_cluster[row].clone(),
                monitoring_cluster: nats_cluster_monitoring_cluster[row].clone(),
                distribute_over_dcs: nats_cluster_distribute_over_dcs[row],
                workload_architecture: nats_cluster_workload_architecture[row].clone(),
                docker_image_nats: nats_cluster_docker_image_nats[row],
                docker_image_nats_exporter: nats_cluster_docker_image_nats_exporter[row],
                nats_clients_port: nats_cluster_nats_clients_port[row],
                nats_cluster_port: nats_cluster_nats_cluster_port[row],
                nats_http_mon_port: nats_cluster_nats_http_mon_port[row],
                nats_prometheus_port: nats_cluster_nats_prometheus_port[row],
                instance_memory_mb: nats_cluster_instance_memory_mb[row],
                children_nats_jetstream_stream: nats_cluster_children_nats_jetstream_stream[row].clone(),
                children_nats_deployment_instance: nats_cluster_children_nats_deployment_instance[row].clone(),
            });
        }

        let nats_deployment_instance_instance_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_deployment_instance_nats_server: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_deployment_instance_parent: Vec<TableRowPointerNatsCluster> = ::bincode::deserialize_from(&mut cursor)?;

        let nats_deployment_instance_len = nats_deployment_instance_parent.len();

        assert_eq!(nats_deployment_instance_len, nats_deployment_instance_instance_id.len());
        assert_eq!(nats_deployment_instance_len, nats_deployment_instance_nats_server.len());

        let mut rows_nats_deployment_instance: Vec<TableRowNatsDeploymentInstance> = Vec::with_capacity(nats_deployment_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..nats_deployment_instance_len {
            rows_nats_deployment_instance.push(TableRowNatsDeploymentInstance {
                instance_id: nats_deployment_instance_instance_id[row],
                nats_server: nats_deployment_instance_nats_server[row],
                parent: nats_deployment_instance_parent[row],
            });
        }

        let nats_jetstream_stream_stream_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_jetstream_stream_stream_type: Vec<TableRowPointerVersionedType> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_jetstream_stream_max_bytes: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_jetstream_stream_max_msg_size: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_jetstream_stream_enable_subjects: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_jetstream_stream_parent: Vec<TableRowPointerNatsCluster> = ::bincode::deserialize_from(&mut cursor)?;
        let nats_jetstream_stream_referrers_ch_nats_stream_import__stream: Vec<Vec<TableRowPointerChNatsStreamImport>> = ::bincode::deserialize_from(&mut cursor)?;

        let nats_jetstream_stream_len = nats_jetstream_stream_referrers_ch_nats_stream_import__stream.len();

        assert_eq!(nats_jetstream_stream_len, nats_jetstream_stream_stream_name.len());
        assert_eq!(nats_jetstream_stream_len, nats_jetstream_stream_stream_type.len());
        assert_eq!(nats_jetstream_stream_len, nats_jetstream_stream_max_bytes.len());
        assert_eq!(nats_jetstream_stream_len, nats_jetstream_stream_max_msg_size.len());
        assert_eq!(nats_jetstream_stream_len, nats_jetstream_stream_enable_subjects.len());
        assert_eq!(nats_jetstream_stream_len, nats_jetstream_stream_parent.len());

        let mut rows_nats_jetstream_stream: Vec<TableRowNatsJetstreamStream> = Vec::with_capacity(nats_jetstream_stream_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..nats_jetstream_stream_len {
            rows_nats_jetstream_stream.push(TableRowNatsJetstreamStream {
                stream_name: nats_jetstream_stream_stream_name[row].clone(),
                stream_type: nats_jetstream_stream_stream_type[row],
                max_bytes: nats_jetstream_stream_max_bytes[row],
                max_msg_size: nats_jetstream_stream_max_msg_size[row],
                enable_subjects: nats_jetstream_stream_enable_subjects[row],
                parent: nats_jetstream_stream_parent[row],
                referrers_ch_nats_stream_import__stream: nats_jetstream_stream_referrers_ch_nats_stream_import__stream[row].clone(),
            });
        }

        let network_network_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let network_cidr: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let network_referrers_network_interface__if_network: Vec<Vec<TableRowPointerNetworkInterface>> = ::bincode::deserialize_from(&mut cursor)?;

        let network_len = network_referrers_network_interface__if_network.len();

        assert_eq!(network_len, network_network_name.len());
        assert_eq!(network_len, network_cidr.len());

        let mut rows_network: Vec<TableRowNetwork> = Vec::with_capacity(network_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..network_len {
            rows_network.push(TableRowNetwork {
                network_name: network_network_name[row].clone(),
                cidr: network_cidr[row].clone(),
                referrers_network_interface__if_network: network_referrers_network_interface__if_network[row].clone(),
            });
        }

        let network_interface_if_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let network_interface_if_network: Vec<TableRowPointerNetwork> = ::bincode::deserialize_from(&mut cursor)?;
        let network_interface_if_ip: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let network_interface_if_prefix: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let network_interface_if_vlan: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let network_interface_parent: Vec<TableRowPointerServer> = ::bincode::deserialize_from(&mut cursor)?;
        let network_interface_referrers_server__ssh_interface: Vec<Vec<TableRowPointerServer>> = ::bincode::deserialize_from(&mut cursor)?;

        let network_interface_len = network_interface_referrers_server__ssh_interface.len();

        assert_eq!(network_interface_len, network_interface_if_name.len());
        assert_eq!(network_interface_len, network_interface_if_network.len());
        assert_eq!(network_interface_len, network_interface_if_ip.len());
        assert_eq!(network_interface_len, network_interface_if_prefix.len());
        assert_eq!(network_interface_len, network_interface_if_vlan.len());
        assert_eq!(network_interface_len, network_interface_parent.len());

        let mut rows_network_interface: Vec<TableRowNetworkInterface> = Vec::with_capacity(network_interface_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..network_interface_len {
            rows_network_interface.push(TableRowNetworkInterface {
                if_name: network_interface_if_name[row].clone(),
                if_network: network_interface_if_network[row],
                if_ip: network_interface_if_ip[row].clone(),
                if_prefix: network_interface_if_prefix[row],
                if_vlan: network_interface_if_vlan[row],
                parent: network_interface_parent[row],
                referrers_server__ssh_interface: network_interface_referrers_server__ssh_interface[row].clone(),
            });
        }

        let nixpkgs_environment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nixpkgs_environment_version: Vec<TableRowPointerNixpkgsVersion> = ::bincode::deserialize_from(&mut cursor)?;
        let nixpkgs_environment_referrers_server__nixpkgs_environment: Vec<Vec<TableRowPointerServer>> = ::bincode::deserialize_from(&mut cursor)?;
        let nixpkgs_environment_referrers_rust_compilation_environment__nixpkgs_environment: Vec<Vec<TableRowPointerRustCompilationEnvironment>> = ::bincode::deserialize_from(&mut cursor)?;

        let nixpkgs_environment_len = nixpkgs_environment_referrers_rust_compilation_environment__nixpkgs_environment.len();

        assert_eq!(nixpkgs_environment_len, nixpkgs_environment_name.len());
        assert_eq!(nixpkgs_environment_len, nixpkgs_environment_version.len());
        assert_eq!(nixpkgs_environment_len, nixpkgs_environment_referrers_server__nixpkgs_environment.len());

        let mut rows_nixpkgs_environment: Vec<TableRowNixpkgsEnvironment> = Vec::with_capacity(nixpkgs_environment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..nixpkgs_environment_len {
            rows_nixpkgs_environment.push(TableRowNixpkgsEnvironment {
                name: nixpkgs_environment_name[row].clone(),
                version: nixpkgs_environment_version[row],
                referrers_server__nixpkgs_environment: nixpkgs_environment_referrers_server__nixpkgs_environment[row].clone(),
                referrers_rust_compilation_environment__nixpkgs_environment: nixpkgs_environment_referrers_rust_compilation_environment__nixpkgs_environment[row].clone(),
            });
        }

        let nixpkgs_version_version: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nixpkgs_version_checksum: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nixpkgs_version_tarball_checksum: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nixpkgs_version_referrers_nixpkgs_environment__version: Vec<Vec<TableRowPointerNixpkgsEnvironment>> = ::bincode::deserialize_from(&mut cursor)?;

        let nixpkgs_version_len = nixpkgs_version_referrers_nixpkgs_environment__version.len();

        assert_eq!(nixpkgs_version_len, nixpkgs_version_version.len());
        assert_eq!(nixpkgs_version_len, nixpkgs_version_checksum.len());
        assert_eq!(nixpkgs_version_len, nixpkgs_version_tarball_checksum.len());

        let mut rows_nixpkgs_version: Vec<TableRowNixpkgsVersion> = Vec::with_capacity(nixpkgs_version_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..nixpkgs_version_len {
            rows_nixpkgs_version.push(TableRowNixpkgsVersion {
                version: nixpkgs_version_version[row].clone(),
                checksum: nixpkgs_version_checksum[row].clone(),
                tarball_checksum: nixpkgs_version_tarball_checksum[row].clone(),
                referrers_nixpkgs_environment__version: nixpkgs_version_referrers_nixpkgs_environment__version[row].clone(),
            });
        }

        let nomad_namespace_namespace: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_description: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_pg_deployment__namespace: Vec<Vec<TableRowPointerPgDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_ch_deployment__namespace: Vec<Vec<TableRowPointerChDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_ch_keeper_deployment__namespace: Vec<Vec<TableRowPointerChKeeperDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_nats_cluster__namespace: Vec<Vec<TableRowPointerNatsCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_backend_application_deployment__namespace: Vec<Vec<TableRowPointerBackendApplicationDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_frontend_application_deployment__namespace: Vec<Vec<TableRowPointerFrontendApplicationDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_minio_cluster__namespace: Vec<Vec<TableRowPointerMinioCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_monitoring_cluster__namespace: Vec<Vec<TableRowPointerMonitoringCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_grafana__namespace: Vec<Vec<TableRowPointerGrafana>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_loki_cluster__namespace: Vec<Vec<TableRowPointerLokiCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_tempo_cluster__namespace: Vec<Vec<TableRowPointerTempoCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let nomad_namespace_referrers_blackbox_deployment__namespace: Vec<Vec<TableRowPointerBlackboxDeployment>> = ::bincode::deserialize_from(&mut cursor)?;

        let nomad_namespace_len = nomad_namespace_referrers_blackbox_deployment__namespace.len();

        assert_eq!(nomad_namespace_len, nomad_namespace_namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_description.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_pg_deployment__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_ch_deployment__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_ch_keeper_deployment__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_nats_cluster__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_backend_application_deployment__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_frontend_application_deployment__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_minio_cluster__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_monitoring_cluster__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_grafana__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_loki_cluster__namespace.len());
        assert_eq!(nomad_namespace_len, nomad_namespace_referrers_tempo_cluster__namespace.len());

        let mut rows_nomad_namespace: Vec<TableRowNomadNamespace> = Vec::with_capacity(nomad_namespace_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..nomad_namespace_len {
            rows_nomad_namespace.push(TableRowNomadNamespace {
                namespace: nomad_namespace_namespace[row].clone(),
                description: nomad_namespace_description[row].clone(),
                referrers_pg_deployment__namespace: nomad_namespace_referrers_pg_deployment__namespace[row].clone(),
                referrers_ch_deployment__namespace: nomad_namespace_referrers_ch_deployment__namespace[row].clone(),
                referrers_ch_keeper_deployment__namespace: nomad_namespace_referrers_ch_keeper_deployment__namespace[row].clone(),
                referrers_nats_cluster__namespace: nomad_namespace_referrers_nats_cluster__namespace[row].clone(),
                referrers_backend_application_deployment__namespace: nomad_namespace_referrers_backend_application_deployment__namespace[row].clone(),
                referrers_frontend_application_deployment__namespace: nomad_namespace_referrers_frontend_application_deployment__namespace[row].clone(),
                referrers_minio_cluster__namespace: nomad_namespace_referrers_minio_cluster__namespace[row].clone(),
                referrers_monitoring_cluster__namespace: nomad_namespace_referrers_monitoring_cluster__namespace[row].clone(),
                referrers_grafana__namespace: nomad_namespace_referrers_grafana__namespace[row].clone(),
                referrers_loki_cluster__namespace: nomad_namespace_referrers_loki_cluster__namespace[row].clone(),
                referrers_tempo_cluster__namespace: nomad_namespace_referrers_tempo_cluster__namespace[row].clone(),
                referrers_blackbox_deployment__namespace: nomad_namespace_referrers_blackbox_deployment__namespace[row].clone(),
            });
        }

        let pg_deployment_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_docker_image_pg: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_docker_image_haproxy: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_docker_image_pg_exporter: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_distribute_over_dcs: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_synchronous_replication: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_shared_buffers_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_work_mem_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_maintenance_work_mem_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_overhead_mem_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_max_connections: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_replica_rolling_update_delay_seconds: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_pg_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_pg_master_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_pg_slave_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_patroni_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_haproxy_metrics_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_pg_exporter_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_children_pg_deployment_schemas: Vec<Vec<TableRowPointerPgDeploymentSchemas>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_children_pg_deployment_unmanaged_db: Vec<Vec<TableRowPointerPgDeploymentUnmanagedDb>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_children_pg_deployment_instance: Vec<Vec<TableRowPointerPgDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_deployment_len = pg_deployment_children_pg_deployment_instance.len();

        assert_eq!(pg_deployment_len, pg_deployment_deployment_name.len());
        assert_eq!(pg_deployment_len, pg_deployment_namespace.len());
        assert_eq!(pg_deployment_len, pg_deployment_region.len());
        assert_eq!(pg_deployment_len, pg_deployment_loki_cluster.len());
        assert_eq!(pg_deployment_len, pg_deployment_monitoring_cluster.len());
        assert_eq!(pg_deployment_len, pg_deployment_docker_image_pg.len());
        assert_eq!(pg_deployment_len, pg_deployment_docker_image_haproxy.len());
        assert_eq!(pg_deployment_len, pg_deployment_docker_image_pg_exporter.len());
        assert_eq!(pg_deployment_len, pg_deployment_workload_architecture.len());
        assert_eq!(pg_deployment_len, pg_deployment_distribute_over_dcs.len());
        assert_eq!(pg_deployment_len, pg_deployment_synchronous_replication.len());
        assert_eq!(pg_deployment_len, pg_deployment_shared_buffers_mb.len());
        assert_eq!(pg_deployment_len, pg_deployment_work_mem_mb.len());
        assert_eq!(pg_deployment_len, pg_deployment_maintenance_work_mem_mb.len());
        assert_eq!(pg_deployment_len, pg_deployment_overhead_mem_mb.len());
        assert_eq!(pg_deployment_len, pg_deployment_max_connections.len());
        assert_eq!(pg_deployment_len, pg_deployment_replica_rolling_update_delay_seconds.len());
        assert_eq!(pg_deployment_len, pg_deployment_instance_pg_port.len());
        assert_eq!(pg_deployment_len, pg_deployment_instance_pg_master_port.len());
        assert_eq!(pg_deployment_len, pg_deployment_instance_pg_slave_port.len());
        assert_eq!(pg_deployment_len, pg_deployment_instance_patroni_port.len());
        assert_eq!(pg_deployment_len, pg_deployment_instance_haproxy_metrics_port.len());
        assert_eq!(pg_deployment_len, pg_deployment_instance_pg_exporter_port.len());
        assert_eq!(pg_deployment_len, pg_deployment_children_pg_deployment_schemas.len());
        assert_eq!(pg_deployment_len, pg_deployment_children_pg_deployment_unmanaged_db.len());

        let mut rows_pg_deployment: Vec<TableRowPgDeployment> = Vec::with_capacity(pg_deployment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_deployment_len {
            rows_pg_deployment.push(TableRowPgDeployment {
                deployment_name: pg_deployment_deployment_name[row].clone(),
                namespace: pg_deployment_namespace[row],
                region: pg_deployment_region[row],
                loki_cluster: pg_deployment_loki_cluster[row].clone(),
                monitoring_cluster: pg_deployment_monitoring_cluster[row].clone(),
                docker_image_pg: pg_deployment_docker_image_pg[row],
                docker_image_haproxy: pg_deployment_docker_image_haproxy[row],
                docker_image_pg_exporter: pg_deployment_docker_image_pg_exporter[row],
                workload_architecture: pg_deployment_workload_architecture[row].clone(),
                distribute_over_dcs: pg_deployment_distribute_over_dcs[row],
                synchronous_replication: pg_deployment_synchronous_replication[row],
                shared_buffers_mb: pg_deployment_shared_buffers_mb[row],
                work_mem_mb: pg_deployment_work_mem_mb[row],
                maintenance_work_mem_mb: pg_deployment_maintenance_work_mem_mb[row],
                overhead_mem_mb: pg_deployment_overhead_mem_mb[row],
                max_connections: pg_deployment_max_connections[row],
                replica_rolling_update_delay_seconds: pg_deployment_replica_rolling_update_delay_seconds[row],
                instance_pg_port: pg_deployment_instance_pg_port[row],
                instance_pg_master_port: pg_deployment_instance_pg_master_port[row],
                instance_pg_slave_port: pg_deployment_instance_pg_slave_port[row],
                instance_patroni_port: pg_deployment_instance_patroni_port[row],
                instance_haproxy_metrics_port: pg_deployment_instance_haproxy_metrics_port[row],
                instance_pg_exporter_port: pg_deployment_instance_pg_exporter_port[row],
                children_pg_deployment_schemas: pg_deployment_children_pg_deployment_schemas[row].clone(),
                children_pg_deployment_unmanaged_db: pg_deployment_children_pg_deployment_unmanaged_db[row].clone(),
                children_pg_deployment_instance: pg_deployment_children_pg_deployment_instance[row].clone(),
            });
        }

        let pg_deployment_instance_instance_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_pg_server: Vec<TableRowPointerServerVolume> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_instance_parent: Vec<TableRowPointerPgDeployment> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_deployment_instance_len = pg_deployment_instance_parent.len();

        assert_eq!(pg_deployment_instance_len, pg_deployment_instance_instance_id.len());
        assert_eq!(pg_deployment_instance_len, pg_deployment_instance_pg_server.len());

        let mut rows_pg_deployment_instance: Vec<TableRowPgDeploymentInstance> = Vec::with_capacity(pg_deployment_instance_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_deployment_instance_len {
            rows_pg_deployment_instance.push(TableRowPgDeploymentInstance {
                instance_id: pg_deployment_instance_instance_id[row],
                pg_server: pg_deployment_instance_pg_server[row],
                parent: pg_deployment_instance_parent[row],
            });
        }

        let pg_deployment_schemas_db_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_schemas_pg_schema: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_schemas_parent: Vec<TableRowPointerPgDeployment> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_deployment_schemas_len = pg_deployment_schemas_parent.len();

        assert_eq!(pg_deployment_schemas_len, pg_deployment_schemas_db_name.len());
        assert_eq!(pg_deployment_schemas_len, pg_deployment_schemas_pg_schema.len());

        let mut rows_pg_deployment_schemas: Vec<TableRowPgDeploymentSchemas> = Vec::with_capacity(pg_deployment_schemas_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_deployment_schemas_len {
            rows_pg_deployment_schemas.push(TableRowPgDeploymentSchemas {
                db_name: pg_deployment_schemas_db_name[row].clone(),
                pg_schema: pg_deployment_schemas_pg_schema[row],
                parent: pg_deployment_schemas_parent[row],
            });
        }

        let pg_deployment_unmanaged_db_db_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_unmanaged_db_parent: Vec<TableRowPointerPgDeployment> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_deployment_unmanaged_db_referrers_grafana__database: Vec<Vec<TableRowPointerGrafana>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_deployment_unmanaged_db_len = pg_deployment_unmanaged_db_referrers_grafana__database.len();

        assert_eq!(pg_deployment_unmanaged_db_len, pg_deployment_unmanaged_db_db_name.len());
        assert_eq!(pg_deployment_unmanaged_db_len, pg_deployment_unmanaged_db_parent.len());

        let mut rows_pg_deployment_unmanaged_db: Vec<TableRowPgDeploymentUnmanagedDb> = Vec::with_capacity(pg_deployment_unmanaged_db_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_deployment_unmanaged_db_len {
            rows_pg_deployment_unmanaged_db.push(TableRowPgDeploymentUnmanagedDb {
                db_name: pg_deployment_unmanaged_db_db_name[row].clone(),
                parent: pg_deployment_unmanaged_db_parent[row],
                referrers_grafana__database: pg_deployment_unmanaged_db_referrers_grafana__database[row].clone(),
            });
        }

        let pg_mat_view_mview_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mat_view_update_frequency: Vec<TableRowPointerPgMatViewUpdateFrequency> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mat_view_parent: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mat_view_children_pg_mat_view_test: Vec<Vec<TableRowPointerPgMatViewTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_mat_view_len = pg_mat_view_children_pg_mat_view_test.len();

        assert_eq!(pg_mat_view_len, pg_mat_view_mview_name.len());
        assert_eq!(pg_mat_view_len, pg_mat_view_update_frequency.len());
        assert_eq!(pg_mat_view_len, pg_mat_view_parent.len());

        let mut rows_pg_mat_view: Vec<TableRowPgMatView> = Vec::with_capacity(pg_mat_view_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_mat_view_len {
            rows_pg_mat_view.push(TableRowPgMatView {
                mview_name: pg_mat_view_mview_name[row].clone(),
                update_frequency: pg_mat_view_update_frequency[row],
                parent: pg_mat_view_parent[row],
                children_pg_mat_view_test: pg_mat_view_children_pg_mat_view_test[row].clone(),
            });
        }

        let pg_mat_view_test_expected_data: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mat_view_test_test_dataset: Vec<TableRowPointerPgTestDataset> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mat_view_test_parent: Vec<TableRowPointerPgMatView> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_mat_view_test_len = pg_mat_view_test_parent.len();

        assert_eq!(pg_mat_view_test_len, pg_mat_view_test_expected_data.len());
        assert_eq!(pg_mat_view_test_len, pg_mat_view_test_test_dataset.len());

        let mut rows_pg_mat_view_test: Vec<TableRowPgMatViewTest> = Vec::with_capacity(pg_mat_view_test_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_mat_view_test_len {
            rows_pg_mat_view_test.push(TableRowPgMatViewTest {
                expected_data: pg_mat_view_test_expected_data[row].clone(),
                test_dataset: pg_mat_view_test_test_dataset[row],
                parent: pg_mat_view_test_parent[row],
            });
        }

        let pg_mat_view_update_frequency_frequency: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mat_view_update_frequency_referrers_pg_mat_view__update_frequency: Vec<Vec<TableRowPointerPgMatView>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_mat_view_update_frequency_len = pg_mat_view_update_frequency_referrers_pg_mat_view__update_frequency.len();

        assert_eq!(pg_mat_view_update_frequency_len, pg_mat_view_update_frequency_frequency.len());

        let mut rows_pg_mat_view_update_frequency: Vec<TableRowPgMatViewUpdateFrequency> = Vec::with_capacity(pg_mat_view_update_frequency_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_mat_view_update_frequency_len {
            rows_pg_mat_view_update_frequency.push(TableRowPgMatViewUpdateFrequency {
                frequency: pg_mat_view_update_frequency_frequency[row].clone(),
                referrers_pg_mat_view__update_frequency: pg_mat_view_update_frequency_referrers_pg_mat_view__update_frequency[row].clone(),
            });
        }

        let pg_migration_time: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_migration_upgrade: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_migration_downgrade: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_migration_needs_admin: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_migration_parent: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_migration_len = pg_migration_parent.len();

        assert_eq!(pg_migration_len, pg_migration_time.len());
        assert_eq!(pg_migration_len, pg_migration_upgrade.len());
        assert_eq!(pg_migration_len, pg_migration_downgrade.len());
        assert_eq!(pg_migration_len, pg_migration_needs_admin.len());

        let mut rows_pg_migration: Vec<TableRowPgMigration> = Vec::with_capacity(pg_migration_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_migration_len {
            rows_pg_migration.push(TableRowPgMigration {
                time: pg_migration_time[row],
                upgrade: pg_migration_upgrade[row].clone(),
                downgrade: pg_migration_downgrade[row].clone(),
                needs_admin: pg_migration_needs_admin[row],
                parent: pg_migration_parent[row],
            });
        }

        let pg_mutator_mutator_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mutator_mutator_expression: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mutator_seqscan_ok: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mutator_parent: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mutator_children_pg_mutator_test: Vec<Vec<TableRowPointerPgMutatorTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_mutator_len = pg_mutator_children_pg_mutator_test.len();

        assert_eq!(pg_mutator_len, pg_mutator_mutator_name.len());
        assert_eq!(pg_mutator_len, pg_mutator_mutator_expression.len());
        assert_eq!(pg_mutator_len, pg_mutator_seqscan_ok.len());
        assert_eq!(pg_mutator_len, pg_mutator_parent.len());

        let mut rows_pg_mutator: Vec<TableRowPgMutator> = Vec::with_capacity(pg_mutator_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_mutator_len {
            rows_pg_mutator.push(TableRowPgMutator {
                mutator_name: pg_mutator_mutator_name[row].clone(),
                mutator_expression: pg_mutator_mutator_expression[row].clone(),
                seqscan_ok: pg_mutator_seqscan_ok[row],
                parent: pg_mutator_parent[row],
                children_pg_mutator_test: pg_mutator_children_pg_mutator_test[row].clone(),
            });
        }

        let pg_mutator_test_arguments: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mutator_test_test_dataset: Vec<TableRowPointerPgTestDataset> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mutator_test_resulting_data: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_mutator_test_parent: Vec<TableRowPointerPgMutator> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_mutator_test_len = pg_mutator_test_parent.len();

        assert_eq!(pg_mutator_test_len, pg_mutator_test_arguments.len());
        assert_eq!(pg_mutator_test_len, pg_mutator_test_test_dataset.len());
        assert_eq!(pg_mutator_test_len, pg_mutator_test_resulting_data.len());

        let mut rows_pg_mutator_test: Vec<TableRowPgMutatorTest> = Vec::with_capacity(pg_mutator_test_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_mutator_test_len {
            rows_pg_mutator_test.push(TableRowPgMutatorTest {
                arguments: pg_mutator_test_arguments[row].clone(),
                test_dataset: pg_mutator_test_test_dataset[row],
                resulting_data: pg_mutator_test_resulting_data[row].clone(),
                parent: pg_mutator_test_parent[row],
            });
        }

        let pg_query_query_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_query_expression: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_is_mutating: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_seqscan_ok: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_opt_fields: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_parent: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_children_pg_query_test: Vec<Vec<TableRowPointerPgQueryTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_query_len = pg_query_children_pg_query_test.len();

        assert_eq!(pg_query_len, pg_query_query_name.len());
        assert_eq!(pg_query_len, pg_query_query_expression.len());
        assert_eq!(pg_query_len, pg_query_is_mutating.len());
        assert_eq!(pg_query_len, pg_query_seqscan_ok.len());
        assert_eq!(pg_query_len, pg_query_opt_fields.len());
        assert_eq!(pg_query_len, pg_query_parent.len());

        let mut rows_pg_query: Vec<TableRowPgQuery> = Vec::with_capacity(pg_query_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_query_len {
            rows_pg_query.push(TableRowPgQuery {
                query_name: pg_query_query_name[row].clone(),
                query_expression: pg_query_query_expression[row].clone(),
                is_mutating: pg_query_is_mutating[row],
                seqscan_ok: pg_query_seqscan_ok[row],
                opt_fields: pg_query_opt_fields[row].clone(),
                parent: pg_query_parent[row],
                children_pg_query_test: pg_query_children_pg_query_test[row].clone(),
            });
        }

        let pg_query_test_arguments: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_test_outputs: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_test_test_dataset: Vec<TableRowPointerPgTestDataset> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_query_test_parent: Vec<TableRowPointerPgQuery> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_query_test_len = pg_query_test_parent.len();

        assert_eq!(pg_query_test_len, pg_query_test_arguments.len());
        assert_eq!(pg_query_test_len, pg_query_test_outputs.len());
        assert_eq!(pg_query_test_len, pg_query_test_test_dataset.len());

        let mut rows_pg_query_test: Vec<TableRowPgQueryTest> = Vec::with_capacity(pg_query_test_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_query_test_len {
            rows_pg_query_test.push(TableRowPgQueryTest {
                arguments: pg_query_test_arguments[row].clone(),
                outputs: pg_query_test_outputs[row].clone(),
                test_dataset: pg_query_test_test_dataset[row],
                parent: pg_query_test_parent[row],
            });
        }

        let pg_schema_schema_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_children_pg_migration: Vec<Vec<TableRowPointerPgMigration>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_children_pg_query: Vec<Vec<TableRowPointerPgQuery>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_children_pg_mutator: Vec<Vec<TableRowPointerPgMutator>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_children_pg_transaction: Vec<Vec<TableRowPointerPgTransaction>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_children_pg_mat_view: Vec<Vec<TableRowPointerPgMatView>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_children_pg_test_dataset: Vec<Vec<TableRowPointerPgTestDataset>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_referrers_pg_deployment_schemas__pg_schema: Vec<Vec<TableRowPointerPgDeploymentSchemas>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_schema_referrers_backend_application_pg_shard__pg_schema: Vec<Vec<TableRowPointerBackendApplicationPgShard>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_schema_len = pg_schema_referrers_backend_application_pg_shard__pg_schema.len();

        assert_eq!(pg_schema_len, pg_schema_schema_name.len());
        assert_eq!(pg_schema_len, pg_schema_children_pg_migration.len());
        assert_eq!(pg_schema_len, pg_schema_children_pg_query.len());
        assert_eq!(pg_schema_len, pg_schema_children_pg_mutator.len());
        assert_eq!(pg_schema_len, pg_schema_children_pg_transaction.len());
        assert_eq!(pg_schema_len, pg_schema_children_pg_mat_view.len());
        assert_eq!(pg_schema_len, pg_schema_children_pg_test_dataset.len());
        assert_eq!(pg_schema_len, pg_schema_referrers_pg_deployment_schemas__pg_schema.len());

        let mut rows_pg_schema: Vec<TableRowPgSchema> = Vec::with_capacity(pg_schema_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_schema_len {
            rows_pg_schema.push(TableRowPgSchema {
                schema_name: pg_schema_schema_name[row].clone(),
                children_pg_migration: pg_schema_children_pg_migration[row].clone(),
                children_pg_query: pg_schema_children_pg_query[row].clone(),
                children_pg_mutator: pg_schema_children_pg_mutator[row].clone(),
                children_pg_transaction: pg_schema_children_pg_transaction[row].clone(),
                children_pg_mat_view: pg_schema_children_pg_mat_view[row].clone(),
                children_pg_test_dataset: pg_schema_children_pg_test_dataset[row].clone(),
                referrers_pg_deployment_schemas__pg_schema: pg_schema_referrers_pg_deployment_schemas__pg_schema[row].clone(),
                referrers_backend_application_pg_shard__pg_schema: pg_schema_referrers_backend_application_pg_shard__pg_schema[row].clone(),
            });
        }

        let pg_test_dataset_dataset_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_test_dataset_dataset_contents: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_test_dataset_min_time: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_test_dataset_parent: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_test_dataset_referrers_pg_query_test__test_dataset: Vec<Vec<TableRowPointerPgQueryTest>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_test_dataset_referrers_pg_mutator_test__test_dataset: Vec<Vec<TableRowPointerPgMutatorTest>> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_test_dataset_referrers_pg_mat_view_test__test_dataset: Vec<Vec<TableRowPointerPgMatViewTest>> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_test_dataset_len = pg_test_dataset_referrers_pg_mat_view_test__test_dataset.len();

        assert_eq!(pg_test_dataset_len, pg_test_dataset_dataset_name.len());
        assert_eq!(pg_test_dataset_len, pg_test_dataset_dataset_contents.len());
        assert_eq!(pg_test_dataset_len, pg_test_dataset_min_time.len());
        assert_eq!(pg_test_dataset_len, pg_test_dataset_parent.len());
        assert_eq!(pg_test_dataset_len, pg_test_dataset_referrers_pg_query_test__test_dataset.len());
        assert_eq!(pg_test_dataset_len, pg_test_dataset_referrers_pg_mutator_test__test_dataset.len());

        let mut rows_pg_test_dataset: Vec<TableRowPgTestDataset> = Vec::with_capacity(pg_test_dataset_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_test_dataset_len {
            rows_pg_test_dataset.push(TableRowPgTestDataset {
                dataset_name: pg_test_dataset_dataset_name[row].clone(),
                dataset_contents: pg_test_dataset_dataset_contents[row].clone(),
                min_time: pg_test_dataset_min_time[row],
                parent: pg_test_dataset_parent[row],
                referrers_pg_query_test__test_dataset: pg_test_dataset_referrers_pg_query_test__test_dataset[row].clone(),
                referrers_pg_mutator_test__test_dataset: pg_test_dataset_referrers_pg_mutator_test__test_dataset[row].clone(),
                referrers_pg_mat_view_test__test_dataset: pg_test_dataset_referrers_pg_mat_view_test__test_dataset[row].clone(),
            });
        }

        let pg_transaction_transaction_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_transaction_steps: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_transaction_is_read_only: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let pg_transaction_parent: Vec<TableRowPointerPgSchema> = ::bincode::deserialize_from(&mut cursor)?;

        let pg_transaction_len = pg_transaction_parent.len();

        assert_eq!(pg_transaction_len, pg_transaction_transaction_name.len());
        assert_eq!(pg_transaction_len, pg_transaction_steps.len());
        assert_eq!(pg_transaction_len, pg_transaction_is_read_only.len());

        let mut rows_pg_transaction: Vec<TableRowPgTransaction> = Vec::with_capacity(pg_transaction_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..pg_transaction_len {
            rows_pg_transaction.push(TableRowPgTransaction {
                transaction_name: pg_transaction_transaction_name[row].clone(),
                steps: pg_transaction_steps[row].clone(),
                is_read_only: pg_transaction_is_read_only[row],
                parent: pg_transaction_parent[row],
            });
        }

        let region_region_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let region_availability_mode: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let region_tld: Vec<TableRowPointerTld> = ::bincode::deserialize_from(&mut cursor)?;
        let region_is_dns_master: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let region_is_dns_slave: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let region_has_coprocessor_dc: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let region_docker_image_external_lb: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let region_nomad_disable_log_collection: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_datacenter__region: Vec<Vec<TableRowPointerDatacenter>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_docker_registry_instance__region: Vec<Vec<TableRowPointerDockerRegistryInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_pg_deployment__region: Vec<Vec<TableRowPointerPgDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_ch_deployment__region: Vec<Vec<TableRowPointerChDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_ch_keeper_deployment__region: Vec<Vec<TableRowPointerChKeeperDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_nats_cluster__region: Vec<Vec<TableRowPointerNatsCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_backend_application_deployment__region: Vec<Vec<TableRowPointerBackendApplicationDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_frontend_application_deployment__region: Vec<Vec<TableRowPointerFrontendApplicationDeployment>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_minio_cluster__region: Vec<Vec<TableRowPointerMinioCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_monitoring_cluster__region: Vec<Vec<TableRowPointerMonitoringCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_grafana__region: Vec<Vec<TableRowPointerGrafana>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_loki_cluster__region: Vec<Vec<TableRowPointerLokiCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_tempo_cluster__region: Vec<Vec<TableRowPointerTempoCluster>> = ::bincode::deserialize_from(&mut cursor)?;
        let region_referrers_blackbox_deployment__region: Vec<Vec<TableRowPointerBlackboxDeployment>> = ::bincode::deserialize_from(&mut cursor)?;

        let region_len = region_referrers_blackbox_deployment__region.len();

        assert_eq!(region_len, region_region_name.len());
        assert_eq!(region_len, region_availability_mode.len());
        assert_eq!(region_len, region_tld.len());
        assert_eq!(region_len, region_is_dns_master.len());
        assert_eq!(region_len, region_is_dns_slave.len());
        assert_eq!(region_len, region_has_coprocessor_dc.len());
        assert_eq!(region_len, region_docker_image_external_lb.len());
        assert_eq!(region_len, region_nomad_disable_log_collection.len());
        assert_eq!(region_len, region_referrers_datacenter__region.len());
        assert_eq!(region_len, region_referrers_docker_registry_instance__region.len());
        assert_eq!(region_len, region_referrers_pg_deployment__region.len());
        assert_eq!(region_len, region_referrers_ch_deployment__region.len());
        assert_eq!(region_len, region_referrers_ch_keeper_deployment__region.len());
        assert_eq!(region_len, region_referrers_nats_cluster__region.len());
        assert_eq!(region_len, region_referrers_backend_application_deployment__region.len());
        assert_eq!(region_len, region_referrers_frontend_application_deployment__region.len());
        assert_eq!(region_len, region_referrers_minio_cluster__region.len());
        assert_eq!(region_len, region_referrers_monitoring_cluster__region.len());
        assert_eq!(region_len, region_referrers_grafana__region.len());
        assert_eq!(region_len, region_referrers_loki_cluster__region.len());
        assert_eq!(region_len, region_referrers_tempo_cluster__region.len());

        let mut rows_region: Vec<TableRowRegion> = Vec::with_capacity(region_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..region_len {
            rows_region.push(TableRowRegion {
                region_name: region_region_name[row].clone(),
                availability_mode: region_availability_mode[row].clone(),
                tld: region_tld[row],
                is_dns_master: region_is_dns_master[row],
                is_dns_slave: region_is_dns_slave[row],
                has_coprocessor_dc: region_has_coprocessor_dc[row],
                docker_image_external_lb: region_docker_image_external_lb[row],
                nomad_disable_log_collection: region_nomad_disable_log_collection[row],
                referrers_datacenter__region: region_referrers_datacenter__region[row].clone(),
                referrers_docker_registry_instance__region: region_referrers_docker_registry_instance__region[row].clone(),
                referrers_pg_deployment__region: region_referrers_pg_deployment__region[row].clone(),
                referrers_ch_deployment__region: region_referrers_ch_deployment__region[row].clone(),
                referrers_ch_keeper_deployment__region: region_referrers_ch_keeper_deployment__region[row].clone(),
                referrers_nats_cluster__region: region_referrers_nats_cluster__region[row].clone(),
                referrers_backend_application_deployment__region: region_referrers_backend_application_deployment__region[row].clone(),
                referrers_frontend_application_deployment__region: region_referrers_frontend_application_deployment__region[row].clone(),
                referrers_minio_cluster__region: region_referrers_minio_cluster__region[row].clone(),
                referrers_monitoring_cluster__region: region_referrers_monitoring_cluster__region[row].clone(),
                referrers_grafana__region: region_referrers_grafana__region[row].clone(),
                referrers_loki_cluster__region: region_referrers_loki_cluster__region[row].clone(),
                referrers_tempo_cluster__region: region_referrers_tempo_cluster__region[row].clone(),
                referrers_blackbox_deployment__region: region_referrers_blackbox_deployment__region[row].clone(),
            });
        }

        let rust_compilation_environment_env_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_compilation_environment_rust_edition: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_compilation_environment_nixpkgs_environment: Vec<TableRowPointerNixpkgsEnvironment> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_compilation_environment_environment_kind: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_compilation_environment_children_rust_crate_version: Vec<Vec<TableRowPointerRustCrateVersion>> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_compilation_environment_referrers_backend_application__build_environment: Vec<Vec<TableRowPointerBackendApplication>> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_compilation_environment_referrers_frontend_application__build_environment: Vec<Vec<TableRowPointerFrontendApplication>> = ::bincode::deserialize_from(&mut cursor)?;

        let rust_compilation_environment_len = rust_compilation_environment_referrers_frontend_application__build_environment.len();

        assert_eq!(rust_compilation_environment_len, rust_compilation_environment_env_name.len());
        assert_eq!(rust_compilation_environment_len, rust_compilation_environment_rust_edition.len());
        assert_eq!(rust_compilation_environment_len, rust_compilation_environment_nixpkgs_environment.len());
        assert_eq!(rust_compilation_environment_len, rust_compilation_environment_environment_kind.len());
        assert_eq!(rust_compilation_environment_len, rust_compilation_environment_children_rust_crate_version.len());
        assert_eq!(rust_compilation_environment_len, rust_compilation_environment_referrers_backend_application__build_environment.len());

        let mut rows_rust_compilation_environment: Vec<TableRowRustCompilationEnvironment> = Vec::with_capacity(rust_compilation_environment_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..rust_compilation_environment_len {
            rows_rust_compilation_environment.push(TableRowRustCompilationEnvironment {
                env_name: rust_compilation_environment_env_name[row].clone(),
                rust_edition: rust_compilation_environment_rust_edition[row].clone(),
                nixpkgs_environment: rust_compilation_environment_nixpkgs_environment[row],
                environment_kind: rust_compilation_environment_environment_kind[row].clone(),
                children_rust_crate_version: rust_compilation_environment_children_rust_crate_version[row].clone(),
                referrers_backend_application__build_environment: rust_compilation_environment_referrers_backend_application__build_environment[row].clone(),
                referrers_frontend_application__build_environment: rust_compilation_environment_referrers_frontend_application__build_environment[row].clone(),
            });
        }

        let rust_crate_version_crate_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_crate_version_version: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_crate_version_features: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_crate_version_default_features: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let rust_crate_version_parent: Vec<TableRowPointerRustCompilationEnvironment> = ::bincode::deserialize_from(&mut cursor)?;

        let rust_crate_version_len = rust_crate_version_parent.len();

        assert_eq!(rust_crate_version_len, rust_crate_version_crate_name.len());
        assert_eq!(rust_crate_version_len, rust_crate_version_version.len());
        assert_eq!(rust_crate_version_len, rust_crate_version_features.len());
        assert_eq!(rust_crate_version_len, rust_crate_version_default_features.len());

        let mut rows_rust_crate_version: Vec<TableRowRustCrateVersion> = Vec::with_capacity(rust_crate_version_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..rust_crate_version_len {
            rows_rust_crate_version.push(TableRowRustCrateVersion {
                crate_name: rust_crate_version_crate_name[row].clone(),
                version: rust_crate_version_version[row].clone(),
                features: rust_crate_version_features[row].clone(),
                default_features: rust_crate_version_default_features[row],
                parent: rust_crate_version_parent[row],
            });
        }

        let server_hostname: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_dc: Vec<TableRowPointerDatacenter> = ::bincode::deserialize_from(&mut cursor)?;
        let server_ssh_interface: Vec<TableRowPointerNetworkInterface> = ::bincode::deserialize_from(&mut cursor)?;
        let server_root_disk: Vec<TableRowPointerServerDisk> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_consul_master: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_nomad_master: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_vault_instance: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_dns_master: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_dns_slave: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_ingress: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_vpn_gateway: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_coprocessor_gateway: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_is_router: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_public_ipv6_address: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_public_ipv6_address_prefix: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_nixpkgs_environment: Vec<TableRowPointerNixpkgsEnvironment> = ::bincode::deserialize_from(&mut cursor)?;
        let server_run_unassigned_workloads: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_children_server_label: Vec<Vec<TableRowPointerServerLabel>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_children_server_disk: Vec<Vec<TableRowPointerServerDisk>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_children_server_volume: Vec<Vec<TableRowPointerServerVolume>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_children_server_root_volume: Vec<Vec<TableRowPointerServerRootVolume>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_children_server_xfs_volume: Vec<Vec<TableRowPointerServerXfsVolume>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_children_network_interface: Vec<Vec<TableRowPointerNetworkInterface>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_children_server_zpool: Vec<Vec<TableRowPointerServerZpool>> = ::bincode::deserialize_from(&mut cursor)?;

        let server_len = server_children_server_zpool.len();

        assert_eq!(server_len, server_hostname.len());
        assert_eq!(server_len, server_dc.len());
        assert_eq!(server_len, server_ssh_interface.len());
        assert_eq!(server_len, server_root_disk.len());
        assert_eq!(server_len, server_is_consul_master.len());
        assert_eq!(server_len, server_is_nomad_master.len());
        assert_eq!(server_len, server_is_vault_instance.len());
        assert_eq!(server_len, server_is_dns_master.len());
        assert_eq!(server_len, server_is_dns_slave.len());
        assert_eq!(server_len, server_is_ingress.len());
        assert_eq!(server_len, server_is_vpn_gateway.len());
        assert_eq!(server_len, server_is_coprocessor_gateway.len());
        assert_eq!(server_len, server_is_router.len());
        assert_eq!(server_len, server_public_ipv6_address.len());
        assert_eq!(server_len, server_public_ipv6_address_prefix.len());
        assert_eq!(server_len, server_kind.len());
        assert_eq!(server_len, server_nixpkgs_environment.len());
        assert_eq!(server_len, server_run_unassigned_workloads.len());
        assert_eq!(server_len, server_children_server_label.len());
        assert_eq!(server_len, server_children_server_disk.len());
        assert_eq!(server_len, server_children_server_volume.len());
        assert_eq!(server_len, server_children_server_root_volume.len());
        assert_eq!(server_len, server_children_server_xfs_volume.len());
        assert_eq!(server_len, server_children_network_interface.len());

        let mut rows_server: Vec<TableRowServer> = Vec::with_capacity(server_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_len {
            rows_server.push(TableRowServer {
                hostname: server_hostname[row].clone(),
                dc: server_dc[row],
                ssh_interface: server_ssh_interface[row],
                root_disk: server_root_disk[row],
                is_consul_master: server_is_consul_master[row],
                is_nomad_master: server_is_nomad_master[row],
                is_vault_instance: server_is_vault_instance[row],
                is_dns_master: server_is_dns_master[row],
                is_dns_slave: server_is_dns_slave[row],
                is_ingress: server_is_ingress[row],
                is_vpn_gateway: server_is_vpn_gateway[row],
                is_coprocessor_gateway: server_is_coprocessor_gateway[row],
                is_router: server_is_router[row],
                public_ipv6_address: server_public_ipv6_address[row].clone(),
                public_ipv6_address_prefix: server_public_ipv6_address_prefix[row],
                kind: server_kind[row].clone(),
                nixpkgs_environment: server_nixpkgs_environment[row],
                run_unassigned_workloads: server_run_unassigned_workloads[row],
                children_server_label: server_children_server_label[row].clone(),
                children_server_disk: server_children_server_disk[row].clone(),
                children_server_volume: server_children_server_volume[row].clone(),
                children_server_root_volume: server_children_server_root_volume[row].clone(),
                children_server_xfs_volume: server_children_server_xfs_volume[row].clone(),
                children_network_interface: server_children_network_interface[row].clone(),
                children_server_zpool: server_children_server_zpool[row].clone(),
            });
        }

        let server_disk_disk_id: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_disk_kind: Vec<TableRowPointerDiskKind> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_xfs_format: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_extra_config: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_capacity_bytes: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_parent: Vec<TableRowPointerServer> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_referrers_server__root_disk: Vec<Vec<TableRowPointerServer>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_referrers_server_xfs_volume__xfs_disk: Vec<Vec<TableRowPointerServerXfsVolume>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_referrers_server_zpool_spare__disk_id: Vec<Vec<TableRowPointerServerZpoolSpare>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_referrers_server_zpool_cache__disk_id: Vec<Vec<TableRowPointerServerZpoolCache>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_referrers_server_zpool_log__disk_id: Vec<Vec<TableRowPointerServerZpoolLog>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_disk_referrers_server_zpool_vdev_disk__disk_id: Vec<Vec<TableRowPointerServerZpoolVdevDisk>> = ::bincode::deserialize_from(&mut cursor)?;

        let server_disk_len = server_disk_referrers_server_zpool_vdev_disk__disk_id.len();

        assert_eq!(server_disk_len, server_disk_disk_id.len());
        assert_eq!(server_disk_len, server_disk_disk_kind.len());
        assert_eq!(server_disk_len, server_disk_xfs_format.len());
        assert_eq!(server_disk_len, server_disk_extra_config.len());
        assert_eq!(server_disk_len, server_disk_capacity_bytes.len());
        assert_eq!(server_disk_len, server_disk_parent.len());
        assert_eq!(server_disk_len, server_disk_referrers_server__root_disk.len());
        assert_eq!(server_disk_len, server_disk_referrers_server_xfs_volume__xfs_disk.len());
        assert_eq!(server_disk_len, server_disk_referrers_server_zpool_spare__disk_id.len());
        assert_eq!(server_disk_len, server_disk_referrers_server_zpool_cache__disk_id.len());
        assert_eq!(server_disk_len, server_disk_referrers_server_zpool_log__disk_id.len());

        let mut rows_server_disk: Vec<TableRowServerDisk> = Vec::with_capacity(server_disk_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_disk_len {
            rows_server_disk.push(TableRowServerDisk {
                disk_id: server_disk_disk_id[row].clone(),
                disk_kind: server_disk_disk_kind[row],
                xfs_format: server_disk_xfs_format[row],
                extra_config: server_disk_extra_config[row].clone(),
                capacity_bytes: server_disk_capacity_bytes[row],
                parent: server_disk_parent[row],
                referrers_server__root_disk: server_disk_referrers_server__root_disk[row].clone(),
                referrers_server_xfs_volume__xfs_disk: server_disk_referrers_server_xfs_volume__xfs_disk[row].clone(),
                referrers_server_zpool_spare__disk_id: server_disk_referrers_server_zpool_spare__disk_id[row].clone(),
                referrers_server_zpool_cache__disk_id: server_disk_referrers_server_zpool_cache__disk_id[row].clone(),
                referrers_server_zpool_log__disk_id: server_disk_referrers_server_zpool_log__disk_id[row].clone(),
                referrers_server_zpool_vdev_disk__disk_id: server_disk_referrers_server_zpool_vdev_disk__disk_id[row].clone(),
            });
        }

        let server_kind_kind: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_cores: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_memory_bytes: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_bare_metal: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_non_eligible_reason: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_children_server_kind_attribute: Vec<Vec<TableRowPointerServerKindAttribute>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_referrers_datacenter__default_server_kind: Vec<Vec<TableRowPointerDatacenter>> = ::bincode::deserialize_from(&mut cursor)?;

        let server_kind_len = server_kind_referrers_datacenter__default_server_kind.len();

        assert_eq!(server_kind_len, server_kind_kind.len());
        assert_eq!(server_kind_len, server_kind_cores.len());
        assert_eq!(server_kind_len, server_kind_memory_bytes.len());
        assert_eq!(server_kind_len, server_kind_architecture.len());
        assert_eq!(server_kind_len, server_kind_bare_metal.len());
        assert_eq!(server_kind_len, server_kind_non_eligible_reason.len());
        assert_eq!(server_kind_len, server_kind_children_server_kind_attribute.len());

        let mut rows_server_kind: Vec<TableRowServerKind> = Vec::with_capacity(server_kind_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_kind_len {
            rows_server_kind.push(TableRowServerKind {
                kind: server_kind_kind[row].clone(),
                cores: server_kind_cores[row],
                memory_bytes: server_kind_memory_bytes[row],
                architecture: server_kind_architecture[row].clone(),
                bare_metal: server_kind_bare_metal[row],
                non_eligible_reason: server_kind_non_eligible_reason[row].clone(),
                children_server_kind_attribute: server_kind_children_server_kind_attribute[row].clone(),
                referrers_datacenter__default_server_kind: server_kind_referrers_datacenter__default_server_kind[row].clone(),
            });
        }

        let server_kind_attribute_key: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_attribute_value: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_kind_attribute_parent: Vec<TableRowPointerServerKind> = ::bincode::deserialize_from(&mut cursor)?;

        let server_kind_attribute_len = server_kind_attribute_parent.len();

        assert_eq!(server_kind_attribute_len, server_kind_attribute_key.len());
        assert_eq!(server_kind_attribute_len, server_kind_attribute_value.len());

        let mut rows_server_kind_attribute: Vec<TableRowServerKindAttribute> = Vec::with_capacity(server_kind_attribute_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_kind_attribute_len {
            rows_server_kind_attribute.push(TableRowServerKindAttribute {
                key: server_kind_attribute_key[row].clone(),
                value: server_kind_attribute_value[row].clone(),
                parent: server_kind_attribute_parent[row],
            });
        }

        let server_label_label_name: Vec<TableRowPointerValidServerLabels> = ::bincode::deserialize_from(&mut cursor)?;
        let server_label_label_value: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_label_parent: Vec<TableRowPointerServer> = ::bincode::deserialize_from(&mut cursor)?;

        let server_label_len = server_label_parent.len();

        assert_eq!(server_label_len, server_label_label_name.len());
        assert_eq!(server_label_len, server_label_label_value.len());

        let mut rows_server_label: Vec<TableRowServerLabel> = Vec::with_capacity(server_label_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_label_len {
            rows_server_label.push(TableRowServerLabel {
                label_name: server_label_label_name[row],
                label_value: server_label_label_value[row].clone(),
                parent: server_label_parent[row],
            });
        }

        let server_root_volume_volume_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_root_volume_intended_usage: Vec<TableRowPointerServerVolumeUsageContract> = ::bincode::deserialize_from(&mut cursor)?;
        let server_root_volume_mountpoint: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_root_volume_zfs_recordsize: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_root_volume_zfs_compression: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_root_volume_zfs_encryption: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_root_volume_parent: Vec<TableRowPointerServer> = ::bincode::deserialize_from(&mut cursor)?;

        let server_root_volume_len = server_root_volume_parent.len();

        assert_eq!(server_root_volume_len, server_root_volume_volume_name.len());
        assert_eq!(server_root_volume_len, server_root_volume_intended_usage.len());
        assert_eq!(server_root_volume_len, server_root_volume_mountpoint.len());
        assert_eq!(server_root_volume_len, server_root_volume_zfs_recordsize.len());
        assert_eq!(server_root_volume_len, server_root_volume_zfs_compression.len());
        assert_eq!(server_root_volume_len, server_root_volume_zfs_encryption.len());

        let mut rows_server_root_volume: Vec<TableRowServerRootVolume> = Vec::with_capacity(server_root_volume_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_root_volume_len {
            rows_server_root_volume.push(TableRowServerRootVolume {
                volume_name: server_root_volume_volume_name[row].clone(),
                intended_usage: server_root_volume_intended_usage[row],
                mountpoint: server_root_volume_mountpoint[row].clone(),
                zfs_recordsize: server_root_volume_zfs_recordsize[row].clone(),
                zfs_compression: server_root_volume_zfs_compression[row],
                zfs_encryption: server_root_volume_zfs_encryption[row],
                parent: server_root_volume_parent[row],
            });
        }

        let server_volume_volume_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_mountpoint: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_intended_usage: Vec<TableRowPointerServerVolumeUsageContract> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_source: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_parent: Vec<TableRowPointerServer> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_pg_deployment_instance__pg_server: Vec<Vec<TableRowPointerPgDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_ch_deployment_instance__ch_server: Vec<Vec<TableRowPointerChDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_ch_keeper_deployment_instance__keeper_server: Vec<Vec<TableRowPointerChKeeperDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_nats_deployment_instance__nats_server: Vec<Vec<TableRowPointerNatsDeploymentInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_minio_instance__instance_volume: Vec<Vec<TableRowPointerMinioInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_monitoring_instance__monitoring_server: Vec<Vec<TableRowPointerMonitoringInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_alertmanager_instance__alertmanager_server: Vec<Vec<TableRowPointerAlertmanagerInstance>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_referrers_blackbox_deployment_task_mount__server_volume: Vec<Vec<TableRowPointerBlackboxDeploymentTaskMount>> = ::bincode::deserialize_from(&mut cursor)?;

        let server_volume_len = server_volume_referrers_blackbox_deployment_task_mount__server_volume.len();

        assert_eq!(server_volume_len, server_volume_volume_name.len());
        assert_eq!(server_volume_len, server_volume_mountpoint.len());
        assert_eq!(server_volume_len, server_volume_intended_usage.len());
        assert_eq!(server_volume_len, server_volume_source.len());
        assert_eq!(server_volume_len, server_volume_parent.len());
        assert_eq!(server_volume_len, server_volume_referrers_pg_deployment_instance__pg_server.len());
        assert_eq!(server_volume_len, server_volume_referrers_ch_deployment_instance__ch_server.len());
        assert_eq!(server_volume_len, server_volume_referrers_ch_keeper_deployment_instance__keeper_server.len());
        assert_eq!(server_volume_len, server_volume_referrers_nats_deployment_instance__nats_server.len());
        assert_eq!(server_volume_len, server_volume_referrers_minio_instance__instance_volume.len());
        assert_eq!(server_volume_len, server_volume_referrers_monitoring_instance__monitoring_server.len());
        assert_eq!(server_volume_len, server_volume_referrers_alertmanager_instance__alertmanager_server.len());

        let mut rows_server_volume: Vec<TableRowServerVolume> = Vec::with_capacity(server_volume_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_volume_len {
            rows_server_volume.push(TableRowServerVolume {
                volume_name: server_volume_volume_name[row].clone(),
                mountpoint: server_volume_mountpoint[row].clone(),
                intended_usage: server_volume_intended_usage[row],
                source: server_volume_source[row].clone(),
                parent: server_volume_parent[row],
                referrers_pg_deployment_instance__pg_server: server_volume_referrers_pg_deployment_instance__pg_server[row].clone(),
                referrers_ch_deployment_instance__ch_server: server_volume_referrers_ch_deployment_instance__ch_server[row].clone(),
                referrers_ch_keeper_deployment_instance__keeper_server: server_volume_referrers_ch_keeper_deployment_instance__keeper_server[row].clone(),
                referrers_nats_deployment_instance__nats_server: server_volume_referrers_nats_deployment_instance__nats_server[row].clone(),
                referrers_minio_instance__instance_volume: server_volume_referrers_minio_instance__instance_volume[row].clone(),
                referrers_monitoring_instance__monitoring_server: server_volume_referrers_monitoring_instance__monitoring_server[row].clone(),
                referrers_alertmanager_instance__alertmanager_server: server_volume_referrers_alertmanager_instance__alertmanager_server[row].clone(),
                referrers_blackbox_deployment_task_mount__server_volume: server_volume_referrers_blackbox_deployment_task_mount__server_volume[row].clone(),
            });
        }

        let server_volume_usage_contract_usage_contract: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_usage_contract_referrers_server_volume__intended_usage: Vec<Vec<TableRowPointerServerVolume>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_usage_contract_referrers_server_root_volume__intended_usage: Vec<Vec<TableRowPointerServerRootVolume>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_usage_contract_referrers_server_xfs_volume__intended_usage: Vec<Vec<TableRowPointerServerXfsVolume>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_volume_usage_contract_referrers_server_zfs_dataset__intended_usage: Vec<Vec<TableRowPointerServerZfsDataset>> = ::bincode::deserialize_from(&mut cursor)?;

        let server_volume_usage_contract_len = server_volume_usage_contract_referrers_server_zfs_dataset__intended_usage.len();

        assert_eq!(server_volume_usage_contract_len, server_volume_usage_contract_usage_contract.len());
        assert_eq!(server_volume_usage_contract_len, server_volume_usage_contract_referrers_server_volume__intended_usage.len());
        assert_eq!(server_volume_usage_contract_len, server_volume_usage_contract_referrers_server_root_volume__intended_usage.len());
        assert_eq!(server_volume_usage_contract_len, server_volume_usage_contract_referrers_server_xfs_volume__intended_usage.len());

        let mut rows_server_volume_usage_contract: Vec<TableRowServerVolumeUsageContract> = Vec::with_capacity(server_volume_usage_contract_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_volume_usage_contract_len {
            rows_server_volume_usage_contract.push(TableRowServerVolumeUsageContract {
                usage_contract: server_volume_usage_contract_usage_contract[row].clone(),
                referrers_server_volume__intended_usage: server_volume_usage_contract_referrers_server_volume__intended_usage[row].clone(),
                referrers_server_root_volume__intended_usage: server_volume_usage_contract_referrers_server_root_volume__intended_usage[row].clone(),
                referrers_server_xfs_volume__intended_usage: server_volume_usage_contract_referrers_server_xfs_volume__intended_usage[row].clone(),
                referrers_server_zfs_dataset__intended_usage: server_volume_usage_contract_referrers_server_zfs_dataset__intended_usage[row].clone(),
            });
        }

        let server_xfs_volume_volume_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_xfs_volume_xfs_disk: Vec<TableRowPointerServerDisk> = ::bincode::deserialize_from(&mut cursor)?;
        let server_xfs_volume_intended_usage: Vec<TableRowPointerServerVolumeUsageContract> = ::bincode::deserialize_from(&mut cursor)?;
        let server_xfs_volume_parent: Vec<TableRowPointerServer> = ::bincode::deserialize_from(&mut cursor)?;

        let server_xfs_volume_len = server_xfs_volume_parent.len();

        assert_eq!(server_xfs_volume_len, server_xfs_volume_volume_name.len());
        assert_eq!(server_xfs_volume_len, server_xfs_volume_xfs_disk.len());
        assert_eq!(server_xfs_volume_len, server_xfs_volume_intended_usage.len());

        let mut rows_server_xfs_volume: Vec<TableRowServerXfsVolume> = Vec::with_capacity(server_xfs_volume_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_xfs_volume_len {
            rows_server_xfs_volume.push(TableRowServerXfsVolume {
                volume_name: server_xfs_volume_volume_name[row].clone(),
                xfs_disk: server_xfs_volume_xfs_disk[row],
                intended_usage: server_xfs_volume_intended_usage[row],
                parent: server_xfs_volume_parent[row],
            });
        }

        let server_zfs_dataset_dataset_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zfs_dataset_intended_usage: Vec<TableRowPointerServerVolumeUsageContract> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zfs_dataset_zfs_recordsize: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zfs_dataset_zfs_compression: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zfs_dataset_zfs_encryption: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zfs_dataset_parent: Vec<TableRowPointerServerZpool> = ::bincode::deserialize_from(&mut cursor)?;

        let server_zfs_dataset_len = server_zfs_dataset_parent.len();

        assert_eq!(server_zfs_dataset_len, server_zfs_dataset_dataset_name.len());
        assert_eq!(server_zfs_dataset_len, server_zfs_dataset_intended_usage.len());
        assert_eq!(server_zfs_dataset_len, server_zfs_dataset_zfs_recordsize.len());
        assert_eq!(server_zfs_dataset_len, server_zfs_dataset_zfs_compression.len());
        assert_eq!(server_zfs_dataset_len, server_zfs_dataset_zfs_encryption.len());

        let mut rows_server_zfs_dataset: Vec<TableRowServerZfsDataset> = Vec::with_capacity(server_zfs_dataset_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_zfs_dataset_len {
            rows_server_zfs_dataset.push(TableRowServerZfsDataset {
                dataset_name: server_zfs_dataset_dataset_name[row].clone(),
                intended_usage: server_zfs_dataset_intended_usage[row],
                zfs_recordsize: server_zfs_dataset_zfs_recordsize[row].clone(),
                zfs_compression: server_zfs_dataset_zfs_compression[row],
                zfs_encryption: server_zfs_dataset_zfs_encryption[row],
                parent: server_zfs_dataset_parent[row],
            });
        }

        let server_zpool_zpool_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_is_redundant: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_parent: Vec<TableRowPointerServer> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_children_server_zpool_vdev: Vec<Vec<TableRowPointerServerZpoolVdev>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_children_server_zpool_spare: Vec<Vec<TableRowPointerServerZpoolSpare>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_children_server_zpool_cache: Vec<Vec<TableRowPointerServerZpoolCache>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_children_server_zpool_log: Vec<Vec<TableRowPointerServerZpoolLog>> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_children_server_zfs_dataset: Vec<Vec<TableRowPointerServerZfsDataset>> = ::bincode::deserialize_from(&mut cursor)?;

        let server_zpool_len = server_zpool_children_server_zfs_dataset.len();

        assert_eq!(server_zpool_len, server_zpool_zpool_name.len());
        assert_eq!(server_zpool_len, server_zpool_is_redundant.len());
        assert_eq!(server_zpool_len, server_zpool_parent.len());
        assert_eq!(server_zpool_len, server_zpool_children_server_zpool_vdev.len());
        assert_eq!(server_zpool_len, server_zpool_children_server_zpool_spare.len());
        assert_eq!(server_zpool_len, server_zpool_children_server_zpool_cache.len());
        assert_eq!(server_zpool_len, server_zpool_children_server_zpool_log.len());

        let mut rows_server_zpool: Vec<TableRowServerZpool> = Vec::with_capacity(server_zpool_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_zpool_len {
            rows_server_zpool.push(TableRowServerZpool {
                zpool_name: server_zpool_zpool_name[row].clone(),
                is_redundant: server_zpool_is_redundant[row],
                parent: server_zpool_parent[row],
                children_server_zpool_vdev: server_zpool_children_server_zpool_vdev[row].clone(),
                children_server_zpool_spare: server_zpool_children_server_zpool_spare[row].clone(),
                children_server_zpool_cache: server_zpool_children_server_zpool_cache[row].clone(),
                children_server_zpool_log: server_zpool_children_server_zpool_log[row].clone(),
                children_server_zfs_dataset: server_zpool_children_server_zfs_dataset[row].clone(),
            });
        }

        let server_zpool_cache_disk_id: Vec<TableRowPointerServerDisk> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_cache_parent: Vec<TableRowPointerServerZpool> = ::bincode::deserialize_from(&mut cursor)?;

        let server_zpool_cache_len = server_zpool_cache_parent.len();

        assert_eq!(server_zpool_cache_len, server_zpool_cache_disk_id.len());

        let mut rows_server_zpool_cache: Vec<TableRowServerZpoolCache> = Vec::with_capacity(server_zpool_cache_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_zpool_cache_len {
            rows_server_zpool_cache.push(TableRowServerZpoolCache {
                disk_id: server_zpool_cache_disk_id[row],
                parent: server_zpool_cache_parent[row],
            });
        }

        let server_zpool_log_disk_id: Vec<TableRowPointerServerDisk> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_log_parent: Vec<TableRowPointerServerZpool> = ::bincode::deserialize_from(&mut cursor)?;

        let server_zpool_log_len = server_zpool_log_parent.len();

        assert_eq!(server_zpool_log_len, server_zpool_log_disk_id.len());

        let mut rows_server_zpool_log: Vec<TableRowServerZpoolLog> = Vec::with_capacity(server_zpool_log_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_zpool_log_len {
            rows_server_zpool_log.push(TableRowServerZpoolLog {
                disk_id: server_zpool_log_disk_id[row],
                parent: server_zpool_log_parent[row],
            });
        }

        let server_zpool_spare_disk_id: Vec<TableRowPointerServerDisk> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_spare_parent: Vec<TableRowPointerServerZpool> = ::bincode::deserialize_from(&mut cursor)?;

        let server_zpool_spare_len = server_zpool_spare_parent.len();

        assert_eq!(server_zpool_spare_len, server_zpool_spare_disk_id.len());

        let mut rows_server_zpool_spare: Vec<TableRowServerZpoolSpare> = Vec::with_capacity(server_zpool_spare_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_zpool_spare_len {
            rows_server_zpool_spare.push(TableRowServerZpoolSpare {
                disk_id: server_zpool_spare_disk_id[row],
                parent: server_zpool_spare_parent[row],
            });
        }

        let server_zpool_vdev_vdev_number: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_vdev_vdev_type: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_vdev_parent: Vec<TableRowPointerServerZpool> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_vdev_children_server_zpool_vdev_disk: Vec<Vec<TableRowPointerServerZpoolVdevDisk>> = ::bincode::deserialize_from(&mut cursor)?;

        let server_zpool_vdev_len = server_zpool_vdev_children_server_zpool_vdev_disk.len();

        assert_eq!(server_zpool_vdev_len, server_zpool_vdev_vdev_number.len());
        assert_eq!(server_zpool_vdev_len, server_zpool_vdev_vdev_type.len());
        assert_eq!(server_zpool_vdev_len, server_zpool_vdev_parent.len());

        let mut rows_server_zpool_vdev: Vec<TableRowServerZpoolVdev> = Vec::with_capacity(server_zpool_vdev_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_zpool_vdev_len {
            rows_server_zpool_vdev.push(TableRowServerZpoolVdev {
                vdev_number: server_zpool_vdev_vdev_number[row],
                vdev_type: server_zpool_vdev_vdev_type[row].clone(),
                parent: server_zpool_vdev_parent[row],
                children_server_zpool_vdev_disk: server_zpool_vdev_children_server_zpool_vdev_disk[row].clone(),
            });
        }

        let server_zpool_vdev_disk_disk_id: Vec<TableRowPointerServerDisk> = ::bincode::deserialize_from(&mut cursor)?;
        let server_zpool_vdev_disk_parent: Vec<TableRowPointerServerZpoolVdev> = ::bincode::deserialize_from(&mut cursor)?;

        let server_zpool_vdev_disk_len = server_zpool_vdev_disk_parent.len();

        assert_eq!(server_zpool_vdev_disk_len, server_zpool_vdev_disk_disk_id.len());

        let mut rows_server_zpool_vdev_disk: Vec<TableRowServerZpoolVdevDisk> = Vec::with_capacity(server_zpool_vdev_disk_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..server_zpool_vdev_disk_len {
            rows_server_zpool_vdev_disk.push(TableRowServerZpoolVdevDisk {
                disk_id: server_zpool_vdev_disk_disk_id[row],
                parent: server_zpool_vdev_disk_parent[row],
            });
        }

        let subnet_router_floating_ip_ip_address: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;

        let subnet_router_floating_ip_len = subnet_router_floating_ip_ip_address.len();


        let mut rows_subnet_router_floating_ip: Vec<TableRowSubnetRouterFloatingIp> = Vec::with_capacity(subnet_router_floating_ip_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..subnet_router_floating_ip_len {
            rows_subnet_router_floating_ip.push(TableRowSubnetRouterFloatingIp {
                ip_address: subnet_router_floating_ip_ip_address[row].clone(),
            });
        }

        let telegram_bot_bot_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let telegram_bot_bot_token: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let telegram_bot_referrers_monitoring_cluster_alert_group__telegram_bot: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>> = ::bincode::deserialize_from(&mut cursor)?;

        let telegram_bot_len = telegram_bot_referrers_monitoring_cluster_alert_group__telegram_bot.len();

        assert_eq!(telegram_bot_len, telegram_bot_bot_name.len());
        assert_eq!(telegram_bot_len, telegram_bot_bot_token.len());

        let mut rows_telegram_bot: Vec<TableRowTelegramBot> = Vec::with_capacity(telegram_bot_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..telegram_bot_len {
            rows_telegram_bot.push(TableRowTelegramBot {
                bot_name: telegram_bot_bot_name[row].clone(),
                bot_token: telegram_bot_bot_token[row].clone(),
                referrers_monitoring_cluster_alert_group__telegram_bot: telegram_bot_referrers_monitoring_cluster_alert_group__telegram_bot[row].clone(),
            });
        }

        let telegram_channel_channel_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let telegram_channel_channel_id: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let telegram_channel_referrers_monitoring_cluster_alert_group__telegram_channel: Vec<Vec<TableRowPointerMonitoringClusterAlertGroup>> = ::bincode::deserialize_from(&mut cursor)?;

        let telegram_channel_len = telegram_channel_referrers_monitoring_cluster_alert_group__telegram_channel.len();

        assert_eq!(telegram_channel_len, telegram_channel_channel_name.len());
        assert_eq!(telegram_channel_len, telegram_channel_channel_id.len());

        let mut rows_telegram_channel: Vec<TableRowTelegramChannel> = Vec::with_capacity(telegram_channel_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..telegram_channel_len {
            rows_telegram_channel.push(TableRowTelegramChannel {
                channel_name: telegram_channel_channel_name[row].clone(),
                channel_id: telegram_channel_channel_id[row],
                referrers_monitoring_cluster_alert_group__telegram_channel: telegram_channel_referrers_monitoring_cluster_alert_group__telegram_channel[row].clone(),
            });
        }

        let tempo_cluster_cluster_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_namespace: Vec<TableRowPointerNomadNamespace> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_region: Vec<TableRowPointerRegion> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_workload_architecture: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_docker_image: Vec<TableRowPointerDockerImagePin> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_is_region_default: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_loki_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_monitoring_cluster: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_storage_bucket: Vec<TableRowPointerMinioBucket> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_grpc_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_p2p_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_otlp_http_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_otlp_grpc_port: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_tempo_instances: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_placement: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_trace_retention_days: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let tempo_cluster_memory_mb: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;

        let tempo_cluster_len = tempo_cluster_memory_mb.len();

        assert_eq!(tempo_cluster_len, tempo_cluster_cluster_name.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_namespace.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_region.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_workload_architecture.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_docker_image.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_is_region_default.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_loki_cluster.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_monitoring_cluster.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_storage_bucket.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_http_port.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_grpc_port.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_p2p_port.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_otlp_http_port.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_otlp_grpc_port.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_tempo_instances.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_placement.len());
        assert_eq!(tempo_cluster_len, tempo_cluster_trace_retention_days.len());

        let mut rows_tempo_cluster: Vec<TableRowTempoCluster> = Vec::with_capacity(tempo_cluster_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..tempo_cluster_len {
            rows_tempo_cluster.push(TableRowTempoCluster {
                cluster_name: tempo_cluster_cluster_name[row].clone(),
                namespace: tempo_cluster_namespace[row],
                region: tempo_cluster_region[row],
                workload_architecture: tempo_cluster_workload_architecture[row].clone(),
                docker_image: tempo_cluster_docker_image[row],
                is_region_default: tempo_cluster_is_region_default[row],
                loki_cluster: tempo_cluster_loki_cluster[row].clone(),
                monitoring_cluster: tempo_cluster_monitoring_cluster[row].clone(),
                storage_bucket: tempo_cluster_storage_bucket[row],
                http_port: tempo_cluster_http_port[row],
                grpc_port: tempo_cluster_grpc_port[row],
                p2p_port: tempo_cluster_p2p_port[row],
                otlp_http_port: tempo_cluster_otlp_http_port[row],
                otlp_grpc_port: tempo_cluster_otlp_grpc_port[row],
                tempo_instances: tempo_cluster_tempo_instances[row],
                placement: tempo_cluster_placement[row].clone(),
                trace_retention_days: tempo_cluster_trace_retention_days[row],
                memory_mb: tempo_cluster_memory_mb[row],
            });
        }

        let tld_domain: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let tld_expose_admin: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let tld_automatic_certificates: Vec<bool> = ::bincode::deserialize_from(&mut cursor)?;
        let tld_referrers_region__tld: Vec<Vec<TableRowPointerRegion>> = ::bincode::deserialize_from(&mut cursor)?;
        let tld_referrers_backend_application_deployment_ingress__tld: Vec<Vec<TableRowPointerBackendApplicationDeploymentIngress>> = ::bincode::deserialize_from(&mut cursor)?;
        let tld_referrers_frontend_application_deployment_ingress__tld: Vec<Vec<TableRowPointerFrontendApplicationDeploymentIngress>> = ::bincode::deserialize_from(&mut cursor)?;

        let tld_len = tld_referrers_frontend_application_deployment_ingress__tld.len();

        assert_eq!(tld_len, tld_domain.len());
        assert_eq!(tld_len, tld_expose_admin.len());
        assert_eq!(tld_len, tld_automatic_certificates.len());
        assert_eq!(tld_len, tld_referrers_region__tld.len());
        assert_eq!(tld_len, tld_referrers_backend_application_deployment_ingress__tld.len());

        let mut rows_tld: Vec<TableRowTld> = Vec::with_capacity(tld_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..tld_len {
            rows_tld.push(TableRowTld {
                domain: tld_domain[row].clone(),
                expose_admin: tld_expose_admin[row],
                automatic_certificates: tld_automatic_certificates[row],
                referrers_region__tld: tld_referrers_region__tld[row].clone(),
                referrers_backend_application_deployment_ingress__tld: tld_referrers_backend_application_deployment_ingress__tld[row].clone(),
                referrers_frontend_application_deployment_ingress__tld: tld_referrers_frontend_application_deployment_ingress__tld[row].clone(),
            });
        }

        let unique_application_names_application_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let unique_application_names_source: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;

        let unique_application_names_len = unique_application_names_source.len();

        assert_eq!(unique_application_names_len, unique_application_names_application_name.len());

        let mut rows_unique_application_names: Vec<TableRowUniqueApplicationNames> = Vec::with_capacity(unique_application_names_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..unique_application_names_len {
            rows_unique_application_names.push(TableRowUniqueApplicationNames {
                application_name: unique_application_names_application_name[row].clone(),
                source: unique_application_names_source[row].clone(),
            });
        }

        let unique_deployment_names_deployment_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let unique_deployment_names_source: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;

        let unique_deployment_names_len = unique_deployment_names_source.len();

        assert_eq!(unique_deployment_names_len, unique_deployment_names_deployment_name.len());

        let mut rows_unique_deployment_names: Vec<TableRowUniqueDeploymentNames> = Vec::with_capacity(unique_deployment_names_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..unique_deployment_names_len {
            rows_unique_deployment_names.push(TableRowUniqueDeploymentNames {
                deployment_name: unique_deployment_names_deployment_name[row].clone(),
                source: unique_deployment_names_source[row].clone(),
            });
        }

        let valid_server_labels_label_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let valid_server_labels_referrers_server_label__label_name: Vec<Vec<TableRowPointerServerLabel>> = ::bincode::deserialize_from(&mut cursor)?;

        let valid_server_labels_len = valid_server_labels_referrers_server_label__label_name.len();

        assert_eq!(valid_server_labels_len, valid_server_labels_label_name.len());

        let mut rows_valid_server_labels: Vec<TableRowValidServerLabels> = Vec::with_capacity(valid_server_labels_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..valid_server_labels_len {
            rows_valid_server_labels.push(TableRowValidServerLabels {
                label_name: valid_server_labels_label_name[row].clone(),
                referrers_server_label__label_name: valid_server_labels_referrers_server_label__label_name[row].clone(),
            });
        }

        let versioned_type_type_name: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_children_versioned_type_snapshot: Vec<Vec<TableRowPointerVersionedTypeSnapshot>> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_children_versioned_type_migration: Vec<Vec<TableRowPointerVersionedTypeMigration>> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_referrers_nats_jetstream_stream__stream_type: Vec<Vec<TableRowPointerNatsJetstreamStream>> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_referrers_backend_application_nats_stream__stream_type: Vec<Vec<TableRowPointerBackendApplicationNatsStream>> = ::bincode::deserialize_from(&mut cursor)?;

        let versioned_type_len = versioned_type_referrers_backend_application_nats_stream__stream_type.len();

        assert_eq!(versioned_type_len, versioned_type_type_name.len());
        assert_eq!(versioned_type_len, versioned_type_children_versioned_type_snapshot.len());
        assert_eq!(versioned_type_len, versioned_type_children_versioned_type_migration.len());
        assert_eq!(versioned_type_len, versioned_type_referrers_nats_jetstream_stream__stream_type.len());

        let mut rows_versioned_type: Vec<TableRowVersionedType> = Vec::with_capacity(versioned_type_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..versioned_type_len {
            rows_versioned_type.push(TableRowVersionedType {
                type_name: versioned_type_type_name[row].clone(),
                children_versioned_type_snapshot: versioned_type_children_versioned_type_snapshot[row].clone(),
                children_versioned_type_migration: versioned_type_children_versioned_type_migration[row].clone(),
                referrers_nats_jetstream_stream__stream_type: versioned_type_referrers_nats_jetstream_stream__stream_type[row].clone(),
                referrers_backend_application_nats_stream__stream_type: versioned_type_referrers_backend_application_nats_stream__stream_type[row].clone(),
            });
        }

        let versioned_type_migration_version: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_migration_migration_source: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_migration_parent: Vec<TableRowPointerVersionedType> = ::bincode::deserialize_from(&mut cursor)?;

        let versioned_type_migration_len = versioned_type_migration_parent.len();

        assert_eq!(versioned_type_migration_len, versioned_type_migration_version.len());
        assert_eq!(versioned_type_migration_len, versioned_type_migration_migration_source.len());

        let mut rows_versioned_type_migration: Vec<TableRowVersionedTypeMigration> = Vec::with_capacity(versioned_type_migration_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..versioned_type_migration_len {
            rows_versioned_type_migration.push(TableRowVersionedTypeMigration {
                version: versioned_type_migration_version[row],
                migration_source: versioned_type_migration_migration_source[row].clone(),
                parent: versioned_type_migration_parent[row],
            });
        }

        let versioned_type_snapshot_version: Vec<i64> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_snapshot_snapshot_source: Vec<::std::string::String> = ::bincode::deserialize_from(&mut cursor)?;
        let versioned_type_snapshot_parent: Vec<TableRowPointerVersionedType> = ::bincode::deserialize_from(&mut cursor)?;

        let versioned_type_snapshot_len = versioned_type_snapshot_parent.len();

        assert_eq!(versioned_type_snapshot_len, versioned_type_snapshot_version.len());
        assert_eq!(versioned_type_snapshot_len, versioned_type_snapshot_snapshot_source.len());

        let mut rows_versioned_type_snapshot: Vec<TableRowVersionedTypeSnapshot> = Vec::with_capacity(versioned_type_snapshot_len);
        #[allow(clippy::needless_range_loop)]
        for row in 0..versioned_type_snapshot_len {
            rows_versioned_type_snapshot.push(TableRowVersionedTypeSnapshot {
                version: versioned_type_snapshot_version[row],
                snapshot_source: versioned_type_snapshot_snapshot_source[row].clone(),
                parent: versioned_type_snapshot_parent[row],
            });
        }


        assert_eq!(cursor.position() as usize, input.len());

        Ok(Database {
            admin_ssh_keys: TableDefinitionAdminSshKeys {
                rows: rows_admin_ssh_keys,
                c_contents: admin_ssh_keys_contents,
            },
            alert: TableDefinitionAlert {
                rows: rows_alert,
                c_alert_name: alert_alert_name,
                c_expr: alert_expr,
                c_description: alert_description,
                c_for_time: alert_for_time,
                c_severity: alert_severity,
                c_parent: alert_parent,
                c_children_alert_trigger_test: alert_children_alert_trigger_test,
            },
            alert_group: TableDefinitionAlertGroup {
                rows: rows_alert_group,
                c_alert_group_name: alert_group_alert_group_name,
                c_children_alert: alert_group_children_alert,
                c_referrers_monitoring_cluster_alert_group__alert_group_name: alert_group_referrers_monitoring_cluster_alert_group__alert_group_name,
            },
            alert_trigger_test: TableDefinitionAlertTriggerTest {
                rows: rows_alert_trigger_test,
                c_expected_message: alert_trigger_test_expected_message,
                c_expected_labels: alert_trigger_test_expected_labels,
                c_eval_time: alert_trigger_test_eval_time,
                c_interval: alert_trigger_test_interval,
                c_input_series: alert_trigger_test_input_series,
                c_parent: alert_trigger_test_parent,
            },
            alertmanager_instance: TableDefinitionAlertmanagerInstance {
                rows: rows_alertmanager_instance,
                c_instance_id: alertmanager_instance_instance_id,
                c_alertmanager_server: alertmanager_instance_alertmanager_server,
                c_parent: alertmanager_instance_parent,
            },
            backend_application: TableDefinitionBackendApplication {
                rows: rows_backend_application,
                c_application_name: backend_application_application_name,
                c_build_environment: backend_application_build_environment,
                c_children_backend_application_background_job: backend_application_children_backend_application_background_job,
                c_children_backend_application_config: backend_application_children_backend_application_config,
                c_children_backend_application_s3_bucket: backend_application_children_backend_application_s3_bucket,
                c_children_backend_application_pg_shard: backend_application_children_backend_application_pg_shard,
                c_children_backend_application_ch_shard: backend_application_children_backend_application_ch_shard,
                c_children_backend_application_nats_stream: backend_application_children_backend_application_nats_stream,
                c_children_backend_http_endpoint: backend_application_children_backend_http_endpoint,
                c_referrers_backend_application_deployment__application_name: backend_application_referrers_backend_application_deployment__application_name,
            },
            backend_application_background_job: TableDefinitionBackendApplicationBackgroundJob {
                rows: rows_backend_application_background_job,
                c_job_name: backend_application_background_job_job_name,
                c_parent: backend_application_background_job_parent,
            },
            backend_application_ch_shard: TableDefinitionBackendApplicationChShard {
                rows: rows_backend_application_ch_shard,
                c_shard_name: backend_application_ch_shard_shard_name,
                c_ch_schema: backend_application_ch_shard_ch_schema,
                c_used_queries: backend_application_ch_shard_used_queries,
                c_used_inserters: backend_application_ch_shard_used_inserters,
                c_used_mutators: backend_application_ch_shard_used_mutators,
                c_parent: backend_application_ch_shard_parent,
            },
            backend_application_config: TableDefinitionBackendApplicationConfig {
                rows: rows_backend_application_config,
                c_config_name: backend_application_config_config_name,
                c_config_type: backend_application_config_config_type,
                c_default_value: backend_application_config_default_value,
                c_min_value: backend_application_config_min_value,
                c_max_value: backend_application_config_max_value,
                c_regex_check: backend_application_config_regex_check,
                c_parent: backend_application_config_parent,
            },
            backend_application_deployment: TableDefinitionBackendApplicationDeployment {
                rows: rows_backend_application_deployment,
                c_deployment_name: backend_application_deployment_deployment_name,
                c_namespace: backend_application_deployment_namespace,
                c_application_name: backend_application_deployment_application_name,
                c_workload_architecture: backend_application_deployment_workload_architecture,
                c_count: backend_application_deployment_count,
                c_placement: backend_application_deployment_placement,
                c_pg_shard_wiring: backend_application_deployment_pg_shard_wiring,
                c_ch_shard_wiring: backend_application_deployment_ch_shard_wiring,
                c_nats_stream_wiring: backend_application_deployment_nats_stream_wiring,
                c_s3_bucket_wiring: backend_application_deployment_s3_bucket_wiring,
                c_config: backend_application_deployment_config,
                c_http_port: backend_application_deployment_http_port,
                c_memory_mb: backend_application_deployment_memory_mb,
                c_region: backend_application_deployment_region,
                c_loki_cluster: backend_application_deployment_loki_cluster,
                c_monitoring_cluster: backend_application_deployment_monitoring_cluster,
                c_tracing_cluster: backend_application_deployment_tracing_cluster,
                c_referrers_backend_application_deployment_ingress__deployment: backend_application_deployment_referrers_backend_application_deployment_ingress__deployment,
            },
            backend_application_deployment_ingress: TableDefinitionBackendApplicationDeploymentIngress {
                rows: rows_backend_application_deployment_ingress,
                c_deployment: backend_application_deployment_ingress_deployment,
                c_mountpoint: backend_application_deployment_ingress_mountpoint,
                c_subdomain: backend_application_deployment_ingress_subdomain,
                c_tld: backend_application_deployment_ingress_tld,
                c_endpoint_list: backend_application_deployment_ingress_endpoint_list,
            },
            backend_application_nats_stream: TableDefinitionBackendApplicationNatsStream {
                rows: rows_backend_application_nats_stream,
                c_stream_name: backend_application_nats_stream_stream_name,
                c_stream_type: backend_application_nats_stream_stream_type,
                c_enable_consumer: backend_application_nats_stream_enable_consumer,
                c_enable_producer: backend_application_nats_stream_enable_producer,
                c_is_batch_consumer: backend_application_nats_stream_is_batch_consumer,
                c_enable_subjects: backend_application_nats_stream_enable_subjects,
                c_parent: backend_application_nats_stream_parent,
            },
            backend_application_pg_shard: TableDefinitionBackendApplicationPgShard {
                rows: rows_backend_application_pg_shard,
                c_shard_name: backend_application_pg_shard_shard_name,
                c_pg_schema: backend_application_pg_shard_pg_schema,
                c_used_queries: backend_application_pg_shard_used_queries,
                c_used_mutators: backend_application_pg_shard_used_mutators,
                c_used_transactions: backend_application_pg_shard_used_transactions,
                c_parent: backend_application_pg_shard_parent,
            },
            backend_application_s3_bucket: TableDefinitionBackendApplicationS3Bucket {
                rows: rows_backend_application_s3_bucket,
                c_bucket_name: backend_application_s3_bucket_bucket_name,
                c_parent: backend_application_s3_bucket_parent,
            },
            backend_http_endpoint: TableDefinitionBackendHttpEndpoint {
                rows: rows_backend_http_endpoint,
                c_http_endpoint_name: backend_http_endpoint_http_endpoint_name,
                c_path: backend_http_endpoint_path,
                c_http_method: backend_http_endpoint_http_method,
                c_input_body_type: backend_http_endpoint_input_body_type,
                c_output_body_type: backend_http_endpoint_output_body_type,
                c_data_type: backend_http_endpoint_data_type,
                c_max_input_body_size_bytes: backend_http_endpoint_max_input_body_size_bytes,
                c_needs_headers: backend_http_endpoint_needs_headers,
                c_receive_body_as_stream: backend_http_endpoint_receive_body_as_stream,
                c_parent: backend_http_endpoint_parent,
                c_referrers_frontend_application_used_endpoint__backend_endpoint: backend_http_endpoint_referrers_frontend_application_used_endpoint__backend_endpoint,
                c_referrers_frontend_application_external_link__backend_endpoint: backend_http_endpoint_referrers_frontend_application_external_link__backend_endpoint,
            },
            blackbox_deployment: TableDefinitionBlackboxDeployment {
                rows: rows_blackbox_deployment,
                c_deployment_name: blackbox_deployment_deployment_name,
                c_namespace: blackbox_deployment_namespace,
                c_region: blackbox_deployment_region,
                c_loki_cluster: blackbox_deployment_loki_cluster,
                c_monitoring_cluster: blackbox_deployment_monitoring_cluster,
                c_children_blackbox_deployment_group: blackbox_deployment_children_blackbox_deployment_group,
                c_children_blackbox_deployment_service_registration: blackbox_deployment_children_blackbox_deployment_service_registration,
            },
            blackbox_deployment_group: TableDefinitionBlackboxDeploymentGroup {
                rows: rows_blackbox_deployment_group,
                c_group_name: blackbox_deployment_group_group_name,
                c_count: blackbox_deployment_group_count,
                c_workload_architecture: blackbox_deployment_group_workload_architecture,
                c_placement: blackbox_deployment_group_placement,
                c_parent: blackbox_deployment_group_parent,
                c_children_blackbox_deployment_port: blackbox_deployment_group_children_blackbox_deployment_port,
                c_children_blackbox_deployment_task: blackbox_deployment_group_children_blackbox_deployment_task,
                c_children_blackbox_deployment_service_instance: blackbox_deployment_group_children_blackbox_deployment_service_instance,
            },
            blackbox_deployment_local_file: TableDefinitionBlackboxDeploymentLocalFile {
                rows: rows_blackbox_deployment_local_file,
                c_local_file_name: blackbox_deployment_local_file_local_file_name,
                c_local_file_contents: blackbox_deployment_local_file_local_file_contents,
                c_mode: blackbox_deployment_local_file_mode,
                c_parent: blackbox_deployment_local_file_parent,
            },
            blackbox_deployment_port: TableDefinitionBlackboxDeploymentPort {
                rows: rows_blackbox_deployment_port,
                c_port: blackbox_deployment_port_port,
                c_port_description: blackbox_deployment_port_port_description,
                c_protocol: blackbox_deployment_port_protocol,
                c_parent: blackbox_deployment_port_parent,
                c_referrers_blackbox_deployment_service_instance__port: blackbox_deployment_port_referrers_blackbox_deployment_service_instance__port,
            },
            blackbox_deployment_service_instance: TableDefinitionBlackboxDeploymentServiceInstance {
                rows: rows_blackbox_deployment_service_instance,
                c_service_registration: blackbox_deployment_service_instance_service_registration,
                c_port: blackbox_deployment_service_instance_port,
                c_parent: blackbox_deployment_service_instance_parent,
            },
            blackbox_deployment_service_registration: TableDefinitionBlackboxDeploymentServiceRegistration {
                rows: rows_blackbox_deployment_service_registration,
                c_service_name: blackbox_deployment_service_registration_service_name,
                c_scrape_prometheus_metrics: blackbox_deployment_service_registration_scrape_prometheus_metrics,
                c_prometheus_metrics_path: blackbox_deployment_service_registration_prometheus_metrics_path,
                c_min_instances: blackbox_deployment_service_registration_min_instances,
                c_parent: blackbox_deployment_service_registration_parent,
                c_referrers_blackbox_deployment_service_instance__service_registration: blackbox_deployment_service_registration_referrers_blackbox_deployment_service_instance__service_registration,
            },
            blackbox_deployment_task: TableDefinitionBlackboxDeploymentTask {
                rows: rows_blackbox_deployment_task,
                c_task_name: blackbox_deployment_task_task_name,
                c_docker_image: blackbox_deployment_task_docker_image,
                c_docker_image_set: blackbox_deployment_task_docker_image_set,
                c_memory_mb: blackbox_deployment_task_memory_mb,
                c_memory_oversubscription_mb: blackbox_deployment_task_memory_oversubscription_mb,
                c_entrypoint: blackbox_deployment_task_entrypoint,
                c_args: blackbox_deployment_task_args,
                c_parent: blackbox_deployment_task_parent,
                c_children_blackbox_deployment_task_mount: blackbox_deployment_task_children_blackbox_deployment_task_mount,
                c_children_blackbox_deployment_vault_secret: blackbox_deployment_task_children_blackbox_deployment_vault_secret,
                c_children_blackbox_deployment_local_file: blackbox_deployment_task_children_blackbox_deployment_local_file,
            },
            blackbox_deployment_task_mount: TableDefinitionBlackboxDeploymentTaskMount {
                rows: rows_blackbox_deployment_task_mount,
                c_target_path: blackbox_deployment_task_mount_target_path,
                c_server_volume: blackbox_deployment_task_mount_server_volume,
                c_parent: blackbox_deployment_task_mount_parent,
            },
            blackbox_deployment_vault_secret: TableDefinitionBlackboxDeploymentVaultSecret {
                rows: rows_blackbox_deployment_vault_secret,
                c_secret_name: blackbox_deployment_vault_secret_secret_name,
                c_target_file_name: blackbox_deployment_vault_secret_target_file_name,
                c_target_env_var_name: blackbox_deployment_vault_secret_target_env_var_name,
                c_parent: blackbox_deployment_vault_secret_parent,
            },
            ch_deployment: TableDefinitionChDeployment {
                rows: rows_ch_deployment,
                c_deployment_name: ch_deployment_deployment_name,
                c_namespace: ch_deployment_namespace,
                c_region: ch_deployment_region,
                c_loki_cluster: ch_deployment_loki_cluster,
                c_monitoring_cluster: ch_deployment_monitoring_cluster,
                c_docker_image: ch_deployment_docker_image,
                c_workload_architecture: ch_deployment_workload_architecture,
                c_keeper: ch_deployment_keeper,
                c_extra_memory_mb: ch_deployment_extra_memory_mb,
                c_mark_cache_size_mb: ch_deployment_mark_cache_size_mb,
                c_index_mark_cache_size_mb: ch_deployment_index_mark_cache_size_mb,
                c_uncompressed_cache_size_mb: ch_deployment_uncompressed_cache_size_mb,
                c_compiled_expression_cache_size_mb: ch_deployment_compiled_expression_cache_size_mb,
                c_query_cache_size_mb: ch_deployment_query_cache_size_mb,
                c_max_thread_pool_size: ch_deployment_max_thread_pool_size,
                c_max_concurrent_queries: ch_deployment_max_concurrent_queries,
                c_merge_max_block_size: ch_deployment_merge_max_block_size,
                c_max_bytes_to_merge_at_max_space_in_pool_mb: ch_deployment_max_bytes_to_merge_at_max_space_in_pool_mb,
                c_max_query_execution_time_seconds: ch_deployment_max_query_execution_time_seconds,
                c_queue_max_wait_ms: ch_deployment_queue_max_wait_ms,
                c_distribute_over_dcs: ch_deployment_distribute_over_dcs,
                c_native_port: ch_deployment_native_port,
                c_http_port: ch_deployment_http_port,
                c_replication_port: ch_deployment_replication_port,
                c_prometheus_port: ch_deployment_prometheus_port,
                c_children_ch_deployment_instance: ch_deployment_children_ch_deployment_instance,
                c_children_ch_deployment_schemas: ch_deployment_children_ch_deployment_schemas,
            },
            ch_deployment_instance: TableDefinitionChDeploymentInstance {
                rows: rows_ch_deployment_instance,
                c_instance_id: ch_deployment_instance_instance_id,
                c_ch_server: ch_deployment_instance_ch_server,
                c_parent: ch_deployment_instance_parent,
            },
            ch_deployment_schemas: TableDefinitionChDeploymentSchemas {
                rows: rows_ch_deployment_schemas,
                c_db_name: ch_deployment_schemas_db_name,
                c_ch_schema: ch_deployment_schemas_ch_schema,
                c_parent: ch_deployment_schemas_parent,
                c_children_ch_nats_stream_import: ch_deployment_schemas_children_ch_nats_stream_import,
            },
            ch_keeper_deployment: TableDefinitionChKeeperDeployment {
                rows: rows_ch_keeper_deployment,
                c_deployment_name: ch_keeper_deployment_deployment_name,
                c_namespace: ch_keeper_deployment_namespace,
                c_region: ch_keeper_deployment_region,
                c_loki_cluster: ch_keeper_deployment_loki_cluster,
                c_monitoring_cluster: ch_keeper_deployment_monitoring_cluster,
                c_docker_image: ch_keeper_deployment_docker_image,
                c_workload_architecture: ch_keeper_deployment_workload_architecture,
                c_distribute_over_dcs: ch_keeper_deployment_distribute_over_dcs,
                c_memory_mb: ch_keeper_deployment_memory_mb,
                c_keeper_port: ch_keeper_deployment_keeper_port,
                c_raft_port: ch_keeper_deployment_raft_port,
                c_prometheus_port: ch_keeper_deployment_prometheus_port,
                c_children_ch_keeper_deployment_instance: ch_keeper_deployment_children_ch_keeper_deployment_instance,
                c_referrers_ch_deployment__keeper: ch_keeper_deployment_referrers_ch_deployment__keeper,
            },
            ch_keeper_deployment_instance: TableDefinitionChKeeperDeploymentInstance {
                rows: rows_ch_keeper_deployment_instance,
                c_instance_id: ch_keeper_deployment_instance_instance_id,
                c_keeper_server: ch_keeper_deployment_instance_keeper_server,
                c_parent: ch_keeper_deployment_instance_parent,
            },
            ch_migration: TableDefinitionChMigration {
                rows: rows_ch_migration,
                c_time: ch_migration_time,
                c_upgrade: ch_migration_upgrade,
                c_downgrade: ch_migration_downgrade,
                c_needs_admin: ch_migration_needs_admin,
                c_parent: ch_migration_parent,
            },
            ch_mutator: TableDefinitionChMutator {
                rows: rows_ch_mutator,
                c_mutator_name: ch_mutator_mutator_name,
                c_mutator_expression: ch_mutator_mutator_expression,
                c_parent: ch_mutator_parent,
                c_children_ch_mutator_test: ch_mutator_children_ch_mutator_test,
            },
            ch_mutator_test: TableDefinitionChMutatorTest {
                rows: rows_ch_mutator_test,
                c_arguments: ch_mutator_test_arguments,
                c_test_dataset: ch_mutator_test_test_dataset,
                c_resulting_data: ch_mutator_test_resulting_data,
                c_parent: ch_mutator_test_parent,
            },
            ch_nats_stream_import: TableDefinitionChNatsStreamImport {
                rows: rows_ch_nats_stream_import,
                c_consumer_name: ch_nats_stream_import_consumer_name,
                c_into_table: ch_nats_stream_import_into_table,
                c_stream: ch_nats_stream_import_stream,
                c_parent: ch_nats_stream_import_parent,
            },
            ch_query: TableDefinitionChQuery {
                rows: rows_ch_query,
                c_query_name: ch_query_query_name,
                c_query_expression: ch_query_query_expression,
                c_opt_fields: ch_query_opt_fields,
                c_parent: ch_query_parent,
                c_children_ch_query_test: ch_query_children_ch_query_test,
            },
            ch_query_test: TableDefinitionChQueryTest {
                rows: rows_ch_query_test,
                c_arguments: ch_query_test_arguments,
                c_outputs: ch_query_test_outputs,
                c_test_dataset: ch_query_test_test_dataset,
                c_parent: ch_query_test_parent,
            },
            ch_schema: TableDefinitionChSchema {
                rows: rows_ch_schema,
                c_schema_name: ch_schema_schema_name,
                c_children_ch_migration: ch_schema_children_ch_migration,
                c_children_ch_query: ch_schema_children_ch_query,
                c_children_ch_mutator: ch_schema_children_ch_mutator,
                c_children_ch_test_dataset: ch_schema_children_ch_test_dataset,
                c_referrers_ch_deployment_schemas__ch_schema: ch_schema_referrers_ch_deployment_schemas__ch_schema,
                c_referrers_backend_application_ch_shard__ch_schema: ch_schema_referrers_backend_application_ch_shard__ch_schema,
            },
            ch_test_dataset: TableDefinitionChTestDataset {
                rows: rows_ch_test_dataset,
                c_dataset_name: ch_test_dataset_dataset_name,
                c_dataset_contents: ch_test_dataset_dataset_contents,
                c_min_time: ch_test_dataset_min_time,
                c_parent: ch_test_dataset_parent,
                c_referrers_ch_query_test__test_dataset: ch_test_dataset_referrers_ch_query_test__test_dataset,
                c_referrers_ch_mutator_test__test_dataset: ch_test_dataset_referrers_ch_mutator_test__test_dataset,
            },
            datacenter: TableDefinitionDatacenter {
                rows: rows_datacenter,
                c_dc_name: datacenter_dc_name,
                c_region: datacenter_region,
                c_network_cidr: datacenter_network_cidr,
                c_allow_small_subnets: datacenter_allow_small_subnets,
                c_implementation: datacenter_implementation,
                c_implementation_settings: datacenter_implementation_settings,
                c_default_server_kind: datacenter_default_server_kind,
                c_disk_ids_policy: datacenter_disk_ids_policy,
                c_router_subnet_vlan_id: datacenter_router_subnet_vlan_id,
                c_referrers_server__dc: datacenter_referrers_server__dc,
            },
            disk_kind: TableDefinitionDiskKind {
                rows: rows_disk_kind,
                c_kind: disk_kind_kind,
                c_medium: disk_kind_medium,
                c_is_elastic: disk_kind_is_elastic,
                c_min_capacity_bytes: disk_kind_min_capacity_bytes,
                c_max_capacity_bytes: disk_kind_max_capacity_bytes,
                c_capacity_bytes: disk_kind_capacity_bytes,
                c_has_extra_config: disk_kind_has_extra_config,
                c_non_eligible_reason: disk_kind_non_eligible_reason,
                c_referrers_server_disk__disk_kind: disk_kind_referrers_server_disk__disk_kind,
            },
            docker_image: TableDefinitionDockerImage {
                rows: rows_docker_image,
                c_checksum: docker_image_checksum,
                c_image_set: docker_image_image_set,
                c_repository: docker_image_repository,
                c_architecture: docker_image_architecture,
                c_tag: docker_image_tag,
                c_referrers_docker_image_pin_images__checksum: docker_image_referrers_docker_image_pin_images__checksum,
            },
            docker_image_pin: TableDefinitionDockerImagePin {
                rows: rows_docker_image_pin,
                c_pin_name: docker_image_pin_pin_name,
                c_children_docker_image_pin_images: docker_image_pin_children_docker_image_pin_images,
                c_referrers_region__docker_image_external_lb: docker_image_pin_referrers_region__docker_image_external_lb,
                c_referrers_docker_registry_instance__docker_image: docker_image_pin_referrers_docker_registry_instance__docker_image,
                c_referrers_pg_deployment__docker_image_pg: docker_image_pin_referrers_pg_deployment__docker_image_pg,
                c_referrers_pg_deployment__docker_image_haproxy: docker_image_pin_referrers_pg_deployment__docker_image_haproxy,
                c_referrers_pg_deployment__docker_image_pg_exporter: docker_image_pin_referrers_pg_deployment__docker_image_pg_exporter,
                c_referrers_ch_deployment__docker_image: docker_image_pin_referrers_ch_deployment__docker_image,
                c_referrers_ch_keeper_deployment__docker_image: docker_image_pin_referrers_ch_keeper_deployment__docker_image,
                c_referrers_nats_cluster__docker_image_nats: docker_image_pin_referrers_nats_cluster__docker_image_nats,
                c_referrers_nats_cluster__docker_image_nats_exporter: docker_image_pin_referrers_nats_cluster__docker_image_nats_exporter,
                c_referrers_minio_cluster__docker_image_minio: docker_image_pin_referrers_minio_cluster__docker_image_minio,
                c_referrers_minio_cluster__docker_image_minio_mc: docker_image_pin_referrers_minio_cluster__docker_image_minio_mc,
                c_referrers_minio_cluster__docker_image_nginx: docker_image_pin_referrers_minio_cluster__docker_image_nginx,
                c_referrers_monitoring_cluster__docker_image_prometheus: docker_image_pin_referrers_monitoring_cluster__docker_image_prometheus,
                c_referrers_monitoring_cluster__docker_image_alertmanager: docker_image_pin_referrers_monitoring_cluster__docker_image_alertmanager,
                c_referrers_monitoring_cluster__docker_image_victoriametrics: docker_image_pin_referrers_monitoring_cluster__docker_image_victoriametrics,
                c_referrers_grafana__docker_image_grafana: docker_image_pin_referrers_grafana__docker_image_grafana,
                c_referrers_grafana__docker_image_promxy: docker_image_pin_referrers_grafana__docker_image_promxy,
                c_referrers_loki_cluster__docker_image_loki: docker_image_pin_referrers_loki_cluster__docker_image_loki,
                c_referrers_tempo_cluster__docker_image: docker_image_pin_referrers_tempo_cluster__docker_image,
                c_referrers_blackbox_deployment_task__docker_image: docker_image_pin_referrers_blackbox_deployment_task__docker_image,
            },
            docker_image_pin_images: TableDefinitionDockerImagePinImages {
                rows: rows_docker_image_pin_images,
                c_checksum: docker_image_pin_images_checksum,
                c_parent: docker_image_pin_images_parent,
            },
            docker_image_set: TableDefinitionDockerImageSet {
                rows: rows_docker_image_set,
                c_set_name: docker_image_set_set_name,
                c_referrers_docker_image__image_set: docker_image_set_referrers_docker_image__image_set,
                c_referrers_blackbox_deployment_task__docker_image_set: docker_image_set_referrers_blackbox_deployment_task__docker_image_set,
            },
            docker_registry_instance: TableDefinitionDockerRegistryInstance {
                rows: rows_docker_registry_instance,
                c_region: docker_registry_instance_region,
                c_minio_bucket: docker_registry_instance_minio_bucket,
                c_memory_mb: docker_registry_instance_memory_mb,
                c_docker_image: docker_registry_instance_docker_image,
            },
            frontend_application: TableDefinitionFrontendApplication {
                rows: rows_frontend_application,
                c_application_name: frontend_application_application_name,
                c_build_environment: frontend_application_build_environment,
                c_index_page_title: frontend_application_index_page_title,
                c_children_frontend_page: frontend_application_children_frontend_page,
                c_children_frontend_application_used_endpoint: frontend_application_children_frontend_application_used_endpoint,
                c_children_frontend_application_external_link: frontend_application_children_frontend_application_external_link,
                c_children_frontend_application_external_page: frontend_application_children_frontend_application_external_page,
                c_referrers_frontend_application_deployment__application_name: frontend_application_referrers_frontend_application_deployment__application_name,
            },
            frontend_application_deployment: TableDefinitionFrontendApplicationDeployment {
                rows: rows_frontend_application_deployment,
                c_deployment_name: frontend_application_deployment_deployment_name,
                c_application_name: frontend_application_deployment_application_name,
                c_namespace: frontend_application_deployment_namespace,
                c_explicit_endpoint_wiring: frontend_application_deployment_explicit_endpoint_wiring,
                c_workload_backend_architecture: frontend_application_deployment_workload_backend_architecture,
                c_placement: frontend_application_deployment_placement,
                c_link_wiring: frontend_application_deployment_link_wiring,
                c_page_wiring: frontend_application_deployment_page_wiring,
                c_count: frontend_application_deployment_count,
                c_http_port: frontend_application_deployment_http_port,
                c_memory_mb: frontend_application_deployment_memory_mb,
                c_region: frontend_application_deployment_region,
                c_loki_cluster: frontend_application_deployment_loki_cluster,
                c_referrers_frontend_application_deployment_ingress__deployment: frontend_application_deployment_referrers_frontend_application_deployment_ingress__deployment,
            },
            frontend_application_deployment_ingress: TableDefinitionFrontendApplicationDeploymentIngress {
                rows: rows_frontend_application_deployment_ingress,
                c_deployment: frontend_application_deployment_ingress_deployment,
                c_mountpoint: frontend_application_deployment_ingress_mountpoint,
                c_subdomain: frontend_application_deployment_ingress_subdomain,
                c_tld: frontend_application_deployment_ingress_tld,
            },
            frontend_application_external_link: TableDefinitionFrontendApplicationExternalLink {
                rows: rows_frontend_application_external_link,
                c_link_name: frontend_application_external_link_link_name,
                c_backend_endpoint: frontend_application_external_link_backend_endpoint,
                c_parent: frontend_application_external_link_parent,
            },
            frontend_application_external_page: TableDefinitionFrontendApplicationExternalPage {
                rows: rows_frontend_application_external_page,
                c_link_name: frontend_application_external_page_link_name,
                c_frontend_page: frontend_application_external_page_frontend_page,
                c_parent: frontend_application_external_page_parent,
            },
            frontend_application_used_endpoint: TableDefinitionFrontendApplicationUsedEndpoint {
                rows: rows_frontend_application_used_endpoint,
                c_endpoint_name: frontend_application_used_endpoint_endpoint_name,
                c_backend_endpoint: frontend_application_used_endpoint_backend_endpoint,
                c_parent: frontend_application_used_endpoint_parent,
            },
            frontend_page: TableDefinitionFrontendPage {
                rows: rows_frontend_page,
                c_page_name: frontend_page_page_name,
                c_path: frontend_page_path,
                c_parent: frontend_page_parent,
                c_referrers_frontend_application_external_page__frontend_page: frontend_page_referrers_frontend_application_external_page__frontend_page,
            },
            global_settings: TableDefinitionGlobalSettings {
                rows: rows_global_settings,
                c_project_name: global_settings_project_name,
                c_docker_registry_port: global_settings_docker_registry_port,
                c_docker_registry_service_name: global_settings_docker_registry_service_name,
                c_aws_artefacts_s3_bucket_name: global_settings_aws_artefacts_s3_bucket_name,
                c_local_docker_cache_port: global_settings_local_docker_cache_port,
                c_admin_email: global_settings_admin_email,
                c_google_cloud_project_id: global_settings_google_cloud_project_id,
                c_google_cloud_artefacts_bucket_name: global_settings_google_cloud_artefacts_bucket_name,
                c_disable_consul_quorum_tests: global_settings_disable_consul_quorum_tests,
                c_disable_nomad_quorum_tests: global_settings_disable_nomad_quorum_tests,
                c_disable_vault_quorum_tests: global_settings_disable_vault_quorum_tests,
                c_disable_dns_quorum_tests: global_settings_disable_dns_quorum_tests,
                c_disable_deployment_min_server_tests: global_settings_disable_deployment_min_server_tests,
                c_disable_deployment_min_ingress_tests: global_settings_disable_deployment_min_ingress_tests,
                c_disable_region_docker_registry_tests: global_settings_disable_region_docker_registry_tests,
                c_disable_region_monitoring_tests: global_settings_disable_region_monitoring_tests,
                c_disable_region_tracing_tests: global_settings_disable_region_tracing_tests,
                c_disable_region_logging_tests: global_settings_disable_region_logging_tests,
                c_disable_vpn_gateway_tests: global_settings_disable_vpn_gateway_tests,
                c_hetzner_inter_dc_vlan_id: global_settings_hetzner_inter_dc_vlan_id,
                c_experimental_enable_arm64_support: global_settings_experimental_enable_arm64_support,
                c_update_edl_public_ips_from_terraform: global_settings_update_edl_public_ips_from_terraform,
                c_enable_ipv6: global_settings_enable_ipv6,
                c_force_ipv6: global_settings_force_ipv6,
            },
            grafana: TableDefinitionGrafana {
                rows: rows_grafana,
                c_deployment_name: grafana_deployment_name,
                c_namespace: grafana_namespace,
                c_region: grafana_region,
                c_placement: grafana_placement,
                c_workload_architecture: grafana_workload_architecture,
                c_docker_image_grafana: grafana_docker_image_grafana,
                c_docker_image_promxy: grafana_docker_image_promxy,
                c_loki_cluster: grafana_loki_cluster,
                c_monitoring_cluster: grafana_monitoring_cluster,
                c_port: grafana_port,
                c_promxy_port: grafana_promxy_port,
                c_instance_count: grafana_instance_count,
                c_database: grafana_database,
                c_memory_mb: grafana_memory_mb,
                c_promxy_memory_mb: grafana_promxy_memory_mb,
            },
            grafana_dashboard: TableDefinitionGrafanaDashboard {
                rows: rows_grafana_dashboard,
                c_filename: grafana_dashboard_filename,
                c_contents: grafana_dashboard_contents,
            },
            http_endpoint_data_type: TableDefinitionHttpEndpointDataType {
                rows: rows_http_endpoint_data_type,
                c_http_endpoint_data_type: http_endpoint_data_type_http_endpoint_data_type,
                c_referrers_backend_http_endpoint__data_type: http_endpoint_data_type_referrers_backend_http_endpoint__data_type,
            },
            http_methods: TableDefinitionHttpMethods {
                rows: rows_http_methods,
                c_http_method_name: http_methods_http_method_name,
                c_referrers_backend_http_endpoint__http_method: http_methods_referrers_backend_http_endpoint__http_method,
            },
            loki_cluster: TableDefinitionLokiCluster {
                rows: rows_loki_cluster,
                c_cluster_name: loki_cluster_cluster_name,
                c_namespace: loki_cluster_namespace,
                c_region: loki_cluster_region,
                c_workload_architecture: loki_cluster_workload_architecture,
                c_docker_image_loki: loki_cluster_docker_image_loki,
                c_is_region_default: loki_cluster_is_region_default,
                c_loki_cluster: loki_cluster_loki_cluster,
                c_monitoring_cluster: loki_cluster_monitoring_cluster,
                c_storage_bucket: loki_cluster_storage_bucket,
                c_retention_period_days: loki_cluster_retention_period_days,
                c_loki_writer_http_port: loki_cluster_loki_writer_http_port,
                c_loki_writer_grpc_port: loki_cluster_loki_writer_grpc_port,
                c_loki_reader_http_port: loki_cluster_loki_reader_http_port,
                c_loki_reader_grpc_port: loki_cluster_loki_reader_grpc_port,
                c_loki_backend_http_port: loki_cluster_loki_backend_http_port,
                c_loki_backend_grpc_port: loki_cluster_loki_backend_grpc_port,
                c_loki_writers: loki_cluster_loki_writers,
                c_loki_readers: loki_cluster_loki_readers,
                c_writer_placement: loki_cluster_writer_placement,
                c_reader_placement: loki_cluster_reader_placement,
                c_backend_placement: loki_cluster_backend_placement,
                c_loki_reader_memory_mb: loki_cluster_loki_reader_memory_mb,
                c_loki_writer_memory_mb: loki_cluster_loki_writer_memory_mb,
                c_loki_backend_memory_mb: loki_cluster_loki_backend_memory_mb,
            },
            minio_bucket: TableDefinitionMinioBucket {
                rows: rows_minio_bucket,
                c_bucket_name: minio_bucket_bucket_name,
                c_locking_enabled: minio_bucket_locking_enabled,
                c_parent: minio_bucket_parent,
                c_referrers_docker_registry_instance__minio_bucket: minio_bucket_referrers_docker_registry_instance__minio_bucket,
                c_referrers_loki_cluster__storage_bucket: minio_bucket_referrers_loki_cluster__storage_bucket,
                c_referrers_tempo_cluster__storage_bucket: minio_bucket_referrers_tempo_cluster__storage_bucket,
            },
            minio_cluster: TableDefinitionMinioCluster {
                rows: rows_minio_cluster,
                c_cluster_name: minio_cluster_cluster_name,
                c_namespace: minio_cluster_namespace,
                c_region: minio_cluster_region,
                c_workload_architecture: minio_cluster_workload_architecture,
                c_docker_image_minio: minio_cluster_docker_image_minio,
                c_docker_image_minio_mc: minio_cluster_docker_image_minio_mc,
                c_docker_image_nginx: minio_cluster_docker_image_nginx,
                c_api_port: minio_cluster_api_port,
                c_console_port: minio_cluster_console_port,
                c_lb_port: minio_cluster_lb_port,
                c_loki_cluster: minio_cluster_loki_cluster,
                c_monitoring_cluster: minio_cluster_monitoring_cluster,
                c_expected_zfs_recordsize: minio_cluster_expected_zfs_recordsize,
                c_distribute_over_dcs: minio_cluster_distribute_over_dcs,
                c_instance_memory_mb: minio_cluster_instance_memory_mb,
                c_lb_memory_mb: minio_cluster_lb_memory_mb,
                c_consul_service_name: minio_cluster_consul_service_name,
                c_children_minio_instance: minio_cluster_children_minio_instance,
                c_children_minio_bucket: minio_cluster_children_minio_bucket,
            },
            minio_instance: TableDefinitionMinioInstance {
                rows: rows_minio_instance,
                c_instance_id: minio_instance_instance_id,
                c_instance_volume: minio_instance_instance_volume,
                c_parent: minio_instance_parent,
            },
            monitoring_cluster: TableDefinitionMonitoringCluster {
                rows: rows_monitoring_cluster,
                c_cluster_name: monitoring_cluster_cluster_name,
                c_namespace: monitoring_cluster_namespace,
                c_region: monitoring_cluster_region,
                c_is_region_default: monitoring_cluster_is_region_default,
                c_workload_architecture: monitoring_cluster_workload_architecture,
                c_docker_image_prometheus: monitoring_cluster_docker_image_prometheus,
                c_docker_image_alertmanager: monitoring_cluster_docker_image_alertmanager,
                c_docker_image_victoriametrics: monitoring_cluster_docker_image_victoriametrics,
                c_prometheus_memory_mb: monitoring_cluster_prometheus_memory_mb,
                c_victoriametrics_memory_mb: monitoring_cluster_victoriametrics_memory_mb,
                c_alertmanager_memory_mb: monitoring_cluster_alertmanager_memory_mb,
                c_prometheus_port: monitoring_cluster_prometheus_port,
                c_victoriametrics_port: monitoring_cluster_victoriametrics_port,
                c_alertmanager_port: monitoring_cluster_alertmanager_port,
                c_alertmanager_p2p_port: monitoring_cluster_alertmanager_p2p_port,
                c_victoriametrics_retention_months: monitoring_cluster_victoriametrics_retention_months,
                c_children_monitoring_cluster_scraped_metric: monitoring_cluster_children_monitoring_cluster_scraped_metric,
                c_children_monitoring_cluster_alert_group: monitoring_cluster_children_monitoring_cluster_alert_group,
                c_children_monitoring_instance: monitoring_cluster_children_monitoring_instance,
                c_children_alertmanager_instance: monitoring_cluster_children_alertmanager_instance,
            },
            monitoring_cluster_alert_group: TableDefinitionMonitoringClusterAlertGroup {
                rows: rows_monitoring_cluster_alert_group,
                c_alert_group_name: monitoring_cluster_alert_group_alert_group_name,
                c_telegram_channel: monitoring_cluster_alert_group_telegram_channel,
                c_telegram_bot: monitoring_cluster_alert_group_telegram_bot,
                c_parent: monitoring_cluster_alert_group_parent,
            },
            monitoring_cluster_scraped_metric: TableDefinitionMonitoringClusterScrapedMetric {
                rows: rows_monitoring_cluster_scraped_metric,
                c_metric_name: monitoring_cluster_scraped_metric_metric_name,
                c_expression: monitoring_cluster_scraped_metric_expression,
                c_parent: monitoring_cluster_scraped_metric_parent,
            },
            monitoring_instance: TableDefinitionMonitoringInstance {
                rows: rows_monitoring_instance,
                c_instance_id: monitoring_instance_instance_id,
                c_monitoring_server: monitoring_instance_monitoring_server,
                c_parent: monitoring_instance_parent,
            },
            nats_cluster: TableDefinitionNatsCluster {
                rows: rows_nats_cluster,
                c_cluster_name: nats_cluster_cluster_name,
                c_namespace: nats_cluster_namespace,
                c_region: nats_cluster_region,
                c_loki_cluster: nats_cluster_loki_cluster,
                c_monitoring_cluster: nats_cluster_monitoring_cluster,
                c_distribute_over_dcs: nats_cluster_distribute_over_dcs,
                c_workload_architecture: nats_cluster_workload_architecture,
                c_docker_image_nats: nats_cluster_docker_image_nats,
                c_docker_image_nats_exporter: nats_cluster_docker_image_nats_exporter,
                c_nats_clients_port: nats_cluster_nats_clients_port,
                c_nats_cluster_port: nats_cluster_nats_cluster_port,
                c_nats_http_mon_port: nats_cluster_nats_http_mon_port,
                c_nats_prometheus_port: nats_cluster_nats_prometheus_port,
                c_instance_memory_mb: nats_cluster_instance_memory_mb,
                c_children_nats_jetstream_stream: nats_cluster_children_nats_jetstream_stream,
                c_children_nats_deployment_instance: nats_cluster_children_nats_deployment_instance,
            },
            nats_deployment_instance: TableDefinitionNatsDeploymentInstance {
                rows: rows_nats_deployment_instance,
                c_instance_id: nats_deployment_instance_instance_id,
                c_nats_server: nats_deployment_instance_nats_server,
                c_parent: nats_deployment_instance_parent,
            },
            nats_jetstream_stream: TableDefinitionNatsJetstreamStream {
                rows: rows_nats_jetstream_stream,
                c_stream_name: nats_jetstream_stream_stream_name,
                c_stream_type: nats_jetstream_stream_stream_type,
                c_max_bytes: nats_jetstream_stream_max_bytes,
                c_max_msg_size: nats_jetstream_stream_max_msg_size,
                c_enable_subjects: nats_jetstream_stream_enable_subjects,
                c_parent: nats_jetstream_stream_parent,
                c_referrers_ch_nats_stream_import__stream: nats_jetstream_stream_referrers_ch_nats_stream_import__stream,
            },
            network: TableDefinitionNetwork {
                rows: rows_network,
                c_network_name: network_network_name,
                c_cidr: network_cidr,
                c_referrers_network_interface__if_network: network_referrers_network_interface__if_network,
            },
            network_interface: TableDefinitionNetworkInterface {
                rows: rows_network_interface,
                c_if_name: network_interface_if_name,
                c_if_network: network_interface_if_network,
                c_if_ip: network_interface_if_ip,
                c_if_prefix: network_interface_if_prefix,
                c_if_vlan: network_interface_if_vlan,
                c_parent: network_interface_parent,
                c_referrers_server__ssh_interface: network_interface_referrers_server__ssh_interface,
            },
            nixpkgs_environment: TableDefinitionNixpkgsEnvironment {
                rows: rows_nixpkgs_environment,
                c_name: nixpkgs_environment_name,
                c_version: nixpkgs_environment_version,
                c_referrers_server__nixpkgs_environment: nixpkgs_environment_referrers_server__nixpkgs_environment,
                c_referrers_rust_compilation_environment__nixpkgs_environment: nixpkgs_environment_referrers_rust_compilation_environment__nixpkgs_environment,
            },
            nixpkgs_version: TableDefinitionNixpkgsVersion {
                rows: rows_nixpkgs_version,
                c_version: nixpkgs_version_version,
                c_checksum: nixpkgs_version_checksum,
                c_tarball_checksum: nixpkgs_version_tarball_checksum,
                c_referrers_nixpkgs_environment__version: nixpkgs_version_referrers_nixpkgs_environment__version,
            },
            nomad_namespace: TableDefinitionNomadNamespace {
                rows: rows_nomad_namespace,
                c_namespace: nomad_namespace_namespace,
                c_description: nomad_namespace_description,
                c_referrers_pg_deployment__namespace: nomad_namespace_referrers_pg_deployment__namespace,
                c_referrers_ch_deployment__namespace: nomad_namespace_referrers_ch_deployment__namespace,
                c_referrers_ch_keeper_deployment__namespace: nomad_namespace_referrers_ch_keeper_deployment__namespace,
                c_referrers_nats_cluster__namespace: nomad_namespace_referrers_nats_cluster__namespace,
                c_referrers_backend_application_deployment__namespace: nomad_namespace_referrers_backend_application_deployment__namespace,
                c_referrers_frontend_application_deployment__namespace: nomad_namespace_referrers_frontend_application_deployment__namespace,
                c_referrers_minio_cluster__namespace: nomad_namespace_referrers_minio_cluster__namespace,
                c_referrers_monitoring_cluster__namespace: nomad_namespace_referrers_monitoring_cluster__namespace,
                c_referrers_grafana__namespace: nomad_namespace_referrers_grafana__namespace,
                c_referrers_loki_cluster__namespace: nomad_namespace_referrers_loki_cluster__namespace,
                c_referrers_tempo_cluster__namespace: nomad_namespace_referrers_tempo_cluster__namespace,
                c_referrers_blackbox_deployment__namespace: nomad_namespace_referrers_blackbox_deployment__namespace,
            },
            pg_deployment: TableDefinitionPgDeployment {
                rows: rows_pg_deployment,
                c_deployment_name: pg_deployment_deployment_name,
                c_namespace: pg_deployment_namespace,
                c_region: pg_deployment_region,
                c_loki_cluster: pg_deployment_loki_cluster,
                c_monitoring_cluster: pg_deployment_monitoring_cluster,
                c_docker_image_pg: pg_deployment_docker_image_pg,
                c_docker_image_haproxy: pg_deployment_docker_image_haproxy,
                c_docker_image_pg_exporter: pg_deployment_docker_image_pg_exporter,
                c_workload_architecture: pg_deployment_workload_architecture,
                c_distribute_over_dcs: pg_deployment_distribute_over_dcs,
                c_synchronous_replication: pg_deployment_synchronous_replication,
                c_shared_buffers_mb: pg_deployment_shared_buffers_mb,
                c_work_mem_mb: pg_deployment_work_mem_mb,
                c_maintenance_work_mem_mb: pg_deployment_maintenance_work_mem_mb,
                c_overhead_mem_mb: pg_deployment_overhead_mem_mb,
                c_max_connections: pg_deployment_max_connections,
                c_replica_rolling_update_delay_seconds: pg_deployment_replica_rolling_update_delay_seconds,
                c_instance_pg_port: pg_deployment_instance_pg_port,
                c_instance_pg_master_port: pg_deployment_instance_pg_master_port,
                c_instance_pg_slave_port: pg_deployment_instance_pg_slave_port,
                c_instance_patroni_port: pg_deployment_instance_patroni_port,
                c_instance_haproxy_metrics_port: pg_deployment_instance_haproxy_metrics_port,
                c_instance_pg_exporter_port: pg_deployment_instance_pg_exporter_port,
                c_children_pg_deployment_schemas: pg_deployment_children_pg_deployment_schemas,
                c_children_pg_deployment_unmanaged_db: pg_deployment_children_pg_deployment_unmanaged_db,
                c_children_pg_deployment_instance: pg_deployment_children_pg_deployment_instance,
            },
            pg_deployment_instance: TableDefinitionPgDeploymentInstance {
                rows: rows_pg_deployment_instance,
                c_instance_id: pg_deployment_instance_instance_id,
                c_pg_server: pg_deployment_instance_pg_server,
                c_parent: pg_deployment_instance_parent,
            },
            pg_deployment_schemas: TableDefinitionPgDeploymentSchemas {
                rows: rows_pg_deployment_schemas,
                c_db_name: pg_deployment_schemas_db_name,
                c_pg_schema: pg_deployment_schemas_pg_schema,
                c_parent: pg_deployment_schemas_parent,
            },
            pg_deployment_unmanaged_db: TableDefinitionPgDeploymentUnmanagedDb {
                rows: rows_pg_deployment_unmanaged_db,
                c_db_name: pg_deployment_unmanaged_db_db_name,
                c_parent: pg_deployment_unmanaged_db_parent,
                c_referrers_grafana__database: pg_deployment_unmanaged_db_referrers_grafana__database,
            },
            pg_mat_view: TableDefinitionPgMatView {
                rows: rows_pg_mat_view,
                c_mview_name: pg_mat_view_mview_name,
                c_update_frequency: pg_mat_view_update_frequency,
                c_parent: pg_mat_view_parent,
                c_children_pg_mat_view_test: pg_mat_view_children_pg_mat_view_test,
            },
            pg_mat_view_test: TableDefinitionPgMatViewTest {
                rows: rows_pg_mat_view_test,
                c_expected_data: pg_mat_view_test_expected_data,
                c_test_dataset: pg_mat_view_test_test_dataset,
                c_parent: pg_mat_view_test_parent,
            },
            pg_mat_view_update_frequency: TableDefinitionPgMatViewUpdateFrequency {
                rows: rows_pg_mat_view_update_frequency,
                c_frequency: pg_mat_view_update_frequency_frequency,
                c_referrers_pg_mat_view__update_frequency: pg_mat_view_update_frequency_referrers_pg_mat_view__update_frequency,
            },
            pg_migration: TableDefinitionPgMigration {
                rows: rows_pg_migration,
                c_time: pg_migration_time,
                c_upgrade: pg_migration_upgrade,
                c_downgrade: pg_migration_downgrade,
                c_needs_admin: pg_migration_needs_admin,
                c_parent: pg_migration_parent,
            },
            pg_mutator: TableDefinitionPgMutator {
                rows: rows_pg_mutator,
                c_mutator_name: pg_mutator_mutator_name,
                c_mutator_expression: pg_mutator_mutator_expression,
                c_seqscan_ok: pg_mutator_seqscan_ok,
                c_parent: pg_mutator_parent,
                c_children_pg_mutator_test: pg_mutator_children_pg_mutator_test,
            },
            pg_mutator_test: TableDefinitionPgMutatorTest {
                rows: rows_pg_mutator_test,
                c_arguments: pg_mutator_test_arguments,
                c_test_dataset: pg_mutator_test_test_dataset,
                c_resulting_data: pg_mutator_test_resulting_data,
                c_parent: pg_mutator_test_parent,
            },
            pg_query: TableDefinitionPgQuery {
                rows: rows_pg_query,
                c_query_name: pg_query_query_name,
                c_query_expression: pg_query_query_expression,
                c_is_mutating: pg_query_is_mutating,
                c_seqscan_ok: pg_query_seqscan_ok,
                c_opt_fields: pg_query_opt_fields,
                c_parent: pg_query_parent,
                c_children_pg_query_test: pg_query_children_pg_query_test,
            },
            pg_query_test: TableDefinitionPgQueryTest {
                rows: rows_pg_query_test,
                c_arguments: pg_query_test_arguments,
                c_outputs: pg_query_test_outputs,
                c_test_dataset: pg_query_test_test_dataset,
                c_parent: pg_query_test_parent,
            },
            pg_schema: TableDefinitionPgSchema {
                rows: rows_pg_schema,
                c_schema_name: pg_schema_schema_name,
                c_children_pg_migration: pg_schema_children_pg_migration,
                c_children_pg_query: pg_schema_children_pg_query,
                c_children_pg_mutator: pg_schema_children_pg_mutator,
                c_children_pg_transaction: pg_schema_children_pg_transaction,
                c_children_pg_mat_view: pg_schema_children_pg_mat_view,
                c_children_pg_test_dataset: pg_schema_children_pg_test_dataset,
                c_referrers_pg_deployment_schemas__pg_schema: pg_schema_referrers_pg_deployment_schemas__pg_schema,
                c_referrers_backend_application_pg_shard__pg_schema: pg_schema_referrers_backend_application_pg_shard__pg_schema,
            },
            pg_test_dataset: TableDefinitionPgTestDataset {
                rows: rows_pg_test_dataset,
                c_dataset_name: pg_test_dataset_dataset_name,
                c_dataset_contents: pg_test_dataset_dataset_contents,
                c_min_time: pg_test_dataset_min_time,
                c_parent: pg_test_dataset_parent,
                c_referrers_pg_query_test__test_dataset: pg_test_dataset_referrers_pg_query_test__test_dataset,
                c_referrers_pg_mutator_test__test_dataset: pg_test_dataset_referrers_pg_mutator_test__test_dataset,
                c_referrers_pg_mat_view_test__test_dataset: pg_test_dataset_referrers_pg_mat_view_test__test_dataset,
            },
            pg_transaction: TableDefinitionPgTransaction {
                rows: rows_pg_transaction,
                c_transaction_name: pg_transaction_transaction_name,
                c_steps: pg_transaction_steps,
                c_is_read_only: pg_transaction_is_read_only,
                c_parent: pg_transaction_parent,
            },
            region: TableDefinitionRegion {
                rows: rows_region,
                c_region_name: region_region_name,
                c_availability_mode: region_availability_mode,
                c_tld: region_tld,
                c_is_dns_master: region_is_dns_master,
                c_is_dns_slave: region_is_dns_slave,
                c_has_coprocessor_dc: region_has_coprocessor_dc,
                c_docker_image_external_lb: region_docker_image_external_lb,
                c_nomad_disable_log_collection: region_nomad_disable_log_collection,
                c_referrers_datacenter__region: region_referrers_datacenter__region,
                c_referrers_docker_registry_instance__region: region_referrers_docker_registry_instance__region,
                c_referrers_pg_deployment__region: region_referrers_pg_deployment__region,
                c_referrers_ch_deployment__region: region_referrers_ch_deployment__region,
                c_referrers_ch_keeper_deployment__region: region_referrers_ch_keeper_deployment__region,
                c_referrers_nats_cluster__region: region_referrers_nats_cluster__region,
                c_referrers_backend_application_deployment__region: region_referrers_backend_application_deployment__region,
                c_referrers_frontend_application_deployment__region: region_referrers_frontend_application_deployment__region,
                c_referrers_minio_cluster__region: region_referrers_minio_cluster__region,
                c_referrers_monitoring_cluster__region: region_referrers_monitoring_cluster__region,
                c_referrers_grafana__region: region_referrers_grafana__region,
                c_referrers_loki_cluster__region: region_referrers_loki_cluster__region,
                c_referrers_tempo_cluster__region: region_referrers_tempo_cluster__region,
                c_referrers_blackbox_deployment__region: region_referrers_blackbox_deployment__region,
            },
            rust_compilation_environment: TableDefinitionRustCompilationEnvironment {
                rows: rows_rust_compilation_environment,
                c_env_name: rust_compilation_environment_env_name,
                c_rust_edition: rust_compilation_environment_rust_edition,
                c_nixpkgs_environment: rust_compilation_environment_nixpkgs_environment,
                c_environment_kind: rust_compilation_environment_environment_kind,
                c_children_rust_crate_version: rust_compilation_environment_children_rust_crate_version,
                c_referrers_backend_application__build_environment: rust_compilation_environment_referrers_backend_application__build_environment,
                c_referrers_frontend_application__build_environment: rust_compilation_environment_referrers_frontend_application__build_environment,
            },
            rust_crate_version: TableDefinitionRustCrateVersion {
                rows: rows_rust_crate_version,
                c_crate_name: rust_crate_version_crate_name,
                c_version: rust_crate_version_version,
                c_features: rust_crate_version_features,
                c_default_features: rust_crate_version_default_features,
                c_parent: rust_crate_version_parent,
            },
            server: TableDefinitionServer {
                rows: rows_server,
                c_hostname: server_hostname,
                c_dc: server_dc,
                c_ssh_interface: server_ssh_interface,
                c_root_disk: server_root_disk,
                c_is_consul_master: server_is_consul_master,
                c_is_nomad_master: server_is_nomad_master,
                c_is_vault_instance: server_is_vault_instance,
                c_is_dns_master: server_is_dns_master,
                c_is_dns_slave: server_is_dns_slave,
                c_is_ingress: server_is_ingress,
                c_is_vpn_gateway: server_is_vpn_gateway,
                c_is_coprocessor_gateway: server_is_coprocessor_gateway,
                c_is_router: server_is_router,
                c_public_ipv6_address: server_public_ipv6_address,
                c_public_ipv6_address_prefix: server_public_ipv6_address_prefix,
                c_kind: server_kind,
                c_nixpkgs_environment: server_nixpkgs_environment,
                c_run_unassigned_workloads: server_run_unassigned_workloads,
                c_children_server_label: server_children_server_label,
                c_children_server_disk: server_children_server_disk,
                c_children_server_volume: server_children_server_volume,
                c_children_server_root_volume: server_children_server_root_volume,
                c_children_server_xfs_volume: server_children_server_xfs_volume,
                c_children_network_interface: server_children_network_interface,
                c_children_server_zpool: server_children_server_zpool,
            },
            server_disk: TableDefinitionServerDisk {
                rows: rows_server_disk,
                c_disk_id: server_disk_disk_id,
                c_disk_kind: server_disk_disk_kind,
                c_xfs_format: server_disk_xfs_format,
                c_extra_config: server_disk_extra_config,
                c_capacity_bytes: server_disk_capacity_bytes,
                c_parent: server_disk_parent,
                c_referrers_server__root_disk: server_disk_referrers_server__root_disk,
                c_referrers_server_xfs_volume__xfs_disk: server_disk_referrers_server_xfs_volume__xfs_disk,
                c_referrers_server_zpool_spare__disk_id: server_disk_referrers_server_zpool_spare__disk_id,
                c_referrers_server_zpool_cache__disk_id: server_disk_referrers_server_zpool_cache__disk_id,
                c_referrers_server_zpool_log__disk_id: server_disk_referrers_server_zpool_log__disk_id,
                c_referrers_server_zpool_vdev_disk__disk_id: server_disk_referrers_server_zpool_vdev_disk__disk_id,
            },
            server_kind: TableDefinitionServerKind {
                rows: rows_server_kind,
                c_kind: server_kind_kind,
                c_cores: server_kind_cores,
                c_memory_bytes: server_kind_memory_bytes,
                c_architecture: server_kind_architecture,
                c_bare_metal: server_kind_bare_metal,
                c_non_eligible_reason: server_kind_non_eligible_reason,
                c_children_server_kind_attribute: server_kind_children_server_kind_attribute,
                c_referrers_datacenter__default_server_kind: server_kind_referrers_datacenter__default_server_kind,
            },
            server_kind_attribute: TableDefinitionServerKindAttribute {
                rows: rows_server_kind_attribute,
                c_key: server_kind_attribute_key,
                c_value: server_kind_attribute_value,
                c_parent: server_kind_attribute_parent,
            },
            server_label: TableDefinitionServerLabel {
                rows: rows_server_label,
                c_label_name: server_label_label_name,
                c_label_value: server_label_label_value,
                c_parent: server_label_parent,
            },
            server_root_volume: TableDefinitionServerRootVolume {
                rows: rows_server_root_volume,
                c_volume_name: server_root_volume_volume_name,
                c_intended_usage: server_root_volume_intended_usage,
                c_mountpoint: server_root_volume_mountpoint,
                c_zfs_recordsize: server_root_volume_zfs_recordsize,
                c_zfs_compression: server_root_volume_zfs_compression,
                c_zfs_encryption: server_root_volume_zfs_encryption,
                c_parent: server_root_volume_parent,
            },
            server_volume: TableDefinitionServerVolume {
                rows: rows_server_volume,
                c_volume_name: server_volume_volume_name,
                c_mountpoint: server_volume_mountpoint,
                c_intended_usage: server_volume_intended_usage,
                c_source: server_volume_source,
                c_parent: server_volume_parent,
                c_referrers_pg_deployment_instance__pg_server: server_volume_referrers_pg_deployment_instance__pg_server,
                c_referrers_ch_deployment_instance__ch_server: server_volume_referrers_ch_deployment_instance__ch_server,
                c_referrers_ch_keeper_deployment_instance__keeper_server: server_volume_referrers_ch_keeper_deployment_instance__keeper_server,
                c_referrers_nats_deployment_instance__nats_server: server_volume_referrers_nats_deployment_instance__nats_server,
                c_referrers_minio_instance__instance_volume: server_volume_referrers_minio_instance__instance_volume,
                c_referrers_monitoring_instance__monitoring_server: server_volume_referrers_monitoring_instance__monitoring_server,
                c_referrers_alertmanager_instance__alertmanager_server: server_volume_referrers_alertmanager_instance__alertmanager_server,
                c_referrers_blackbox_deployment_task_mount__server_volume: server_volume_referrers_blackbox_deployment_task_mount__server_volume,
            },
            server_volume_usage_contract: TableDefinitionServerVolumeUsageContract {
                rows: rows_server_volume_usage_contract,
                c_usage_contract: server_volume_usage_contract_usage_contract,
                c_referrers_server_volume__intended_usage: server_volume_usage_contract_referrers_server_volume__intended_usage,
                c_referrers_server_root_volume__intended_usage: server_volume_usage_contract_referrers_server_root_volume__intended_usage,
                c_referrers_server_xfs_volume__intended_usage: server_volume_usage_contract_referrers_server_xfs_volume__intended_usage,
                c_referrers_server_zfs_dataset__intended_usage: server_volume_usage_contract_referrers_server_zfs_dataset__intended_usage,
            },
            server_xfs_volume: TableDefinitionServerXfsVolume {
                rows: rows_server_xfs_volume,
                c_volume_name: server_xfs_volume_volume_name,
                c_xfs_disk: server_xfs_volume_xfs_disk,
                c_intended_usage: server_xfs_volume_intended_usage,
                c_parent: server_xfs_volume_parent,
            },
            server_zfs_dataset: TableDefinitionServerZfsDataset {
                rows: rows_server_zfs_dataset,
                c_dataset_name: server_zfs_dataset_dataset_name,
                c_intended_usage: server_zfs_dataset_intended_usage,
                c_zfs_recordsize: server_zfs_dataset_zfs_recordsize,
                c_zfs_compression: server_zfs_dataset_zfs_compression,
                c_zfs_encryption: server_zfs_dataset_zfs_encryption,
                c_parent: server_zfs_dataset_parent,
            },
            server_zpool: TableDefinitionServerZpool {
                rows: rows_server_zpool,
                c_zpool_name: server_zpool_zpool_name,
                c_is_redundant: server_zpool_is_redundant,
                c_parent: server_zpool_parent,
                c_children_server_zpool_vdev: server_zpool_children_server_zpool_vdev,
                c_children_server_zpool_spare: server_zpool_children_server_zpool_spare,
                c_children_server_zpool_cache: server_zpool_children_server_zpool_cache,
                c_children_server_zpool_log: server_zpool_children_server_zpool_log,
                c_children_server_zfs_dataset: server_zpool_children_server_zfs_dataset,
            },
            server_zpool_cache: TableDefinitionServerZpoolCache {
                rows: rows_server_zpool_cache,
                c_disk_id: server_zpool_cache_disk_id,
                c_parent: server_zpool_cache_parent,
            },
            server_zpool_log: TableDefinitionServerZpoolLog {
                rows: rows_server_zpool_log,
                c_disk_id: server_zpool_log_disk_id,
                c_parent: server_zpool_log_parent,
            },
            server_zpool_spare: TableDefinitionServerZpoolSpare {
                rows: rows_server_zpool_spare,
                c_disk_id: server_zpool_spare_disk_id,
                c_parent: server_zpool_spare_parent,
            },
            server_zpool_vdev: TableDefinitionServerZpoolVdev {
                rows: rows_server_zpool_vdev,
                c_vdev_number: server_zpool_vdev_vdev_number,
                c_vdev_type: server_zpool_vdev_vdev_type,
                c_parent: server_zpool_vdev_parent,
                c_children_server_zpool_vdev_disk: server_zpool_vdev_children_server_zpool_vdev_disk,
            },
            server_zpool_vdev_disk: TableDefinitionServerZpoolVdevDisk {
                rows: rows_server_zpool_vdev_disk,
                c_disk_id: server_zpool_vdev_disk_disk_id,
                c_parent: server_zpool_vdev_disk_parent,
            },
            subnet_router_floating_ip: TableDefinitionSubnetRouterFloatingIp {
                rows: rows_subnet_router_floating_ip,
                c_ip_address: subnet_router_floating_ip_ip_address,
            },
            telegram_bot: TableDefinitionTelegramBot {
                rows: rows_telegram_bot,
                c_bot_name: telegram_bot_bot_name,
                c_bot_token: telegram_bot_bot_token,
                c_referrers_monitoring_cluster_alert_group__telegram_bot: telegram_bot_referrers_monitoring_cluster_alert_group__telegram_bot,
            },
            telegram_channel: TableDefinitionTelegramChannel {
                rows: rows_telegram_channel,
                c_channel_name: telegram_channel_channel_name,
                c_channel_id: telegram_channel_channel_id,
                c_referrers_monitoring_cluster_alert_group__telegram_channel: telegram_channel_referrers_monitoring_cluster_alert_group__telegram_channel,
            },
            tempo_cluster: TableDefinitionTempoCluster {
                rows: rows_tempo_cluster,
                c_cluster_name: tempo_cluster_cluster_name,
                c_namespace: tempo_cluster_namespace,
                c_region: tempo_cluster_region,
                c_workload_architecture: tempo_cluster_workload_architecture,
                c_docker_image: tempo_cluster_docker_image,
                c_is_region_default: tempo_cluster_is_region_default,
                c_loki_cluster: tempo_cluster_loki_cluster,
                c_monitoring_cluster: tempo_cluster_monitoring_cluster,
                c_storage_bucket: tempo_cluster_storage_bucket,
                c_http_port: tempo_cluster_http_port,
                c_grpc_port: tempo_cluster_grpc_port,
                c_p2p_port: tempo_cluster_p2p_port,
                c_otlp_http_port: tempo_cluster_otlp_http_port,
                c_otlp_grpc_port: tempo_cluster_otlp_grpc_port,
                c_tempo_instances: tempo_cluster_tempo_instances,
                c_placement: tempo_cluster_placement,
                c_trace_retention_days: tempo_cluster_trace_retention_days,
                c_memory_mb: tempo_cluster_memory_mb,
            },
            tld: TableDefinitionTld {
                rows: rows_tld,
                c_domain: tld_domain,
                c_expose_admin: tld_expose_admin,
                c_automatic_certificates: tld_automatic_certificates,
                c_referrers_region__tld: tld_referrers_region__tld,
                c_referrers_backend_application_deployment_ingress__tld: tld_referrers_backend_application_deployment_ingress__tld,
                c_referrers_frontend_application_deployment_ingress__tld: tld_referrers_frontend_application_deployment_ingress__tld,
            },
            unique_application_names: TableDefinitionUniqueApplicationNames {
                rows: rows_unique_application_names,
                c_application_name: unique_application_names_application_name,
                c_source: unique_application_names_source,
            },
            unique_deployment_names: TableDefinitionUniqueDeploymentNames {
                rows: rows_unique_deployment_names,
                c_deployment_name: unique_deployment_names_deployment_name,
                c_source: unique_deployment_names_source,
            },
            valid_server_labels: TableDefinitionValidServerLabels {
                rows: rows_valid_server_labels,
                c_label_name: valid_server_labels_label_name,
                c_referrers_server_label__label_name: valid_server_labels_referrers_server_label__label_name,
            },
            versioned_type: TableDefinitionVersionedType {
                rows: rows_versioned_type,
                c_type_name: versioned_type_type_name,
                c_children_versioned_type_snapshot: versioned_type_children_versioned_type_snapshot,
                c_children_versioned_type_migration: versioned_type_children_versioned_type_migration,
                c_referrers_nats_jetstream_stream__stream_type: versioned_type_referrers_nats_jetstream_stream__stream_type,
                c_referrers_backend_application_nats_stream__stream_type: versioned_type_referrers_backend_application_nats_stream__stream_type,
            },
            versioned_type_migration: TableDefinitionVersionedTypeMigration {
                rows: rows_versioned_type_migration,
                c_version: versioned_type_migration_version,
                c_migration_source: versioned_type_migration_migration_source,
                c_parent: versioned_type_migration_parent,
            },
            versioned_type_snapshot: TableDefinitionVersionedTypeSnapshot {
                rows: rows_versioned_type_snapshot,
                c_version: versioned_type_snapshot_version,
                c_snapshot_source: versioned_type_snapshot_snapshot_source,
                c_parent: versioned_type_snapshot_parent,
            },
        })
    }
}

// Table definition implementations
impl TableDefinitionAdminSshKeys {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerAdminSshKeys> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerAdminSshKeys(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerAdminSshKeys) -> &TableRowAdminSshKeys {
        &self.rows[ptr.0]
    }

    pub fn c_contents(&self, ptr: TableRowPointerAdminSshKeys) -> &::std::string::String {
        &self.c_contents[ptr.0]
    }

}

impl TableDefinitionAlert {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerAlert> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerAlert(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerAlert) -> &TableRowAlert {
        &self.rows[ptr.0]
    }

    pub fn c_alert_name(&self, ptr: TableRowPointerAlert) -> &::std::string::String {
        &self.c_alert_name[ptr.0]
    }

    pub fn c_expr(&self, ptr: TableRowPointerAlert) -> &::std::string::String {
        &self.c_expr[ptr.0]
    }

    pub fn c_description(&self, ptr: TableRowPointerAlert) -> &::std::string::String {
        &self.c_description[ptr.0]
    }

    pub fn c_for_time(&self, ptr: TableRowPointerAlert) -> &::std::string::String {
        &self.c_for_time[ptr.0]
    }

    pub fn c_severity(&self, ptr: TableRowPointerAlert) -> i64 {
        self.c_severity[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerAlert) -> TableRowPointerAlertGroup {
        self.c_parent[ptr.0]
    }

    pub fn c_children_alert_trigger_test(&self, ptr: TableRowPointerAlert) -> &[TableRowPointerAlertTriggerTest] {
        &self.c_children_alert_trigger_test[ptr.0]
    }

}

impl TableDefinitionAlertGroup {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerAlertGroup> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerAlertGroup(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerAlertGroup) -> &TableRowAlertGroup {
        &self.rows[ptr.0]
    }

    pub fn c_alert_group_name(&self, ptr: TableRowPointerAlertGroup) -> &::std::string::String {
        &self.c_alert_group_name[ptr.0]
    }

    pub fn c_children_alert(&self, ptr: TableRowPointerAlertGroup) -> &[TableRowPointerAlert] {
        &self.c_children_alert[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster_alert_group__alert_group_name(&self, ptr: TableRowPointerAlertGroup) -> &[TableRowPointerMonitoringClusterAlertGroup] {
        &self.c_referrers_monitoring_cluster_alert_group__alert_group_name[ptr.0]
    }

}

impl TableDefinitionAlertTriggerTest {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerAlertTriggerTest> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerAlertTriggerTest(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerAlertTriggerTest) -> &TableRowAlertTriggerTest {
        &self.rows[ptr.0]
    }

    pub fn c_expected_message(&self, ptr: TableRowPointerAlertTriggerTest) -> &::std::string::String {
        &self.c_expected_message[ptr.0]
    }

    pub fn c_expected_labels(&self, ptr: TableRowPointerAlertTriggerTest) -> &::std::string::String {
        &self.c_expected_labels[ptr.0]
    }

    pub fn c_eval_time(&self, ptr: TableRowPointerAlertTriggerTest) -> &::std::string::String {
        &self.c_eval_time[ptr.0]
    }

    pub fn c_interval(&self, ptr: TableRowPointerAlertTriggerTest) -> &::std::string::String {
        &self.c_interval[ptr.0]
    }

    pub fn c_input_series(&self, ptr: TableRowPointerAlertTriggerTest) -> &::std::string::String {
        &self.c_input_series[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerAlertTriggerTest) -> TableRowPointerAlert {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionAlertmanagerInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerAlertmanagerInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerAlertmanagerInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerAlertmanagerInstance) -> &TableRowAlertmanagerInstance {
        &self.rows[ptr.0]
    }

    pub fn c_instance_id(&self, ptr: TableRowPointerAlertmanagerInstance) -> i64 {
        self.c_instance_id[ptr.0]
    }

    pub fn c_alertmanager_server(&self, ptr: TableRowPointerAlertmanagerInstance) -> TableRowPointerServerVolume {
        self.c_alertmanager_server[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerAlertmanagerInstance) -> TableRowPointerMonitoringCluster {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBackendApplication {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplication> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplication(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplication) -> &TableRowBackendApplication {
        &self.rows[ptr.0]
    }

    pub fn c_application_name(&self, ptr: TableRowPointerBackendApplication) -> &::std::string::String {
        &self.c_application_name[ptr.0]
    }

    pub fn c_build_environment(&self, ptr: TableRowPointerBackendApplication) -> TableRowPointerRustCompilationEnvironment {
        self.c_build_environment[ptr.0]
    }

    pub fn c_children_backend_application_background_job(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendApplicationBackgroundJob] {
        &self.c_children_backend_application_background_job[ptr.0]
    }

    pub fn c_children_backend_application_config(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendApplicationConfig] {
        &self.c_children_backend_application_config[ptr.0]
    }

    pub fn c_children_backend_application_s3_bucket(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendApplicationS3Bucket] {
        &self.c_children_backend_application_s3_bucket[ptr.0]
    }

    pub fn c_children_backend_application_pg_shard(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendApplicationPgShard] {
        &self.c_children_backend_application_pg_shard[ptr.0]
    }

    pub fn c_children_backend_application_ch_shard(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendApplicationChShard] {
        &self.c_children_backend_application_ch_shard[ptr.0]
    }

    pub fn c_children_backend_application_nats_stream(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendApplicationNatsStream] {
        &self.c_children_backend_application_nats_stream[ptr.0]
    }

    pub fn c_children_backend_http_endpoint(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendHttpEndpoint] {
        &self.c_children_backend_http_endpoint[ptr.0]
    }

    pub fn c_referrers_backend_application_deployment__application_name(&self, ptr: TableRowPointerBackendApplication) -> &[TableRowPointerBackendApplicationDeployment] {
        &self.c_referrers_backend_application_deployment__application_name[ptr.0]
    }

}

impl TableDefinitionBackendApplicationBackgroundJob {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationBackgroundJob> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationBackgroundJob(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationBackgroundJob) -> &TableRowBackendApplicationBackgroundJob {
        &self.rows[ptr.0]
    }

    pub fn c_job_name(&self, ptr: TableRowPointerBackendApplicationBackgroundJob) -> &::std::string::String {
        &self.c_job_name[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBackendApplicationBackgroundJob) -> TableRowPointerBackendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBackendApplicationChShard {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationChShard> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationChShard(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationChShard) -> &TableRowBackendApplicationChShard {
        &self.rows[ptr.0]
    }

    pub fn c_shard_name(&self, ptr: TableRowPointerBackendApplicationChShard) -> &::std::string::String {
        &self.c_shard_name[ptr.0]
    }

    pub fn c_ch_schema(&self, ptr: TableRowPointerBackendApplicationChShard) -> TableRowPointerChSchema {
        self.c_ch_schema[ptr.0]
    }

    pub fn c_used_queries(&self, ptr: TableRowPointerBackendApplicationChShard) -> &::std::string::String {
        &self.c_used_queries[ptr.0]
    }

    pub fn c_used_inserters(&self, ptr: TableRowPointerBackendApplicationChShard) -> &::std::string::String {
        &self.c_used_inserters[ptr.0]
    }

    pub fn c_used_mutators(&self, ptr: TableRowPointerBackendApplicationChShard) -> &::std::string::String {
        &self.c_used_mutators[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBackendApplicationChShard) -> TableRowPointerBackendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBackendApplicationConfig {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationConfig> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationConfig(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationConfig) -> &TableRowBackendApplicationConfig {
        &self.rows[ptr.0]
    }

    pub fn c_config_name(&self, ptr: TableRowPointerBackendApplicationConfig) -> &::std::string::String {
        &self.c_config_name[ptr.0]
    }

    pub fn c_config_type(&self, ptr: TableRowPointerBackendApplicationConfig) -> &::std::string::String {
        &self.c_config_type[ptr.0]
    }

    pub fn c_default_value(&self, ptr: TableRowPointerBackendApplicationConfig) -> &::std::string::String {
        &self.c_default_value[ptr.0]
    }

    pub fn c_min_value(&self, ptr: TableRowPointerBackendApplicationConfig) -> &::std::string::String {
        &self.c_min_value[ptr.0]
    }

    pub fn c_max_value(&self, ptr: TableRowPointerBackendApplicationConfig) -> &::std::string::String {
        &self.c_max_value[ptr.0]
    }

    pub fn c_regex_check(&self, ptr: TableRowPointerBackendApplicationConfig) -> &::std::string::String {
        &self.c_regex_check[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBackendApplicationConfig) -> TableRowPointerBackendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBackendApplicationDeployment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationDeployment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationDeployment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &TableRowBackendApplicationDeployment {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerBackendApplicationDeployment) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_application_name(&self, ptr: TableRowPointerBackendApplicationDeployment) -> TableRowPointerBackendApplication {
        self.c_application_name[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_count(&self, ptr: TableRowPointerBackendApplicationDeployment) -> i64 {
        self.c_count[ptr.0]
    }

    pub fn c_placement(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_placement[ptr.0]
    }

    pub fn c_pg_shard_wiring(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_pg_shard_wiring[ptr.0]
    }

    pub fn c_ch_shard_wiring(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_ch_shard_wiring[ptr.0]
    }

    pub fn c_nats_stream_wiring(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_nats_stream_wiring[ptr.0]
    }

    pub fn c_s3_bucket_wiring(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_s3_bucket_wiring[ptr.0]
    }

    pub fn c_config(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_config[ptr.0]
    }

    pub fn c_http_port(&self, ptr: TableRowPointerBackendApplicationDeployment) -> i64 {
        self.c_http_port[ptr.0]
    }

    pub fn c_memory_mb(&self, ptr: TableRowPointerBackendApplicationDeployment) -> i64 {
        self.c_memory_mb[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerBackendApplicationDeployment) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_tracing_cluster(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &::std::string::String {
        &self.c_tracing_cluster[ptr.0]
    }

    pub fn c_referrers_backend_application_deployment_ingress__deployment(&self, ptr: TableRowPointerBackendApplicationDeployment) -> &[TableRowPointerBackendApplicationDeploymentIngress] {
        &self.c_referrers_backend_application_deployment_ingress__deployment[ptr.0]
    }

}

impl TableDefinitionBackendApplicationDeploymentIngress {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationDeploymentIngress> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationDeploymentIngress(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationDeploymentIngress) -> &TableRowBackendApplicationDeploymentIngress {
        &self.rows[ptr.0]
    }

    pub fn c_deployment(&self, ptr: TableRowPointerBackendApplicationDeploymentIngress) -> TableRowPointerBackendApplicationDeployment {
        self.c_deployment[ptr.0]
    }

    pub fn c_mountpoint(&self, ptr: TableRowPointerBackendApplicationDeploymentIngress) -> &::std::string::String {
        &self.c_mountpoint[ptr.0]
    }

    pub fn c_subdomain(&self, ptr: TableRowPointerBackendApplicationDeploymentIngress) -> &::std::string::String {
        &self.c_subdomain[ptr.0]
    }

    pub fn c_tld(&self, ptr: TableRowPointerBackendApplicationDeploymentIngress) -> TableRowPointerTld {
        self.c_tld[ptr.0]
    }

    pub fn c_endpoint_list(&self, ptr: TableRowPointerBackendApplicationDeploymentIngress) -> &::std::string::String {
        &self.c_endpoint_list[ptr.0]
    }

}

impl TableDefinitionBackendApplicationNatsStream {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationNatsStream> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationNatsStream(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> &TableRowBackendApplicationNatsStream {
        &self.rows[ptr.0]
    }

    pub fn c_stream_name(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> &::std::string::String {
        &self.c_stream_name[ptr.0]
    }

    pub fn c_stream_type(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> TableRowPointerVersionedType {
        self.c_stream_type[ptr.0]
    }

    pub fn c_enable_consumer(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> bool {
        self.c_enable_consumer[ptr.0]
    }

    pub fn c_enable_producer(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> bool {
        self.c_enable_producer[ptr.0]
    }

    pub fn c_is_batch_consumer(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> bool {
        self.c_is_batch_consumer[ptr.0]
    }

    pub fn c_enable_subjects(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> bool {
        self.c_enable_subjects[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBackendApplicationNatsStream) -> TableRowPointerBackendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBackendApplicationPgShard {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationPgShard> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationPgShard(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationPgShard) -> &TableRowBackendApplicationPgShard {
        &self.rows[ptr.0]
    }

    pub fn c_shard_name(&self, ptr: TableRowPointerBackendApplicationPgShard) -> &::std::string::String {
        &self.c_shard_name[ptr.0]
    }

    pub fn c_pg_schema(&self, ptr: TableRowPointerBackendApplicationPgShard) -> TableRowPointerPgSchema {
        self.c_pg_schema[ptr.0]
    }

    pub fn c_used_queries(&self, ptr: TableRowPointerBackendApplicationPgShard) -> &::std::string::String {
        &self.c_used_queries[ptr.0]
    }

    pub fn c_used_mutators(&self, ptr: TableRowPointerBackendApplicationPgShard) -> &::std::string::String {
        &self.c_used_mutators[ptr.0]
    }

    pub fn c_used_transactions(&self, ptr: TableRowPointerBackendApplicationPgShard) -> &::std::string::String {
        &self.c_used_transactions[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBackendApplicationPgShard) -> TableRowPointerBackendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBackendApplicationS3Bucket {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendApplicationS3Bucket> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendApplicationS3Bucket(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendApplicationS3Bucket) -> &TableRowBackendApplicationS3Bucket {
        &self.rows[ptr.0]
    }

    pub fn c_bucket_name(&self, ptr: TableRowPointerBackendApplicationS3Bucket) -> &::std::string::String {
        &self.c_bucket_name[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBackendApplicationS3Bucket) -> TableRowPointerBackendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBackendHttpEndpoint {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBackendHttpEndpoint> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBackendHttpEndpoint(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBackendHttpEndpoint) -> &TableRowBackendHttpEndpoint {
        &self.rows[ptr.0]
    }

    pub fn c_http_endpoint_name(&self, ptr: TableRowPointerBackendHttpEndpoint) -> &::std::string::String {
        &self.c_http_endpoint_name[ptr.0]
    }

    pub fn c_path(&self, ptr: TableRowPointerBackendHttpEndpoint) -> &::std::string::String {
        &self.c_path[ptr.0]
    }

    pub fn c_http_method(&self, ptr: TableRowPointerBackendHttpEndpoint) -> TableRowPointerHttpMethods {
        self.c_http_method[ptr.0]
    }

    pub fn c_input_body_type(&self, ptr: TableRowPointerBackendHttpEndpoint) -> &::std::string::String {
        &self.c_input_body_type[ptr.0]
    }

    pub fn c_output_body_type(&self, ptr: TableRowPointerBackendHttpEndpoint) -> &::std::string::String {
        &self.c_output_body_type[ptr.0]
    }

    pub fn c_data_type(&self, ptr: TableRowPointerBackendHttpEndpoint) -> TableRowPointerHttpEndpointDataType {
        self.c_data_type[ptr.0]
    }

    pub fn c_max_input_body_size_bytes(&self, ptr: TableRowPointerBackendHttpEndpoint) -> i64 {
        self.c_max_input_body_size_bytes[ptr.0]
    }

    pub fn c_needs_headers(&self, ptr: TableRowPointerBackendHttpEndpoint) -> bool {
        self.c_needs_headers[ptr.0]
    }

    pub fn c_receive_body_as_stream(&self, ptr: TableRowPointerBackendHttpEndpoint) -> bool {
        self.c_receive_body_as_stream[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBackendHttpEndpoint) -> TableRowPointerBackendApplication {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_frontend_application_used_endpoint__backend_endpoint(&self, ptr: TableRowPointerBackendHttpEndpoint) -> &[TableRowPointerFrontendApplicationUsedEndpoint] {
        &self.c_referrers_frontend_application_used_endpoint__backend_endpoint[ptr.0]
    }

    pub fn c_referrers_frontend_application_external_link__backend_endpoint(&self, ptr: TableRowPointerBackendHttpEndpoint) -> &[TableRowPointerFrontendApplicationExternalLink] {
        &self.c_referrers_frontend_application_external_link__backend_endpoint[ptr.0]
    }

}

impl TableDefinitionBlackboxDeployment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeployment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeployment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeployment) -> &TableRowBlackboxDeployment {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerBlackboxDeployment) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerBlackboxDeployment) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerBlackboxDeployment) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerBlackboxDeployment) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerBlackboxDeployment) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_children_blackbox_deployment_group(&self, ptr: TableRowPointerBlackboxDeployment) -> &[TableRowPointerBlackboxDeploymentGroup] {
        &self.c_children_blackbox_deployment_group[ptr.0]
    }

    pub fn c_children_blackbox_deployment_service_registration(&self, ptr: TableRowPointerBlackboxDeployment) -> &[TableRowPointerBlackboxDeploymentServiceRegistration] {
        &self.c_children_blackbox_deployment_service_registration[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentGroup {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentGroup> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentGroup(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> &TableRowBlackboxDeploymentGroup {
        &self.rows[ptr.0]
    }

    pub fn c_group_name(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> &::std::string::String {
        &self.c_group_name[ptr.0]
    }

    pub fn c_count(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> i64 {
        self.c_count[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_placement(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> &::std::string::String {
        &self.c_placement[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> TableRowPointerBlackboxDeployment {
        self.c_parent[ptr.0]
    }

    pub fn c_children_blackbox_deployment_port(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> &[TableRowPointerBlackboxDeploymentPort] {
        &self.c_children_blackbox_deployment_port[ptr.0]
    }

    pub fn c_children_blackbox_deployment_task(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> &[TableRowPointerBlackboxDeploymentTask] {
        &self.c_children_blackbox_deployment_task[ptr.0]
    }

    pub fn c_children_blackbox_deployment_service_instance(&self, ptr: TableRowPointerBlackboxDeploymentGroup) -> &[TableRowPointerBlackboxDeploymentServiceInstance] {
        &self.c_children_blackbox_deployment_service_instance[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentLocalFile {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentLocalFile> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentLocalFile(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentLocalFile) -> &TableRowBlackboxDeploymentLocalFile {
        &self.rows[ptr.0]
    }

    pub fn c_local_file_name(&self, ptr: TableRowPointerBlackboxDeploymentLocalFile) -> &::std::string::String {
        &self.c_local_file_name[ptr.0]
    }

    pub fn c_local_file_contents(&self, ptr: TableRowPointerBlackboxDeploymentLocalFile) -> &::std::string::String {
        &self.c_local_file_contents[ptr.0]
    }

    pub fn c_mode(&self, ptr: TableRowPointerBlackboxDeploymentLocalFile) -> &::std::string::String {
        &self.c_mode[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentLocalFile) -> TableRowPointerBlackboxDeploymentTask {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentPort {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentPort> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentPort(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentPort) -> &TableRowBlackboxDeploymentPort {
        &self.rows[ptr.0]
    }

    pub fn c_port(&self, ptr: TableRowPointerBlackboxDeploymentPort) -> i64 {
        self.c_port[ptr.0]
    }

    pub fn c_port_description(&self, ptr: TableRowPointerBlackboxDeploymentPort) -> &::std::string::String {
        &self.c_port_description[ptr.0]
    }

    pub fn c_protocol(&self, ptr: TableRowPointerBlackboxDeploymentPort) -> &::std::string::String {
        &self.c_protocol[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentPort) -> TableRowPointerBlackboxDeploymentGroup {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_blackbox_deployment_service_instance__port(&self, ptr: TableRowPointerBlackboxDeploymentPort) -> &[TableRowPointerBlackboxDeploymentServiceInstance] {
        &self.c_referrers_blackbox_deployment_service_instance__port[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentServiceInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentServiceInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentServiceInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentServiceInstance) -> &TableRowBlackboxDeploymentServiceInstance {
        &self.rows[ptr.0]
    }

    pub fn c_service_registration(&self, ptr: TableRowPointerBlackboxDeploymentServiceInstance) -> TableRowPointerBlackboxDeploymentServiceRegistration {
        self.c_service_registration[ptr.0]
    }

    pub fn c_port(&self, ptr: TableRowPointerBlackboxDeploymentServiceInstance) -> TableRowPointerBlackboxDeploymentPort {
        self.c_port[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentServiceInstance) -> TableRowPointerBlackboxDeploymentGroup {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentServiceRegistration {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentServiceRegistration> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentServiceRegistration(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentServiceRegistration) -> &TableRowBlackboxDeploymentServiceRegistration {
        &self.rows[ptr.0]
    }

    pub fn c_service_name(&self, ptr: TableRowPointerBlackboxDeploymentServiceRegistration) -> &::std::string::String {
        &self.c_service_name[ptr.0]
    }

    pub fn c_scrape_prometheus_metrics(&self, ptr: TableRowPointerBlackboxDeploymentServiceRegistration) -> bool {
        self.c_scrape_prometheus_metrics[ptr.0]
    }

    pub fn c_prometheus_metrics_path(&self, ptr: TableRowPointerBlackboxDeploymentServiceRegistration) -> &::std::string::String {
        &self.c_prometheus_metrics_path[ptr.0]
    }

    pub fn c_min_instances(&self, ptr: TableRowPointerBlackboxDeploymentServiceRegistration) -> i64 {
        self.c_min_instances[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentServiceRegistration) -> TableRowPointerBlackboxDeployment {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_blackbox_deployment_service_instance__service_registration(&self, ptr: TableRowPointerBlackboxDeploymentServiceRegistration) -> &[TableRowPointerBlackboxDeploymentServiceInstance] {
        &self.c_referrers_blackbox_deployment_service_instance__service_registration[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentTask {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentTask> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentTask(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> &TableRowBlackboxDeploymentTask {
        &self.rows[ptr.0]
    }

    pub fn c_task_name(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> &::std::string::String {
        &self.c_task_name[ptr.0]
    }

    pub fn c_docker_image(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> TableRowPointerDockerImagePin {
        self.c_docker_image[ptr.0]
    }

    pub fn c_docker_image_set(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> TableRowPointerDockerImageSet {
        self.c_docker_image_set[ptr.0]
    }

    pub fn c_memory_mb(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> i64 {
        self.c_memory_mb[ptr.0]
    }

    pub fn c_memory_oversubscription_mb(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> i64 {
        self.c_memory_oversubscription_mb[ptr.0]
    }

    pub fn c_entrypoint(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> &::std::string::String {
        &self.c_entrypoint[ptr.0]
    }

    pub fn c_args(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> &::std::string::String {
        &self.c_args[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> TableRowPointerBlackboxDeploymentGroup {
        self.c_parent[ptr.0]
    }

    pub fn c_children_blackbox_deployment_task_mount(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> &[TableRowPointerBlackboxDeploymentTaskMount] {
        &self.c_children_blackbox_deployment_task_mount[ptr.0]
    }

    pub fn c_children_blackbox_deployment_vault_secret(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> &[TableRowPointerBlackboxDeploymentVaultSecret] {
        &self.c_children_blackbox_deployment_vault_secret[ptr.0]
    }

    pub fn c_children_blackbox_deployment_local_file(&self, ptr: TableRowPointerBlackboxDeploymentTask) -> &[TableRowPointerBlackboxDeploymentLocalFile] {
        &self.c_children_blackbox_deployment_local_file[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentTaskMount {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentTaskMount> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentTaskMount(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentTaskMount) -> &TableRowBlackboxDeploymentTaskMount {
        &self.rows[ptr.0]
    }

    pub fn c_target_path(&self, ptr: TableRowPointerBlackboxDeploymentTaskMount) -> &::std::string::String {
        &self.c_target_path[ptr.0]
    }

    pub fn c_server_volume(&self, ptr: TableRowPointerBlackboxDeploymentTaskMount) -> TableRowPointerServerVolume {
        self.c_server_volume[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentTaskMount) -> TableRowPointerBlackboxDeploymentTask {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionBlackboxDeploymentVaultSecret {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerBlackboxDeploymentVaultSecret> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerBlackboxDeploymentVaultSecret(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerBlackboxDeploymentVaultSecret) -> &TableRowBlackboxDeploymentVaultSecret {
        &self.rows[ptr.0]
    }

    pub fn c_secret_name(&self, ptr: TableRowPointerBlackboxDeploymentVaultSecret) -> &::std::string::String {
        &self.c_secret_name[ptr.0]
    }

    pub fn c_target_file_name(&self, ptr: TableRowPointerBlackboxDeploymentVaultSecret) -> &::std::string::String {
        &self.c_target_file_name[ptr.0]
    }

    pub fn c_target_env_var_name(&self, ptr: TableRowPointerBlackboxDeploymentVaultSecret) -> &::std::string::String {
        &self.c_target_env_var_name[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerBlackboxDeploymentVaultSecret) -> TableRowPointerBlackboxDeploymentTask {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionChDeployment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChDeployment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChDeployment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChDeployment) -> &TableRowChDeployment {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerChDeployment) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerChDeployment) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerChDeployment) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerChDeployment) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerChDeployment) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_docker_image(&self, ptr: TableRowPointerChDeployment) -> TableRowPointerDockerImagePin {
        self.c_docker_image[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerChDeployment) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_keeper(&self, ptr: TableRowPointerChDeployment) -> TableRowPointerChKeeperDeployment {
        self.c_keeper[ptr.0]
    }

    pub fn c_extra_memory_mb(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_extra_memory_mb[ptr.0]
    }

    pub fn c_mark_cache_size_mb(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_mark_cache_size_mb[ptr.0]
    }

    pub fn c_index_mark_cache_size_mb(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_index_mark_cache_size_mb[ptr.0]
    }

    pub fn c_uncompressed_cache_size_mb(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_uncompressed_cache_size_mb[ptr.0]
    }

    pub fn c_compiled_expression_cache_size_mb(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_compiled_expression_cache_size_mb[ptr.0]
    }

    pub fn c_query_cache_size_mb(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_query_cache_size_mb[ptr.0]
    }

    pub fn c_max_thread_pool_size(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_max_thread_pool_size[ptr.0]
    }

    pub fn c_max_concurrent_queries(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_max_concurrent_queries[ptr.0]
    }

    pub fn c_merge_max_block_size(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_merge_max_block_size[ptr.0]
    }

    pub fn c_max_bytes_to_merge_at_max_space_in_pool_mb(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_max_bytes_to_merge_at_max_space_in_pool_mb[ptr.0]
    }

    pub fn c_max_query_execution_time_seconds(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_max_query_execution_time_seconds[ptr.0]
    }

    pub fn c_queue_max_wait_ms(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_queue_max_wait_ms[ptr.0]
    }

    pub fn c_distribute_over_dcs(&self, ptr: TableRowPointerChDeployment) -> bool {
        self.c_distribute_over_dcs[ptr.0]
    }

    pub fn c_native_port(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_native_port[ptr.0]
    }

    pub fn c_http_port(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_http_port[ptr.0]
    }

    pub fn c_replication_port(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_replication_port[ptr.0]
    }

    pub fn c_prometheus_port(&self, ptr: TableRowPointerChDeployment) -> i64 {
        self.c_prometheus_port[ptr.0]
    }

    pub fn c_children_ch_deployment_instance(&self, ptr: TableRowPointerChDeployment) -> &[TableRowPointerChDeploymentInstance] {
        &self.c_children_ch_deployment_instance[ptr.0]
    }

    pub fn c_children_ch_deployment_schemas(&self, ptr: TableRowPointerChDeployment) -> &[TableRowPointerChDeploymentSchemas] {
        &self.c_children_ch_deployment_schemas[ptr.0]
    }

}

impl TableDefinitionChDeploymentInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChDeploymentInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChDeploymentInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChDeploymentInstance) -> &TableRowChDeploymentInstance {
        &self.rows[ptr.0]
    }

    pub fn c_instance_id(&self, ptr: TableRowPointerChDeploymentInstance) -> i64 {
        self.c_instance_id[ptr.0]
    }

    pub fn c_ch_server(&self, ptr: TableRowPointerChDeploymentInstance) -> TableRowPointerServerVolume {
        self.c_ch_server[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChDeploymentInstance) -> TableRowPointerChDeployment {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionChDeploymentSchemas {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChDeploymentSchemas> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChDeploymentSchemas(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChDeploymentSchemas) -> &TableRowChDeploymentSchemas {
        &self.rows[ptr.0]
    }

    pub fn c_db_name(&self, ptr: TableRowPointerChDeploymentSchemas) -> &::std::string::String {
        &self.c_db_name[ptr.0]
    }

    pub fn c_ch_schema(&self, ptr: TableRowPointerChDeploymentSchemas) -> TableRowPointerChSchema {
        self.c_ch_schema[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChDeploymentSchemas) -> TableRowPointerChDeployment {
        self.c_parent[ptr.0]
    }

    pub fn c_children_ch_nats_stream_import(&self, ptr: TableRowPointerChDeploymentSchemas) -> &[TableRowPointerChNatsStreamImport] {
        &self.c_children_ch_nats_stream_import[ptr.0]
    }

}

impl TableDefinitionChKeeperDeployment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChKeeperDeployment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChKeeperDeployment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChKeeperDeployment) -> &TableRowChKeeperDeployment {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerChKeeperDeployment) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerChKeeperDeployment) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerChKeeperDeployment) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerChKeeperDeployment) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerChKeeperDeployment) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_docker_image(&self, ptr: TableRowPointerChKeeperDeployment) -> TableRowPointerDockerImagePin {
        self.c_docker_image[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerChKeeperDeployment) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_distribute_over_dcs(&self, ptr: TableRowPointerChKeeperDeployment) -> bool {
        self.c_distribute_over_dcs[ptr.0]
    }

    pub fn c_memory_mb(&self, ptr: TableRowPointerChKeeperDeployment) -> i64 {
        self.c_memory_mb[ptr.0]
    }

    pub fn c_keeper_port(&self, ptr: TableRowPointerChKeeperDeployment) -> i64 {
        self.c_keeper_port[ptr.0]
    }

    pub fn c_raft_port(&self, ptr: TableRowPointerChKeeperDeployment) -> i64 {
        self.c_raft_port[ptr.0]
    }

    pub fn c_prometheus_port(&self, ptr: TableRowPointerChKeeperDeployment) -> i64 {
        self.c_prometheus_port[ptr.0]
    }

    pub fn c_children_ch_keeper_deployment_instance(&self, ptr: TableRowPointerChKeeperDeployment) -> &[TableRowPointerChKeeperDeploymentInstance] {
        &self.c_children_ch_keeper_deployment_instance[ptr.0]
    }

    pub fn c_referrers_ch_deployment__keeper(&self, ptr: TableRowPointerChKeeperDeployment) -> &[TableRowPointerChDeployment] {
        &self.c_referrers_ch_deployment__keeper[ptr.0]
    }

}

impl TableDefinitionChKeeperDeploymentInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChKeeperDeploymentInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChKeeperDeploymentInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChKeeperDeploymentInstance) -> &TableRowChKeeperDeploymentInstance {
        &self.rows[ptr.0]
    }

    pub fn c_instance_id(&self, ptr: TableRowPointerChKeeperDeploymentInstance) -> i64 {
        self.c_instance_id[ptr.0]
    }

    pub fn c_keeper_server(&self, ptr: TableRowPointerChKeeperDeploymentInstance) -> TableRowPointerServerVolume {
        self.c_keeper_server[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChKeeperDeploymentInstance) -> TableRowPointerChKeeperDeployment {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionChMigration {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChMigration> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChMigration(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChMigration) -> &TableRowChMigration {
        &self.rows[ptr.0]
    }

    pub fn c_time(&self, ptr: TableRowPointerChMigration) -> i64 {
        self.c_time[ptr.0]
    }

    pub fn c_upgrade(&self, ptr: TableRowPointerChMigration) -> &::std::string::String {
        &self.c_upgrade[ptr.0]
    }

    pub fn c_downgrade(&self, ptr: TableRowPointerChMigration) -> &::std::string::String {
        &self.c_downgrade[ptr.0]
    }

    pub fn c_needs_admin(&self, ptr: TableRowPointerChMigration) -> bool {
        self.c_needs_admin[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChMigration) -> TableRowPointerChSchema {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionChMutator {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChMutator> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChMutator(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChMutator) -> &TableRowChMutator {
        &self.rows[ptr.0]
    }

    pub fn c_mutator_name(&self, ptr: TableRowPointerChMutator) -> &::std::string::String {
        &self.c_mutator_name[ptr.0]
    }

    pub fn c_mutator_expression(&self, ptr: TableRowPointerChMutator) -> &::std::string::String {
        &self.c_mutator_expression[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChMutator) -> TableRowPointerChSchema {
        self.c_parent[ptr.0]
    }

    pub fn c_children_ch_mutator_test(&self, ptr: TableRowPointerChMutator) -> &[TableRowPointerChMutatorTest] {
        &self.c_children_ch_mutator_test[ptr.0]
    }

}

impl TableDefinitionChMutatorTest {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChMutatorTest> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChMutatorTest(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChMutatorTest) -> &TableRowChMutatorTest {
        &self.rows[ptr.0]
    }

    pub fn c_arguments(&self, ptr: TableRowPointerChMutatorTest) -> &::std::string::String {
        &self.c_arguments[ptr.0]
    }

    pub fn c_test_dataset(&self, ptr: TableRowPointerChMutatorTest) -> TableRowPointerChTestDataset {
        self.c_test_dataset[ptr.0]
    }

    pub fn c_resulting_data(&self, ptr: TableRowPointerChMutatorTest) -> &::std::string::String {
        &self.c_resulting_data[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChMutatorTest) -> TableRowPointerChMutator {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionChNatsStreamImport {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChNatsStreamImport> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChNatsStreamImport(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChNatsStreamImport) -> &TableRowChNatsStreamImport {
        &self.rows[ptr.0]
    }

    pub fn c_consumer_name(&self, ptr: TableRowPointerChNatsStreamImport) -> &::std::string::String {
        &self.c_consumer_name[ptr.0]
    }

    pub fn c_into_table(&self, ptr: TableRowPointerChNatsStreamImport) -> &::std::string::String {
        &self.c_into_table[ptr.0]
    }

    pub fn c_stream(&self, ptr: TableRowPointerChNatsStreamImport) -> TableRowPointerNatsJetstreamStream {
        self.c_stream[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChNatsStreamImport) -> TableRowPointerChDeploymentSchemas {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionChQuery {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChQuery> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChQuery(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChQuery) -> &TableRowChQuery {
        &self.rows[ptr.0]
    }

    pub fn c_query_name(&self, ptr: TableRowPointerChQuery) -> &::std::string::String {
        &self.c_query_name[ptr.0]
    }

    pub fn c_query_expression(&self, ptr: TableRowPointerChQuery) -> &::std::string::String {
        &self.c_query_expression[ptr.0]
    }

    pub fn c_opt_fields(&self, ptr: TableRowPointerChQuery) -> &::std::string::String {
        &self.c_opt_fields[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChQuery) -> TableRowPointerChSchema {
        self.c_parent[ptr.0]
    }

    pub fn c_children_ch_query_test(&self, ptr: TableRowPointerChQuery) -> &[TableRowPointerChQueryTest] {
        &self.c_children_ch_query_test[ptr.0]
    }

}

impl TableDefinitionChQueryTest {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChQueryTest> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChQueryTest(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChQueryTest) -> &TableRowChQueryTest {
        &self.rows[ptr.0]
    }

    pub fn c_arguments(&self, ptr: TableRowPointerChQueryTest) -> &::std::string::String {
        &self.c_arguments[ptr.0]
    }

    pub fn c_outputs(&self, ptr: TableRowPointerChQueryTest) -> &::std::string::String {
        &self.c_outputs[ptr.0]
    }

    pub fn c_test_dataset(&self, ptr: TableRowPointerChQueryTest) -> TableRowPointerChTestDataset {
        self.c_test_dataset[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChQueryTest) -> TableRowPointerChQuery {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionChSchema {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChSchema> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChSchema(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChSchema) -> &TableRowChSchema {
        &self.rows[ptr.0]
    }

    pub fn c_schema_name(&self, ptr: TableRowPointerChSchema) -> &::std::string::String {
        &self.c_schema_name[ptr.0]
    }

    pub fn c_children_ch_migration(&self, ptr: TableRowPointerChSchema) -> &[TableRowPointerChMigration] {
        &self.c_children_ch_migration[ptr.0]
    }

    pub fn c_children_ch_query(&self, ptr: TableRowPointerChSchema) -> &[TableRowPointerChQuery] {
        &self.c_children_ch_query[ptr.0]
    }

    pub fn c_children_ch_mutator(&self, ptr: TableRowPointerChSchema) -> &[TableRowPointerChMutator] {
        &self.c_children_ch_mutator[ptr.0]
    }

    pub fn c_children_ch_test_dataset(&self, ptr: TableRowPointerChSchema) -> &[TableRowPointerChTestDataset] {
        &self.c_children_ch_test_dataset[ptr.0]
    }

    pub fn c_referrers_ch_deployment_schemas__ch_schema(&self, ptr: TableRowPointerChSchema) -> &[TableRowPointerChDeploymentSchemas] {
        &self.c_referrers_ch_deployment_schemas__ch_schema[ptr.0]
    }

    pub fn c_referrers_backend_application_ch_shard__ch_schema(&self, ptr: TableRowPointerChSchema) -> &[TableRowPointerBackendApplicationChShard] {
        &self.c_referrers_backend_application_ch_shard__ch_schema[ptr.0]
    }

}

impl TableDefinitionChTestDataset {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerChTestDataset> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerChTestDataset(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerChTestDataset) -> &TableRowChTestDataset {
        &self.rows[ptr.0]
    }

    pub fn c_dataset_name(&self, ptr: TableRowPointerChTestDataset) -> &::std::string::String {
        &self.c_dataset_name[ptr.0]
    }

    pub fn c_dataset_contents(&self, ptr: TableRowPointerChTestDataset) -> &::std::string::String {
        &self.c_dataset_contents[ptr.0]
    }

    pub fn c_min_time(&self, ptr: TableRowPointerChTestDataset) -> i64 {
        self.c_min_time[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerChTestDataset) -> TableRowPointerChSchema {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_ch_query_test__test_dataset(&self, ptr: TableRowPointerChTestDataset) -> &[TableRowPointerChQueryTest] {
        &self.c_referrers_ch_query_test__test_dataset[ptr.0]
    }

    pub fn c_referrers_ch_mutator_test__test_dataset(&self, ptr: TableRowPointerChTestDataset) -> &[TableRowPointerChMutatorTest] {
        &self.c_referrers_ch_mutator_test__test_dataset[ptr.0]
    }

}

impl TableDefinitionDatacenter {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerDatacenter> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerDatacenter(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerDatacenter) -> &TableRowDatacenter {
        &self.rows[ptr.0]
    }

    pub fn c_dc_name(&self, ptr: TableRowPointerDatacenter) -> &::std::string::String {
        &self.c_dc_name[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerDatacenter) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_network_cidr(&self, ptr: TableRowPointerDatacenter) -> &::std::string::String {
        &self.c_network_cidr[ptr.0]
    }

    pub fn c_allow_small_subnets(&self, ptr: TableRowPointerDatacenter) -> bool {
        self.c_allow_small_subnets[ptr.0]
    }

    pub fn c_implementation(&self, ptr: TableRowPointerDatacenter) -> &::std::string::String {
        &self.c_implementation[ptr.0]
    }

    pub fn c_implementation_settings(&self, ptr: TableRowPointerDatacenter) -> &::std::string::String {
        &self.c_implementation_settings[ptr.0]
    }

    pub fn c_default_server_kind(&self, ptr: TableRowPointerDatacenter) -> TableRowPointerServerKind {
        self.c_default_server_kind[ptr.0]
    }

    pub fn c_disk_ids_policy(&self, ptr: TableRowPointerDatacenter) -> &::std::string::String {
        &self.c_disk_ids_policy[ptr.0]
    }

    pub fn c_router_subnet_vlan_id(&self, ptr: TableRowPointerDatacenter) -> i64 {
        self.c_router_subnet_vlan_id[ptr.0]
    }

    pub fn c_referrers_server__dc(&self, ptr: TableRowPointerDatacenter) -> &[TableRowPointerServer] {
        &self.c_referrers_server__dc[ptr.0]
    }

}

impl TableDefinitionDiskKind {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerDiskKind> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerDiskKind(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerDiskKind) -> &TableRowDiskKind {
        &self.rows[ptr.0]
    }

    pub fn c_kind(&self, ptr: TableRowPointerDiskKind) -> &::std::string::String {
        &self.c_kind[ptr.0]
    }

    pub fn c_medium(&self, ptr: TableRowPointerDiskKind) -> &::std::string::String {
        &self.c_medium[ptr.0]
    }

    pub fn c_is_elastic(&self, ptr: TableRowPointerDiskKind) -> bool {
        self.c_is_elastic[ptr.0]
    }

    pub fn c_min_capacity_bytes(&self, ptr: TableRowPointerDiskKind) -> i64 {
        self.c_min_capacity_bytes[ptr.0]
    }

    pub fn c_max_capacity_bytes(&self, ptr: TableRowPointerDiskKind) -> i64 {
        self.c_max_capacity_bytes[ptr.0]
    }

    pub fn c_capacity_bytes(&self, ptr: TableRowPointerDiskKind) -> i64 {
        self.c_capacity_bytes[ptr.0]
    }

    pub fn c_has_extra_config(&self, ptr: TableRowPointerDiskKind) -> bool {
        self.c_has_extra_config[ptr.0]
    }

    pub fn c_non_eligible_reason(&self, ptr: TableRowPointerDiskKind) -> &::std::string::String {
        &self.c_non_eligible_reason[ptr.0]
    }

    pub fn c_referrers_server_disk__disk_kind(&self, ptr: TableRowPointerDiskKind) -> &[TableRowPointerServerDisk] {
        &self.c_referrers_server_disk__disk_kind[ptr.0]
    }

}

impl TableDefinitionDockerImage {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerDockerImage> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerDockerImage(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerDockerImage) -> &TableRowDockerImage {
        &self.rows[ptr.0]
    }

    pub fn c_checksum(&self, ptr: TableRowPointerDockerImage) -> &::std::string::String {
        &self.c_checksum[ptr.0]
    }

    pub fn c_image_set(&self, ptr: TableRowPointerDockerImage) -> TableRowPointerDockerImageSet {
        self.c_image_set[ptr.0]
    }

    pub fn c_repository(&self, ptr: TableRowPointerDockerImage) -> &::std::string::String {
        &self.c_repository[ptr.0]
    }

    pub fn c_architecture(&self, ptr: TableRowPointerDockerImage) -> &::std::string::String {
        &self.c_architecture[ptr.0]
    }

    pub fn c_tag(&self, ptr: TableRowPointerDockerImage) -> &::std::string::String {
        &self.c_tag[ptr.0]
    }

    pub fn c_referrers_docker_image_pin_images__checksum(&self, ptr: TableRowPointerDockerImage) -> &[TableRowPointerDockerImagePinImages] {
        &self.c_referrers_docker_image_pin_images__checksum[ptr.0]
    }

}

impl TableDefinitionDockerImagePin {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerDockerImagePin> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerDockerImagePin(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerDockerImagePin) -> &TableRowDockerImagePin {
        &self.rows[ptr.0]
    }

    pub fn c_pin_name(&self, ptr: TableRowPointerDockerImagePin) -> &::std::string::String {
        &self.c_pin_name[ptr.0]
    }

    pub fn c_children_docker_image_pin_images(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerDockerImagePinImages] {
        &self.c_children_docker_image_pin_images[ptr.0]
    }

    pub fn c_referrers_region__docker_image_external_lb(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerRegion] {
        &self.c_referrers_region__docker_image_external_lb[ptr.0]
    }

    pub fn c_referrers_docker_registry_instance__docker_image(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerDockerRegistryInstance] {
        &self.c_referrers_docker_registry_instance__docker_image[ptr.0]
    }

    pub fn c_referrers_pg_deployment__docker_image_pg(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerPgDeployment] {
        &self.c_referrers_pg_deployment__docker_image_pg[ptr.0]
    }

    pub fn c_referrers_pg_deployment__docker_image_haproxy(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerPgDeployment] {
        &self.c_referrers_pg_deployment__docker_image_haproxy[ptr.0]
    }

    pub fn c_referrers_pg_deployment__docker_image_pg_exporter(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerPgDeployment] {
        &self.c_referrers_pg_deployment__docker_image_pg_exporter[ptr.0]
    }

    pub fn c_referrers_ch_deployment__docker_image(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerChDeployment] {
        &self.c_referrers_ch_deployment__docker_image[ptr.0]
    }

    pub fn c_referrers_ch_keeper_deployment__docker_image(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerChKeeperDeployment] {
        &self.c_referrers_ch_keeper_deployment__docker_image[ptr.0]
    }

    pub fn c_referrers_nats_cluster__docker_image_nats(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerNatsCluster] {
        &self.c_referrers_nats_cluster__docker_image_nats[ptr.0]
    }

    pub fn c_referrers_nats_cluster__docker_image_nats_exporter(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerNatsCluster] {
        &self.c_referrers_nats_cluster__docker_image_nats_exporter[ptr.0]
    }

    pub fn c_referrers_minio_cluster__docker_image_minio(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerMinioCluster] {
        &self.c_referrers_minio_cluster__docker_image_minio[ptr.0]
    }

    pub fn c_referrers_minio_cluster__docker_image_minio_mc(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerMinioCluster] {
        &self.c_referrers_minio_cluster__docker_image_minio_mc[ptr.0]
    }

    pub fn c_referrers_minio_cluster__docker_image_nginx(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerMinioCluster] {
        &self.c_referrers_minio_cluster__docker_image_nginx[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster__docker_image_prometheus(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerMonitoringCluster] {
        &self.c_referrers_monitoring_cluster__docker_image_prometheus[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster__docker_image_alertmanager(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerMonitoringCluster] {
        &self.c_referrers_monitoring_cluster__docker_image_alertmanager[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster__docker_image_victoriametrics(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerMonitoringCluster] {
        &self.c_referrers_monitoring_cluster__docker_image_victoriametrics[ptr.0]
    }

    pub fn c_referrers_grafana__docker_image_grafana(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerGrafana] {
        &self.c_referrers_grafana__docker_image_grafana[ptr.0]
    }

    pub fn c_referrers_grafana__docker_image_promxy(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerGrafana] {
        &self.c_referrers_grafana__docker_image_promxy[ptr.0]
    }

    pub fn c_referrers_loki_cluster__docker_image_loki(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerLokiCluster] {
        &self.c_referrers_loki_cluster__docker_image_loki[ptr.0]
    }

    pub fn c_referrers_tempo_cluster__docker_image(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerTempoCluster] {
        &self.c_referrers_tempo_cluster__docker_image[ptr.0]
    }

    pub fn c_referrers_blackbox_deployment_task__docker_image(&self, ptr: TableRowPointerDockerImagePin) -> &[TableRowPointerBlackboxDeploymentTask] {
        &self.c_referrers_blackbox_deployment_task__docker_image[ptr.0]
    }

}

impl TableDefinitionDockerImagePinImages {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerDockerImagePinImages> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerDockerImagePinImages(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerDockerImagePinImages) -> &TableRowDockerImagePinImages {
        &self.rows[ptr.0]
    }

    pub fn c_checksum(&self, ptr: TableRowPointerDockerImagePinImages) -> TableRowPointerDockerImage {
        self.c_checksum[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerDockerImagePinImages) -> TableRowPointerDockerImagePin {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionDockerImageSet {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerDockerImageSet> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerDockerImageSet(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerDockerImageSet) -> &TableRowDockerImageSet {
        &self.rows[ptr.0]
    }

    pub fn c_set_name(&self, ptr: TableRowPointerDockerImageSet) -> &::std::string::String {
        &self.c_set_name[ptr.0]
    }

    pub fn c_referrers_docker_image__image_set(&self, ptr: TableRowPointerDockerImageSet) -> &[TableRowPointerDockerImage] {
        &self.c_referrers_docker_image__image_set[ptr.0]
    }

    pub fn c_referrers_blackbox_deployment_task__docker_image_set(&self, ptr: TableRowPointerDockerImageSet) -> &[TableRowPointerBlackboxDeploymentTask] {
        &self.c_referrers_blackbox_deployment_task__docker_image_set[ptr.0]
    }

}

impl TableDefinitionDockerRegistryInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerDockerRegistryInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerDockerRegistryInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerDockerRegistryInstance) -> &TableRowDockerRegistryInstance {
        &self.rows[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerDockerRegistryInstance) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_minio_bucket(&self, ptr: TableRowPointerDockerRegistryInstance) -> TableRowPointerMinioBucket {
        self.c_minio_bucket[ptr.0]
    }

    pub fn c_memory_mb(&self, ptr: TableRowPointerDockerRegistryInstance) -> i64 {
        self.c_memory_mb[ptr.0]
    }

    pub fn c_docker_image(&self, ptr: TableRowPointerDockerRegistryInstance) -> TableRowPointerDockerImagePin {
        self.c_docker_image[ptr.0]
    }

}

impl TableDefinitionFrontendApplication {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerFrontendApplication> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerFrontendApplication(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerFrontendApplication) -> &TableRowFrontendApplication {
        &self.rows[ptr.0]
    }

    pub fn c_application_name(&self, ptr: TableRowPointerFrontendApplication) -> &::std::string::String {
        &self.c_application_name[ptr.0]
    }

    pub fn c_build_environment(&self, ptr: TableRowPointerFrontendApplication) -> TableRowPointerRustCompilationEnvironment {
        self.c_build_environment[ptr.0]
    }

    pub fn c_index_page_title(&self, ptr: TableRowPointerFrontendApplication) -> &::std::string::String {
        &self.c_index_page_title[ptr.0]
    }

    pub fn c_children_frontend_page(&self, ptr: TableRowPointerFrontendApplication) -> &[TableRowPointerFrontendPage] {
        &self.c_children_frontend_page[ptr.0]
    }

    pub fn c_children_frontend_application_used_endpoint(&self, ptr: TableRowPointerFrontendApplication) -> &[TableRowPointerFrontendApplicationUsedEndpoint] {
        &self.c_children_frontend_application_used_endpoint[ptr.0]
    }

    pub fn c_children_frontend_application_external_link(&self, ptr: TableRowPointerFrontendApplication) -> &[TableRowPointerFrontendApplicationExternalLink] {
        &self.c_children_frontend_application_external_link[ptr.0]
    }

    pub fn c_children_frontend_application_external_page(&self, ptr: TableRowPointerFrontendApplication) -> &[TableRowPointerFrontendApplicationExternalPage] {
        &self.c_children_frontend_application_external_page[ptr.0]
    }

    pub fn c_referrers_frontend_application_deployment__application_name(&self, ptr: TableRowPointerFrontendApplication) -> &[TableRowPointerFrontendApplicationDeployment] {
        &self.c_referrers_frontend_application_deployment__application_name[ptr.0]
    }

}

impl TableDefinitionFrontendApplicationDeployment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerFrontendApplicationDeployment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerFrontendApplicationDeployment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &TableRowFrontendApplicationDeployment {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_application_name(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> TableRowPointerFrontendApplication {
        self.c_application_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_explicit_endpoint_wiring(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &::std::string::String {
        &self.c_explicit_endpoint_wiring[ptr.0]
    }

    pub fn c_workload_backend_architecture(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &::std::string::String {
        &self.c_workload_backend_architecture[ptr.0]
    }

    pub fn c_placement(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &::std::string::String {
        &self.c_placement[ptr.0]
    }

    pub fn c_link_wiring(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &::std::string::String {
        &self.c_link_wiring[ptr.0]
    }

    pub fn c_page_wiring(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &::std::string::String {
        &self.c_page_wiring[ptr.0]
    }

    pub fn c_count(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> i64 {
        self.c_count[ptr.0]
    }

    pub fn c_http_port(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> i64 {
        self.c_http_port[ptr.0]
    }

    pub fn c_memory_mb(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> i64 {
        self.c_memory_mb[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_referrers_frontend_application_deployment_ingress__deployment(&self, ptr: TableRowPointerFrontendApplicationDeployment) -> &[TableRowPointerFrontendApplicationDeploymentIngress] {
        &self.c_referrers_frontend_application_deployment_ingress__deployment[ptr.0]
    }

}

impl TableDefinitionFrontendApplicationDeploymentIngress {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerFrontendApplicationDeploymentIngress> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerFrontendApplicationDeploymentIngress(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerFrontendApplicationDeploymentIngress) -> &TableRowFrontendApplicationDeploymentIngress {
        &self.rows[ptr.0]
    }

    pub fn c_deployment(&self, ptr: TableRowPointerFrontendApplicationDeploymentIngress) -> TableRowPointerFrontendApplicationDeployment {
        self.c_deployment[ptr.0]
    }

    pub fn c_mountpoint(&self, ptr: TableRowPointerFrontendApplicationDeploymentIngress) -> &::std::string::String {
        &self.c_mountpoint[ptr.0]
    }

    pub fn c_subdomain(&self, ptr: TableRowPointerFrontendApplicationDeploymentIngress) -> &::std::string::String {
        &self.c_subdomain[ptr.0]
    }

    pub fn c_tld(&self, ptr: TableRowPointerFrontendApplicationDeploymentIngress) -> TableRowPointerTld {
        self.c_tld[ptr.0]
    }

}

impl TableDefinitionFrontendApplicationExternalLink {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerFrontendApplicationExternalLink> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerFrontendApplicationExternalLink(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerFrontendApplicationExternalLink) -> &TableRowFrontendApplicationExternalLink {
        &self.rows[ptr.0]
    }

    pub fn c_link_name(&self, ptr: TableRowPointerFrontendApplicationExternalLink) -> &::std::string::String {
        &self.c_link_name[ptr.0]
    }

    pub fn c_backend_endpoint(&self, ptr: TableRowPointerFrontendApplicationExternalLink) -> TableRowPointerBackendHttpEndpoint {
        self.c_backend_endpoint[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerFrontendApplicationExternalLink) -> TableRowPointerFrontendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionFrontendApplicationExternalPage {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerFrontendApplicationExternalPage> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerFrontendApplicationExternalPage(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerFrontendApplicationExternalPage) -> &TableRowFrontendApplicationExternalPage {
        &self.rows[ptr.0]
    }

    pub fn c_link_name(&self, ptr: TableRowPointerFrontendApplicationExternalPage) -> &::std::string::String {
        &self.c_link_name[ptr.0]
    }

    pub fn c_frontend_page(&self, ptr: TableRowPointerFrontendApplicationExternalPage) -> TableRowPointerFrontendPage {
        self.c_frontend_page[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerFrontendApplicationExternalPage) -> TableRowPointerFrontendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionFrontendApplicationUsedEndpoint {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerFrontendApplicationUsedEndpoint> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerFrontendApplicationUsedEndpoint(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerFrontendApplicationUsedEndpoint) -> &TableRowFrontendApplicationUsedEndpoint {
        &self.rows[ptr.0]
    }

    pub fn c_endpoint_name(&self, ptr: TableRowPointerFrontendApplicationUsedEndpoint) -> &::std::string::String {
        &self.c_endpoint_name[ptr.0]
    }

    pub fn c_backend_endpoint(&self, ptr: TableRowPointerFrontendApplicationUsedEndpoint) -> TableRowPointerBackendHttpEndpoint {
        self.c_backend_endpoint[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerFrontendApplicationUsedEndpoint) -> TableRowPointerFrontendApplication {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionFrontendPage {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerFrontendPage> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerFrontendPage(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerFrontendPage) -> &TableRowFrontendPage {
        &self.rows[ptr.0]
    }

    pub fn c_page_name(&self, ptr: TableRowPointerFrontendPage) -> &::std::string::String {
        &self.c_page_name[ptr.0]
    }

    pub fn c_path(&self, ptr: TableRowPointerFrontendPage) -> &::std::string::String {
        &self.c_path[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerFrontendPage) -> TableRowPointerFrontendApplication {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_frontend_application_external_page__frontend_page(&self, ptr: TableRowPointerFrontendPage) -> &[TableRowPointerFrontendApplicationExternalPage] {
        &self.c_referrers_frontend_application_external_page__frontend_page[ptr.0]
    }

}

impl TableDefinitionGlobalSettings {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerGlobalSettings> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerGlobalSettings(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerGlobalSettings) -> &TableRowGlobalSettings {
        &self.rows[ptr.0]
    }

    pub fn c_project_name(&self, ptr: TableRowPointerGlobalSettings) -> &::std::string::String {
        &self.c_project_name[ptr.0]
    }

    pub fn c_docker_registry_port(&self, ptr: TableRowPointerGlobalSettings) -> i64 {
        self.c_docker_registry_port[ptr.0]
    }

    pub fn c_docker_registry_service_name(&self, ptr: TableRowPointerGlobalSettings) -> &::std::string::String {
        &self.c_docker_registry_service_name[ptr.0]
    }

    pub fn c_aws_artefacts_s3_bucket_name(&self, ptr: TableRowPointerGlobalSettings) -> &::std::string::String {
        &self.c_aws_artefacts_s3_bucket_name[ptr.0]
    }

    pub fn c_local_docker_cache_port(&self, ptr: TableRowPointerGlobalSettings) -> i64 {
        self.c_local_docker_cache_port[ptr.0]
    }

    pub fn c_admin_email(&self, ptr: TableRowPointerGlobalSettings) -> &::std::string::String {
        &self.c_admin_email[ptr.0]
    }

    pub fn c_google_cloud_project_id(&self, ptr: TableRowPointerGlobalSettings) -> &::std::string::String {
        &self.c_google_cloud_project_id[ptr.0]
    }

    pub fn c_google_cloud_artefacts_bucket_name(&self, ptr: TableRowPointerGlobalSettings) -> &::std::string::String {
        &self.c_google_cloud_artefacts_bucket_name[ptr.0]
    }

    pub fn c_disable_consul_quorum_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_consul_quorum_tests[ptr.0]
    }

    pub fn c_disable_nomad_quorum_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_nomad_quorum_tests[ptr.0]
    }

    pub fn c_disable_vault_quorum_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_vault_quorum_tests[ptr.0]
    }

    pub fn c_disable_dns_quorum_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_dns_quorum_tests[ptr.0]
    }

    pub fn c_disable_deployment_min_server_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_deployment_min_server_tests[ptr.0]
    }

    pub fn c_disable_deployment_min_ingress_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_deployment_min_ingress_tests[ptr.0]
    }

    pub fn c_disable_region_docker_registry_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_region_docker_registry_tests[ptr.0]
    }

    pub fn c_disable_region_monitoring_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_region_monitoring_tests[ptr.0]
    }

    pub fn c_disable_region_tracing_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_region_tracing_tests[ptr.0]
    }

    pub fn c_disable_region_logging_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_region_logging_tests[ptr.0]
    }

    pub fn c_disable_vpn_gateway_tests(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_disable_vpn_gateway_tests[ptr.0]
    }

    pub fn c_hetzner_inter_dc_vlan_id(&self, ptr: TableRowPointerGlobalSettings) -> i64 {
        self.c_hetzner_inter_dc_vlan_id[ptr.0]
    }

    pub fn c_experimental_enable_arm64_support(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_experimental_enable_arm64_support[ptr.0]
    }

    pub fn c_update_edl_public_ips_from_terraform(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_update_edl_public_ips_from_terraform[ptr.0]
    }

    pub fn c_enable_ipv6(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_enable_ipv6[ptr.0]
    }

    pub fn c_force_ipv6(&self, ptr: TableRowPointerGlobalSettings) -> bool {
        self.c_force_ipv6[ptr.0]
    }

}

impl TableDefinitionGrafana {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerGrafana> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerGrafana(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerGrafana) -> &TableRowGrafana {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerGrafana) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerGrafana) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerGrafana) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_placement(&self, ptr: TableRowPointerGrafana) -> &::std::string::String {
        &self.c_placement[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerGrafana) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_docker_image_grafana(&self, ptr: TableRowPointerGrafana) -> TableRowPointerDockerImagePin {
        self.c_docker_image_grafana[ptr.0]
    }

    pub fn c_docker_image_promxy(&self, ptr: TableRowPointerGrafana) -> TableRowPointerDockerImagePin {
        self.c_docker_image_promxy[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerGrafana) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerGrafana) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_port(&self, ptr: TableRowPointerGrafana) -> i64 {
        self.c_port[ptr.0]
    }

    pub fn c_promxy_port(&self, ptr: TableRowPointerGrafana) -> i64 {
        self.c_promxy_port[ptr.0]
    }

    pub fn c_instance_count(&self, ptr: TableRowPointerGrafana) -> i64 {
        self.c_instance_count[ptr.0]
    }

    pub fn c_database(&self, ptr: TableRowPointerGrafana) -> TableRowPointerPgDeploymentUnmanagedDb {
        self.c_database[ptr.0]
    }

    pub fn c_memory_mb(&self, ptr: TableRowPointerGrafana) -> i64 {
        self.c_memory_mb[ptr.0]
    }

    pub fn c_promxy_memory_mb(&self, ptr: TableRowPointerGrafana) -> i64 {
        self.c_promxy_memory_mb[ptr.0]
    }

}

impl TableDefinitionGrafanaDashboard {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerGrafanaDashboard> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerGrafanaDashboard(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerGrafanaDashboard) -> &TableRowGrafanaDashboard {
        &self.rows[ptr.0]
    }

    pub fn c_filename(&self, ptr: TableRowPointerGrafanaDashboard) -> &::std::string::String {
        &self.c_filename[ptr.0]
    }

    pub fn c_contents(&self, ptr: TableRowPointerGrafanaDashboard) -> &::std::string::String {
        &self.c_contents[ptr.0]
    }

}

impl TableDefinitionHttpEndpointDataType {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerHttpEndpointDataType> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerHttpEndpointDataType(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerHttpEndpointDataType) -> &TableRowHttpEndpointDataType {
        &self.rows[ptr.0]
    }

    pub fn c_http_endpoint_data_type(&self, ptr: TableRowPointerHttpEndpointDataType) -> &::std::string::String {
        &self.c_http_endpoint_data_type[ptr.0]
    }

    pub fn c_referrers_backend_http_endpoint__data_type(&self, ptr: TableRowPointerHttpEndpointDataType) -> &[TableRowPointerBackendHttpEndpoint] {
        &self.c_referrers_backend_http_endpoint__data_type[ptr.0]
    }

}

impl TableDefinitionHttpMethods {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerHttpMethods> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerHttpMethods(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerHttpMethods) -> &TableRowHttpMethods {
        &self.rows[ptr.0]
    }

    pub fn c_http_method_name(&self, ptr: TableRowPointerHttpMethods) -> &::std::string::String {
        &self.c_http_method_name[ptr.0]
    }

    pub fn c_referrers_backend_http_endpoint__http_method(&self, ptr: TableRowPointerHttpMethods) -> &[TableRowPointerBackendHttpEndpoint] {
        &self.c_referrers_backend_http_endpoint__http_method[ptr.0]
    }

}

impl TableDefinitionLokiCluster {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerLokiCluster> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerLokiCluster(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerLokiCluster) -> &TableRowLokiCluster {
        &self.rows[ptr.0]
    }

    pub fn c_cluster_name(&self, ptr: TableRowPointerLokiCluster) -> &::std::string::String {
        &self.c_cluster_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerLokiCluster) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerLokiCluster) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerLokiCluster) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_docker_image_loki(&self, ptr: TableRowPointerLokiCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_loki[ptr.0]
    }

    pub fn c_is_region_default(&self, ptr: TableRowPointerLokiCluster) -> bool {
        self.c_is_region_default[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerLokiCluster) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerLokiCluster) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_storage_bucket(&self, ptr: TableRowPointerLokiCluster) -> TableRowPointerMinioBucket {
        self.c_storage_bucket[ptr.0]
    }

    pub fn c_retention_period_days(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_retention_period_days[ptr.0]
    }

    pub fn c_loki_writer_http_port(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_writer_http_port[ptr.0]
    }

    pub fn c_loki_writer_grpc_port(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_writer_grpc_port[ptr.0]
    }

    pub fn c_loki_reader_http_port(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_reader_http_port[ptr.0]
    }

    pub fn c_loki_reader_grpc_port(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_reader_grpc_port[ptr.0]
    }

    pub fn c_loki_backend_http_port(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_backend_http_port[ptr.0]
    }

    pub fn c_loki_backend_grpc_port(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_backend_grpc_port[ptr.0]
    }

    pub fn c_loki_writers(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_writers[ptr.0]
    }

    pub fn c_loki_readers(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_readers[ptr.0]
    }

    pub fn c_writer_placement(&self, ptr: TableRowPointerLokiCluster) -> &::std::string::String {
        &self.c_writer_placement[ptr.0]
    }

    pub fn c_reader_placement(&self, ptr: TableRowPointerLokiCluster) -> &::std::string::String {
        &self.c_reader_placement[ptr.0]
    }

    pub fn c_backend_placement(&self, ptr: TableRowPointerLokiCluster) -> &::std::string::String {
        &self.c_backend_placement[ptr.0]
    }

    pub fn c_loki_reader_memory_mb(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_reader_memory_mb[ptr.0]
    }

    pub fn c_loki_writer_memory_mb(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_writer_memory_mb[ptr.0]
    }

    pub fn c_loki_backend_memory_mb(&self, ptr: TableRowPointerLokiCluster) -> i64 {
        self.c_loki_backend_memory_mb[ptr.0]
    }

}

impl TableDefinitionMinioBucket {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerMinioBucket> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerMinioBucket(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerMinioBucket) -> &TableRowMinioBucket {
        &self.rows[ptr.0]
    }

    pub fn c_bucket_name(&self, ptr: TableRowPointerMinioBucket) -> &::std::string::String {
        &self.c_bucket_name[ptr.0]
    }

    pub fn c_locking_enabled(&self, ptr: TableRowPointerMinioBucket) -> bool {
        self.c_locking_enabled[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerMinioBucket) -> TableRowPointerMinioCluster {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_docker_registry_instance__minio_bucket(&self, ptr: TableRowPointerMinioBucket) -> &[TableRowPointerDockerRegistryInstance] {
        &self.c_referrers_docker_registry_instance__minio_bucket[ptr.0]
    }

    pub fn c_referrers_loki_cluster__storage_bucket(&self, ptr: TableRowPointerMinioBucket) -> &[TableRowPointerLokiCluster] {
        &self.c_referrers_loki_cluster__storage_bucket[ptr.0]
    }

    pub fn c_referrers_tempo_cluster__storage_bucket(&self, ptr: TableRowPointerMinioBucket) -> &[TableRowPointerTempoCluster] {
        &self.c_referrers_tempo_cluster__storage_bucket[ptr.0]
    }

}

impl TableDefinitionMinioCluster {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerMinioCluster> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerMinioCluster(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerMinioCluster) -> &TableRowMinioCluster {
        &self.rows[ptr.0]
    }

    pub fn c_cluster_name(&self, ptr: TableRowPointerMinioCluster) -> &::std::string::String {
        &self.c_cluster_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerMinioCluster) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerMinioCluster) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerMinioCluster) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_docker_image_minio(&self, ptr: TableRowPointerMinioCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_minio[ptr.0]
    }

    pub fn c_docker_image_minio_mc(&self, ptr: TableRowPointerMinioCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_minio_mc[ptr.0]
    }

    pub fn c_docker_image_nginx(&self, ptr: TableRowPointerMinioCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_nginx[ptr.0]
    }

    pub fn c_api_port(&self, ptr: TableRowPointerMinioCluster) -> i64 {
        self.c_api_port[ptr.0]
    }

    pub fn c_console_port(&self, ptr: TableRowPointerMinioCluster) -> i64 {
        self.c_console_port[ptr.0]
    }

    pub fn c_lb_port(&self, ptr: TableRowPointerMinioCluster) -> i64 {
        self.c_lb_port[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerMinioCluster) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerMinioCluster) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_expected_zfs_recordsize(&self, ptr: TableRowPointerMinioCluster) -> &::std::string::String {
        &self.c_expected_zfs_recordsize[ptr.0]
    }

    pub fn c_distribute_over_dcs(&self, ptr: TableRowPointerMinioCluster) -> bool {
        self.c_distribute_over_dcs[ptr.0]
    }

    pub fn c_instance_memory_mb(&self, ptr: TableRowPointerMinioCluster) -> i64 {
        self.c_instance_memory_mb[ptr.0]
    }

    pub fn c_lb_memory_mb(&self, ptr: TableRowPointerMinioCluster) -> i64 {
        self.c_lb_memory_mb[ptr.0]
    }

    pub fn c_consul_service_name(&self, ptr: TableRowPointerMinioCluster) -> &::std::string::String {
        &self.c_consul_service_name[ptr.0]
    }

    pub fn c_children_minio_instance(&self, ptr: TableRowPointerMinioCluster) -> &[TableRowPointerMinioInstance] {
        &self.c_children_minio_instance[ptr.0]
    }

    pub fn c_children_minio_bucket(&self, ptr: TableRowPointerMinioCluster) -> &[TableRowPointerMinioBucket] {
        &self.c_children_minio_bucket[ptr.0]
    }

}

impl TableDefinitionMinioInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerMinioInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerMinioInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerMinioInstance) -> &TableRowMinioInstance {
        &self.rows[ptr.0]
    }

    pub fn c_instance_id(&self, ptr: TableRowPointerMinioInstance) -> i64 {
        self.c_instance_id[ptr.0]
    }

    pub fn c_instance_volume(&self, ptr: TableRowPointerMinioInstance) -> TableRowPointerServerVolume {
        self.c_instance_volume[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerMinioInstance) -> TableRowPointerMinioCluster {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionMonitoringCluster {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerMonitoringCluster> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerMonitoringCluster(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerMonitoringCluster) -> &TableRowMonitoringCluster {
        &self.rows[ptr.0]
    }

    pub fn c_cluster_name(&self, ptr: TableRowPointerMonitoringCluster) -> &::std::string::String {
        &self.c_cluster_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerMonitoringCluster) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerMonitoringCluster) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_is_region_default(&self, ptr: TableRowPointerMonitoringCluster) -> bool {
        self.c_is_region_default[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerMonitoringCluster) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_docker_image_prometheus(&self, ptr: TableRowPointerMonitoringCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_prometheus[ptr.0]
    }

    pub fn c_docker_image_alertmanager(&self, ptr: TableRowPointerMonitoringCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_alertmanager[ptr.0]
    }

    pub fn c_docker_image_victoriametrics(&self, ptr: TableRowPointerMonitoringCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_victoriametrics[ptr.0]
    }

    pub fn c_prometheus_memory_mb(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_prometheus_memory_mb[ptr.0]
    }

    pub fn c_victoriametrics_memory_mb(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_victoriametrics_memory_mb[ptr.0]
    }

    pub fn c_alertmanager_memory_mb(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_alertmanager_memory_mb[ptr.0]
    }

    pub fn c_prometheus_port(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_prometheus_port[ptr.0]
    }

    pub fn c_victoriametrics_port(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_victoriametrics_port[ptr.0]
    }

    pub fn c_alertmanager_port(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_alertmanager_port[ptr.0]
    }

    pub fn c_alertmanager_p2p_port(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_alertmanager_p2p_port[ptr.0]
    }

    pub fn c_victoriametrics_retention_months(&self, ptr: TableRowPointerMonitoringCluster) -> i64 {
        self.c_victoriametrics_retention_months[ptr.0]
    }

    pub fn c_children_monitoring_cluster_scraped_metric(&self, ptr: TableRowPointerMonitoringCluster) -> &[TableRowPointerMonitoringClusterScrapedMetric] {
        &self.c_children_monitoring_cluster_scraped_metric[ptr.0]
    }

    pub fn c_children_monitoring_cluster_alert_group(&self, ptr: TableRowPointerMonitoringCluster) -> &[TableRowPointerMonitoringClusterAlertGroup] {
        &self.c_children_monitoring_cluster_alert_group[ptr.0]
    }

    pub fn c_children_monitoring_instance(&self, ptr: TableRowPointerMonitoringCluster) -> &[TableRowPointerMonitoringInstance] {
        &self.c_children_monitoring_instance[ptr.0]
    }

    pub fn c_children_alertmanager_instance(&self, ptr: TableRowPointerMonitoringCluster) -> &[TableRowPointerAlertmanagerInstance] {
        &self.c_children_alertmanager_instance[ptr.0]
    }

}

impl TableDefinitionMonitoringClusterAlertGroup {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerMonitoringClusterAlertGroup> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerMonitoringClusterAlertGroup(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerMonitoringClusterAlertGroup) -> &TableRowMonitoringClusterAlertGroup {
        &self.rows[ptr.0]
    }

    pub fn c_alert_group_name(&self, ptr: TableRowPointerMonitoringClusterAlertGroup) -> TableRowPointerAlertGroup {
        self.c_alert_group_name[ptr.0]
    }

    pub fn c_telegram_channel(&self, ptr: TableRowPointerMonitoringClusterAlertGroup) -> TableRowPointerTelegramChannel {
        self.c_telegram_channel[ptr.0]
    }

    pub fn c_telegram_bot(&self, ptr: TableRowPointerMonitoringClusterAlertGroup) -> TableRowPointerTelegramBot {
        self.c_telegram_bot[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerMonitoringClusterAlertGroup) -> TableRowPointerMonitoringCluster {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionMonitoringClusterScrapedMetric {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerMonitoringClusterScrapedMetric> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerMonitoringClusterScrapedMetric(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerMonitoringClusterScrapedMetric) -> &TableRowMonitoringClusterScrapedMetric {
        &self.rows[ptr.0]
    }

    pub fn c_metric_name(&self, ptr: TableRowPointerMonitoringClusterScrapedMetric) -> &::std::string::String {
        &self.c_metric_name[ptr.0]
    }

    pub fn c_expression(&self, ptr: TableRowPointerMonitoringClusterScrapedMetric) -> &::std::string::String {
        &self.c_expression[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerMonitoringClusterScrapedMetric) -> TableRowPointerMonitoringCluster {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionMonitoringInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerMonitoringInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerMonitoringInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerMonitoringInstance) -> &TableRowMonitoringInstance {
        &self.rows[ptr.0]
    }

    pub fn c_instance_id(&self, ptr: TableRowPointerMonitoringInstance) -> i64 {
        self.c_instance_id[ptr.0]
    }

    pub fn c_monitoring_server(&self, ptr: TableRowPointerMonitoringInstance) -> TableRowPointerServerVolume {
        self.c_monitoring_server[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerMonitoringInstance) -> TableRowPointerMonitoringCluster {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionNatsCluster {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNatsCluster> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNatsCluster(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNatsCluster) -> &TableRowNatsCluster {
        &self.rows[ptr.0]
    }

    pub fn c_cluster_name(&self, ptr: TableRowPointerNatsCluster) -> &::std::string::String {
        &self.c_cluster_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerNatsCluster) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerNatsCluster) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerNatsCluster) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerNatsCluster) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_distribute_over_dcs(&self, ptr: TableRowPointerNatsCluster) -> bool {
        self.c_distribute_over_dcs[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerNatsCluster) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_docker_image_nats(&self, ptr: TableRowPointerNatsCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_nats[ptr.0]
    }

    pub fn c_docker_image_nats_exporter(&self, ptr: TableRowPointerNatsCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image_nats_exporter[ptr.0]
    }

    pub fn c_nats_clients_port(&self, ptr: TableRowPointerNatsCluster) -> i64 {
        self.c_nats_clients_port[ptr.0]
    }

    pub fn c_nats_cluster_port(&self, ptr: TableRowPointerNatsCluster) -> i64 {
        self.c_nats_cluster_port[ptr.0]
    }

    pub fn c_nats_http_mon_port(&self, ptr: TableRowPointerNatsCluster) -> i64 {
        self.c_nats_http_mon_port[ptr.0]
    }

    pub fn c_nats_prometheus_port(&self, ptr: TableRowPointerNatsCluster) -> i64 {
        self.c_nats_prometheus_port[ptr.0]
    }

    pub fn c_instance_memory_mb(&self, ptr: TableRowPointerNatsCluster) -> i64 {
        self.c_instance_memory_mb[ptr.0]
    }

    pub fn c_children_nats_jetstream_stream(&self, ptr: TableRowPointerNatsCluster) -> &[TableRowPointerNatsJetstreamStream] {
        &self.c_children_nats_jetstream_stream[ptr.0]
    }

    pub fn c_children_nats_deployment_instance(&self, ptr: TableRowPointerNatsCluster) -> &[TableRowPointerNatsDeploymentInstance] {
        &self.c_children_nats_deployment_instance[ptr.0]
    }

}

impl TableDefinitionNatsDeploymentInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNatsDeploymentInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNatsDeploymentInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNatsDeploymentInstance) -> &TableRowNatsDeploymentInstance {
        &self.rows[ptr.0]
    }

    pub fn c_instance_id(&self, ptr: TableRowPointerNatsDeploymentInstance) -> i64 {
        self.c_instance_id[ptr.0]
    }

    pub fn c_nats_server(&self, ptr: TableRowPointerNatsDeploymentInstance) -> TableRowPointerServerVolume {
        self.c_nats_server[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerNatsDeploymentInstance) -> TableRowPointerNatsCluster {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionNatsJetstreamStream {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNatsJetstreamStream> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNatsJetstreamStream(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNatsJetstreamStream) -> &TableRowNatsJetstreamStream {
        &self.rows[ptr.0]
    }

    pub fn c_stream_name(&self, ptr: TableRowPointerNatsJetstreamStream) -> &::std::string::String {
        &self.c_stream_name[ptr.0]
    }

    pub fn c_stream_type(&self, ptr: TableRowPointerNatsJetstreamStream) -> TableRowPointerVersionedType {
        self.c_stream_type[ptr.0]
    }

    pub fn c_max_bytes(&self, ptr: TableRowPointerNatsJetstreamStream) -> i64 {
        self.c_max_bytes[ptr.0]
    }

    pub fn c_max_msg_size(&self, ptr: TableRowPointerNatsJetstreamStream) -> i64 {
        self.c_max_msg_size[ptr.0]
    }

    pub fn c_enable_subjects(&self, ptr: TableRowPointerNatsJetstreamStream) -> bool {
        self.c_enable_subjects[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerNatsJetstreamStream) -> TableRowPointerNatsCluster {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_ch_nats_stream_import__stream(&self, ptr: TableRowPointerNatsJetstreamStream) -> &[TableRowPointerChNatsStreamImport] {
        &self.c_referrers_ch_nats_stream_import__stream[ptr.0]
    }

}

impl TableDefinitionNetwork {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNetwork> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNetwork(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNetwork) -> &TableRowNetwork {
        &self.rows[ptr.0]
    }

    pub fn c_network_name(&self, ptr: TableRowPointerNetwork) -> &::std::string::String {
        &self.c_network_name[ptr.0]
    }

    pub fn c_cidr(&self, ptr: TableRowPointerNetwork) -> &::std::string::String {
        &self.c_cidr[ptr.0]
    }

    pub fn c_referrers_network_interface__if_network(&self, ptr: TableRowPointerNetwork) -> &[TableRowPointerNetworkInterface] {
        &self.c_referrers_network_interface__if_network[ptr.0]
    }

}

impl TableDefinitionNetworkInterface {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNetworkInterface> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNetworkInterface(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNetworkInterface) -> &TableRowNetworkInterface {
        &self.rows[ptr.0]
    }

    pub fn c_if_name(&self, ptr: TableRowPointerNetworkInterface) -> &::std::string::String {
        &self.c_if_name[ptr.0]
    }

    pub fn c_if_network(&self, ptr: TableRowPointerNetworkInterface) -> TableRowPointerNetwork {
        self.c_if_network[ptr.0]
    }

    pub fn c_if_ip(&self, ptr: TableRowPointerNetworkInterface) -> &::std::string::String {
        &self.c_if_ip[ptr.0]
    }

    pub fn c_if_prefix(&self, ptr: TableRowPointerNetworkInterface) -> i64 {
        self.c_if_prefix[ptr.0]
    }

    pub fn c_if_vlan(&self, ptr: TableRowPointerNetworkInterface) -> i64 {
        self.c_if_vlan[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerNetworkInterface) -> TableRowPointerServer {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_server__ssh_interface(&self, ptr: TableRowPointerNetworkInterface) -> &[TableRowPointerServer] {
        &self.c_referrers_server__ssh_interface[ptr.0]
    }

}

impl TableDefinitionNixpkgsEnvironment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNixpkgsEnvironment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNixpkgsEnvironment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNixpkgsEnvironment) -> &TableRowNixpkgsEnvironment {
        &self.rows[ptr.0]
    }

    pub fn c_name(&self, ptr: TableRowPointerNixpkgsEnvironment) -> &::std::string::String {
        &self.c_name[ptr.0]
    }

    pub fn c_version(&self, ptr: TableRowPointerNixpkgsEnvironment) -> TableRowPointerNixpkgsVersion {
        self.c_version[ptr.0]
    }

    pub fn c_referrers_server__nixpkgs_environment(&self, ptr: TableRowPointerNixpkgsEnvironment) -> &[TableRowPointerServer] {
        &self.c_referrers_server__nixpkgs_environment[ptr.0]
    }

    pub fn c_referrers_rust_compilation_environment__nixpkgs_environment(&self, ptr: TableRowPointerNixpkgsEnvironment) -> &[TableRowPointerRustCompilationEnvironment] {
        &self.c_referrers_rust_compilation_environment__nixpkgs_environment[ptr.0]
    }

}

impl TableDefinitionNixpkgsVersion {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNixpkgsVersion> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNixpkgsVersion(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNixpkgsVersion) -> &TableRowNixpkgsVersion {
        &self.rows[ptr.0]
    }

    pub fn c_version(&self, ptr: TableRowPointerNixpkgsVersion) -> &::std::string::String {
        &self.c_version[ptr.0]
    }

    pub fn c_checksum(&self, ptr: TableRowPointerNixpkgsVersion) -> &::std::string::String {
        &self.c_checksum[ptr.0]
    }

    pub fn c_tarball_checksum(&self, ptr: TableRowPointerNixpkgsVersion) -> &::std::string::String {
        &self.c_tarball_checksum[ptr.0]
    }

    pub fn c_referrers_nixpkgs_environment__version(&self, ptr: TableRowPointerNixpkgsVersion) -> &[TableRowPointerNixpkgsEnvironment] {
        &self.c_referrers_nixpkgs_environment__version[ptr.0]
    }

}

impl TableDefinitionNomadNamespace {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerNomadNamespace> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerNomadNamespace(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerNomadNamespace) -> &TableRowNomadNamespace {
        &self.rows[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerNomadNamespace) -> &::std::string::String {
        &self.c_namespace[ptr.0]
    }

    pub fn c_description(&self, ptr: TableRowPointerNomadNamespace) -> &::std::string::String {
        &self.c_description[ptr.0]
    }

    pub fn c_referrers_pg_deployment__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerPgDeployment] {
        &self.c_referrers_pg_deployment__namespace[ptr.0]
    }

    pub fn c_referrers_ch_deployment__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerChDeployment] {
        &self.c_referrers_ch_deployment__namespace[ptr.0]
    }

    pub fn c_referrers_ch_keeper_deployment__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerChKeeperDeployment] {
        &self.c_referrers_ch_keeper_deployment__namespace[ptr.0]
    }

    pub fn c_referrers_nats_cluster__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerNatsCluster] {
        &self.c_referrers_nats_cluster__namespace[ptr.0]
    }

    pub fn c_referrers_backend_application_deployment__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerBackendApplicationDeployment] {
        &self.c_referrers_backend_application_deployment__namespace[ptr.0]
    }

    pub fn c_referrers_frontend_application_deployment__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerFrontendApplicationDeployment] {
        &self.c_referrers_frontend_application_deployment__namespace[ptr.0]
    }

    pub fn c_referrers_minio_cluster__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerMinioCluster] {
        &self.c_referrers_minio_cluster__namespace[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerMonitoringCluster] {
        &self.c_referrers_monitoring_cluster__namespace[ptr.0]
    }

    pub fn c_referrers_grafana__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerGrafana] {
        &self.c_referrers_grafana__namespace[ptr.0]
    }

    pub fn c_referrers_loki_cluster__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerLokiCluster] {
        &self.c_referrers_loki_cluster__namespace[ptr.0]
    }

    pub fn c_referrers_tempo_cluster__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerTempoCluster] {
        &self.c_referrers_tempo_cluster__namespace[ptr.0]
    }

    pub fn c_referrers_blackbox_deployment__namespace(&self, ptr: TableRowPointerNomadNamespace) -> &[TableRowPointerBlackboxDeployment] {
        &self.c_referrers_blackbox_deployment__namespace[ptr.0]
    }

}

impl TableDefinitionPgDeployment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgDeployment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgDeployment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgDeployment) -> &TableRowPgDeployment {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerPgDeployment) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerPgDeployment) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerPgDeployment) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerPgDeployment) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerPgDeployment) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_docker_image_pg(&self, ptr: TableRowPointerPgDeployment) -> TableRowPointerDockerImagePin {
        self.c_docker_image_pg[ptr.0]
    }

    pub fn c_docker_image_haproxy(&self, ptr: TableRowPointerPgDeployment) -> TableRowPointerDockerImagePin {
        self.c_docker_image_haproxy[ptr.0]
    }

    pub fn c_docker_image_pg_exporter(&self, ptr: TableRowPointerPgDeployment) -> TableRowPointerDockerImagePin {
        self.c_docker_image_pg_exporter[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerPgDeployment) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_distribute_over_dcs(&self, ptr: TableRowPointerPgDeployment) -> bool {
        self.c_distribute_over_dcs[ptr.0]
    }

    pub fn c_synchronous_replication(&self, ptr: TableRowPointerPgDeployment) -> bool {
        self.c_synchronous_replication[ptr.0]
    }

    pub fn c_shared_buffers_mb(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_shared_buffers_mb[ptr.0]
    }

    pub fn c_work_mem_mb(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_work_mem_mb[ptr.0]
    }

    pub fn c_maintenance_work_mem_mb(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_maintenance_work_mem_mb[ptr.0]
    }

    pub fn c_overhead_mem_mb(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_overhead_mem_mb[ptr.0]
    }

    pub fn c_max_connections(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_max_connections[ptr.0]
    }

    pub fn c_replica_rolling_update_delay_seconds(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_replica_rolling_update_delay_seconds[ptr.0]
    }

    pub fn c_instance_pg_port(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_instance_pg_port[ptr.0]
    }

    pub fn c_instance_pg_master_port(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_instance_pg_master_port[ptr.0]
    }

    pub fn c_instance_pg_slave_port(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_instance_pg_slave_port[ptr.0]
    }

    pub fn c_instance_patroni_port(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_instance_patroni_port[ptr.0]
    }

    pub fn c_instance_haproxy_metrics_port(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_instance_haproxy_metrics_port[ptr.0]
    }

    pub fn c_instance_pg_exporter_port(&self, ptr: TableRowPointerPgDeployment) -> i64 {
        self.c_instance_pg_exporter_port[ptr.0]
    }

    pub fn c_children_pg_deployment_schemas(&self, ptr: TableRowPointerPgDeployment) -> &[TableRowPointerPgDeploymentSchemas] {
        &self.c_children_pg_deployment_schemas[ptr.0]
    }

    pub fn c_children_pg_deployment_unmanaged_db(&self, ptr: TableRowPointerPgDeployment) -> &[TableRowPointerPgDeploymentUnmanagedDb] {
        &self.c_children_pg_deployment_unmanaged_db[ptr.0]
    }

    pub fn c_children_pg_deployment_instance(&self, ptr: TableRowPointerPgDeployment) -> &[TableRowPointerPgDeploymentInstance] {
        &self.c_children_pg_deployment_instance[ptr.0]
    }

}

impl TableDefinitionPgDeploymentInstance {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgDeploymentInstance> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgDeploymentInstance(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgDeploymentInstance) -> &TableRowPgDeploymentInstance {
        &self.rows[ptr.0]
    }

    pub fn c_instance_id(&self, ptr: TableRowPointerPgDeploymentInstance) -> i64 {
        self.c_instance_id[ptr.0]
    }

    pub fn c_pg_server(&self, ptr: TableRowPointerPgDeploymentInstance) -> TableRowPointerServerVolume {
        self.c_pg_server[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgDeploymentInstance) -> TableRowPointerPgDeployment {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionPgDeploymentSchemas {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgDeploymentSchemas> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgDeploymentSchemas(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgDeploymentSchemas) -> &TableRowPgDeploymentSchemas {
        &self.rows[ptr.0]
    }

    pub fn c_db_name(&self, ptr: TableRowPointerPgDeploymentSchemas) -> &::std::string::String {
        &self.c_db_name[ptr.0]
    }

    pub fn c_pg_schema(&self, ptr: TableRowPointerPgDeploymentSchemas) -> TableRowPointerPgSchema {
        self.c_pg_schema[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgDeploymentSchemas) -> TableRowPointerPgDeployment {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionPgDeploymentUnmanagedDb {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgDeploymentUnmanagedDb> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgDeploymentUnmanagedDb(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgDeploymentUnmanagedDb) -> &TableRowPgDeploymentUnmanagedDb {
        &self.rows[ptr.0]
    }

    pub fn c_db_name(&self, ptr: TableRowPointerPgDeploymentUnmanagedDb) -> &::std::string::String {
        &self.c_db_name[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgDeploymentUnmanagedDb) -> TableRowPointerPgDeployment {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_grafana__database(&self, ptr: TableRowPointerPgDeploymentUnmanagedDb) -> &[TableRowPointerGrafana] {
        &self.c_referrers_grafana__database[ptr.0]
    }

}

impl TableDefinitionPgMatView {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgMatView> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgMatView(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgMatView) -> &TableRowPgMatView {
        &self.rows[ptr.0]
    }

    pub fn c_mview_name(&self, ptr: TableRowPointerPgMatView) -> &::std::string::String {
        &self.c_mview_name[ptr.0]
    }

    pub fn c_update_frequency(&self, ptr: TableRowPointerPgMatView) -> TableRowPointerPgMatViewUpdateFrequency {
        self.c_update_frequency[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgMatView) -> TableRowPointerPgSchema {
        self.c_parent[ptr.0]
    }

    pub fn c_children_pg_mat_view_test(&self, ptr: TableRowPointerPgMatView) -> &[TableRowPointerPgMatViewTest] {
        &self.c_children_pg_mat_view_test[ptr.0]
    }

}

impl TableDefinitionPgMatViewTest {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgMatViewTest> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgMatViewTest(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgMatViewTest) -> &TableRowPgMatViewTest {
        &self.rows[ptr.0]
    }

    pub fn c_expected_data(&self, ptr: TableRowPointerPgMatViewTest) -> &::std::string::String {
        &self.c_expected_data[ptr.0]
    }

    pub fn c_test_dataset(&self, ptr: TableRowPointerPgMatViewTest) -> TableRowPointerPgTestDataset {
        self.c_test_dataset[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgMatViewTest) -> TableRowPointerPgMatView {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionPgMatViewUpdateFrequency {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgMatViewUpdateFrequency> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgMatViewUpdateFrequency(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgMatViewUpdateFrequency) -> &TableRowPgMatViewUpdateFrequency {
        &self.rows[ptr.0]
    }

    pub fn c_frequency(&self, ptr: TableRowPointerPgMatViewUpdateFrequency) -> &::std::string::String {
        &self.c_frequency[ptr.0]
    }

    pub fn c_referrers_pg_mat_view__update_frequency(&self, ptr: TableRowPointerPgMatViewUpdateFrequency) -> &[TableRowPointerPgMatView] {
        &self.c_referrers_pg_mat_view__update_frequency[ptr.0]
    }

}

impl TableDefinitionPgMigration {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgMigration> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgMigration(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgMigration) -> &TableRowPgMigration {
        &self.rows[ptr.0]
    }

    pub fn c_time(&self, ptr: TableRowPointerPgMigration) -> i64 {
        self.c_time[ptr.0]
    }

    pub fn c_upgrade(&self, ptr: TableRowPointerPgMigration) -> &::std::string::String {
        &self.c_upgrade[ptr.0]
    }

    pub fn c_downgrade(&self, ptr: TableRowPointerPgMigration) -> &::std::string::String {
        &self.c_downgrade[ptr.0]
    }

    pub fn c_needs_admin(&self, ptr: TableRowPointerPgMigration) -> bool {
        self.c_needs_admin[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgMigration) -> TableRowPointerPgSchema {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionPgMutator {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgMutator> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgMutator(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgMutator) -> &TableRowPgMutator {
        &self.rows[ptr.0]
    }

    pub fn c_mutator_name(&self, ptr: TableRowPointerPgMutator) -> &::std::string::String {
        &self.c_mutator_name[ptr.0]
    }

    pub fn c_mutator_expression(&self, ptr: TableRowPointerPgMutator) -> &::std::string::String {
        &self.c_mutator_expression[ptr.0]
    }

    pub fn c_seqscan_ok(&self, ptr: TableRowPointerPgMutator) -> bool {
        self.c_seqscan_ok[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgMutator) -> TableRowPointerPgSchema {
        self.c_parent[ptr.0]
    }

    pub fn c_children_pg_mutator_test(&self, ptr: TableRowPointerPgMutator) -> &[TableRowPointerPgMutatorTest] {
        &self.c_children_pg_mutator_test[ptr.0]
    }

}

impl TableDefinitionPgMutatorTest {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgMutatorTest> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgMutatorTest(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgMutatorTest) -> &TableRowPgMutatorTest {
        &self.rows[ptr.0]
    }

    pub fn c_arguments(&self, ptr: TableRowPointerPgMutatorTest) -> &::std::string::String {
        &self.c_arguments[ptr.0]
    }

    pub fn c_test_dataset(&self, ptr: TableRowPointerPgMutatorTest) -> TableRowPointerPgTestDataset {
        self.c_test_dataset[ptr.0]
    }

    pub fn c_resulting_data(&self, ptr: TableRowPointerPgMutatorTest) -> &::std::string::String {
        &self.c_resulting_data[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgMutatorTest) -> TableRowPointerPgMutator {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionPgQuery {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgQuery> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgQuery(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgQuery) -> &TableRowPgQuery {
        &self.rows[ptr.0]
    }

    pub fn c_query_name(&self, ptr: TableRowPointerPgQuery) -> &::std::string::String {
        &self.c_query_name[ptr.0]
    }

    pub fn c_query_expression(&self, ptr: TableRowPointerPgQuery) -> &::std::string::String {
        &self.c_query_expression[ptr.0]
    }

    pub fn c_is_mutating(&self, ptr: TableRowPointerPgQuery) -> bool {
        self.c_is_mutating[ptr.0]
    }

    pub fn c_seqscan_ok(&self, ptr: TableRowPointerPgQuery) -> bool {
        self.c_seqscan_ok[ptr.0]
    }

    pub fn c_opt_fields(&self, ptr: TableRowPointerPgQuery) -> &::std::string::String {
        &self.c_opt_fields[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgQuery) -> TableRowPointerPgSchema {
        self.c_parent[ptr.0]
    }

    pub fn c_children_pg_query_test(&self, ptr: TableRowPointerPgQuery) -> &[TableRowPointerPgQueryTest] {
        &self.c_children_pg_query_test[ptr.0]
    }

}

impl TableDefinitionPgQueryTest {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgQueryTest> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgQueryTest(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgQueryTest) -> &TableRowPgQueryTest {
        &self.rows[ptr.0]
    }

    pub fn c_arguments(&self, ptr: TableRowPointerPgQueryTest) -> &::std::string::String {
        &self.c_arguments[ptr.0]
    }

    pub fn c_outputs(&self, ptr: TableRowPointerPgQueryTest) -> &::std::string::String {
        &self.c_outputs[ptr.0]
    }

    pub fn c_test_dataset(&self, ptr: TableRowPointerPgQueryTest) -> TableRowPointerPgTestDataset {
        self.c_test_dataset[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgQueryTest) -> TableRowPointerPgQuery {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionPgSchema {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgSchema> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgSchema(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgSchema) -> &TableRowPgSchema {
        &self.rows[ptr.0]
    }

    pub fn c_schema_name(&self, ptr: TableRowPointerPgSchema) -> &::std::string::String {
        &self.c_schema_name[ptr.0]
    }

    pub fn c_children_pg_migration(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerPgMigration] {
        &self.c_children_pg_migration[ptr.0]
    }

    pub fn c_children_pg_query(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerPgQuery] {
        &self.c_children_pg_query[ptr.0]
    }

    pub fn c_children_pg_mutator(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerPgMutator] {
        &self.c_children_pg_mutator[ptr.0]
    }

    pub fn c_children_pg_transaction(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerPgTransaction] {
        &self.c_children_pg_transaction[ptr.0]
    }

    pub fn c_children_pg_mat_view(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerPgMatView] {
        &self.c_children_pg_mat_view[ptr.0]
    }

    pub fn c_children_pg_test_dataset(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerPgTestDataset] {
        &self.c_children_pg_test_dataset[ptr.0]
    }

    pub fn c_referrers_pg_deployment_schemas__pg_schema(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerPgDeploymentSchemas] {
        &self.c_referrers_pg_deployment_schemas__pg_schema[ptr.0]
    }

    pub fn c_referrers_backend_application_pg_shard__pg_schema(&self, ptr: TableRowPointerPgSchema) -> &[TableRowPointerBackendApplicationPgShard] {
        &self.c_referrers_backend_application_pg_shard__pg_schema[ptr.0]
    }

}

impl TableDefinitionPgTestDataset {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgTestDataset> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgTestDataset(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgTestDataset) -> &TableRowPgTestDataset {
        &self.rows[ptr.0]
    }

    pub fn c_dataset_name(&self, ptr: TableRowPointerPgTestDataset) -> &::std::string::String {
        &self.c_dataset_name[ptr.0]
    }

    pub fn c_dataset_contents(&self, ptr: TableRowPointerPgTestDataset) -> &::std::string::String {
        &self.c_dataset_contents[ptr.0]
    }

    pub fn c_min_time(&self, ptr: TableRowPointerPgTestDataset) -> i64 {
        self.c_min_time[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgTestDataset) -> TableRowPointerPgSchema {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_pg_query_test__test_dataset(&self, ptr: TableRowPointerPgTestDataset) -> &[TableRowPointerPgQueryTest] {
        &self.c_referrers_pg_query_test__test_dataset[ptr.0]
    }

    pub fn c_referrers_pg_mutator_test__test_dataset(&self, ptr: TableRowPointerPgTestDataset) -> &[TableRowPointerPgMutatorTest] {
        &self.c_referrers_pg_mutator_test__test_dataset[ptr.0]
    }

    pub fn c_referrers_pg_mat_view_test__test_dataset(&self, ptr: TableRowPointerPgTestDataset) -> &[TableRowPointerPgMatViewTest] {
        &self.c_referrers_pg_mat_view_test__test_dataset[ptr.0]
    }

}

impl TableDefinitionPgTransaction {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerPgTransaction> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerPgTransaction(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerPgTransaction) -> &TableRowPgTransaction {
        &self.rows[ptr.0]
    }

    pub fn c_transaction_name(&self, ptr: TableRowPointerPgTransaction) -> &::std::string::String {
        &self.c_transaction_name[ptr.0]
    }

    pub fn c_steps(&self, ptr: TableRowPointerPgTransaction) -> &::std::string::String {
        &self.c_steps[ptr.0]
    }

    pub fn c_is_read_only(&self, ptr: TableRowPointerPgTransaction) -> bool {
        self.c_is_read_only[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerPgTransaction) -> TableRowPointerPgSchema {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionRegion {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerRegion> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerRegion(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerRegion) -> &TableRowRegion {
        &self.rows[ptr.0]
    }

    pub fn c_region_name(&self, ptr: TableRowPointerRegion) -> &::std::string::String {
        &self.c_region_name[ptr.0]
    }

    pub fn c_availability_mode(&self, ptr: TableRowPointerRegion) -> &::std::string::String {
        &self.c_availability_mode[ptr.0]
    }

    pub fn c_tld(&self, ptr: TableRowPointerRegion) -> TableRowPointerTld {
        self.c_tld[ptr.0]
    }

    pub fn c_is_dns_master(&self, ptr: TableRowPointerRegion) -> bool {
        self.c_is_dns_master[ptr.0]
    }

    pub fn c_is_dns_slave(&self, ptr: TableRowPointerRegion) -> bool {
        self.c_is_dns_slave[ptr.0]
    }

    pub fn c_has_coprocessor_dc(&self, ptr: TableRowPointerRegion) -> bool {
        self.c_has_coprocessor_dc[ptr.0]
    }

    pub fn c_docker_image_external_lb(&self, ptr: TableRowPointerRegion) -> TableRowPointerDockerImagePin {
        self.c_docker_image_external_lb[ptr.0]
    }

    pub fn c_nomad_disable_log_collection(&self, ptr: TableRowPointerRegion) -> bool {
        self.c_nomad_disable_log_collection[ptr.0]
    }

    pub fn c_referrers_datacenter__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerDatacenter] {
        &self.c_referrers_datacenter__region[ptr.0]
    }

    pub fn c_referrers_docker_registry_instance__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerDockerRegistryInstance] {
        &self.c_referrers_docker_registry_instance__region[ptr.0]
    }

    pub fn c_referrers_pg_deployment__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerPgDeployment] {
        &self.c_referrers_pg_deployment__region[ptr.0]
    }

    pub fn c_referrers_ch_deployment__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerChDeployment] {
        &self.c_referrers_ch_deployment__region[ptr.0]
    }

    pub fn c_referrers_ch_keeper_deployment__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerChKeeperDeployment] {
        &self.c_referrers_ch_keeper_deployment__region[ptr.0]
    }

    pub fn c_referrers_nats_cluster__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerNatsCluster] {
        &self.c_referrers_nats_cluster__region[ptr.0]
    }

    pub fn c_referrers_backend_application_deployment__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerBackendApplicationDeployment] {
        &self.c_referrers_backend_application_deployment__region[ptr.0]
    }

    pub fn c_referrers_frontend_application_deployment__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerFrontendApplicationDeployment] {
        &self.c_referrers_frontend_application_deployment__region[ptr.0]
    }

    pub fn c_referrers_minio_cluster__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerMinioCluster] {
        &self.c_referrers_minio_cluster__region[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerMonitoringCluster] {
        &self.c_referrers_monitoring_cluster__region[ptr.0]
    }

    pub fn c_referrers_grafana__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerGrafana] {
        &self.c_referrers_grafana__region[ptr.0]
    }

    pub fn c_referrers_loki_cluster__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerLokiCluster] {
        &self.c_referrers_loki_cluster__region[ptr.0]
    }

    pub fn c_referrers_tempo_cluster__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerTempoCluster] {
        &self.c_referrers_tempo_cluster__region[ptr.0]
    }

    pub fn c_referrers_blackbox_deployment__region(&self, ptr: TableRowPointerRegion) -> &[TableRowPointerBlackboxDeployment] {
        &self.c_referrers_blackbox_deployment__region[ptr.0]
    }

}

impl TableDefinitionRustCompilationEnvironment {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerRustCompilationEnvironment> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerRustCompilationEnvironment(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerRustCompilationEnvironment) -> &TableRowRustCompilationEnvironment {
        &self.rows[ptr.0]
    }

    pub fn c_env_name(&self, ptr: TableRowPointerRustCompilationEnvironment) -> &::std::string::String {
        &self.c_env_name[ptr.0]
    }

    pub fn c_rust_edition(&self, ptr: TableRowPointerRustCompilationEnvironment) -> &::std::string::String {
        &self.c_rust_edition[ptr.0]
    }

    pub fn c_nixpkgs_environment(&self, ptr: TableRowPointerRustCompilationEnvironment) -> TableRowPointerNixpkgsEnvironment {
        self.c_nixpkgs_environment[ptr.0]
    }

    pub fn c_environment_kind(&self, ptr: TableRowPointerRustCompilationEnvironment) -> &::std::string::String {
        &self.c_environment_kind[ptr.0]
    }

    pub fn c_children_rust_crate_version(&self, ptr: TableRowPointerRustCompilationEnvironment) -> &[TableRowPointerRustCrateVersion] {
        &self.c_children_rust_crate_version[ptr.0]
    }

    pub fn c_referrers_backend_application__build_environment(&self, ptr: TableRowPointerRustCompilationEnvironment) -> &[TableRowPointerBackendApplication] {
        &self.c_referrers_backend_application__build_environment[ptr.0]
    }

    pub fn c_referrers_frontend_application__build_environment(&self, ptr: TableRowPointerRustCompilationEnvironment) -> &[TableRowPointerFrontendApplication] {
        &self.c_referrers_frontend_application__build_environment[ptr.0]
    }

}

impl TableDefinitionRustCrateVersion {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerRustCrateVersion> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerRustCrateVersion(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerRustCrateVersion) -> &TableRowRustCrateVersion {
        &self.rows[ptr.0]
    }

    pub fn c_crate_name(&self, ptr: TableRowPointerRustCrateVersion) -> &::std::string::String {
        &self.c_crate_name[ptr.0]
    }

    pub fn c_version(&self, ptr: TableRowPointerRustCrateVersion) -> &::std::string::String {
        &self.c_version[ptr.0]
    }

    pub fn c_features(&self, ptr: TableRowPointerRustCrateVersion) -> &::std::string::String {
        &self.c_features[ptr.0]
    }

    pub fn c_default_features(&self, ptr: TableRowPointerRustCrateVersion) -> bool {
        self.c_default_features[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerRustCrateVersion) -> TableRowPointerRustCompilationEnvironment {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServer {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServer> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServer(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServer) -> &TableRowServer {
        &self.rows[ptr.0]
    }

    pub fn c_hostname(&self, ptr: TableRowPointerServer) -> &::std::string::String {
        &self.c_hostname[ptr.0]
    }

    pub fn c_dc(&self, ptr: TableRowPointerServer) -> TableRowPointerDatacenter {
        self.c_dc[ptr.0]
    }

    pub fn c_ssh_interface(&self, ptr: TableRowPointerServer) -> TableRowPointerNetworkInterface {
        self.c_ssh_interface[ptr.0]
    }

    pub fn c_root_disk(&self, ptr: TableRowPointerServer) -> TableRowPointerServerDisk {
        self.c_root_disk[ptr.0]
    }

    pub fn c_is_consul_master(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_consul_master[ptr.0]
    }

    pub fn c_is_nomad_master(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_nomad_master[ptr.0]
    }

    pub fn c_is_vault_instance(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_vault_instance[ptr.0]
    }

    pub fn c_is_dns_master(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_dns_master[ptr.0]
    }

    pub fn c_is_dns_slave(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_dns_slave[ptr.0]
    }

    pub fn c_is_ingress(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_ingress[ptr.0]
    }

    pub fn c_is_vpn_gateway(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_vpn_gateway[ptr.0]
    }

    pub fn c_is_coprocessor_gateway(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_coprocessor_gateway[ptr.0]
    }

    pub fn c_is_router(&self, ptr: TableRowPointerServer) -> bool {
        self.c_is_router[ptr.0]
    }

    pub fn c_public_ipv6_address(&self, ptr: TableRowPointerServer) -> &::std::string::String {
        &self.c_public_ipv6_address[ptr.0]
    }

    pub fn c_public_ipv6_address_prefix(&self, ptr: TableRowPointerServer) -> i64 {
        self.c_public_ipv6_address_prefix[ptr.0]
    }

    pub fn c_kind(&self, ptr: TableRowPointerServer) -> &::std::string::String {
        &self.c_kind[ptr.0]
    }

    pub fn c_nixpkgs_environment(&self, ptr: TableRowPointerServer) -> TableRowPointerNixpkgsEnvironment {
        self.c_nixpkgs_environment[ptr.0]
    }

    pub fn c_run_unassigned_workloads(&self, ptr: TableRowPointerServer) -> bool {
        self.c_run_unassigned_workloads[ptr.0]
    }

    pub fn c_children_server_label(&self, ptr: TableRowPointerServer) -> &[TableRowPointerServerLabel] {
        &self.c_children_server_label[ptr.0]
    }

    pub fn c_children_server_disk(&self, ptr: TableRowPointerServer) -> &[TableRowPointerServerDisk] {
        &self.c_children_server_disk[ptr.0]
    }

    pub fn c_children_server_volume(&self, ptr: TableRowPointerServer) -> &[TableRowPointerServerVolume] {
        &self.c_children_server_volume[ptr.0]
    }

    pub fn c_children_server_root_volume(&self, ptr: TableRowPointerServer) -> &[TableRowPointerServerRootVolume] {
        &self.c_children_server_root_volume[ptr.0]
    }

    pub fn c_children_server_xfs_volume(&self, ptr: TableRowPointerServer) -> &[TableRowPointerServerXfsVolume] {
        &self.c_children_server_xfs_volume[ptr.0]
    }

    pub fn c_children_network_interface(&self, ptr: TableRowPointerServer) -> &[TableRowPointerNetworkInterface] {
        &self.c_children_network_interface[ptr.0]
    }

    pub fn c_children_server_zpool(&self, ptr: TableRowPointerServer) -> &[TableRowPointerServerZpool] {
        &self.c_children_server_zpool[ptr.0]
    }

}

impl TableDefinitionServerDisk {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerDisk> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerDisk(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerDisk) -> &TableRowServerDisk {
        &self.rows[ptr.0]
    }

    pub fn c_disk_id(&self, ptr: TableRowPointerServerDisk) -> &::std::string::String {
        &self.c_disk_id[ptr.0]
    }

    pub fn c_disk_kind(&self, ptr: TableRowPointerServerDisk) -> TableRowPointerDiskKind {
        self.c_disk_kind[ptr.0]
    }

    pub fn c_xfs_format(&self, ptr: TableRowPointerServerDisk) -> bool {
        self.c_xfs_format[ptr.0]
    }

    pub fn c_extra_config(&self, ptr: TableRowPointerServerDisk) -> &::std::string::String {
        &self.c_extra_config[ptr.0]
    }

    pub fn c_capacity_bytes(&self, ptr: TableRowPointerServerDisk) -> i64 {
        self.c_capacity_bytes[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerDisk) -> TableRowPointerServer {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_server__root_disk(&self, ptr: TableRowPointerServerDisk) -> &[TableRowPointerServer] {
        &self.c_referrers_server__root_disk[ptr.0]
    }

    pub fn c_referrers_server_xfs_volume__xfs_disk(&self, ptr: TableRowPointerServerDisk) -> &[TableRowPointerServerXfsVolume] {
        &self.c_referrers_server_xfs_volume__xfs_disk[ptr.0]
    }

    pub fn c_referrers_server_zpool_spare__disk_id(&self, ptr: TableRowPointerServerDisk) -> &[TableRowPointerServerZpoolSpare] {
        &self.c_referrers_server_zpool_spare__disk_id[ptr.0]
    }

    pub fn c_referrers_server_zpool_cache__disk_id(&self, ptr: TableRowPointerServerDisk) -> &[TableRowPointerServerZpoolCache] {
        &self.c_referrers_server_zpool_cache__disk_id[ptr.0]
    }

    pub fn c_referrers_server_zpool_log__disk_id(&self, ptr: TableRowPointerServerDisk) -> &[TableRowPointerServerZpoolLog] {
        &self.c_referrers_server_zpool_log__disk_id[ptr.0]
    }

    pub fn c_referrers_server_zpool_vdev_disk__disk_id(&self, ptr: TableRowPointerServerDisk) -> &[TableRowPointerServerZpoolVdevDisk] {
        &self.c_referrers_server_zpool_vdev_disk__disk_id[ptr.0]
    }

}

impl TableDefinitionServerKind {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerKind> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerKind(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerKind) -> &TableRowServerKind {
        &self.rows[ptr.0]
    }

    pub fn c_kind(&self, ptr: TableRowPointerServerKind) -> &::std::string::String {
        &self.c_kind[ptr.0]
    }

    pub fn c_cores(&self, ptr: TableRowPointerServerKind) -> i64 {
        self.c_cores[ptr.0]
    }

    pub fn c_memory_bytes(&self, ptr: TableRowPointerServerKind) -> i64 {
        self.c_memory_bytes[ptr.0]
    }

    pub fn c_architecture(&self, ptr: TableRowPointerServerKind) -> &::std::string::String {
        &self.c_architecture[ptr.0]
    }

    pub fn c_bare_metal(&self, ptr: TableRowPointerServerKind) -> bool {
        self.c_bare_metal[ptr.0]
    }

    pub fn c_non_eligible_reason(&self, ptr: TableRowPointerServerKind) -> &::std::string::String {
        &self.c_non_eligible_reason[ptr.0]
    }

    pub fn c_children_server_kind_attribute(&self, ptr: TableRowPointerServerKind) -> &[TableRowPointerServerKindAttribute] {
        &self.c_children_server_kind_attribute[ptr.0]
    }

    pub fn c_referrers_datacenter__default_server_kind(&self, ptr: TableRowPointerServerKind) -> &[TableRowPointerDatacenter] {
        &self.c_referrers_datacenter__default_server_kind[ptr.0]
    }

}

impl TableDefinitionServerKindAttribute {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerKindAttribute> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerKindAttribute(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerKindAttribute) -> &TableRowServerKindAttribute {
        &self.rows[ptr.0]
    }

    pub fn c_key(&self, ptr: TableRowPointerServerKindAttribute) -> &::std::string::String {
        &self.c_key[ptr.0]
    }

    pub fn c_value(&self, ptr: TableRowPointerServerKindAttribute) -> &::std::string::String {
        &self.c_value[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerKindAttribute) -> TableRowPointerServerKind {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerLabel {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerLabel> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerLabel(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerLabel) -> &TableRowServerLabel {
        &self.rows[ptr.0]
    }

    pub fn c_label_name(&self, ptr: TableRowPointerServerLabel) -> TableRowPointerValidServerLabels {
        self.c_label_name[ptr.0]
    }

    pub fn c_label_value(&self, ptr: TableRowPointerServerLabel) -> &::std::string::String {
        &self.c_label_value[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerLabel) -> TableRowPointerServer {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerRootVolume {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerRootVolume> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerRootVolume(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerRootVolume) -> &TableRowServerRootVolume {
        &self.rows[ptr.0]
    }

    pub fn c_volume_name(&self, ptr: TableRowPointerServerRootVolume) -> &::std::string::String {
        &self.c_volume_name[ptr.0]
    }

    pub fn c_intended_usage(&self, ptr: TableRowPointerServerRootVolume) -> TableRowPointerServerVolumeUsageContract {
        self.c_intended_usage[ptr.0]
    }

    pub fn c_mountpoint(&self, ptr: TableRowPointerServerRootVolume) -> &::std::string::String {
        &self.c_mountpoint[ptr.0]
    }

    pub fn c_zfs_recordsize(&self, ptr: TableRowPointerServerRootVolume) -> &::std::string::String {
        &self.c_zfs_recordsize[ptr.0]
    }

    pub fn c_zfs_compression(&self, ptr: TableRowPointerServerRootVolume) -> bool {
        self.c_zfs_compression[ptr.0]
    }

    pub fn c_zfs_encryption(&self, ptr: TableRowPointerServerRootVolume) -> bool {
        self.c_zfs_encryption[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerRootVolume) -> TableRowPointerServer {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerVolume {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerVolume> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerVolume(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerVolume) -> &TableRowServerVolume {
        &self.rows[ptr.0]
    }

    pub fn c_volume_name(&self, ptr: TableRowPointerServerVolume) -> &::std::string::String {
        &self.c_volume_name[ptr.0]
    }

    pub fn c_mountpoint(&self, ptr: TableRowPointerServerVolume) -> &::std::string::String {
        &self.c_mountpoint[ptr.0]
    }

    pub fn c_intended_usage(&self, ptr: TableRowPointerServerVolume) -> TableRowPointerServerVolumeUsageContract {
        self.c_intended_usage[ptr.0]
    }

    pub fn c_source(&self, ptr: TableRowPointerServerVolume) -> &::std::string::String {
        &self.c_source[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerVolume) -> TableRowPointerServer {
        self.c_parent[ptr.0]
    }

    pub fn c_referrers_pg_deployment_instance__pg_server(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerPgDeploymentInstance] {
        &self.c_referrers_pg_deployment_instance__pg_server[ptr.0]
    }

    pub fn c_referrers_ch_deployment_instance__ch_server(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerChDeploymentInstance] {
        &self.c_referrers_ch_deployment_instance__ch_server[ptr.0]
    }

    pub fn c_referrers_ch_keeper_deployment_instance__keeper_server(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerChKeeperDeploymentInstance] {
        &self.c_referrers_ch_keeper_deployment_instance__keeper_server[ptr.0]
    }

    pub fn c_referrers_nats_deployment_instance__nats_server(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerNatsDeploymentInstance] {
        &self.c_referrers_nats_deployment_instance__nats_server[ptr.0]
    }

    pub fn c_referrers_minio_instance__instance_volume(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerMinioInstance] {
        &self.c_referrers_minio_instance__instance_volume[ptr.0]
    }

    pub fn c_referrers_monitoring_instance__monitoring_server(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerMonitoringInstance] {
        &self.c_referrers_monitoring_instance__monitoring_server[ptr.0]
    }

    pub fn c_referrers_alertmanager_instance__alertmanager_server(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerAlertmanagerInstance] {
        &self.c_referrers_alertmanager_instance__alertmanager_server[ptr.0]
    }

    pub fn c_referrers_blackbox_deployment_task_mount__server_volume(&self, ptr: TableRowPointerServerVolume) -> &[TableRowPointerBlackboxDeploymentTaskMount] {
        &self.c_referrers_blackbox_deployment_task_mount__server_volume[ptr.0]
    }

}

impl TableDefinitionServerVolumeUsageContract {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerVolumeUsageContract> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerVolumeUsageContract(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerVolumeUsageContract) -> &TableRowServerVolumeUsageContract {
        &self.rows[ptr.0]
    }

    pub fn c_usage_contract(&self, ptr: TableRowPointerServerVolumeUsageContract) -> &::std::string::String {
        &self.c_usage_contract[ptr.0]
    }

    pub fn c_referrers_server_volume__intended_usage(&self, ptr: TableRowPointerServerVolumeUsageContract) -> &[TableRowPointerServerVolume] {
        &self.c_referrers_server_volume__intended_usage[ptr.0]
    }

    pub fn c_referrers_server_root_volume__intended_usage(&self, ptr: TableRowPointerServerVolumeUsageContract) -> &[TableRowPointerServerRootVolume] {
        &self.c_referrers_server_root_volume__intended_usage[ptr.0]
    }

    pub fn c_referrers_server_xfs_volume__intended_usage(&self, ptr: TableRowPointerServerVolumeUsageContract) -> &[TableRowPointerServerXfsVolume] {
        &self.c_referrers_server_xfs_volume__intended_usage[ptr.0]
    }

    pub fn c_referrers_server_zfs_dataset__intended_usage(&self, ptr: TableRowPointerServerVolumeUsageContract) -> &[TableRowPointerServerZfsDataset] {
        &self.c_referrers_server_zfs_dataset__intended_usage[ptr.0]
    }

}

impl TableDefinitionServerXfsVolume {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerXfsVolume> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerXfsVolume(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerXfsVolume) -> &TableRowServerXfsVolume {
        &self.rows[ptr.0]
    }

    pub fn c_volume_name(&self, ptr: TableRowPointerServerXfsVolume) -> &::std::string::String {
        &self.c_volume_name[ptr.0]
    }

    pub fn c_xfs_disk(&self, ptr: TableRowPointerServerXfsVolume) -> TableRowPointerServerDisk {
        self.c_xfs_disk[ptr.0]
    }

    pub fn c_intended_usage(&self, ptr: TableRowPointerServerXfsVolume) -> TableRowPointerServerVolumeUsageContract {
        self.c_intended_usage[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerXfsVolume) -> TableRowPointerServer {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerZfsDataset {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerZfsDataset> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerZfsDataset(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerZfsDataset) -> &TableRowServerZfsDataset {
        &self.rows[ptr.0]
    }

    pub fn c_dataset_name(&self, ptr: TableRowPointerServerZfsDataset) -> &::std::string::String {
        &self.c_dataset_name[ptr.0]
    }

    pub fn c_intended_usage(&self, ptr: TableRowPointerServerZfsDataset) -> TableRowPointerServerVolumeUsageContract {
        self.c_intended_usage[ptr.0]
    }

    pub fn c_zfs_recordsize(&self, ptr: TableRowPointerServerZfsDataset) -> &::std::string::String {
        &self.c_zfs_recordsize[ptr.0]
    }

    pub fn c_zfs_compression(&self, ptr: TableRowPointerServerZfsDataset) -> bool {
        self.c_zfs_compression[ptr.0]
    }

    pub fn c_zfs_encryption(&self, ptr: TableRowPointerServerZfsDataset) -> bool {
        self.c_zfs_encryption[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerZfsDataset) -> TableRowPointerServerZpool {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerZpool {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerZpool> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerZpool(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerZpool) -> &TableRowServerZpool {
        &self.rows[ptr.0]
    }

    pub fn c_zpool_name(&self, ptr: TableRowPointerServerZpool) -> &::std::string::String {
        &self.c_zpool_name[ptr.0]
    }

    pub fn c_is_redundant(&self, ptr: TableRowPointerServerZpool) -> bool {
        self.c_is_redundant[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerZpool) -> TableRowPointerServer {
        self.c_parent[ptr.0]
    }

    pub fn c_children_server_zpool_vdev(&self, ptr: TableRowPointerServerZpool) -> &[TableRowPointerServerZpoolVdev] {
        &self.c_children_server_zpool_vdev[ptr.0]
    }

    pub fn c_children_server_zpool_spare(&self, ptr: TableRowPointerServerZpool) -> &[TableRowPointerServerZpoolSpare] {
        &self.c_children_server_zpool_spare[ptr.0]
    }

    pub fn c_children_server_zpool_cache(&self, ptr: TableRowPointerServerZpool) -> &[TableRowPointerServerZpoolCache] {
        &self.c_children_server_zpool_cache[ptr.0]
    }

    pub fn c_children_server_zpool_log(&self, ptr: TableRowPointerServerZpool) -> &[TableRowPointerServerZpoolLog] {
        &self.c_children_server_zpool_log[ptr.0]
    }

    pub fn c_children_server_zfs_dataset(&self, ptr: TableRowPointerServerZpool) -> &[TableRowPointerServerZfsDataset] {
        &self.c_children_server_zfs_dataset[ptr.0]
    }

}

impl TableDefinitionServerZpoolCache {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerZpoolCache> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerZpoolCache(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerZpoolCache) -> &TableRowServerZpoolCache {
        &self.rows[ptr.0]
    }

    pub fn c_disk_id(&self, ptr: TableRowPointerServerZpoolCache) -> TableRowPointerServerDisk {
        self.c_disk_id[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerZpoolCache) -> TableRowPointerServerZpool {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerZpoolLog {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerZpoolLog> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerZpoolLog(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerZpoolLog) -> &TableRowServerZpoolLog {
        &self.rows[ptr.0]
    }

    pub fn c_disk_id(&self, ptr: TableRowPointerServerZpoolLog) -> TableRowPointerServerDisk {
        self.c_disk_id[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerZpoolLog) -> TableRowPointerServerZpool {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerZpoolSpare {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerZpoolSpare> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerZpoolSpare(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerZpoolSpare) -> &TableRowServerZpoolSpare {
        &self.rows[ptr.0]
    }

    pub fn c_disk_id(&self, ptr: TableRowPointerServerZpoolSpare) -> TableRowPointerServerDisk {
        self.c_disk_id[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerZpoolSpare) -> TableRowPointerServerZpool {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionServerZpoolVdev {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerZpoolVdev> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerZpoolVdev(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerZpoolVdev) -> &TableRowServerZpoolVdev {
        &self.rows[ptr.0]
    }

    pub fn c_vdev_number(&self, ptr: TableRowPointerServerZpoolVdev) -> i64 {
        self.c_vdev_number[ptr.0]
    }

    pub fn c_vdev_type(&self, ptr: TableRowPointerServerZpoolVdev) -> &::std::string::String {
        &self.c_vdev_type[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerZpoolVdev) -> TableRowPointerServerZpool {
        self.c_parent[ptr.0]
    }

    pub fn c_children_server_zpool_vdev_disk(&self, ptr: TableRowPointerServerZpoolVdev) -> &[TableRowPointerServerZpoolVdevDisk] {
        &self.c_children_server_zpool_vdev_disk[ptr.0]
    }

}

impl TableDefinitionServerZpoolVdevDisk {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerServerZpoolVdevDisk> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerServerZpoolVdevDisk(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerServerZpoolVdevDisk) -> &TableRowServerZpoolVdevDisk {
        &self.rows[ptr.0]
    }

    pub fn c_disk_id(&self, ptr: TableRowPointerServerZpoolVdevDisk) -> TableRowPointerServerDisk {
        self.c_disk_id[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerServerZpoolVdevDisk) -> TableRowPointerServerZpoolVdev {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionSubnetRouterFloatingIp {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerSubnetRouterFloatingIp> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerSubnetRouterFloatingIp(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerSubnetRouterFloatingIp) -> &TableRowSubnetRouterFloatingIp {
        &self.rows[ptr.0]
    }

    pub fn c_ip_address(&self, ptr: TableRowPointerSubnetRouterFloatingIp) -> &::std::string::String {
        &self.c_ip_address[ptr.0]
    }

}

impl TableDefinitionTelegramBot {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerTelegramBot> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerTelegramBot(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerTelegramBot) -> &TableRowTelegramBot {
        &self.rows[ptr.0]
    }

    pub fn c_bot_name(&self, ptr: TableRowPointerTelegramBot) -> &::std::string::String {
        &self.c_bot_name[ptr.0]
    }

    pub fn c_bot_token(&self, ptr: TableRowPointerTelegramBot) -> &::std::string::String {
        &self.c_bot_token[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster_alert_group__telegram_bot(&self, ptr: TableRowPointerTelegramBot) -> &[TableRowPointerMonitoringClusterAlertGroup] {
        &self.c_referrers_monitoring_cluster_alert_group__telegram_bot[ptr.0]
    }

}

impl TableDefinitionTelegramChannel {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerTelegramChannel> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerTelegramChannel(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerTelegramChannel) -> &TableRowTelegramChannel {
        &self.rows[ptr.0]
    }

    pub fn c_channel_name(&self, ptr: TableRowPointerTelegramChannel) -> &::std::string::String {
        &self.c_channel_name[ptr.0]
    }

    pub fn c_channel_id(&self, ptr: TableRowPointerTelegramChannel) -> i64 {
        self.c_channel_id[ptr.0]
    }

    pub fn c_referrers_monitoring_cluster_alert_group__telegram_channel(&self, ptr: TableRowPointerTelegramChannel) -> &[TableRowPointerMonitoringClusterAlertGroup] {
        &self.c_referrers_monitoring_cluster_alert_group__telegram_channel[ptr.0]
    }

}

impl TableDefinitionTempoCluster {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerTempoCluster> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerTempoCluster(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerTempoCluster) -> &TableRowTempoCluster {
        &self.rows[ptr.0]
    }

    pub fn c_cluster_name(&self, ptr: TableRowPointerTempoCluster) -> &::std::string::String {
        &self.c_cluster_name[ptr.0]
    }

    pub fn c_namespace(&self, ptr: TableRowPointerTempoCluster) -> TableRowPointerNomadNamespace {
        self.c_namespace[ptr.0]
    }

    pub fn c_region(&self, ptr: TableRowPointerTempoCluster) -> TableRowPointerRegion {
        self.c_region[ptr.0]
    }

    pub fn c_workload_architecture(&self, ptr: TableRowPointerTempoCluster) -> &::std::string::String {
        &self.c_workload_architecture[ptr.0]
    }

    pub fn c_docker_image(&self, ptr: TableRowPointerTempoCluster) -> TableRowPointerDockerImagePin {
        self.c_docker_image[ptr.0]
    }

    pub fn c_is_region_default(&self, ptr: TableRowPointerTempoCluster) -> bool {
        self.c_is_region_default[ptr.0]
    }

    pub fn c_loki_cluster(&self, ptr: TableRowPointerTempoCluster) -> &::std::string::String {
        &self.c_loki_cluster[ptr.0]
    }

    pub fn c_monitoring_cluster(&self, ptr: TableRowPointerTempoCluster) -> &::std::string::String {
        &self.c_monitoring_cluster[ptr.0]
    }

    pub fn c_storage_bucket(&self, ptr: TableRowPointerTempoCluster) -> TableRowPointerMinioBucket {
        self.c_storage_bucket[ptr.0]
    }

    pub fn c_http_port(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_http_port[ptr.0]
    }

    pub fn c_grpc_port(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_grpc_port[ptr.0]
    }

    pub fn c_p2p_port(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_p2p_port[ptr.0]
    }

    pub fn c_otlp_http_port(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_otlp_http_port[ptr.0]
    }

    pub fn c_otlp_grpc_port(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_otlp_grpc_port[ptr.0]
    }

    pub fn c_tempo_instances(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_tempo_instances[ptr.0]
    }

    pub fn c_placement(&self, ptr: TableRowPointerTempoCluster) -> &::std::string::String {
        &self.c_placement[ptr.0]
    }

    pub fn c_trace_retention_days(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_trace_retention_days[ptr.0]
    }

    pub fn c_memory_mb(&self, ptr: TableRowPointerTempoCluster) -> i64 {
        self.c_memory_mb[ptr.0]
    }

}

impl TableDefinitionTld {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerTld> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerTld(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerTld) -> &TableRowTld {
        &self.rows[ptr.0]
    }

    pub fn c_domain(&self, ptr: TableRowPointerTld) -> &::std::string::String {
        &self.c_domain[ptr.0]
    }

    pub fn c_expose_admin(&self, ptr: TableRowPointerTld) -> bool {
        self.c_expose_admin[ptr.0]
    }

    pub fn c_automatic_certificates(&self, ptr: TableRowPointerTld) -> bool {
        self.c_automatic_certificates[ptr.0]
    }

    pub fn c_referrers_region__tld(&self, ptr: TableRowPointerTld) -> &[TableRowPointerRegion] {
        &self.c_referrers_region__tld[ptr.0]
    }

    pub fn c_referrers_backend_application_deployment_ingress__tld(&self, ptr: TableRowPointerTld) -> &[TableRowPointerBackendApplicationDeploymentIngress] {
        &self.c_referrers_backend_application_deployment_ingress__tld[ptr.0]
    }

    pub fn c_referrers_frontend_application_deployment_ingress__tld(&self, ptr: TableRowPointerTld) -> &[TableRowPointerFrontendApplicationDeploymentIngress] {
        &self.c_referrers_frontend_application_deployment_ingress__tld[ptr.0]
    }

}

impl TableDefinitionUniqueApplicationNames {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerUniqueApplicationNames> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerUniqueApplicationNames(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerUniqueApplicationNames) -> &TableRowUniqueApplicationNames {
        &self.rows[ptr.0]
    }

    pub fn c_application_name(&self, ptr: TableRowPointerUniqueApplicationNames) -> &::std::string::String {
        &self.c_application_name[ptr.0]
    }

    pub fn c_source(&self, ptr: TableRowPointerUniqueApplicationNames) -> &::std::string::String {
        &self.c_source[ptr.0]
    }

}

impl TableDefinitionUniqueDeploymentNames {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerUniqueDeploymentNames> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerUniqueDeploymentNames(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerUniqueDeploymentNames) -> &TableRowUniqueDeploymentNames {
        &self.rows[ptr.0]
    }

    pub fn c_deployment_name(&self, ptr: TableRowPointerUniqueDeploymentNames) -> &::std::string::String {
        &self.c_deployment_name[ptr.0]
    }

    pub fn c_source(&self, ptr: TableRowPointerUniqueDeploymentNames) -> &::std::string::String {
        &self.c_source[ptr.0]
    }

}

impl TableDefinitionValidServerLabels {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerValidServerLabels> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerValidServerLabels(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerValidServerLabels) -> &TableRowValidServerLabels {
        &self.rows[ptr.0]
    }

    pub fn c_label_name(&self, ptr: TableRowPointerValidServerLabels) -> &::std::string::String {
        &self.c_label_name[ptr.0]
    }

    pub fn c_referrers_server_label__label_name(&self, ptr: TableRowPointerValidServerLabels) -> &[TableRowPointerServerLabel] {
        &self.c_referrers_server_label__label_name[ptr.0]
    }

}

impl TableDefinitionVersionedType {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerVersionedType> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerVersionedType(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerVersionedType) -> &TableRowVersionedType {
        &self.rows[ptr.0]
    }

    pub fn c_type_name(&self, ptr: TableRowPointerVersionedType) -> &::std::string::String {
        &self.c_type_name[ptr.0]
    }

    pub fn c_children_versioned_type_snapshot(&self, ptr: TableRowPointerVersionedType) -> &[TableRowPointerVersionedTypeSnapshot] {
        &self.c_children_versioned_type_snapshot[ptr.0]
    }

    pub fn c_children_versioned_type_migration(&self, ptr: TableRowPointerVersionedType) -> &[TableRowPointerVersionedTypeMigration] {
        &self.c_children_versioned_type_migration[ptr.0]
    }

    pub fn c_referrers_nats_jetstream_stream__stream_type(&self, ptr: TableRowPointerVersionedType) -> &[TableRowPointerNatsJetstreamStream] {
        &self.c_referrers_nats_jetstream_stream__stream_type[ptr.0]
    }

    pub fn c_referrers_backend_application_nats_stream__stream_type(&self, ptr: TableRowPointerVersionedType) -> &[TableRowPointerBackendApplicationNatsStream] {
        &self.c_referrers_backend_application_nats_stream__stream_type[ptr.0]
    }

}

impl TableDefinitionVersionedTypeMigration {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerVersionedTypeMigration> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerVersionedTypeMigration(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerVersionedTypeMigration) -> &TableRowVersionedTypeMigration {
        &self.rows[ptr.0]
    }

    pub fn c_version(&self, ptr: TableRowPointerVersionedTypeMigration) -> i64 {
        self.c_version[ptr.0]
    }

    pub fn c_migration_source(&self, ptr: TableRowPointerVersionedTypeMigration) -> &::std::string::String {
        &self.c_migration_source[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerVersionedTypeMigration) -> TableRowPointerVersionedType {
        self.c_parent[ptr.0]
    }

}

impl TableDefinitionVersionedTypeSnapshot {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_iter(&self) -> impl ::std::iter::Iterator<Item = TableRowPointerVersionedTypeSnapshot> {
        (0..self.rows.len()).map(|idx| {
            TableRowPointerVersionedTypeSnapshot(idx)
        })
    }

    pub fn row(&self, ptr: TableRowPointerVersionedTypeSnapshot) -> &TableRowVersionedTypeSnapshot {
        &self.rows[ptr.0]
    }

    pub fn c_version(&self, ptr: TableRowPointerVersionedTypeSnapshot) -> i64 {
        self.c_version[ptr.0]
    }

    pub fn c_snapshot_source(&self, ptr: TableRowPointerVersionedTypeSnapshot) -> &::std::string::String {
        &self.c_snapshot_source[ptr.0]
    }

    pub fn c_parent(&self, ptr: TableRowPointerVersionedTypeSnapshot) -> TableRowPointerVersionedType {
        self.c_parent[ptr.0]
    }

}

