use std::collections::{BTreeMap, HashMap, HashSet, BTreeSet};

use convert_case::{Case, Casing};
use regex::Regex;

use crate::{database::{
    Database, TableRowPointerPgDeploymentSchemas, TableRowPointerPgDeploymentUnmanagedDb,
    TableRowPointerMinioBucket, TableRowPointerMonitoringCluster, TableRowPointerServer,
    TableRowPointerServerVolume, TableRowPointerTld, TableRowPointerRegion, TableRowPointerLokiCluster, TableRowPointerServerKind, TableRowPointerDockerImage, TableRowPointerChDeploymentSchemas, TableRowPointerNomadNamespace, TableRowPointerCustomSecret,
}, codegen::{l1_provisioning::{consul::consul_tests, nomad::nomad_tests, vault::vault_tests}, CodegenSecrets}, static_analysis::get_global_settings};

use super::{
    http_endpoints::{CorePathSegment, HttpPathTree, PageMethod},
    L1Projections, PlatformValidationError, dns::dns_tests, networking::{admin_service_responds_test, networking_tests, first_region_server}, projections::Projection, docker_images::DockerImageHandle, server_labels::{LabelQuery, LabelDatabase}, dc_impl::system_reserved_memory_bytes,
};

pub struct VaultSecrets {
    pub declare_only: bool,
    pub renew_if_source_changed: bool,
    pub keys: BTreeMap<String, VaultSecretHandle>,
}

#[derive(Default)]
pub struct CustomSecretUsageTypes {
    pub used_in_env: bool,
}

#[derive(Default)]
struct RegionRuntime {
    secrets_from_yml: BTreeMap<TableRowPointerCustomSecret, CustomSecretUsageTypes>,
    vault_secrets: BTreeMap<String, VaultSecrets>,
    consul_services: BTreeMap<String, ConsulServiceState>,
    consul_kv_entries: BTreeMap<String, ConsulKvEntry>,
    provisioning_scripts: BTreeMap<u8, Vec<ProvisioningScript>>,
    nomad_jobs: BTreeMap<String, NomadJob>,
    provisioning_script_resources:
        BTreeMap<&'static str, BTreeMap<String, ProvisioningScriptResource>>,
    routes: BTreeMap<TableRowPointerTld, ExternalLbSubdomainRoutes>,
}

pub struct ServerRuntime {
    server_data: HashMap<TableRowPointerServer, SingleServerData>,
    server_label_locks: BTreeMap<LockedServerLabel, BTreeSet<TableRowPointerServer>>,
    #[allow(dead_code)]
    stateless_memory: Vec<ReservedMemory>,
    // separate policy per secret, most secure
    // one kv engine
    minio_bucket_users: BTreeMap<TableRowPointerMinioBucket, BTreeMap<String, MinIOUser>>,
    minio_users_sealed: bool,
    region_runtime: BTreeMap<TableRowPointerRegion, RegionRuntime>,
    locked_all_server_ports: BTreeMap<u16, LockedPort>,
    pg_access: HashMap<PgAccessKind, PostgresDbCredentials>,
    ch_access: HashMap<ChAccessKind, ClickhouseDbCredentials>,
    // Resource kinds -> resources
    admin_links: BTreeMap<String, Vec<String>>,
    no_servers_volume_locks: HashMap<ServerVolumeKey, ServerVolumeLocks>,
    integration_tests: BTreeMap<String, IntegrationTest>,
    server_kinds: Projection<TableRowPointerServer, TableRowPointerServerKind>,
    admin_dns_ingress_services: BTreeMap<TableRowPointerTld, Vec<String>>,
}

pub enum IntegrationTestCredentials {
    AdminPanel,
    GrafanaCluster(String),
}

pub enum IntegrationTest {
    DnsResolutionWorksARecords {
        target_servers: Vec<String>,
        queries: Vec<(String, Vec<String>)>,
    },
    DnsResolutionARecordCount {
        target_servers: Vec<String>,
        queries: Vec<(String, usize)>,
    },
    DnsResolutionWorksNsRecords {
        target_servers: Vec<String>,
        queries: Vec<(String, Vec<String>)>,
    },
    DnsResolutionWorksPtrRecords {
        target_servers: Vec<String>,
        queries: Vec<(String, Vec<String>)>,
    },
    DnsSecWorksInternal {
        target_servers: Vec<String>,
        source_ip: String,
        server_to_lookup: String,
        server_to_lookup_ip: String,
        region: String,
        tld: String,
    },
    DnsSecWorksExternal {
        target_servers: Vec<String>,
        dns_to_lookup: Vec<String>,
    },
    TcpSocketsOpen {
        target_sockets: Vec<String>,
    },
    PrometheusMetricExists {
        prometheus_server_ip: String,
        prometheus_server_port: i64,
        metric: String,
        should_exist: bool,
    },
    HttpGetRespondsOk {
        server_ips: Vec<String>,
        http_server_port: i64,
        path: String,
    },
    HttpGetRespondsString {
        hostname: Option<String>,
        server_ips: Vec<String>,
        http_server_port: i64,
        path: String,
        is_https: bool,
        expected_string: String,
        use_admin_panel_credentials: Option<IntegrationTestCredentials>,
    },
    PingWorks {
        server_ips: Vec<String>,
    },
    InsideNodePingWorks {
        server_ips: BTreeMap<String, Vec<String>>,
    },
    InsideNodeDnsAResolutionWorks {
        server_ips: BTreeMap<String, Vec<String>>,
    },
    CrossDcSourceIpCheck {
        port_range_start: usize,
        server_to_run_iperf_server_from_with_private_ip: (String, String),
        servers_to_run_iperf_client_from_with_expected_ips: Vec<(String, String)>,
    },
    LokiWriterReaderTest {
        dns_server: String,
        reader_dns_name: String,
        writer_dns_name: String,
        reader_port: i64,
        writer_port: i64,
    },
    LokiStreamExists {
        dns_server: String,
        reader_dns_name: String,
        reader_port: i64,
        stream_identifiers: Vec<(String, String)>,
    },
    TempoSpansWritable {
        dns_server: String,
        service_name: String,
        push_port: i64,
        query_port: i64,
    },
}


#[derive(Eq, PartialEq, Hash)]
pub enum PgAccessKind {
    Managed(TableRowPointerPgDeploymentSchemas),
    UnmanagedRw(TableRowPointerPgDeploymentUnmanagedDb),
}

#[derive(Eq, PartialEq, Hash)]
pub enum ChAccessKind {
    ManagedReadOnly(TableRowPointerChDeploymentSchemas),
    ManagedReadWrite(TableRowPointerChDeploymentSchemas),
}

pub struct AdminService {
    pub service_title: String,
    pub service_kind: String,
    pub service_instance: String,
    pub service_internal_upstream: ConsulServiceHandle,
    pub service_internal_port: u16,
    pub is_https: bool,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidSubdomain {
    pub subdomain: String,
}

pub struct ExternalLbSubdomainRoutes {
    pub tld: TableRowPointerTld,
    pub subdomains: BTreeMap<ValidSubdomain, SubdomainRouteChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteContent {
    InternalUpstream {
        is_https: bool,
        consul_service: ConsulServiceHandle,
        port: u16,
        target_path: String,
        unlimited_body: bool,
    },
}

impl RouteContent {
    pub fn is_epl_app(&self) -> bool {
        match self {
            RouteContent::InternalUpstream { consul_service, .. } => {
                consul_service.service_name.starts_with("epl-app-")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteData {
    pub content: RouteContent,
    pub basic_auth: String,
}

// we can slice only by /path/
// key to map is Vec<String>, all must be unique
pub struct SubdomainRouteChunk {
    pub routes: HttpPathTree<RouteData>,
}

#[derive(Clone)]
pub struct PostgresDbCredentials {
    pub db_host: String,
    pub db_master_port: u16,
    pub db_user: String,
    pub db_password: VaultSecretHandle,
    pub db_database: String,
}

#[derive(Clone)]
pub struct ClickhouseDbCredentials {
    pub db_host: String,
    pub db_http_port: u16,
    pub db_user: String,
    pub db_password: VaultSecretHandle,
    pub db_database: String,
}

#[derive(strum_macros::EnumIter, PartialEq, Eq, Debug)]
pub enum ProvisioningScriptTag {
    L1Resources,             // provision vault secrets
    RunNomadSystemJob,       // all the system jobs, postgres, queues
    SystemResourceProvision, // provision system resources, like database migrations
    EplApplicationBuild,     // build epl applications
    EplApplicationPush,      // build epl applications
    RunNomadAppJob,          // rn nomad epl applications
    NonEssentialsProvision,  // grafana dashboards
}

impl ProvisioningScriptTag {
    fn tag_weight(&self) -> u8 {
        match self {
            ProvisioningScriptTag::L1Resources => 10,
            ProvisioningScriptTag::RunNomadSystemJob => 20,
            ProvisioningScriptTag::SystemResourceProvision => 30,
            ProvisioningScriptTag::EplApplicationBuild => 40,
            ProvisioningScriptTag::EplApplicationPush => 50,
            ProvisioningScriptTag::RunNomadAppJob => 60,
            ProvisioningScriptTag::NonEssentialsProvision => 70,
        }
    }

    #[cfg(test)]
    fn tag_dependencies(&self) -> Vec<ProvisioningScriptTag> {
        match self {
            ProvisioningScriptTag::L1Resources => vec![], // every secret depends on this
            ProvisioningScriptTag::EplApplicationBuild => vec![],
            ProvisioningScriptTag::RunNomadSystemJob => vec![
                ProvisioningScriptTag::L1Resources, // depends on vault secrets
            ],
            ProvisioningScriptTag::SystemResourceProvision => vec![
                ProvisioningScriptTag::RunNomadSystemJob, // depends on running the jobs
            ],
            ProvisioningScriptTag::RunNomadAppJob => vec![
                ProvisioningScriptTag::RunNomadSystemJob, // apps depend on nats/postgres
                ProvisioningScriptTag::L1Resources, // apps depend on vault secrets/consul tokens
            ],
            ProvisioningScriptTag::EplApplicationPush => vec![
                ProvisioningScriptTag::EplApplicationBuild,
                ProvisioningScriptTag::RunNomadSystemJob, // need MinIO
            ],
            ProvisioningScriptTag::NonEssentialsProvision => vec![
                // grafana dashboards needs db
                ProvisioningScriptTag::SystemResourceProvision,
            ]
        }
    }
}

pub struct ProvisioningResourcePath {
    path: String,
}

impl ProvisioningResourcePath {
    pub fn path(&self) -> &str {
        &self.path
    }
}

// static files, sql scripts, etc.
// all must be synced
pub struct ProvisioningScriptResource {
    contents: String,
    is_executable: bool,
    replacement_macros: Vec<ReplacementMacro>,
}

impl ProvisioningScriptResource {
    pub fn is_executable(&self) -> bool {
        self.is_executable
    }

    pub fn replaced_contents(&self, secrets: &CodegenSecrets) -> String {
        if self.replacement_macros.is_empty() {
            return self.contents.clone();
        }

        let mut output = self.contents.clone();

        for rm in &self.replacement_macros {
            let find = &rm.find;
            let (sec_key, replace_with) =
                match &rm.replace {
                    ReplaceWith::EplSecretKeyValue(sec_key) => {
                        match sec_key.as_str() {
                            "admin_panel_htpasswd_file" => {
                                (sec_key, secrets.admin_panel_htpasswd_file.value())
                            }
                            _ => {
                                panic!("Unknown secret key {sec_key} in secret storage to replace macro {find}")
                            }
                        }
                    }
                };
            let new = output.replace(find, &replace_with);
            if new == output {
                panic!("Replacement find {find} with secret key {sec_key} failed");
            }
            output = new;
        }

        output
    }
}

// How to:
// have files for migrations
pub struct ProvisioningScript {
    name: String,
    script: String,
}

impl ProvisioningScript {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn script(&self) -> &str {
        &self.script
    }
}

#[derive(Clone)]
pub struct MinIOUser {
    username: String,
    password: VaultSecretHandle,
    permission: MinIOBucketPermission,
}

#[derive(Clone)]
pub enum MinIOBucketPermission {
    // TODO: read only
    ReadWrite,
}

#[derive(Clone)]
pub struct WrittenConsulKvValue {
    path: String,
}

pub struct ConsulKvEntry {
    content: String,
}

struct ConsulServiceState {
    is_sealed: bool,
    #[allow(dead_code)]
    service_name: String,
    instantiation_count: usize,
    region: TableRowPointerRegion,
}

impl ConsulServiceState {
    fn handle(&self) -> ConsulServiceHandle {
        ConsulServiceHandle {
            service_name: self.service_name.clone(),
            region: self.region,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ConsulServiceHandle {
    region: TableRowPointerRegion,
    service_name: String,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct VaultSecretHandle {
    request: VaultSecretRequest,
    vault_secret_engine: String,
    vault_secret_name: String,
    vault_secret_key: String,
    left: String,
    right: String,
}

impl VaultSecretHandle {
    pub fn template_expression(&self) -> String {
        let engine = &self.vault_secret_engine;
        let name = &self.vault_secret_name;
        let key = &self.vault_secret_key;
        let left = &self.left;
        let right = &self.right;
        //format!(
        //    r#"{left}{{{{ with secret "{engine}/data/{name}" }}}}{{{{ .Data.data.{key} | toJSON }}}}{{{{ end }}}}{right}"#
        //)
        format!(
            r#"{left}{{{{ with secret "{engine}/data/{name}" }}}}{{{{ .Data.data.{key} }}}}{{{{ end }}}}{right}"#
        )
    }

    pub fn surround_expression(&self, left: String, right: String) -> VaultSecretHandle {
        let mut cloned = self.clone();
        cloned.left = left;
        cloned.right = right;
        cloned
    }

    pub fn request(&self) -> &VaultSecretRequest {
        &self.request
    }

    pub fn secret_kv_path(&self) -> String {
        format!("{}/{}", self.vault_secret_engine, self.vault_secret_name)
    }

    pub fn secret_key_data_path(&self) -> String {
        format!(".data.data.{}", self.vault_secret_key)
    }

    pub fn from_epl_kv_secret(name: &str, key: &str, request: VaultSecretRequest) -> VaultSecretHandle {
        Self {
            request,
            vault_secret_engine: "epl".to_string(),
            vault_secret_name: name.to_string(),
            vault_secret_key: key.to_string(),
            left: "".to_string(),
            right: "".to_string(),
        }
    }
}

pub fn compute_server_runtime(
    db: &Database,
    l1proj: &L1Projections,
) -> Result<ServerRuntime, PlatformValidationError> {
    let mut runtime = ServerRuntime::new(db, l1proj);

    // initialize all server data for memory analysis
    for server in db.server().rows_iter() {
        let _ = runtime.fetch_server_data(db, server);
    }

    super::l2_provisioning::deploy_all_components(db, &mut runtime, l1proj)?;
    add_l1_components_tests(db, l1proj, &mut runtime);
    add_nomad_jobs_tests(db, l1proj, &mut runtime);

    Ok(runtime)
}

fn add_l1_components_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    sys_tests(db, l1proj, runtime);
    networking_tests(db, l1proj, runtime);
    dns_tests(db, l1proj, runtime);
    consul_tests(db, l1proj, runtime);
    nomad_tests(db, l1proj, runtime);
    vault_tests(db, l1proj, runtime);
}

fn add_nomad_jobs_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    let mut int_tests_to_add: Vec<(String, IntegrationTest)> = Vec::new();
    for region in db.region().rows_iter() {
        let region_snake = db.region().c_region_name(region).to_case(Case::Snake);
        let jobs = runtime.nomad_jobs(region);
        if let Some(first_srv) = first_region_server(db, region) {
            let iface = l1proj.consul_network_iface.value(first_srv);
            let dns_ip = db.network_interface().c_if_ip(*iface);

            for job in jobs.values() {
                let job_name = job.job_name();
                let job_name_snake = job_name.to_case(Case::Snake);
                if let Some(loki_cluster) = job.loki_cluster() {
                    let loki_name = db.loki_cluster().c_cluster_name(loki_cluster);
                    let loki_snake = loki_name.to_case(Case::Snake);
                    int_tests_to_add.push((
                        format!("nomad_job_region_{region_snake}_job_{job_name_snake}_logs_in_{loki_snake}"),
                        IntegrationTest::LokiStreamExists {
                            dns_server: format!("{dns_ip}:53"),
                            reader_dns_name: format!("epl-loki-{loki_name}-loki-reader.service.consul"),
                            reader_port: db.loki_cluster().c_loki_reader_http_port(loki_cluster),
                            stream_identifiers: vec![
                                ("source_type".to_string(), "nomad_docker".to_string()),
                                ("job_name".to_string(), job_name.to_string()),
                            ],
                        },
                    ))
                }
            }
        }
    }

    for (test_name, test) in int_tests_to_add {
        runtime.add_integration_test(test_name, test);
    }
}

pub fn sys_tests(db: &Database, l1proj: &L1Projections, runtime: &mut ServerRuntime) {
    runtime.add_integration_test(
        format!("admin_panel_responds_responds"),
        admin_service_responds_test(
            db,
            l1proj,
            "admin".to_string(),
            "/",
            "<h1>Eden Platform admin panel</h1>"
        )
    );
}

pub struct VaultSecretBuilder<'a> {
    sr: &'a mut ServerRuntime,
    engine_key: String,
    vault_secret_name: String,
}

#[derive(Clone)]
pub struct FinalizedVaultSecrets {
    #[allow(dead_code)]
    engine_key: String,
    #[allow(dead_code)]
    vault_secret_name: String,
    vault_policy_name: String,
}

impl FinalizedVaultSecrets {
    pub fn vault_policy_name(&self) -> &str {
        &self.vault_policy_name
    }
}

fn is_valid_s3_username_regex(input: &str) -> bool {
    lazy_static! {
        pub static ref S3_USERNAME_REGEX: Regex = Regex::new(r#"^[a-z0-9_-]+$"#).unwrap();
    }

    S3_USERNAME_REGEX.is_match(input)
}

fn is_valid_vault_secret_regex(input: &str) -> bool {
    lazy_static! {
        pub static ref VAULT_SECRET_REGEX: Regex = Regex::new(r#"^[a-z0-9_-]+$"#).unwrap();
    }

    VAULT_SECRET_REGEX.is_match(input)
}

fn is_valid_consul_service_name(input: &str) -> bool {
    lazy_static! {
        pub static ref CONSUL_SNAME_REGEX: Regex = Regex::new(r#"^[a-z0-9-]+$"#).unwrap();
    }

    CONSUL_SNAME_REGEX.is_match(input)
}

fn is_valid_vault_key_name(input: &str) -> bool {
    lazy_static! {
        pub static ref VAULT_KEY_REGEX: Regex = Regex::new(r#"^[/a-z0-9-]+$"#).unwrap();
    }

    VAULT_KEY_REGEX.is_match(input)
}

fn is_valid_integration_test_name(input: &str) -> bool {
    lazy_static! {
        pub static ref INTEGRATION_TEST_REGEX: Regex = Regex::new(r#"^[a-z][a-z0-9_]+$"#).unwrap();
    }

    INTEGRATION_TEST_REGEX.is_match(input)
}

fn is_valid_environment_variable(input: &str) -> bool {
    lazy_static! {
        pub static ref ENV_VAR_REGEX: Regex = Regex::new(r#"^[A-Z][A-Z0-9_]+$"#).unwrap();
    }

    ENV_VAR_REGEX.is_match(input)
}

impl<'a> VaultSecretBuilder<'a> {
    pub fn request_secret(&mut self, region: TableRowPointerRegion, key: &str, request: VaultSecretRequest) -> VaultSecretHandle {
        assert!(
            is_valid_vault_secret_regex(key),
            "Key to the secret must contain only alphanumeric characters and _-, got [{key}]"
        );

        let handle = VaultSecretHandle {
            request,
            vault_secret_engine: self.engine_key.clone(),
            vault_secret_name: self.vault_secret_name.clone(),
            vault_secret_key: key.to_string(),
            left: "".to_string(),
            right: "".to_string(),
        };

        let map = self
            .sr
            .region_runtime
            .entry(region)
            .or_default()
            .vault_secrets
            .get_mut(&self.vault_secret_name)
            .unwrap();

        assert!(
            map.keys.insert(key.to_string(), handle.clone()).is_none(),
            "Requesting duplicate secret with key {key}"
        );

        handle
    }

    pub fn finalize(self) -> FinalizedVaultSecrets {
        FinalizedVaultSecrets {
            engine_key: self.engine_key,
            vault_policy_name: format!("epl-{}", self.vault_secret_name.replace("/", "-")),
            vault_secret_name: self.vault_secret_name,
        }
    }

    pub fn fetch_pg_access(&self, ptr: &PgAccessKind) -> &PostgresDbCredentials {
        self.sr.fetch_pg_access(ptr)
    }

    /// Fetch MinIO bucket credentials to use in configuration
    /// for wanted job.
    ///
    /// Provide vault key for secret builder
    /// User to create
    /// Bucket and its permissions
    pub fn fetch_minio_bucket_credentials(
        &mut self,
        db: &Database,
        key: &str,
        user: &str,
        bucket: TableRowPointerMinioBucket,
        permission: MinIOBucketPermission,
    ) -> VaultSecretHandle {
        assert!(!self.sr.minio_users_sealed);

        assert!(
            is_valid_s3_username_regex(user),
            "MinIO username can only contain alphanumeric characters and _, got [{user}]"
        );

        let region = db.minio_cluster().c_region(db.minio_bucket().c_parent(bucket));
        let secret = self.request_secret(region, key, VaultSecretRequest::AwsSecretKey);
        let entry = self.sr.minio_bucket_users.entry(bucket).or_default();

        assert!(
            !entry.contains_key(user),
            "MinIO user {} is tried to be registered twice",
            user
        );

        assert!(entry
            .insert(
                user.to_string(),
                MinIOUser {
                    username: user.to_string(),
                    password: secret.clone(),
                    permission,
                }
            )
            .is_none());

        secret
    }
}

pub struct CustomConsulPolicyBuilder {
    segments: Vec<String>,
    name: String,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct CustomConsulPolicy {
    source: String,
    name: String,
}

impl CustomConsulPolicy {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn policy_name(&self) -> &str {
        &self.name
    }

    /// token name should always be same as policy name
    #[allow(dead_code)]
    pub fn token_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum VaultSecretRequest {
    ConsulTokenWithPolicy {
        policy: CustomConsulPolicy,
    },
    AlphanumericPassword42Symbols,
    PasswordSaltOfCurrentSecret {
        key_name: String,
    },
    AwsSecretKey,
    Pem,
    ExistingVaultSecret {
        handle: Box<VaultSecretHandle>,
        sprintf: Option<String>,
    },
    SecretsYmlEntry {
        key_name: String,
    },
}

impl ServerRuntime {
    fn new(db: &Database, l1proj: &L1Projections) -> Self {
        let mut region_runtime = BTreeMap::new();
        // init all regions
        for region in db.region().rows_iter() {
            let _ = region_runtime.entry(region).or_default();
        }
        Self {
            server_data: Default::default(),
            region_runtime,
            server_label_locks: Default::default(),
            stateless_memory: Default::default(),
            locked_all_server_ports: Default::default(),
            minio_bucket_users: Default::default(),
            minio_users_sealed: false,
            pg_access: Default::default(),
            ch_access: Default::default(),
            admin_links: Default::default(),
            no_servers_volume_locks: default_locked_volumes("test-server".to_string()),
            integration_tests: Default::default(),
            server_kinds: l1proj.server_kinds.clone(),
            admin_dns_ingress_services: Default::default(),
        }
    }

    pub fn declare_region_custom_secret_needed_as_env(&mut self, region: TableRowPointerRegion, secret: TableRowPointerCustomSecret) {
        let region_runtime = self.region_runtime.get_mut(&region).expect("Must have been initialized");
        let e = region_runtime.secrets_from_yml.entry(secret).or_default();
        e.used_in_env = true;
    }

    pub fn declare_region_custom_secret_needed_as_file(&mut self, region: TableRowPointerRegion, secret: TableRowPointerCustomSecret) {
        let region_runtime = self.region_runtime.get_mut(&region).expect("Must have been initialized");
        let _ = region_runtime.secrets_from_yml.entry(secret).or_default();
    }

    pub fn add_integration_test(&mut self, name: String, test: IntegrationTest) {
        assert!(is_valid_integration_test_name(&name), "All tests must have snake case, received [{name}]");
        integration_test_validation(&test);
        assert!(self.integration_tests.insert(name.clone(), test).is_none(), "Duplicate test detected [{name}]");
    }

    pub fn all_integration_tests(&self) -> &BTreeMap<String, IntegrationTest> {
        &self.integration_tests
    }

    pub fn admin_links(&self) -> &BTreeMap<String, Vec<String>> {
        &self.admin_links
    }

    pub fn nomad_jobs(&self, region: TableRowPointerRegion) -> &BTreeMap<String, NomadJob> {
        &self.region_runtime.get(&region).unwrap().nomad_jobs
    }

    pub fn fetch_server_data(
        &mut self,
        db: &Database,
        server: TableRowPointerServer,
    ) -> &mut SingleServerData {
        self.server_data
            .entry(server)
            .or_insert_with(|| SingleServerData::new(db, server, *self.server_kinds.value(server)))
    }

    pub fn fetch_nomad_job(
        &mut self,
        namespace: TableRowPointerNomadNamespace,
        name: String,
        region: TableRowPointerRegion,
        kind: NomadJobKind,
        stage: NomadJobStage,
    ) -> &mut NomadJob {
        let job = self
            .region_runtime
            .entry(region)
            .or_default()
            .nomad_jobs
            .entry(name.clone())
            .or_insert_with(|| NomadJob {
                job_name: name.clone(),
                custom_blocks: Vec::new(),
                task_groups: Default::default(),
                vault_policy: None,
                namespace,
                kind,
                stage,
                region,
                loki_cluster: None,
                update_strategy: None,
                replacement_macros: Default::default(),
            });

        assert_eq!(job.kind, kind, "Job kinds not equal to instantiated job");
        assert_eq!(job.region, region, "Job regions not equal to instantiated job");
        assert_eq!(
            job.stage, stage,
            "Job stages not equal to instantiated stage"
        );

        job
    }

    pub fn lock_server_with_label(
        &mut self,
        db: &Database,
        label: String,
        server: TableRowPointerServer,
    ) -> Result<LockedServerLabel, PlatformValidationError> {
        self.lock_servers_with_label(db, label, &[server])
    }

    pub fn lock_servers_with_label(
        &mut self,
        db: &Database,
        label: String,
        servers: &[TableRowPointerServer]
    ) -> Result<LockedServerLabel, PlatformValidationError> {
        let label = LockedServerLabel { label };
        if self.server_label_locks.contains_key(&label) {
            assert!(!servers.is_empty());
            return Err(
                PlatformValidationError::AttemptedToLockServerTwiceWithSameLabel {
                    lock_label: label.label,
                    server_hostname: db.server().c_hostname(servers[0]).clone(),
                },
            );
        }
        let mut set = BTreeSet::new();
        for server in servers {
            assert!(set.insert(*server));
        }
        let cloned = label.clone();
        assert!(self.server_label_locks.insert(label, set).is_none());
        Ok(cloned)
    }

    pub fn issue_vault_secret(&mut self, region: TableRowPointerRegion, component: &str) -> VaultSecretBuilder {
        self.issue_vault_secret_internal(region, component, false, false)
    }

    pub fn issue_vault_secret_renew_from_source(&mut self, region: TableRowPointerRegion, component: &str) -> VaultSecretBuilder {
        self.issue_vault_secret_internal(region, component, false, true)
    }

    pub fn declare_vault_secret(&mut self, region: TableRowPointerRegion, component: &str) -> VaultSecretBuilder {
        self.issue_vault_secret_internal(region, component, true, false)
    }


    fn issue_vault_secret_internal(&mut self, region: TableRowPointerRegion, component: &str, declare_only: bool, renew_if_source_changed: bool) -> VaultSecretBuilder {
        assert!(
            is_valid_vault_key_name(component),
            "Vault key name must be kebab case with optional slashes, got {component}"
        );

        let engine_key = "epl";

        let new_map = BTreeMap::new();
        let region_runtime =
            self
                .region_runtime
                .entry(region)
                .or_default();
        assert!(
            region_runtime
                .vault_secrets
                .insert(component.to_string(), VaultSecrets { declare_only, keys: new_map, renew_if_source_changed })
                .is_none(),
            "Cannot create secret builder more than once for component {component}"
        );

        let _ = region_runtime.vault_secrets.entry(component.to_string()).or_insert_with(|| {
            VaultSecrets { declare_only, keys: BTreeMap::new(), renew_if_source_changed }
        });

        VaultSecretBuilder {
            sr: self,
            engine_key: engine_key.to_string(),
            vault_secret_name: component.to_string(),
        }
    }

    /// memory unbound to any server
    pub fn reserve_stateless_memory_mb(&mut self, comment: String, mb: i64) -> ReservedMemory {
        assert!(mb > 0);

        self.stateless_memory.push(ReservedMemory {
            kind: MemoryBindKind::Stateless,
            comment: comment.clone(),
            bytes: mb * 1024 * 1024,
        });
        ReservedMemory {
            kind: MemoryBindKind::Stateless,
            comment,
            bytes: mb * 1024 * 1024,
        }
    }

    /// memory to be used on every server
    pub fn reserve_memory_every_server_mb(
        &mut self,
        db: &Database,
        comment: String,
        mb: i64,
    ) -> Result<ReservedMemory, PlatformValidationError> {
        assert!(mb > 0);

        let bytes = mb * 1024 * 1024;
        for server in db.server().rows_iter() {
            let ssd = self.fetch_server_data(db, server);
            ssd.memory_reservation_log.push(ReservedMemory {
                kind: MemoryBindKind::EveryServer,
                comment: comment.clone(),
                bytes,
            });
            ssd.reserved_memory += bytes;
            if ssd.reserved_memory >= ssd.server_memory {
                return Err(PlatformValidationError::ServerCannotReserveMoreMemory {
                    total_sum: ssd.reserved_memory,
                    server_memory: ssd.server_memory,
                    server_hostname: ssd.hostname.clone(),
                    memory_reservation_log: ssd
                        .memory_reservation_log
                        .iter()
                        .map(|i| (i.comment.clone(), i.bytes))
                        .collect(),
                });
            }
        }

        Ok(ReservedMemory {
            kind: MemoryBindKind::EveryServer,
            comment,
            bytes: mb * 1024 * 1024,
        })
    }

    pub fn lock_port_all_servers(
        &mut self,
        port: u16,
        comment: String,
    ) -> Result<LockedPort, PlatformValidationError> {
        if let Some(e_lp) = self.locked_all_server_ports.get(&port) {
            return Err(
                PlatformValidationError::DoublePortUseForAllServersAttemptDetected {
                    port,
                    previous_use_comment: e_lp.comment.clone(),
                    duplicate_use_comment: comment,
                },
            );
        }

        assert!(self
            .locked_all_server_ports
            .insert(
                port,
                LockedPort {
                    port,
                    comment: comment.clone(),
                    expose_to_all: false,
                }
            )
            .is_none());

        Ok(LockedPort {
            port,
            comment,
            expose_to_all: false,
        })
    }

    pub fn lock_port_all_servers_duplicate_ok(
        &mut self,
        port: u16,
        comment: String,
    ) -> Result<LockedPort, PlatformValidationError> {
        if let Some(e_lp) = self.locked_all_server_ports.get(&port) {
            if e_lp.comment != comment {
                return Err(
                    PlatformValidationError::DoublePortUseForAllServersAttemptDetected {
                        port,
                        previous_use_comment: e_lp.comment.clone(),
                        duplicate_use_comment: comment,
                    },
                );
            }

            return Ok(e_lp.clone());
        }

        assert!(self
            .locked_all_server_ports
            .insert(
                port,
                LockedPort {
                    port,
                    comment: comment.clone(),
                    expose_to_all: false,
                }
            )
            .is_none());

        Ok(LockedPort {
            port,
            comment,
            expose_to_all: false,
        })
    }

    pub fn vault_secrets(&self, region: TableRowPointerRegion) -> &BTreeMap<String, VaultSecrets> {
        &self.region_runtime.get(&region).unwrap().vault_secrets
    }

    pub fn consul_kv_entries(&self, region: TableRowPointerRegion) -> &BTreeMap<String, ConsulKvEntry> {
        &self.region_runtime.get(&region).unwrap().consul_kv_entries
    }

    pub fn frontend_lb_routes(&self, region: TableRowPointerRegion) -> &BTreeMap<TableRowPointerTld, ExternalLbSubdomainRoutes> {
        &self.region_runtime.get(&region).unwrap().routes
    }

    pub fn region_secrets_from_yml(&self, region: TableRowPointerRegion) -> &BTreeMap<TableRowPointerCustomSecret, CustomSecretUsageTypes> {
        &self.region_runtime.get(&region).unwrap().secrets_from_yml
    }

    #[allow(clippy::too_many_arguments)]
    pub fn expose_prefix_in_tld_for_frontend_app(
        &mut self,
        region: TableRowPointerRegion,
        tld: TableRowPointerTld,
        subdomain: &str,
        prefix: &str,
        content: RouteData,
        deployment: &str,
        application_name: &str,
        mountpoint: &[String],
    ) -> Result<(), PlatformValidationError> {
        let vs = ValidSubdomain::new(subdomain)?;
        let mountpoint_str = || format!("/{}/", mountpoint.join("/"));

        let e = self
            .region_runtime
            .entry(region)
            .or_default()
            .routes
            .entry(tld)
            .or_insert_with(|| ExternalLbSubdomainRoutes {
                tld,
                subdomains: Default::default(),
            });
        let subr = e
            .subdomains
            .entry(vs)
            .or_insert_with(|| SubdomainRouteChunk {
                routes: HttpPathTree::root(),
            });

        let mut routes = &mut subr.routes;
        for m in mountpoint {
            routes = routes.fetch_level(m.as_str(), &content).map_err(|e| {
                PlatformValidationError::AppIngressClashError {
                    deployment: deployment.to_string(),
                    application_name: application_name.to_string(),
                    mountpoint: mountpoint_str(),
                    error: format!("{:#?}", e),
                }
            })?;
        }

        routes
            .add_prefix_page(prefix, PageMethod::GET, content.clone())
            .map_err(|e| PlatformValidationError::AppIngressClashError {
                deployment: deployment.to_string(),
                application_name: application_name.to_string(),
                mountpoint: mountpoint_str(),
                error: format!("{:#?}", e),
            })?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn expose_route_in_tld_for_app(
        &mut self,
        region: TableRowPointerRegion,
        tld: TableRowPointerTld,
        subdomain: &str,
        path: &[CorePathSegment],
        content: RouteData,
        page_method: PageMethod,
        deployment: &str,
        application_name: &str,
        mountpoint: &[String],
        mountpoint_str: &str,
    ) -> Result<(), PlatformValidationError> {
        let vs = ValidSubdomain::new(subdomain)?;

        let e = self
            .region_runtime
            .entry(region)
            .or_default()
            .routes
            .entry(tld)
            .or_insert_with(|| ExternalLbSubdomainRoutes {
                tld,
                subdomains: Default::default(),
            });
        let subr = e
            .subdomains
            .entry(vs)
            .or_insert_with(|| SubdomainRouteChunk {
                routes: HttpPathTree::root(),
            });

        let mut routes = &mut subr.routes;
        for m in mountpoint {
            routes = routes.fetch_level(m.as_str(), &content).map_err(|e| {
                PlatformValidationError::AppIngressClashError {
                    deployment: deployment.to_string(),
                    application_name: application_name.to_string(),
                    mountpoint: mountpoint_str.to_string(),
                    error: format!("{:#?}", e),
                }
            })?;
        }
        for (idx, i) in path.iter().enumerate() {
            let is_last = idx + 1 == path.len();
            if is_last {
                match i {
                    CorePathSegment::Text(page_name) => {
                        routes
                            .add_named_page(page_name, page_method.clone(), content.clone())
                            .map_err(|e| PlatformValidationError::AppIngressClashError {
                                deployment: deployment.to_string(),
                                application_name: application_name.to_string(),
                                mountpoint: mountpoint_str.to_string(),
                                error: format!("{:#?}", e),
                            })?;
                    }
                    CorePathSegment::Argument(arg_name, _) => {
                        routes
                            .add_argument_final(arg_name, page_method.clone(), content.clone())
                            .map_err(|e| PlatformValidationError::AppIngressClashError {
                                deployment: deployment.to_string(),
                                application_name: application_name.to_string(),
                                mountpoint: mountpoint_str.to_string(),
                                error: format!("{:#?}", e),
                            })?;
                    }
                    CorePathSegment::LastSlash => {
                        routes
                            .lock_index(page_method.clone(), content.clone())
                            .map_err(|e| PlatformValidationError::AppIngressClashError {
                                deployment: deployment.to_string(),
                                application_name: application_name.to_string(),
                                mountpoint: mountpoint_str.to_string(),
                                error: format!("{:#?}", e),
                            })?;
                    }
                }
            } else {
                match i {
                    CorePathSegment::Text(page_name) => {
                        routes = routes.fetch_level(page_name, &content).map_err(|e| {
                            PlatformValidationError::AppIngressClashError {
                                deployment: deployment.to_string(),
                                application_name: application_name.to_string(),
                                mountpoint: mountpoint_str.to_string(),
                                error: format!("{:#?}", e),
                            }
                        })?;
                    }
                    CorePathSegment::Argument(arg_name, _) => {
                        routes =
                            routes
                                .add_argument_with_tree(arg_name, &content)
                                .map_err(|e| PlatformValidationError::AppIngressClashError {
                                    deployment: deployment.to_string(),
                                    application_name: application_name.to_string(),
                                    mountpoint: mountpoint_str.to_string(),
                                    error: format!("{:#?}", e),
                                })?;
                    }
                    CorePathSegment::LastSlash => {
                        panic!("Shold never be reached, last slash is in upper branch")
                    }
                }
            }
        }

        Ok(())
    }

    pub fn expose_root_route_in_tld(
        &mut self,
        db: &Database,
        region: TableRowPointerRegion,
        tld: TableRowPointerTld,
        subdomain: &str,
        route_data: RouteData,
    ) -> Result<(), PlatformValidationError> {
        let vs = ValidSubdomain::new(subdomain)?;

        let e = self
            .region_runtime
            .entry(region)
            .or_default()
            .routes
            .entry(tld)
            .or_insert_with(|| ExternalLbSubdomainRoutes {
                tld,
                subdomains: Default::default(),
            });

        let subr = e
            .subdomains
            .entry(vs)
            .or_insert_with(|| SubdomainRouteChunk {
                routes: HttpPathTree::root(),
            });
        subr.routes.lock_root(route_data).map_err(|e| {
            match e {
                crate::static_analysis::http_endpoints::HttpPathTreeCheckerErrors::RootPageAlreadySet { .. } => {
                    PlatformValidationError::ExternalLbRouteIsDuplicated {
                        tld: db.tld().c_domain(tld).clone(),
                        subdomain: subdomain.to_string(),
                        path: "/".to_string(),
                    }
                },
                err => {
                    panic!("Unexpected {:?}", err)
                }
            }
        })?;

        Ok(())
    }

    pub fn admin_dns_ingress_entries(&self) -> &BTreeMap<TableRowPointerTld, Vec<String>> {
        &self.admin_dns_ingress_services
    }

    pub fn expose_admin_service(
        &mut self,
        db: &Database,
        adm_svc: AdminService,
    ) -> Result<(), PlatformValidationError> {
        let full_subdomain = format!("adm-{}-{}", adm_svc.service_kind, adm_svc.service_instance);

        assert!(!adm_svc.service_title.contains('\"'));
        assert!(!adm_svc.service_title.contains('\''));

        let tld = get_global_settings(db).admin_tld;
        let ingress_domains = self.admin_dns_ingress_services.entry(tld).or_default();
        if ingress_domains.is_empty() {
            ingress_domains.push("admin".to_string());
        }
        ingress_domains.push(full_subdomain.clone());

        let e = self
            .admin_links
            .entry(adm_svc.service_title.clone())
            .or_default();

        e.push(format!(
            "<a target=\"_blank\" href=\"https://{}.{}\">{}</a>",
            full_subdomain,
            db.tld().c_domain(tld),
            adm_svc.service_instance
        ));

        for region in db.region().rows_iter() {
            self.expose_root_route_in_tld(
                db,
                region,
                tld,
                &full_subdomain,
                RouteData {
                    content: RouteContent::InternalUpstream {
                        is_https: adm_svc.is_https,
                        consul_service: adm_svc.service_internal_upstream.clone(),
                        port: adm_svc.service_internal_port,
                        target_path: "/".to_string(),
                        unlimited_body: false,
                    },
                    basic_auth: "".to_string(),
                },
            )?;
        }

        Ok(())
    }

    pub fn instantiate_and_seal_consul_service(
        &mut self,
        region: TableRowPointerRegion,
        service_name: &str,
    ) -> ConsulServiceHandle {
        let s = self.instantiate_consul_service(region, service_name);
        self.seal_consul_service(&s);
        s
    }

    pub fn consul_kv_write(&mut self, region: TableRowPointerRegion, kv_path: String, content: String) -> WrittenConsulKvValue {
        lazy_static! {
            static ref CONSUL_KV_REGEX: regex::Regex = regex::Regex::new("^epl-kv/[a-z0-9][a-z0-9/_-]+$").unwrap();
        }
        assert!(CONSUL_KV_REGEX.is_match(&kv_path), "Invalid consul kv path [{}], regex [{}]", kv_path, CONSUL_KV_REGEX.as_str());
        let rr = self.region_runtime.get_mut(&region).unwrap();
        assert!(!rr.consul_kv_entries.contains_key(&kv_path), "Double write to consul {kv_path} detected");
        assert!(rr.consul_kv_entries.insert(kv_path.clone(), ConsulKvEntry { content }).is_none());
        WrittenConsulKvValue {
            path: kv_path,
        }
    }

    pub fn fetch_existing_consul_service(&self, region: TableRowPointerRegion, service_name: &str) -> ConsulServiceHandle {
        self
            .region_runtime
            .get(&region)
            .unwrap()
            .consul_services
            .get(service_name)
            .expect("Cannot fetch consul service")
            .handle()
    }

    fn instantiate_consul_service(&mut self, region: TableRowPointerRegion, service_name: &str) -> ConsulServiceHandle {
        assert!(
            is_valid_consul_service_name(service_name),
            "Consul service name must be kebab case, got {service_name}"
        );
        let handle = ConsulServiceHandle {
            region,
            service_name: service_name.to_string(),
        };
        let e = self
            .region_runtime
            .entry(region)
            .or_default()
            .consul_services
            .entry(service_name.to_string())
            .or_insert_with(|| ConsulServiceState {
                region,
                is_sealed: false,
                service_name: service_name.to_string(),
                instantiation_count: 0,
            });

        if e.is_sealed {
            panic!("Consul service {service_name} is already sealed.");
        }

        e.instantiation_count += 1;

        handle
    }

    fn seal_consul_service(&mut self, handle: &ConsulServiceHandle) {
        match self
            .region_runtime
            .entry(handle.region)
            .or_default()
            .consul_services
            .get_mut(&handle.service_name)
        {
            Some(s) => {
                let sn = &handle.service_name;
                assert!(
                    !s.is_sealed,
                    "Service {sn} is already sealed, cannot seal twice"
                );

                s.is_sealed = true;
            }
            None => {
                panic!("Should never happen, if handle is issued service must exist");
            }
        }
    }

    pub fn minio_credentials(
        &self,
    ) -> &BTreeMap<TableRowPointerMinioBucket, BTreeMap<String, MinIOUser>> {
        &self.minio_bucket_users
    }

    pub fn seal_minio_bucket_credentials(&mut self) {
        self.minio_users_sealed = true;
    }

    pub fn server_label_locks(&self) -> &BTreeMap<LockedServerLabel, BTreeSet<TableRowPointerServer>> {
        &self.server_label_locks
    }

    pub fn provisioning_scripts(&self, region: TableRowPointerRegion) -> &BTreeMap<u8, Vec<ProvisioningScript>> {
        &self.region_runtime.get(&region).unwrap().provisioning_scripts
    }

    pub fn provisioning_resources(
        &self,
        region: TableRowPointerRegion,
    ) -> &BTreeMap<&str, BTreeMap<String, ProvisioningScriptResource>> {
        &self.region_runtime.get(&region).unwrap().provisioning_script_resources
    }

    pub fn server_runtime_checks(&self, db: &Database, l1proj: &L1Projections) -> Result<(), PlatformValidationError> {
        self.check_tasks_memory(db, l1proj)?;
        self.check_job_kinds()?;
        self.check_port_locks(db)?;
        self.check_volume_name_clash(db)?;
        self.check_all_task_groups_have_architecture_constraint(db);

        Ok(())
    }

    fn check_all_task_groups_have_architecture_constraint(&self, db: &Database) {
        // these are internal errors, if this fails we screwed up
        for region in db.region().rows_iter() {
            let jobs = self.nomad_jobs(region);
            for job in jobs.values() {
                for (tgn, tg) in &job.task_groups {
                    assert!(
                        tg.architecture_constaint.is_some(),
                        "architecture constraint in region {} for job {} task group {} is undecided",
                        db.region().c_region_name(region),
                        job.job_name(),
                        tgn
                    );

                    let ac = tg.architecture_constaint.as_ref().unwrap();
                    let expected_arch = match ac {
                        NomadArchitectures::Amd64 => "x86_64",
                        NomadArchitectures::Arm64 => "aarch64",
                    };
                    for (tn, task) in &tg.tasks {
                        if let Some(ptr) = &task.docker_image_ptr {
                            let img_arch = db.docker_image().c_architecture(*ptr);
                            assert_eq!(
                                expected_arch,
                                img_arch,
                                "docker image in region {} for nomad job {} task group {} task {} has unexpected architecture {}, expected {}",
                                db.region().c_region_name(region),
                                job.job_name(),
                                tgn,
                                tn,
                                img_arch,
                                expected_arch,
                            );
                        }
                    }
                }
            }
        }
    }

    fn check_volume_name_clash(&self, db: &Database) -> Result<(), PlatformValidationError> {
        for server in db.server().rows_iter() {
            let sdata = self.server_data.get(&server).unwrap();
            let mut keys = HashSet::new();
            for lv in sdata.locked_volumes.values() {
                if !keys.insert(lv.vol_name.clone()) {
                    return Err(PlatformValidationError::NomadVolumeNameClashOnServer {
                        server_hostname: db.server().c_hostname(server).clone(),
                        duplicate_volume_name: lv.vol_name.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn check_port_locks(&self, db: &Database) -> Result<(), PlatformValidationError> {
        for (tserver, sd) in &self.server_data {
            for (lp, lpa) in &sd.locked_ports {
                if let Some(all_lp) = self.locked_all_server_ports.get(lp) {
                    return Err(PlatformValidationError::DoublePortUseAttemptDetected {
                        server_hostname: db.server().c_hostname(*tserver).clone(),
                        port: *lp,
                        previous_use_comment: all_lp.comment.clone(),
                        duplicate_use_comment: lpa.comment.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn check_job_kinds(&self) -> Result<(), PlatformValidationError> {
        for rr in self.region_runtime.values() {
            for (jn, nj) in &rr.nomad_jobs {
                match nj.kind {
                    NomadJobKind::BoundStateful => {
                        // all task groups have count of 1
                        for (tgn, tg) in &nj.task_groups {
                            if tg.label_lock.is_none() {
                                return Err(
                                    PlatformValidationError::NomadStatefulJobHasNoServerLocks {
                                        job_name: jn.clone(),
                                        task_group_name: tgn.clone(),
                                    },
                                );
                            }

                            if tg.count != 1 {
                                return Err(PlatformValidationError::NomadStatefulJobHasNonOneCount {
                                    job_name: jn.clone(),
                                    task_group_name: tgn.clone(),
                                    count_expected: 1,
                                    count_actual: tg.count,
                                });
                            }

                            for (tn, task) in &tg.tasks {
                                for mem in &task.used_reserved_memory {
                                    match &mem.kind {
                                        MemoryBindKind::Server(_) => {}
                                        MemoryBindKind::Stateless | MemoryBindKind::EveryServer => {
                                            return Err(PlatformValidationError::NomadBoundStatefulJobCanOnlyHaveBoundMemory {
                                                job_name: jn.clone(),
                                                task_group_name: tgn.clone(),
                                                task: tn.clone(),
                                                memory_bytes: mem.bytes,
                                                comment: mem.comment.clone(),
                                                actual_kind: format!("{:?}", mem.kind),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    NomadJobKind::Stateless => {
                        for (tgn, tg) in &nj.task_groups {
                            if tg.label_lock.is_some() {
                                return Err(PlatformValidationError::NomadStatelessJobHasServerLocks {
                                    job_name: jn.clone(),
                                    task_group_name: tgn.clone(),
                                    lock_name: tg.label_lock.as_ref().unwrap().label.clone(),
                                });
                            }

                            if tg.count == 0 {
                                return Err(
                                    PlatformValidationError::NomadStatelessJobZeroInstanceCount {
                                        job_name: jn.clone(),
                                        task_group_name: tgn.clone(),
                                        instance_count: 0,
                                    },
                                );
                            }

                            for (tn, task) in &tg.tasks {
                                for mem in &task.used_reserved_memory {
                                    match &mem.kind {
                                        MemoryBindKind::Stateless => {}
                                        MemoryBindKind::Server(_) | MemoryBindKind::EveryServer => {
                                            return Err(PlatformValidationError::NomadStatelessJobCanOnlyHaveStatelessMemory {
                                                job_name: jn.clone(),
                                                task_group_name: tgn.clone(),
                                                task: tn.clone(),
                                                memory_bytes: mem.bytes,
                                                comment: mem.comment.clone(),
                                                actual_kind: format!("{:?}", mem.kind),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    NomadJobKind::SystemStateless => {
                        for (tgn, tg) in &nj.task_groups {
                            if tg.count != 1 {
                                return Err(
                                    PlatformValidationError::NomadSystemStatelessJobNonOneCount {
                                        job_name: jn.clone(),
                                        task_group_name: tgn.clone(),
                                        count_expected: 1,
                                        count_actual: tg.count,
                                    },
                                );
                            }

                            for (tn, task) in &tg.tasks {
                                for mem in &task.used_reserved_memory {
                                    match &mem.kind {
                                        MemoryBindKind::EveryServer => {}
                                        MemoryBindKind::Server(_) | MemoryBindKind::Stateless => {
                                            return Err(PlatformValidationError::NomadSystemStatelessJobCanOnlyHaveEveryServerMemory {
                                                job_name: jn.clone(),
                                                task_group_name: tgn.clone(),
                                                task: tn.clone(),
                                                memory_bytes: mem.bytes,
                                                comment: mem.comment.clone(),
                                                actual_kind: format!("{:?}", mem.kind),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn check_tasks_memory(&self, db: &Database, l1proj: &L1Projections) -> Result<(), PlatformValidationError> {
        self.check_all_tasks_have_memory_assigned()?;
        self.check_memory_availability(db, l1proj)?;

        Ok(())
    }

    fn check_memory_availability(&self, db: &Database, l1proj: &L1Projections) -> Result<(), PlatformValidationError> {
        if db.server().len() == 0 {
            return Ok(());
        }
        // 1. get reserved server memory so far into hash map
        // 2. sort servers by highest available memory to lowest into btree map
        // 3. go through stateless task groups which have no server assigned to it and sort them by memory
        // 4. try fitting in from biggest to lowest memory into servers
        // Assign unique keys in btreemap if values are equal? so every server is ordered.

        // TODO: perform unique hosts check so that same task can't be fit into same server?
        struct MemoryLog<'a> {
            data: &'a SingleServerData,
            stateless_mem: Vec<ReservedMemory>,
            idx: usize,
            used_memory: i64,
            free_memory: i64,
            workload_ids: BTreeSet<i32>,
        }
        impl<'a> MemoryLog<'a> {
            fn count_memory(&self) -> i64 {
                self.data.reserved_memory
                    + self
                        .stateless_mem
                        .iter()
                        .map(|i| -> i64 { i.bytes })
                        .sum::<i64>()
            }

            fn push_stateless_memory(&mut self, memory: ReservedMemory) {
                self.used_memory += memory.bytes;
                self.free_memory -= memory.bytes;
                self.stateless_mem.push(memory);
            }

            fn is_exhausted(&self) -> bool {
                self.free_memory < 0
            }

            fn used_memory(&self) -> i64 {
                self.used_memory
            }
        }
        let mut server_memory_logs: HashMap<TableRowPointerServer, MemoryLog> = HashMap::new();
        let mut server_memory_bytes: BTreeMap<i64, TableRowPointerServer> = BTreeMap::new();
        for (idx, server) in db.server().rows_iter().enumerate() {
            if let Some(sd) = self.server_data.get(&server) {
                let free_memory = sd.server_memory - sd.reserved_memory;
                let key = free_memory * 1024 * 1024 + idx as i64;
                server_memory_bytes.insert(key, server);
                assert!(server_memory_logs
                    .insert(
                        server,
                        MemoryLog {
                            data: sd,
                            stateless_mem: Vec::new(),
                            idx,
                            free_memory,
                            used_memory: sd.reserved_memory,
                            workload_ids: BTreeSet::new(),
                        }
                    )
                    .is_none());
            } else {
                panic!("Uninitialized server data.");
            }
        }

        let mut assign_stateless_placed_memory =
            |needed_bytes_groups: &Vec<Vec<&ReservedMemory>>, placement_query: &LabelQuery, region: TableRowPointerRegion|
                                       -> Result<(), PlatformValidationError>
        {
                let servers_allowed =
                    l1proj.label_database.find_servers(region, placement_query, None).unwrap();
                assert!(!servers_allowed.is_empty());
                // 1. sort servers by most available memory
                // 2. go through each server one by one and try to reserve
                let mut ordered: Vec<TableRowPointerServer> = servers_allowed.iter().cloned().collect();
                ordered.sort_by_key(|srv| {
                    server_memory_logs.get(srv).unwrap().free_memory
                });
                // highest memory first
                ordered.reverse();
                assert!(needed_bytes_groups.len() <= ordered.len(), "Not enough placements in memory?");
                for (idx, needed_bytes) in needed_bytes_groups.iter().enumerate() {
                    let server = ordered[idx];
                    let server_logs = server_memory_logs.get_mut(&server).unwrap();
                    for nb in needed_bytes {
                        server_logs.push_stateless_memory(
                            ReservedMemory {
                                kind: MemoryBindKind::Stateless,
                                comment: nb.comment.clone(),
                                bytes: nb.bytes,
                            }
                        )
                    }

                    if server_logs.is_exhausted() {
                        let mut mem_res_log = server_logs
                            .data
                            .memory_reservation_log
                            .iter()
                            .map(|i| (i.comment.clone(), i.bytes))
                            .collect::<Vec<_>>();
                        for sm in &server_logs.stateless_mem {
                            mem_res_log.push((sm.comment.clone(), sm.bytes));
                        }
                        return Err(
                            PlatformValidationError::ServerCannotReserveMoreStatelessPlacedMemory {
                                server_hostname: db.server().c_hostname(server_logs.data.server).clone(),
                                placement_labels: serde_yaml::to_string(placement_query).unwrap(),
                                memory_reservation_log: mem_res_log,
                                total_sum: server_logs.count_memory(),
                                server_memory: server_logs.data.server_memory,
                                all_placement_servers: ordered.iter().map(|i| db.server().c_hostname(*i).clone()).collect(),
                            },
                        );
                    }
                }

                Ok(())
            };

        struct UnplacedMemory<'a> {
            chunks: Vec<&'a ReservedMemory>,
            distinct_workload_id: i32,
        }
        let mut to_address_unplaced: Vec<(i64, UnplacedMemory)> = Vec::new();
        let mut to_address_placed: Vec<(i64, Vec<Vec<&ReservedMemory>>, &LabelQuery, TableRowPointerRegion)> = Vec::new();
        let mut distinct_workload_id = 0;
        for (region, rr) in &self.region_runtime {
            for nj in rr.nomad_jobs.values() {
                for tg in nj.task_groups().values() {
                    if tg.label_lock().is_none() && tg.placement().is_none() {
                        distinct_workload_id += 1;
                        // stateless task group
                        for _ in 0..tg.count {
                            let mut this_mem = Vec::new();
                            let mut total = 0i64;
                            for task in tg.tasks().values() {
                                for mem in &task.used_reserved_memory {
                                    total += mem.bytes;
                                    this_mem.push(mem);
                                }
                            }
                            to_address_unplaced.push((total, UnplacedMemory {
                                chunks: this_mem,
                                distinct_workload_id,
                            }));
                        }
                    } else if let Some(placement) = tg.placement() {
                        // stateless with placement
                        let mut this_placement_vec = Vec::new();
                        // order by maximum in each group
                        let mut max_total = 0i64;
                        for _ in 0..tg.count {
                            let mut this_mem = Vec::new();
                            let mut total = 0i64;
                            for task in tg.tasks().values() {
                                for mem in &task.used_reserved_memory {
                                    total += mem.bytes;
                                    this_mem.push(mem);
                                }
                            }
                            this_placement_vec.push(this_mem);
                            max_total = max_total.max(total);
                        }
                        to_address_placed.push((max_total, this_placement_vec, placement, *region));
                    } else {
                        // label locks already accounted for, we assume this is locked task group
                        assert!(tg.label_lock().is_some());
                    }
                }
            }
        }

        to_address_placed.sort_by_key(|i| i.0);
        // highest memory requirements first
        to_address_placed.reverse();
        for (_, chunks, placements, region) in &to_address_placed {
            assign_stateless_placed_memory(&chunks, placements, *region)?;
        }

        to_address_unplaced.sort_by_key(|i| i.0);
        // highest memory requirements first
        to_address_unplaced.reverse();

        // remove servers that are not allowed to run unassigned workloads
        // before distributing unassigned workloads
        let mut unplaced_workload_servers: Vec<(i64, TableRowPointerServer)> = Vec::new();
        for (key, srv) in &server_memory_bytes {
            if !db.server().c_run_unassigned_workloads(*srv) {
                unplaced_workload_servers.push((*key, *srv));
            }
        }

        for (key, _) in &unplaced_workload_servers {
            assert!(server_memory_bytes.remove(key).is_some());
        }

        let mut assign_stateless_unplaced_memory =
            |needed_bytes: &UnplacedMemory| -> Result<(), PlatformValidationError> {
                // 1. pick server with highest memory (last btree)
                // 1.1. make sure server doesn't already host same workload id
                // 2. remove it from map
                // 3. add memory to id
                // 4. add it back to map keying by memory size
                let mut found_server = None;

                let mut rejected_servers = Vec::new();
                for (k, v) in server_memory_bytes.iter().rev() {
                    let log = server_memory_logs.get(v).unwrap();
                    if !log.workload_ids.contains(&needed_bytes.distinct_workload_id) {
                        found_server = Some(*k);
                        break;
                    } else {
                        rejected_servers.push(*v);
                    }
                }

                if let Some(key) = found_server {
                    let removed = server_memory_bytes.remove(&key).unwrap();
                    let server_logs = server_memory_logs.get_mut(&removed).unwrap();
                    for nb in &needed_bytes.chunks {
                        server_logs.push_stateless_memory(
                            ReservedMemory {
                                kind: MemoryBindKind::Stateless,
                                comment: nb.comment.clone(),
                                bytes: nb.bytes,
                            }
                        );
                    }
                    if server_logs.is_exhausted() {
                        let mut mem_res_log = server_logs
                            .data
                            .memory_reservation_log
                            .iter()
                            .map(|i| (i.comment.clone(), i.bytes))
                            .collect::<Vec<_>>();
                        for sm in &server_logs.stateless_mem {
                            mem_res_log.push((sm.comment.clone(), sm.bytes));
                        }
                        return Err(
                            PlatformValidationError::ServerCannotReserveMoreStatelessMemory {
                                server_hostname: db.server().c_hostname(server_logs.data.server).clone(),
                                memory_reservation_log: mem_res_log,
                                total_sum: server_logs.count_memory(),
                                server_memory: server_logs.data.server_memory,
                                servers_already_hosting_this_workload:
                                rejected_servers
                                    .iter()
                                    .map(|srv| {
                                        db.server().c_hostname(*srv).clone()
                                    })
                                    .collect()
                            },
                        );
                    }
                    let new_key = server_logs.used_memory() * 1024 * 1024 + server_logs.idx as i64;
                    assert!(server_memory_bytes.insert(new_key, removed).is_none());
                    assert!(server_logs.workload_ids.insert(needed_bytes.distinct_workload_id));
                } else {
                    let mem_res_log =
                        needed_bytes.chunks.iter()
                        .map(|i| {
                            (i.comment.clone(), i.bytes)
                        })
                        .collect();
                    return Err(
                        PlatformValidationError::ServerCannotPlaceStatelessWorkload {
                            needed_bytes: mem_res_log,
                            servers_disallowed_to_run_unassigned_workloads:
                            unplaced_workload_servers
                                .iter()
                                .map(|(_, srv)| {
                                    db.server().c_hostname(*srv).clone()
                                })
                                .collect()
                        }
                    );
                }

                Ok(())
            };

        for (_, chunk) in &to_address_unplaced {
            assign_stateless_unplaced_memory(chunk)?;
        }

        Ok(())
    }

    fn check_all_tasks_have_memory_assigned(&self) -> Result<(), PlatformValidationError> {
        for rr in self.region_runtime.values() {
            for nj in rr.nomad_jobs.values() {
                for (tgn, tg) in &nj.task_groups {
                    for (tskn, task) in &tg.tasks {
                        let mut sum = 0;
                        for rm in &task.used_reserved_memory {
                            sum += rm.bytes;
                        }

                        assert!(sum >= 0, "Memory cannot be negative?");
                        if sum == 0 {
                            return Err(PlatformValidationError::NomadTaskHasNoMemoryAssignedToIt {
                                nomad_job: nj.job_name.clone(),
                                nomad_task_group: tgn.clone(),
                                nomad_task_name: tskn.clone(),
                                memory_bytes: sum,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn add_provisioning_resource(
        &mut self,
        region: TableRowPointerRegion,
        kind: &'static str,
        name: String,
        contents: String,
        is_executable: bool,
        replacement_macros: Vec<ReplacementMacro>,
    ) -> ProvisioningResourcePath {
        assert_ne!(kind, "scripts", "scripts is reserved directory");
        assert_ne!(kind, "apps", "apps is reserved directory");
        assert!(
            kind.is_case(Case::Kebab),
            "Provisioning resource must be kebab case"
        );
        // TODO: check if valid filename?
        let path = format!("{kind}/{name}");
        let m = self
            .region_runtime
            .entry(region)
            .or_default()
            .provisioning_script_resources
            .entry(kind)
            .or_default();
        let r = m.insert(
            name,
            ProvisioningScriptResource {
                contents,
                is_executable,
                replacement_macros,
            },
        );
        assert!(
            r.is_none(),
            "Duplicate filename detected in inserting provisioning resource"
        );

        ProvisioningResourcePath { path }
    }

    /// All of these will be run
    pub fn add_provisioning_script(
        &mut self,
        region: TableRowPointerRegion,
        tag: ProvisioningScriptTag,
        name: &'static str,
        contents: String,
    ) {
        let w = tag.tag_weight();

        let entry = self
            .region_runtime
            .entry(region)
            .or_default()
            .provisioning_scripts
            .entry(w)
            .or_default();
        for e in entry.iter() {
            assert_ne!(
                e.name, name,
                "Duplicate name for provisioning scripts found"
            );
        }
        entry.push(ProvisioningScript {
            name: name.to_string(),
            script: contents,
        })
    }

    pub fn add_pg_access(&mut self, ptr: PgAccessKind, creds: PostgresDbCredentials) {
        assert!(
            self.pg_access.insert(ptr, creds).is_none(),
            "Db access cannot be redefined twice"
        );
    }

    pub fn fetch_pg_access(&self, ptr: &PgAccessKind) -> &PostgresDbCredentials {
        self.pg_access.get(ptr).unwrap()
    }

    pub fn add_ch_access(&mut self, ptr: ChAccessKind, creds: ClickhouseDbCredentials) {
        assert!(
            self.ch_access.insert(ptr, creds).is_none(),
            "Db access cannot be redefined twice"
        );
    }

    pub fn fetch_ch_access(&self, ptr: &ChAccessKind) -> &ClickhouseDbCredentials {
        self.ch_access.get(ptr).unwrap()
    }

    pub fn system_volume_all_servers_read_lock(
        &mut self,
        db: &Database,
        vol: SystemServerVolume,
        lock_reason: String,
    ) -> Result<SuccessfulVolumeLock, PlatformValidationError> {
        let key = ServerVolumeKey::SystemVolume(vol);

        // No servers, this cannot fail
        if db.server().len() == 0 {
            let lock = self
                .no_servers_volume_locks
                .get_mut(&key)
                .expect("System volume not initialized");
            return lock.read_lock(lock_reason);
        }

        // just lock on all servers for now, could be abstracted away
        let mut last_lock = None;
        for srv in db.server().rows_iter() {
            let sd = self.fetch_server_data(db, srv);
            let lock = sd
                .locked_volumes
                .get_mut(&key)
                .expect("System volume not initialized");

            last_lock = Some(lock.read_lock(lock_reason.clone())?);
        }

        Ok(last_lock.unwrap())
    }
}

#[derive(Clone)]
pub struct LockedPort {
    port: u16,
    comment: String,
    #[allow(dead_code)]
    expose_to_all: bool,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LockedServerLabel {
    label: String,
}

#[derive(Clone, Copy, Debug)]
enum ServerVolumeLockContract {
    ReadOnly,
    OneWriterManyReaders,
    MultipleWriters,
    Exclusive,
}

pub struct ServerVolumeLocks {
    hostname: String,
    vol_name: String,
    contract: ServerVolumeLockContract,
    write: Vec<String>,
    read: Vec<String>,
}

// Cloning is only valid when sharing volume across tasks!!!
#[derive(Clone)]
pub struct SuccessfulVolumeLock {
    vol_name: String,
    read_only: bool,
}

impl ServerVolumeLocks {
    fn new(db: &Database, ptr: TableRowPointerServerVolume) -> Self {
        let server = db.server_volume().c_parent(ptr);
        let contract = db.server_volume().c_intended_usage(ptr);
        let vol_name = db.server_volume().c_volume_name(ptr);
        let hostname = db.server().c_hostname(server).clone();
        let contract = match db
            .server_volume_usage_contract()
            .c_usage_contract(contract)
            .as_str()
        {
            "read_only" => ServerVolumeLockContract::ReadOnly,
            "one_writer_many_readers" => ServerVolumeLockContract::OneWriterManyReaders,
            "multiple_writers" => ServerVolumeLockContract::MultipleWriters,
            "exclusive" => ServerVolumeLockContract::Exclusive,
            other => panic!("Unknown volume usage contract: {other}"),
        };
        Self {
            write: Vec::new(),
            read: Vec::new(),
            contract,
            hostname,
            vol_name: vol_name.to_string(),
        }
    }

    fn new_custom(hostname: String, contract: ServerVolumeLockContract, volume_name: &str) -> Self {
        Self {
            write: Vec::new(),
            read: Vec::new(),
            contract,
            hostname,
            vol_name: volume_name.to_string(),
        }
    }

    pub fn write_lock(
        &mut self,
        lock_reason: String,
    ) -> Result<SuccessfulVolumeLock, PlatformValidationError> {
        self.write.push(lock_reason);
        self.check_contract_breach()?;
        Ok(SuccessfulVolumeLock {
            vol_name: self.vol_name.clone(),
            read_only: false,
        })
    }

    pub fn read_lock(
        &mut self,
        lock_reason: String,
    ) -> Result<SuccessfulVolumeLock, PlatformValidationError> {
        self.read.push(lock_reason);
        self.check_contract_breach()?;
        Ok(SuccessfulVolumeLock {
            vol_name: self.vol_name.clone(),
            read_only: true,
        })
    }

    fn check_contract_breach(&self) -> Result<(), PlatformValidationError> {
        let err = || {
            Err(
                PlatformValidationError::ServerVolumeUsageContractBreachDetected {
                    server_hostname: self.hostname.clone(),
                    contract: format!("{:?}", self.contract),
                    read_locks: self.read.clone(),
                    write_locks: self.write.clone(),
                },
            )
        };
        match &self.contract {
            ServerVolumeLockContract::ReadOnly => {
                if !self.write.is_empty() {
                    return err();
                }
            }
            ServerVolumeLockContract::OneWriterManyReaders => {
                if self.write.len() > 1 {
                    return err();
                }
            }
            ServerVolumeLockContract::Exclusive => {
                if self.write.len() + self.read.len() > 1 {
                    return err();
                }
            }
            ServerVolumeLockContract::MultipleWriters => {
                // anything goes
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum MemoryBindKind {
    Server(TableRowPointerServer),
    Stateless,
    EveryServer,
}

#[derive(Clone)]
pub struct ReservedMemory {
    #[allow(dead_code)]
    kind: MemoryBindKind,
    comment: String,
    bytes: i64,
}

impl ReservedMemory {
    pub fn bytes(&self) -> i64 {
        self.bytes
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum SystemServerVolume {
    TlsCertificates,
}

#[derive(PartialEq, Eq, Hash)]
pub enum ServerVolumeKey {
    ServerVolumePtr(TableRowPointerServerVolume),
    SystemVolume(SystemServerVolume),
}

pub struct SingleServerData {
    server: TableRowPointerServer,
    hostname: String,
    locked_ports: BTreeMap<u16, LockedPort>,
    locked_volumes: HashMap<ServerVolumeKey, ServerVolumeLocks>,
    memory_reservation_log: Vec<ReservedMemory>,
    reserved_memory: i64,
    server_memory: i64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NomadJobKind {
    BoundStateful,   // must have bound task groups
    Stateless,       // stateless job can have counts increased
    SystemStateless, // must have no volumes associated
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NomadJobStage {
    SystemJob, // system epl jobs which can be deployed before epl applications
    Application,
}

#[derive(Clone)]
pub enum ReplaceWith {
    EplSecretKeyValue(String),
}

#[derive(Clone)]
pub struct ReplacementMacro {
    pub find: String,
    pub replace: ReplaceWith,
}

#[derive(Clone)]
pub enum NomadUpdateHealthCheckStrategy {
    Checks,
    // uncomment if will ever be needed
    // TaskStates,
    // we don't have manual because we're
    // not interested in manual labor in eden platform
}

// not everything supported but we don't
// need everything either
#[derive(Clone)]
pub struct NomadJobUpdateExplicitStrategy {
    pub max_parallel: u32,
    pub health_check: NomadUpdateHealthCheckStrategy,
    pub min_healthy_time_seconds: u32,
    pub healthy_deadline_seconds: u32,
    pub progress_deadline_seconds: u32,
    pub stagger_seconds: u32,
    pub auto_revert: bool,
}

pub enum JobUpdateStrategy {
    InstantAllAtOnce,
    RollingDefault, // min healthy time 10 seconds, max parallel 1, stagger 30s
    // uncomment if will ever be needed
    //Custom(NomadJobUpdateExplicitStrategy),
}

pub struct NomadJob {
    job_name: String,
    #[allow(dead_code)]
    custom_blocks: Vec<String>,
    task_groups: BTreeMap<String, NomadTaskGroup>,
    vault_policy: Option<FinalizedVaultSecrets>,
    namespace: TableRowPointerNomadNamespace,
    kind: NomadJobKind,
    stage: NomadJobStage,
    region: TableRowPointerRegion,
    loki_cluster: Option<TableRowPointerLokiCluster>,
    replacement_macros: Vec<ReplacementMacro>,
    update_strategy: Option<JobUpdateStrategy>,
}

pub enum ServiceHealthcheck {
    Tcp,
    Http { path: String },
}

pub struct PrometheusScrapeTarget {
    pub path: String,
    pub cluster: TableRowPointerMonitoringCluster,
}

pub struct NomadTaskGroupService {
    handle: ConsulServiceHandle,
    port: String,
    healthcheck: ServiceHealthcheck,
    metrics_collector: Option<PrometheusScrapeTarget>,
}

pub enum NomadArchitectures {
    Amd64,
    Arm64,
}

impl NomadArchitectures {
    pub fn to_nomad_name(&self) -> &'static str {
        match self {
            NomadArchitectures::Amd64 => "x86_64",
            NomadArchitectures::Arm64 => "aarch64",
        }
    }
}

pub struct NomadTaskGroup {
    ports: BTreeMap<String, LockedPort>,
    tasks: BTreeMap<String, NomadTask>,
    shutdown_delay_seconds: i64,
    label_lock: Option<LockedServerLabel>,
    services: BTreeMap<ConsulServiceHandle, NomadTaskGroupService>,
    count: usize,
    architecture_constaint: Option<NomadArchitectures>,
    placement: Option<LabelQuery>,
}

pub struct NomadTaskVolume {
    lock: SuccessfulVolumeLock,
    target_path: String,
}

pub enum TaskFileChangeMode {
    RestartTask,
    ReloadSignal { signal: ChangeSignal },
}

pub enum QuoteMode {
    EOL,
    Inline,
}

pub struct SecureNomadConfig {
    perms: String,
    contents: String,
    eval_as_env: bool,
    file_change_mode: TaskFileChangeMode,
    quote_mode: QuoteMode,
}

pub struct ConsulNomadConfig {
    perms: String,
    change_script: Option<String>,
    consul_value: WrittenConsulKvValue,
}

pub struct NomadLocalFile {
    perms: String,
    contents: String,
}

pub enum NomadTaskLifecyclePhase {
    PostStart,
}

pub struct NomadTaskLifecycle {
    phase: NomadTaskLifecyclePhase,
    sidecar: bool,
}

pub struct NomadTaskTmpfsMount {
    target_path: String,
    memory_bytes: i64,
}

pub struct NomadTask {
    locked_mounts: BTreeSet<String>,
    used_host_volumes: Vec<NomadTaskVolume>,
    used_tmpfs_mounts: Vec<NomadTaskTmpfsMount>,
    used_reserved_memory: Vec<ReservedMemory>,
    // if task exhausts this memory then the task is killed
    memory_oversubscription_mb: u32,
    // configs that will end up under /secrets dir
    secure_configs: BTreeMap<String, SecureNomadConfig>,
    consul_configs: BTreeMap<String, ConsulNomadConfig>,
    local_files: BTreeMap<String, NomadLocalFile>,

    entrypoint: Vec<String>,
    arguments: Vec<String>,

    docker_image: String,
    docker_image_ptr: Option<TableRowPointerDockerImage>,
    env_variables: BTreeMap<String, String>,

    lifecycle: Option<NomadTaskLifecycle>,
}

fn default_locked_volumes(hostname: String) -> HashMap<ServerVolumeKey, ServerVolumeLocks> {
    let mut locked_volumes: HashMap<ServerVolumeKey, ServerVolumeLocks> = Default::default();

    // TLS certs volume
    // provisioned outside initially
    locked_volumes.insert(
        ServerVolumeKey::SystemVolume(SystemServerVolume::TlsCertificates),
        ServerVolumeLocks::new_custom(
            hostname,
            ServerVolumeLockContract::OneWriterManyReaders,
            "ssl_certs",
        ),
    );

    locked_volumes
}

impl SingleServerData {
    fn new(db: &Database, server: TableRowPointerServer, server_kind: TableRowPointerServerKind) -> Self {
        let server_memory = db.server_kind().c_memory_bytes(server_kind);
        let mut res = Self {
            server,
            hostname: db.server().c_hostname(server).clone(),
            locked_ports: Default::default(),
            locked_volumes: default_locked_volumes(db.server().c_hostname(server).clone()),
            memory_reservation_log: Vec::new(),
            reserved_memory: 0,
            server_memory,
        };

        // we should already have checked that system
        // reserved memory is less than server memory
        let _ = res
            .reserve_memory(
                "System reserved".to_string(),
                system_reserved_memory_bytes(),
            )
            .unwrap();
        res
    }

    pub fn lock_port(
        &mut self,
        db: &Database,
        port: u16,
        comment: String,
    ) -> Result<LockedPort, PlatformValidationError> {
        self.lock_port_internal(db, port, comment, false)
    }

    fn lock_port_internal(
        &mut self,
        db: &Database,
        port: u16,
        comment: String,
        expose_to_all: bool,
    ) -> Result<LockedPort, PlatformValidationError> {
        if let Some(prev) = self.locked_ports.get(&port) {
            return Err(PlatformValidationError::DoublePortUseAttemptDetected {
                server_hostname: db.server().c_hostname(self.server).clone(),
                port,
                previous_use_comment: prev.comment.clone(),
                duplicate_use_comment: comment,
            });
        }

        let res = LockedPort {
            port,
            comment,
            expose_to_all,
        };
        assert!(self.locked_ports.insert(port, res.clone()).is_none());

        Ok(res)
    }

    pub fn server_volume_write_lock(
        &mut self,
        db: &Database,
        vol: TableRowPointerServerVolume,
        lock_reason: String,
    ) -> Result<SuccessfulVolumeLock, PlatformValidationError> {
        assert_eq!(
            db.server_volume().c_parent(vol),
            self.server,
            "Volume does not belong to this server data"
        );

        let lock = self
            .locked_volumes
            .entry(ServerVolumeKey::ServerVolumePtr(vol))
            .or_insert_with(|| ServerVolumeLocks::new(db, vol));

        lock.write_lock(lock_reason)
    }

    pub fn reserve_memory_mb(
        &mut self,
        comment: String,
        megabytes: i64,
    ) -> Result<ReservedMemory, PlatformValidationError> {
        self.reserve_memory(comment, megabytes * 1024 * 1024)
    }

    fn reserve_memory(
        &mut self,
        comment: String,
        bytes: i64,
    ) -> Result<ReservedMemory, PlatformValidationError> {
        assert!(bytes > 0);

        let res = ReservedMemory {
            comment: comment.clone(),
            bytes,
            kind: MemoryBindKind::Server(self.server),
        };
        // don't expose clone to anyone
        let res2 = ReservedMemory {
            comment,
            bytes,
            kind: MemoryBindKind::Server(self.server),
        };
        self.memory_reservation_log.push(res);

        self.reserved_memory += bytes;
        if self.reserved_memory >= self.server_memory {
            return Err(PlatformValidationError::ServerCannotReserveMoreMemory {
                total_sum: self.reserved_memory,
                server_memory: self.server_memory,
                server_hostname: self.hostname.clone(),
                memory_reservation_log: self
                    .memory_reservation_log
                    .iter()
                    .map(|i| (i.comment.clone(), i.bytes))
                    .collect(),
            });
        }

        Ok(res2)
    }
}

impl NomadJob {
    pub fn fetch_task_group(&mut self, task_group_name: String) -> &mut NomadTaskGroup {
        self.task_groups
            .entry(task_group_name)
            .or_insert_with(|| NomadTaskGroup {
                ports: Default::default(),
                tasks: Default::default(),
                label_lock: None,
                services: Default::default(),
                count: 1,
                architecture_constaint: None,
                placement: None,
                shutdown_delay_seconds: 0,
            })
    }

    pub fn set_update_strategy(&mut self, strat: JobUpdateStrategy) {
        assert!(self.update_strategy.is_none(), "Job update strategy already assigned");
        self.update_strategy = Some(strat);
    }

    pub fn assign_vault_secrets(&mut self, secrets: FinalizedVaultSecrets) {
        assert!(
            self.vault_policy.is_none(),
            "Cannot assign vault secrets to job twice"
        );
        self.vault_policy = Some(secrets);
    }

    pub fn set_loki_cluster(&mut self, cluster: TableRowPointerLokiCluster) {
        assert!(self.loki_cluster.is_none());
        self.loki_cluster = Some(cluster);
    }

    /// Replacement macros are used to replace values in nomad jobs at later stages in codegen.
    /// Say, during generation of nomad jobs we cannot access epl secrets because that is part of
    /// the codegen. But in codegen step nomad source will be replaced with macros, where
    /// we have access to secrets.
    pub fn add_replacement_macro(&mut self, to_find: String, to_replace_with: ReplaceWith) {
        self.replacement_macros.push(ReplacementMacro {
            find: to_find,
            replace: to_replace_with,
        });
    }

    pub fn replacement_macros(&self) -> &Vec<ReplacementMacro> {
        &self.replacement_macros
    }

    pub fn job_name(&self) -> &str {
        &self.job_name
    }

    pub fn update_strategy(&self) -> &Option<JobUpdateStrategy> {
        &self.update_strategy
    }

    pub fn vault_policy(&self) -> &Option<FinalizedVaultSecrets> {
        &self.vault_policy
    }

    pub fn task_groups(&self) -> &BTreeMap<String, NomadTaskGroup> {
        &self.task_groups
    }

    pub fn job_kind(&self) -> NomadJobKind {
        self.kind
    }

    pub fn job_namespace(&self) -> TableRowPointerNomadNamespace {
        self.namespace
    }

    pub fn job_stage(&self) -> NomadJobStage {
        self.stage
    }

    pub fn region(&self) -> TableRowPointerRegion {
        self.region
    }

    pub fn loki_cluster(&self) -> Option<TableRowPointerLokiCluster> {
        self.loki_cluster.map(|i| i.clone())
    }
}

#[test]
fn test_docker_image_validation() {
    assert!(is_valid_docker_image("cultleader777/patroni-pg@sha256:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0c"));
    assert!(!is_valid_docker_image("cultleader777/patroni-pg@sha256:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0cz"));
    assert!(is_valid_docker_image(
        "patroni-pg@sha256:9bddbbbe30e2eb6030158ecc1e9375e26105f557aa45df4ce66c7abde698db0c"
    ));
    assert!(is_valid_docker_image(
        "quay.io/ceph/ceph@sha256:0560b16bec6e84345f29fb6693cd2430884e6efff16a95d5bdd0bb06d7661c45"
    ));
    assert!(!is_valid_docker_image("@@EPL_APP_IMAGE:some-app@@"));
    assert!(is_valid_docker_image("@@EPL_APP_IMAGE_x86_64:some-app@@"));
    assert!(is_valid_docker_image("@@EPL_APP_IMAGE_arm64:some-app@@"));
    assert!(is_valid_docker_image(
        "epl-docker-registry.service.consul:5000/wookie@sha256:0560b16bec6e84345f29fb6693cd2430884e6efff16a95d5bdd0bb06d7661c45"
    ));
}

fn is_valid_docker_image(input: &str) -> bool {
    lazy_static! {
        pub static ref DOCKER_IMAGE_REGEX: Regex =
            Regex::new(r#"^([a-z0-9.-]+(:[1-9][0-9]+)?/)?([a-zA-Z0-9_-]+/)?[a-zA-Z0-9_-]+@sha256:[0-9a-f]{64}$"#)
                .unwrap();
        pub static ref DOCKER_IMAGE_EPL_APP_REGEX: Regex =
            Regex::new(r#"^@@EPL_APP_IMAGE_(x86_64|arm64):"#)
                .unwrap();
    }

    if DOCKER_IMAGE_EPL_APP_REGEX.is_match(input) && input.ends_with("@@") {
        return true;
    }

    DOCKER_IMAGE_REGEX.is_match(input)
}

lazy_static! {
    static ref HEALTHCHECK_REGEX: regex::Regex = regex::Regex::new(r#"^/[a-zA-Z0-9/-]+$"#).unwrap();
}

#[test]
fn test_valid_healthcheck_path() {
    assert!(HEALTHCHECK_REGEX.is_match("/hello/health"));
    assert!(HEALTHCHECK_REGEX.is_match("/health"));
    assert!(!HEALTHCHECK_REGEX.is_match("health"));
    assert!(!HEALTHCHECK_REGEX.is_match("/he@*)alth"));
    assert!(!HEALTHCHECK_REGEX.is_match("/some \" syntax injection"));
}

impl NomadTaskGroup {
    pub fn fetch_task(&mut self, task_name: String, docker_image_handle: DockerImageHandle) -> &mut NomadTask {
        let docker_image = docker_image_handle.image_placeholder();
        assert!(is_valid_docker_image(docker_image), "Invalid docker image {docker_image}");

        let res = self.tasks.entry(task_name).or_insert_with(|| NomadTask {
            used_tmpfs_mounts: Vec::new(),
            locked_mounts: BTreeSet::new(),
            used_host_volumes: Vec::new(),
            used_reserved_memory: Vec::new(),
            memory_oversubscription_mb: 128,
            consul_configs: Default::default(),
            secure_configs: Default::default(),
            local_files: Default::default(),
            entrypoint: Vec::new(),
            arguments: Vec::new(),
            docker_image: docker_image.to_string(),
            docker_image_ptr: docker_image_handle.docker_image_ptr(),
            env_variables: Default::default(),
            lifecycle: None,
        });

        assert!(res.lifecycle.is_none());

        res
    }

    pub fn set_shutdown_delay_seconds(&mut self, seconds: i64) {
        assert!(seconds >= 0);
        self.shutdown_delay_seconds = seconds;
    }

    pub fn shutdown_delay_seconds(&self) -> i64 {
        self.shutdown_delay_seconds
    }

    pub fn constrain_architecture(&mut self, arch: NomadArchitectures) {
        assert!(self.architecture_constaint.is_none());
        self.architecture_constaint = Some(arch);
    }

    pub fn try_set_placement(
        &mut self,
        db: &Database,
        region: TableRowPointerRegion,
        query: &str,
        context: &str,
        count: usize,
        label_db: &LabelDatabase,
    ) -> Result<(), PlatformValidationError> {
        assert!(self.placement.is_none());
        let reader_placements = label_db.try_to_find_placements(
            db,
            context,
            region,
            query,
            count,
        )?;
        if let Some(reader_placements) = reader_placements {
            self.placement = Some(reader_placements);
        }
        Ok(())
    }

    /// Task which will start after main task is running
    /// Finish its job and exit
    pub fn fetch_post_start_ephemeral_task(
        &mut self,
        task_name: String,
        docker_image_handle: DockerImageHandle,
    ) -> &mut NomadTask {
        let docker_image = docker_image_handle.image_placeholder().to_string();
        assert!(is_valid_docker_image(&docker_image));

        let res = self.tasks.entry(task_name).or_insert_with(|| NomadTask {
            used_host_volumes: Vec::new(),
            used_tmpfs_mounts: Vec::new(),
            locked_mounts: Default::default(),
            used_reserved_memory: Vec::new(),
            consul_configs: Default::default(),
            secure_configs: Default::default(),
            local_files: Default::default(),
            entrypoint: Vec::new(),
            arguments: Vec::new(),
            memory_oversubscription_mb: 128,
            docker_image,
            docker_image_ptr: docker_image_handle.docker_image_ptr(),
            env_variables: Default::default(),
            lifecycle: Some(NomadTaskLifecycle {
                phase: NomadTaskLifecyclePhase::PostStart,
                sidecar: false,
            }),
        });

        assert!(res.lifecycle.is_some());

        res
    }

    pub fn add_locked_port(&mut self, mnemonic: &str, lp: LockedPort) {
        assert!(
            mnemonic.is_case(Case::Snake),
            "Mnemonic {mnemonic} is not snake case"
        );
        assert!(
            self.ports.insert(mnemonic.to_string(), lp).is_none(),
            "Port mnemonic {mnemonic} locked for task group"
        );
    }

    pub fn expose_port_as_tcp_service(
        &mut self,
        port_mnemonic: &str,
        service: &ConsulServiceHandle,
    ) {
        let sn = service.service_name();
        assert!(
            self.ports.contains_key(port_mnemonic),
            "Port {port_mnemonic} doesn't exist"
        );
        assert!(
            self.services
                .insert(
                    service.clone(),
                    NomadTaskGroupService {
                        handle: service.clone(),
                        port: port_mnemonic.to_string(),
                        healthcheck: ServiceHealthcheck::Tcp,
                        metrics_collector: None,
                    }
                )
                .is_none(),
            "Service {sn} already defined."
        );
    }

    pub fn collect_prometheus_metrics(
        &mut self,
        service: &ConsulServiceHandle,
        cluster: TableRowPointerMonitoringCluster,
        path: Option<&str>,
    ) {
        let metrics_path = path.unwrap_or("/metrics");
        assert!(
            HEALTHCHECK_REGEX.is_match(metrics_path),
            "Metrics path {metrics_path} is invalid"
        );

        let sn = self
            .services
            .get_mut(service)
            .expect("Can't find port for service");
        assert!(
            sn.metrics_collector.is_none(),
            "Metrics cluster already set for service {}",
            service.service_name()
        );
        sn.metrics_collector = Some(PrometheusScrapeTarget {
            path: metrics_path.to_string(),
            cluster,
        });
    }

    pub fn set_service_http_healthcheck(
        &mut self,
        service: &ConsulServiceHandle,
        healthcheck_path: &str,
    ) {
        assert!(
            HEALTHCHECK_REGEX.is_match(healthcheck_path),
            "Healthcheck path {healthcheck_path} is invalid"
        );

        let s = self.services.get_mut(service).unwrap();
        s.healthcheck = ServiceHealthcheck::Http {
            path: healthcheck_path.to_string(),
        };
    }

    /// Do this on task groups that need to exist only once per server, like stateful services, like postgres
    pub fn assign_server_lock(&mut self, lock: LockedServerLabel) {
        assert!(self.label_lock.is_none(), "Lock already assigned to group");
        self.label_lock = Some(lock);
    }

    pub fn label_lock(&self) -> &Option<LockedServerLabel> {
        &self.label_lock
    }

    pub fn placement(&self) -> &Option<LabelQuery> {
        &self.placement
    }

    pub fn ports(&self) -> &BTreeMap<String, LockedPort> {
        &self.ports
    }

    pub fn tasks(&self) -> &BTreeMap<String, NomadTask> {
        &self.tasks
    }

    pub fn services(&self) -> &BTreeMap<ConsulServiceHandle, NomadTaskGroupService> {
        &self.services
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn set_count(&mut self, count: usize) {
        assert!(count >= 1, "Count must be more or equal to 1");
        self.count = count;
    }

    pub fn architecture_constraint(&self) -> &Option<NomadArchitectures> {
        &self.architecture_constaint
    }
}

pub enum ChangeSignal {
    SIGHUP,
}

impl NomadTask {
    pub fn add_memory(&mut self, memory: ReservedMemory) {
        self.used_reserved_memory.push(memory);
    }

    pub fn add_tmpfs_mount(&mut self, target_path: String, memory: ReservedMemory) {
        assert!(self.locked_mounts.insert(target_path.clone()), "Duplicate target paths detected: {target_path}");
        let memory_bytes = memory.bytes;

        self.used_tmpfs_mounts.push(NomadTaskTmpfsMount {
            target_path, memory_bytes,
        });
        self.add_memory(memory);
    }

    pub fn set_memory_oversubscription_mb(&mut self, new_value_mb: u32) {
        assert_ne!(new_value_mb, 128, "new value shouldn't be equal default");
        self.memory_oversubscription_mb = new_value_mb;
    }

    /// this value is added to memory nomad field to create memory_max
    /// once application crosses memory it will be urged to gc its own memory
    /// but if it reaches memory_max it will be killed
    pub fn memory_oversubscription_mb(&self) -> u32 {
        self.memory_oversubscription_mb
    }

    pub fn bind_volume(&mut self, lock: SuccessfulVolumeLock, target_path: String) {
        assert!(self.locked_mounts.insert(target_path.clone()), "Duplicate target paths detected: {target_path}");

        self.used_host_volumes
            .push(NomadTaskVolume { lock, target_path })
    }

    /// Returns absolute path
    pub fn add_local_file(&mut self, fname: String, contents: String) -> String {
        let abs_path = format!("/local/{fname}");
        // configs are flat, we strive to pass args to components so they'd be used
        assert!(
            // Allow everyone to access logs inside container, user may not be root
            self.local_files
                .insert(
                    fname,
                    NomadLocalFile {
                        perms: "644".to_string(),
                        contents,
                    }
                )
                .is_none(),
            "duplicate secure config detected"
        );

        abs_path
    }

    pub fn add_executable_local_file(&mut self, fname: String, contents: String) -> String {
        let abs_path = format!("/local/{fname}");
        // configs are flat, we strive to pass args to components so they'd be used
        assert!(
            // Allow everyone to access logs inside container, user may not be root
            self.local_files
                .insert(
                    fname,
                    NomadLocalFile {
                        perms: "755".to_string(),
                        contents,
                    }
                )
                .is_none(),
            "duplicate secure config detected"
        );

        abs_path
    }

    pub fn add_vault_handle_as_file(&mut self, fname: String, handle: &VaultSecretHandle) -> String {
        let abs_path = format!("/secrets/{fname}");
        assert!(
            // Allow everyone to access logs inside container, user may not be root
            self.secure_configs
                .insert(
                    fname,
                    SecureNomadConfig {
                        perms: "644".to_string(),
                        contents: handle.template_expression(),
                        eval_as_env: false,
                        file_change_mode: TaskFileChangeMode::RestartTask,
                        quote_mode: QuoteMode::Inline,
                    }
                )
                .is_none(),
            "duplicate secure config detected"
        );

        abs_path
    }

    /// Returns absolute path
    pub fn add_secure_config(&mut self, fname: String, contents: String) -> String {
        let abs_path = format!("/secrets/{fname}");
        // otherwise nomad file is messed up
        assert!(contents.ends_with("\n"));
        // configs are flat, we strive to pass args to components so they'd be used
        // grafana broke this rule with /secure/provisioning/datasources/datasources.yml
        // assert!(!fname.contains("/"), "Secure config name cannot be nested deeper into folders");
        assert!(
            // Allow everyone to access logs inside container, user may not be root
            self.secure_configs
                .insert(
                    fname,
                    SecureNomadConfig {
                        perms: "644".to_string(),
                        contents,
                        eval_as_env: false,
                        file_change_mode: TaskFileChangeMode::RestartTask,
                        quote_mode: QuoteMode::EOL,
                    }
                )
                .is_none(),
            "duplicate secure config detected"
        );

        abs_path
    }

    pub fn add_secure_config_wchange_signal(&mut self, fname: String, contents: String, change_signal: ChangeSignal) -> String {
        let abs_path = format!("/secrets/{fname}");
        // configs are flat, we strive to pass args to components so they'd be used
        // grafana broke this rule with /secure/provisioning/datasources/datasources.yml
        // assert!(!fname.contains("/"), "Secure config name cannot be nested deeper into folders");
        assert!(
            // Allow everyone to access logs inside container, user may not be root
            self.secure_configs
                .insert(
                    fname,
                    SecureNomadConfig {
                        perms: "644".to_string(),
                        contents,
                        eval_as_env: false,
                        file_change_mode: TaskFileChangeMode::ReloadSignal { signal: change_signal },
                        quote_mode: QuoteMode::EOL,
                    }
                )
                .is_none(),
            "duplicate secure config detected"
        );

        abs_path
    }

    pub fn add_consul_kv_file_with_change_script(&mut self, fname: String, contents: WrittenConsulKvValue, change_script_path: Option<String>) -> String {
        let abs_path = format!("/secrets/{fname}");
        assert!(
            self.consul_configs.insert(
                fname,
                ConsulNomadConfig {
                    perms: "0644".to_string(),
                    change_script: change_script_path,
                    consul_value: contents,
                }
            ).is_none()
        );
        abs_path
    }

    /// Add environment variables
    pub fn add_secure_env_variables(
        &mut self,
        fname: String,
        secrets: &[(&str, &VaultSecretHandle)],
    ) -> String {
        let abs_path = format!("/secrets/{fname}");
        // configs are flat, we strive to pass args to components so they'd be used
        assert!(
            !fname.contains('/'),
            "Secure config name cannot be nested deeper into folders"
        );

        let mut contents = String::new();
        for (var_name, secret) in secrets {
            assert!(
                is_valid_environment_variable(var_name),
                "Environment variables must be in upper snake case"
            );
            contents += &format!("{}={}\n", var_name, secret.template_expression());
        }

        assert!(
            // Allow everyone to access logs inside container, user may not be root
            self.secure_configs
                .insert(
                    fname,
                    SecureNomadConfig {
                        perms: "644".to_string(),
                        contents,
                        eval_as_env: true,
                        file_change_mode: TaskFileChangeMode::RestartTask,
                        quote_mode: QuoteMode::EOL,
                    }
                )
                .is_none(),
            "duplicate secure config detected"
        );

        abs_path
    }

    /// Sets entrypoint for task, if already set to non empty vec will fail
    pub fn set_entrypoint(&mut self, input: Vec<String>) {
        assert!(self.entrypoint.is_empty());
        self.entrypoint = input;
    }

    /// Sets arguments for task, if already set to non empty vec will fail
    pub fn set_arguments(&mut self, input: Vec<String>) {
        assert!(self.arguments.is_empty());
        self.arguments = input;
    }

    pub fn set_env_variable(&mut self, var_name: &str, var_value: &str) {
        assert!(!var_name.contains('\"'));
        assert!(
            is_valid_environment_variable(var_name),
            "Environment variables must be in upper snake case"
        );
        assert!(
            self.env_variables
                .insert(var_name.to_string(), var_value.to_string())
                .is_none(),
            "Environment variable {var_name} already exists"
        );
    }

    pub fn used_memory_mb(&self) -> u64 {
        let mut res = 0;
        for um in &self.used_reserved_memory {
            res += um.bytes;
        }

        (res / (1024 * 1024)).try_into().unwrap()
    }

    pub fn used_host_volumes(&self) -> &[NomadTaskVolume] {
        &self.used_host_volumes
    }

    pub fn used_tmpfs_mounts(&self) -> &[NomadTaskTmpfsMount] {
        &self.used_tmpfs_mounts
    }

    pub fn env_variables(&self) -> &BTreeMap<String, String> {
        &self.env_variables
    }

    pub fn docker_image(&self) -> &str {
        &self.docker_image
    }

    pub fn entrypoint(&self) -> &[String] {
        &self.entrypoint
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub fn secure_configs(&self) -> &BTreeMap<String, SecureNomadConfig> {
        &self.secure_configs
    }

    pub fn consul_configs(&self) -> &BTreeMap<String, ConsulNomadConfig> {
        &self.consul_configs
    }

    pub fn local_files(&self) -> &BTreeMap<String, NomadLocalFile> {
        &self.local_files
    }

    pub fn lifecycle(&self) -> &Option<NomadTaskLifecycle> {
        &self.lifecycle
    }
}

impl NomadTaskLifecycle {
    pub fn phase(&self) -> &NomadTaskLifecyclePhase {
        &self.phase
    }

    pub fn is_sidecar(&self) -> bool {
        self.sidecar
    }
}

impl LockedPort {
    pub fn value(&self) -> u16 {
        self.port
    }
}

impl SuccessfulVolumeLock {
    pub fn nomad_host_volume_name(&self) -> &str {
        &self.vol_name
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}

impl LockedServerLabel {
    pub fn label_name(&self) -> &str {
        &self.label
    }
}

impl CustomConsulPolicyBuilder {
    pub fn new(name: String) -> Self {
        assert!(name.is_case(Case::Snake) || is_valid_consul_service_name(&name));

        Self {
            segments: Vec::new(),
            name,
        }
    }

    pub fn add_kw_read_write(&mut self, path: &str) {
        for s in path.split('/') {
            assert!(s.is_case(Case::Snake) || is_valid_consul_service_name(s))
        }
        // TODO: ensure we don't manipulate both kv twice
        self.segments.push(format!(
            r#"key_prefix "{path}" {{
    policy = "write"
}}
"#
        ))
    }

    pub fn allow_session_write(&mut self) {
        self.segments.push(
            r#"session_prefix "" {
    policy = "write"
}
"#
            .to_string(),
        )
    }

    pub fn allow_service_write(&mut self, service: &ConsulServiceHandle) {
        let service_name = &service.service_name;
        self.segments.push(format!(
            r#"service_prefix "{service_name}" {{
    policy = "write"
}}
"#
        ))
    }

    pub fn build(self) -> CustomConsulPolicy {
        CustomConsulPolicy {
            source: self.segments.join("\n"),
            name: self.name,
        }
    }
}

impl ConsulServiceHandle {
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn service_fqdn(&self) -> String {
        format!("{}.service.consul", self.service_name)
    }
}

impl NomadTaskVolume {
    pub fn lock(&self) -> &SuccessfulVolumeLock {
        &self.lock
    }

    pub fn target_path(&self) -> &str {
        &self.target_path
    }
}

impl NomadTaskTmpfsMount {
    pub fn target_path(&self) -> &str {
        &self.target_path
    }

    pub fn memory_bytes(&self) -> i64 {
        self.memory_bytes
    }
}

impl NomadTaskGroupService {
    pub fn handle(&self) -> &ConsulServiceHandle {
        &self.handle
    }

    pub fn port(&self) -> &str {
        &self.port
    }

    pub fn healthcheck(&self) -> &ServiceHealthcheck {
        &self.healthcheck
    }

    pub fn metrics_collector(&self) -> &Option<PrometheusScrapeTarget> {
        &self.metrics_collector
    }
}

impl SecureNomadConfig {
    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn perms(&self) -> &str {
        &self.perms
    }

    pub fn eval_as_env(&self) -> bool {
        self.eval_as_env
    }

    pub fn file_change_mode(&self) -> &TaskFileChangeMode {
        &self.file_change_mode
    }

    pub fn quote_mode(&self) -> &QuoteMode {
        &self.quote_mode
    }
}

impl ConsulNomadConfig {
    pub fn kv_path(&self) -> &str {
        &self.consul_value.path
    }

    pub fn perms(&self) -> &str {
        &self.perms
    }

    pub fn change_script(&self) -> &Option<String> {
        &self.change_script
    }
}

impl NomadLocalFile {
    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn perms(&self) -> &str {
        &self.perms
    }
}

impl MinIOUser {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &VaultSecretHandle {
        &self.password
    }

    pub fn permission(&self) -> &MinIOBucketPermission {
        &self.permission
    }
}

impl ConsulKvEntry {
    // probably we don't need these getters,
    // remove in the future?
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl ValidSubdomain {
    fn new(input: &str) -> Result<ValidSubdomain, PlatformValidationError> {
        lazy_static! {
            pub static ref SUBDOMAIN_REGEX: Regex = Regex::new(r#"^[a-z0-9-]+$"#).unwrap();
        }

        if !input.is_empty() && !SUBDOMAIN_REGEX.is_match(input) {
            return Err(PlatformValidationError::InvalidSubdomainValue {
                subdomain: input.to_string(),
            });
        }

        Ok(ValidSubdomain {
            subdomain: input.to_string(),
        })
    }
}

fn integration_test_validation(int_test: &IntegrationTest) {
    match int_test {
        IntegrationTest::DnsResolutionWorksARecords { target_servers, .. } => {
            for server in target_servers {
                assert!(server.contains(":"), "Dns server must include port");
            }
        }
        IntegrationTest::DnsResolutionARecordCount { target_servers, .. } => {
            for server in target_servers {
                assert!(server.contains(":"), "Dns server must include port");
            }
        }
        IntegrationTest::DnsResolutionWorksNsRecords { target_servers, .. } => {
            for server in target_servers {
                assert!(server.contains(":"), "Dns server must include port");
            }
        }
        IntegrationTest::DnsResolutionWorksPtrRecords { target_servers, .. } => {
            for server in target_servers {
                assert!(server.contains(":"), "Dns server must include port");
            }
        }
        IntegrationTest::DnsSecWorksExternal { target_servers, .. } => {
            for server in target_servers {
                assert!(!server.contains(":"), "Dns server must not include port for dns sec");
            }
        }
        IntegrationTest::DnsSecWorksInternal { target_servers, .. } => {
            for server in target_servers {
                assert!(!server.contains(":"), "Dns server must not include port for dns sec");
            }
        }
        IntegrationTest::TcpSocketsOpen { target_sockets } => {
            for socket in target_sockets {
                assert!(socket.contains(":"), "TCP sockets must specify a port");
            }
        }
        IntegrationTest::PrometheusMetricExists { prometheus_server_ip, .. } => {
            assert_eq!(prometheus_server_ip.split(".").count(), 4, "Prometheus server ip must be ip address");
        }
        IntegrationTest::HttpGetRespondsOk { server_ips, .. } => {
            for ip in server_ips {
                assert_eq!(ip.split(".").count(), 4, "Http server target must be ip");
            }
        }
        IntegrationTest::HttpGetRespondsString { server_ips, .. } => {
            for ip in server_ips {
                assert_eq!(ip.split(".").count(), 4, "Http server target must be ip");
            }
        }
        IntegrationTest::PingWorks { server_ips } => {
            for ip in server_ips {
                assert_eq!(ip.split(".").count(), 4, "Ping server target must be ip");
            }
        }
        IntegrationTest::LokiWriterReaderTest { dns_server, reader_dns_name, writer_dns_name, .. } => {
            assert!(dns_server.contains(":"), "Dns server must include port");
            assert!(reader_dns_name.ends_with(".service.consul"), "Loki reader must be consul dns entry");
            assert!(writer_dns_name.ends_with(".service.consul"), "Loki writer must be consul dns entry");
        }
        IntegrationTest::LokiStreamExists { dns_server, reader_dns_name, .. } => {
            assert!(dns_server.contains(":"), "Dns server must include port");
            assert!(reader_dns_name.ends_with(".service.consul"), "Loki reader must be consul dns entry");
        },
        IntegrationTest::TempoSpansWritable { dns_server, service_name, .. } => {
            assert!(dns_server.contains(":"), "Dns server must include port");
            assert!(service_name.ends_with(".service.consul"), "Tempo service must be consul dns entry");
        }
        IntegrationTest::InsideNodeDnsAResolutionWorks { server_ips } => {
            for ip in server_ips.keys() {
                assert!(ip.parse::<std::net::Ipv4Addr>().is_ok(), "Cannot parse ipv4 address '{ip}'");
            }
        }
        IntegrationTest::CrossDcSourceIpCheck { port_range_start, server_to_run_iperf_server_from_with_private_ip, servers_to_run_iperf_client_from_with_expected_ips } => {
            assert!(*port_range_start > 1024 && *port_range_start < 65000, "Invalid port range {port_range_start}");
            let ip = &server_to_run_iperf_server_from_with_private_ip.0;
            assert!(ip.parse::<std::net::Ipv4Addr>().is_ok(), "Cannot parse ipv4 address '{ip}'");
            let ip = &server_to_run_iperf_server_from_with_private_ip.1;
            assert!(ip.parse::<std::net::Ipv4Addr>().is_ok(), "Cannot parse ipv4 address '{ip}'");
            for (ip1, ip2) in servers_to_run_iperf_client_from_with_expected_ips {
                assert!(ip1.parse::<std::net::Ipv4Addr>().is_ok(), "Cannot parse ipv4 address '{ip1}'");
                assert!(ip2.parse::<std::net::Ipv4Addr>().is_ok(), "Cannot parse ipv4 address '{ip2}'");
            }
        }
        IntegrationTest::InsideNodePingWorks { server_ips } => {
            for (from_ip, targets) in server_ips {
                assert!(from_ip.parse::<std::net::Ipv4Addr>().is_ok(), "Cannot parse ipv4 address '{from_ip}'");
                for target in targets {
                    assert!(target.parse::<std::net::Ipv4Addr>().is_ok(), "Cannot parse ipv4 address '{target}'");
                }
            }
        }
    }
}

pub fn epl_architecture_to_nomad_architecture(input: &str) -> NomadArchitectures {
    match input {
        "x86_64" => NomadArchitectures::Amd64,
        "arm64" => NomadArchitectures::Arm64,
        _ => panic!("Unknown input architecture {input}")
    }
}

#[test]
fn provisioning_scripts_dependency_order() {
    use strum::IntoEnumIterator;

    for t in ProvisioningScriptTag::iter() {
        let deps = t.tag_dependencies();
        for d in deps {
            assert_ne!(
                t, d,
                "No circular dependencies allowed in provisioning script tags"
            );
            assert!(
                t.tag_weight() > d.tag_weight(),
                "Dependency weight must be less than tag weight"
            );
        }
    }
}
