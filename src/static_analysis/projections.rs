use std::collections::{HashMap, BTreeSet};
use std::{collections::BTreeMap, hash::Hash};

use crate::database::{TableRowPointerFrontendApplication, TableRowPointerFrontendPage, TableRowPointerRegion, TableRowPointerServer, TableRowPointerServerKind};
use crate::{
    codegen,
    database::{
        Database, TableRowPointerBackendApplication, TableRowPointerBackendHttpEndpoint,
        TableRowPointerHttpEndpointDataType, TableRowPointerHttpMethods,
        TableRowPointerVersionedType,
    },
};

use super::applications::{backend_apps_in_region, frontend_apps_in_region};
use super::dc_impl::aws::compute_aws_topology;
use super::dc_impl::bm_simple::compute_bm_simple_topology;
use super::dc_impl::gcloud::compute_gcloud_topology;
use super::databases::clickhouse::{ch_schemas_in_region, check_nats_stream_import_regionality};
use super::databases::postgres::pg_schemas_in_region;
use super::dns::{dns_checks, server_fqdns};
use super::http_endpoints::{
    check_app_duplicate_paths, check_frontend_duplicate_paths, check_http_path, HttpPathTree,
    PathArgs,
};
use super::l2_provisioning::blackbox_deployments::compute_available_resources_projection;
use super::server_disks::server_disk_analysis;
use super::server_labels::build_label_database;
use super::{L1Projections, CloudTopologies, SyncChecksOutputs, get_global_settings, BmTopologies};
use super::networking::{compute_loki_clusters, compute_monitoring_clusters, ensure_no_double_use_minio_buckets, compute_tempo_clusters};
use super::l2_provisioning::epl_app_ingress::ingress_static_analysis;
use super::{
    bw_compat_types::compute_types,
    http_endpoints::{backend_ingress_endpoints, check_http_endpoint, CheckedHttpEndpoint},
    PlatformValidationError, Projections,
};
#[derive(Clone)]
pub struct Projection<T: Copy + Clone + Hash + Eq, V> {
    contents: HashMap<T, V>,
}

impl<T: Copy + Clone + Hash + Eq, V> Projection<T, V> {
    pub fn create(
        iter: impl ::std::iter::Iterator<Item = T>,
        mut fun: impl FnMut(T) -> V,
    ) -> Projection<T, V> {
        let mut contents = HashMap::new();

        for i in iter {
            let v = fun(i);
            let res = contents.insert(i, v);
            assert!(res.is_none());
        }

        Projection { contents }
    }

    pub fn maybe_create<U: std::error::Error>(
        iter: impl ::std::iter::Iterator<Item = T>,
        fun: impl Fn(T) -> Result<V, U>,
    ) -> Result<Projection<T, V>, U> {
        let mut contents = HashMap::new();

        for i in iter {
            let v = fun(i)?;
            let res = contents.insert(i, v);
            assert!(res.is_none());
        }

        Ok(Projection { contents })
    }

    pub fn derive_another<O>(&self, fun: impl Fn(T, &V) -> O) -> Projection<T, O> {
        let mut contents = HashMap::new();

        for (k, v) in self.contents.iter() {
            let v = fun(*k, v);
            let res = contents.insert(*k, v);
            assert!(res.is_none());
        }

        Projection { contents }
    }

    pub fn value(&self, k: T) -> &V {
        self.contents.get(&k).unwrap()
    }
}

pub struct Index<T: Clone + Hash + Eq, V: Copy + Clone> {
    contents: HashMap<T, Vec<V>>,
    empty_vec: Vec<V>,
}

impl<T: Clone + Hash + Eq, V: Copy + Clone> Index<T, V> {
    pub fn create(iter: impl ::std::iter::Iterator<Item = V>, fun: impl Fn(V) -> T) -> Index<T, V> {
        let mut contents: HashMap<T, Vec<V>> = HashMap::new();

        for i in iter {
            let k = fun(i);
            let e = contents.entry(k).or_default();
            e.push(i);
        }

        Index {
            contents,
            empty_vec: Vec::new(),
        }
    }

    pub fn values(&self, key: &T) -> &[V] {
        match self.contents.get(key) {
            Some(res) => res.as_slice(),
            None => self.empty_vec.as_slice(),
        }
    }
}

pub struct VersionedTypeUsageFlags {
    pub binary_deserialization: bool,
    pub binary_serialization: bool,
    pub json_deserialization: bool,
    pub json_serialization: bool,
}

impl VersionedTypeUsageFlags {
    pub fn new() -> VersionedTypeUsageFlags {
        VersionedTypeUsageFlags {
            binary_deserialization: false,
            binary_serialization: false,
            json_deserialization: false,
            json_serialization: false,
        }
    }
}

pub fn create_projections(db: &Database, sync_checks: &SyncChecksOutputs) -> Result<Projections, PlatformValidationError> {
    let default_used_nixpkgs_version = db
        .nixpkgs_environment()
        .rows_iter()
        .find(|i| db.nixpkgs_environment().c_name(*i) == "default_nixpkgs")
        .map(|i| db.nixpkgs_environment().c_version(i))
        .expect("default_nixpkgs environment is undefined");
    let default_used_nixpkgs_checksum = db
        .nixpkgs_version()
        .c_checksum(default_used_nixpkgs_version)
        .clone();
    let default_used_nixpkgs_tarball_checksum = db
        .nixpkgs_version()
        .c_tarball_checksum(default_used_nixpkgs_version)
        .clone();

    let server_fqdns = server_fqdns(db);
    let global_settings = get_global_settings(db);
    let mut server_kinds_by_name: HashMap<String, TableRowPointerServerKind> = HashMap::new();
    for server_kind in db.server_kind().rows_iter() {
        assert!(server_kinds_by_name.insert(db.server_kind().c_kind(server_kind).clone(), server_kind).is_none());
    }
    let server_disk_sizes = server_disk_analysis(db)?;
    let server_kinds: Projection<TableRowPointerServer, TableRowPointerServerKind> =
        Projection::maybe_create(db.server().rows_iter(), |server| {
            let kind_str = db.server().c_kind(server);
            let sk =
                if kind_str == "dc_default" {
                    db.datacenter().c_default_server_kind(db.server().c_dc(server))
                } else {
                    if let Some(sk) = server_kinds_by_name.get(kind_str) {
                        *sk
                    } else {
                        return Err(PlatformValidationError::ServerKindSpecifiedOnServerDoesntExist {
                            server_hostname: db.server().c_hostname(server).clone(),
                            non_existing_server_kind: kind_str.clone(),
                        });
                    }
                };
            if !db.server_kind().c_non_eligible_reason(sk).is_empty() {
                return Err(PlatformValidationError::NodeKindCannotBeUsedInEdenPlatform {
                    server_hostname: db.server().c_hostname(server).clone(),
                    uneligible_server_kind: kind_str.to_string(),
                    reason:db.server_kind().c_non_eligible_reason(sk).clone(),
                });
            }
            let sk_arch = db.server_kind().c_architecture(sk);
            if !global_settings.experimental_enable_arm64_support && sk_arch == "arm64" {
                return Err(PlatformValidationError::Arm64ServerKindsAreNotSupportedYet {
                    server_hostname: db.server().c_hostname(server).clone(),
                    unsupported_server_kind: kind_str.to_string(),
                    unsupported_server_cpu_architecture: sk_arch.clone(),
                });
            }

            let supported_architectures = ["x86_64", "arm64"];
            if !supported_architectures.contains(&sk_arch.as_str()) {
                return Err(PlatformValidationError::UnsupportedEdenPlatformServerArchitecture {
                    server_hostname: db.server().c_hostname(server).clone(),
                    unsupported_server_kind: kind_str.to_string(),
                    unsupported_server_cpu_architecture: sk_arch.clone(),
                    supported_architectures: supported_architectures.iter().map(|i| i.to_string()).collect(),
                });
            }
            return Ok(sk);
        })?;
    let label_database = build_label_database(db)?;

    let mut used_architectures: BTreeSet<String> = BTreeSet::new();
    let mut used_architectures_per_region: BTreeMap<TableRowPointerRegion, BTreeSet<String>> = BTreeMap::new();
    for region in db.region().rows_iter() {
        let mut this_region_architectures = BTreeSet::new();
        for dc in db.region().c_referrers_datacenter__region(region) {
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                let sk = server_kinds.value(*server);
                if !used_architectures.contains(db.server_kind().c_architecture(*sk)) {
                    let _ = used_architectures.insert(db.server_kind().c_architecture(*sk).clone());
                }
                if !this_region_architectures.contains(db.server_kind().c_architecture(*sk)) {
                    let _ = this_region_architectures.insert(db.server_kind().c_architecture(*sk).clone());
                }
            }
        }
        assert!(used_architectures_per_region.insert(region, this_region_architectures).is_none());
    }

    let dns_checks = dns_checks(db, &server_fqdns)?;
    let cloud_topologies = CloudTopologies {
        aws: compute_aws_topology(db, &server_kinds, &server_disk_sizes)?,
        gcloud: compute_gcloud_topology(db, &server_kinds, &server_disk_sizes)?,
    };
    let bm_topologies = BmTopologies {
        bm_simple: compute_bm_simple_topology(db)?,
    };

    let provisioning_server_in_region: Projection<TableRowPointerRegion, Option<TableRowPointerServer>> =
        Projection::create(db.region().rows_iter(), |r| {
            let dc = db.region().c_referrers_datacenter__region(r);
            if dc.is_empty() {
                return None;
            }

            let server = db.datacenter().c_referrers_server__dc(dc[0]);
            if server.is_empty() {
                return None;
            }

            Some(server[0])
        });


    let valid_http_method_by_name: Index<String, TableRowPointerHttpMethods> =
        Index::create(db.http_methods().rows_iter(), |ptr| {
            db.http_methods().c_http_method_name(ptr).clone()
        });

    let valid_http_data_type: Index<String, TableRowPointerHttpEndpointDataType> =
        Index::create(db.http_endpoint_data_type().rows_iter(), |ptr| {
            db.http_endpoint_data_type()
                .c_http_endpoint_data_type(ptr)
                .clone()
        });

    let bw_type_by_name: Index<String, TableRowPointerVersionedType> =
        Index::create(db.versioned_type().rows_iter(), |vtype| {
            db.versioned_type().c_type_name(vtype).clone()
        });

    let backend_ingress_endpoints = backend_ingress_endpoints(db)?;
    let pg_schemas_in_region = pg_schemas_in_region(db);
    check_nats_stream_import_regionality(db)?;
    let ch_schemas_in_region = ch_schemas_in_region(db);

    let checked_http_endpoints: Projection<
        TableRowPointerBackendHttpEndpoint,
        CheckedHttpEndpoint,
    > = Projection::maybe_create(db.backend_http_endpoint().rows_iter(), |endpoint| {
        check_http_endpoint(
            db,
            endpoint,
            &bw_type_by_name,
            &valid_http_method_by_name,
            &valid_http_data_type,
        )
    })?;

    let loki_clusters = compute_loki_clusters(db)?;
    let monitoring_clusters = compute_monitoring_clusters(db)?;
    let tempo_clusters = compute_tempo_clusters(db)?;

    let checked_app_http_paths: Projection<
        TableRowPointerBackendApplication,
        HttpPathTree<TableRowPointerBackendHttpEndpoint>,
    > = Projection::maybe_create(db.backend_application().rows_iter(), |app| {
        check_app_duplicate_paths(db, app, &checked_http_endpoints)
    })?;

    let checked_frontend_pages: Projection<TableRowPointerFrontendPage, PathArgs> =
        Projection::maybe_create(db.frontend_page().rows_iter(), |endpoint| {
            check_http_path(db.frontend_page().c_path(endpoint))
        })?;

    crate::static_analysis::applications::check_frontend_application_endpoint_types(db)?;

    let checked_frontend_http_paths: Projection<
        TableRowPointerFrontendApplication,
        HttpPathTree<TableRowPointerFrontendPage>,
    > = Projection::maybe_create(db.frontend_application().rows_iter(), |app| {
        if db
            .frontend_application()
            .c_children_frontend_page(app)
            .is_empty()
        {
            return Err(PlatformValidationError::FrontendApplicationHasNoPages {
                application_name: db.frontend_application().c_application_name(app).clone(),
            });
        }
        check_frontend_duplicate_paths(db, app, &checked_frontend_pages)
    })?;

    let (mut ingress_dns_entries, region_ingresses) = ingress_static_analysis(db, sync_checks)?;

    let frontend_deployment_page_wirings =
        crate::static_analysis::applications::check_frontend_application_page_wirings(db)?;
    let frontend_deployment_link_wirings =
        crate::static_analysis::applications::check_frontend_application_link_wirings(db)?;

    let application_used_bw_types: Projection<
        TableRowPointerBackendApplication,
        BTreeMap<TableRowPointerVersionedType, VersionedTypeUsageFlags>,
    > = Projection::create(db.backend_application().rows_iter(), |app| {
        let mut res: BTreeMap<TableRowPointerVersionedType, VersionedTypeUsageFlags> =
            BTreeMap::new();

        for stream in db
            .backend_application()
            .c_children_backend_application_nats_stream(app)
            .iter()
        {
            let vt = db.backend_application_nats_stream().c_stream_type(*stream);
            let e = res.entry(vt).or_insert_with(VersionedTypeUsageFlags::new);
            if db
                .backend_application_nats_stream()
                .c_enable_consumer(*stream)
            {
                e.json_deserialization = true;
            }
            if db
                .backend_application_nats_stream()
                .c_enable_producer(*stream)
            {
                e.json_serialization = true;
            }
        }

        for http_endpoint in db
            .backend_application()
            .c_children_backend_http_endpoint(app)
            .iter()
        {
            let ce = checked_http_endpoints.value(*http_endpoint);
            ce.input_body_type.iter().for_each(|ibt| {
                let e = res.entry(*ibt).or_insert_with(VersionedTypeUsageFlags::new);

                e.json_deserialization = true;
            });

            ce.output_body_type.iter().for_each(|ibt| {
                let e = res.entry(*ibt).or_insert_with(VersionedTypeUsageFlags::new);

                e.json_serialization = true;
            });
        }

        res
    });

    crate::static_analysis::applications::check_application_build_environments(db)?;
    let transaction_steps = crate::static_analysis::databases::postgres::check_transaction_steps(db)?;
    crate::static_analysis::databases::clickhouse::check_ch_schema_name_clash(db)?;
    let application_pg_shard_queries =
        crate::static_analysis::applications::check_application_pg_queries(db)?;
    let application_ch_shard_queries =
        crate::static_analysis::applications::check_application_ch_queries(db)?;
    let application_deployment_pg_wirings =
        crate::static_analysis::applications::application_deployments_pg_shard_wiring(db)?;
    let application_deployment_ch_wirings =
        crate::static_analysis::applications::application_deployments_ch_shard_wiring(db)?;
    let application_deployment_stream_wirings =
        crate::static_analysis::applications::application_deployments_nats_stream_wiring(db)?;
    let application_deployment_bucket_wirings =
        crate::static_analysis::applications::application_deployments_s3_buckets_wiring(db)?;
    let application_deployment_configs =
        crate::static_analysis::applications::application_deployments_config(db)?;
    let frontend_deployment_endpoint_wirings =
        crate::static_analysis::applications::frontend_deployments_endpoint_wirings(
            db,
            &backend_ingress_endpoints,
        )?;
    let backend_apps_in_region = backend_apps_in_region(db);
    let frontend_apps_in_region = frontend_apps_in_region(db);
    ensure_no_double_use_minio_buckets(db, &application_deployment_bucket_wirings)?;

    let versioned_types = compute_types(db)?;
    let rust_versioned_type_snippets = codegen::rust::compute_snippets(db, &versioned_types);
    let rust_sources_for_http_endpoints = codegen::rust::backend::compute_http_endpoints(
        db,
        &checked_http_endpoints,
        &rust_versioned_type_snippets,
    );
    let consul_network_iface =
        crate::static_analysis::networking::find_consul_network_interfaces(db)?;
    let internet_network_iface =
        crate::static_analysis::networking::find_internet_network_interfaces(db);
    let vpn_network_iface =
        crate::static_analysis::networking::find_vpn_network_interfaces(db);
    let vpn_gateways =
        crate::static_analysis::networking::vpn_gateways(db, &internet_network_iface, &vpn_network_iface)?;
    let vpn_p2p_links =
        crate::static_analysis::networking::vpn_p2p_links(db, &vpn_gateways)?;

    let series_database = crate::static_analysis::alerts::try_read_prometheus_series_databases()?;
    let parsed_alert_trigger_test_series =
        crate::static_analysis::alerts::verify_alert_trigger_test_series(db, &series_database)?;
    let promtool_test_suite = crate::static_analysis::alerts::generate_promtool_tests(
        db,
        &parsed_alert_trigger_test_series,
    );
    crate::static_analysis::alerts::run_sandboxed_promtool_tests(db, &promtool_test_suite)?;

    let epl_nomad_namespace = db.nomad_namespace()
      .rows_iter()
      .find(|ns| db.nomad_namespace().c_namespace(*ns) == "epl")
      .expect("we should always have this namespace");

    let bb_depl_resources_per_region = compute_available_resources_projection(db);

    let l1proj = L1Projections {
        application_deployment_pg_wirings: &application_deployment_pg_wirings,
        application_deployment_ch_wirings: &application_deployment_ch_wirings,
        application_ch_shard_queries: &application_ch_shard_queries,
        application_deployment_stream_wirings: &application_deployment_stream_wirings,
        application_deployment_bucket_wirings: &application_deployment_bucket_wirings,
        application_deployment_configs: &application_deployment_configs,
        checked_http_endpoints: &checked_http_endpoints,
        checked_frontend_pages: &checked_frontend_pages,
        backend_ingress_endpoints: &backend_ingress_endpoints,
        frontend_deployment_endpoint_wirings: &frontend_deployment_endpoint_wirings,
        frontend_deployment_page_wirings: &frontend_deployment_page_wirings,
        frontend_deployment_link_wirings: &frontend_deployment_link_wirings,
        loki_clusters: &loki_clusters,
        monitoring_clusters: &monitoring_clusters,
        tempo_clusters: &tempo_clusters,
        backend_apps_in_region: &backend_apps_in_region,
        frontend_apps_in_region: &frontend_apps_in_region,
        pg_schemas_in_region: &pg_schemas_in_region,
        ch_schemas_in_region: &ch_schemas_in_region,
        dns_checks: &dns_checks,
        consul_network_iface: &consul_network_iface,
        server_fqdns: &server_fqdns,
        internet_network_iface: &internet_network_iface,
        region_ingresses: &region_ingresses,
        networking: &sync_checks.network,
        server_kinds: &server_kinds,
        used_architectures_per_region: &used_architectures_per_region,
        label_database: &label_database,
        versioned_types: &versioned_types,
        bb_depl_resources_per_region: &bb_depl_resources_per_region,
        epl_nomad_namespace,
    };
    let server_runtime =
        crate::static_analysis::server_runtime::compute_server_runtime(db, &l1proj)?;

    server_runtime.server_runtime_checks(db, &l1proj)?;

    // what a mess, we add ingresses for region after server runtime
    // because only then we know admin service ips
    for region in db.region().rows_iter() {
        let tld = global_settings.admin_tld;
        // only dns master region exposes admin for now
        let is_dns_master = db.region().c_is_dns_master(region);
        if is_dns_master {
            match (&region_ingresses.get(&region), &server_runtime.admin_dns_ingress_entries().get(&tld)) {
                (Some(ingresses), Some(admin_svcs)) => {
                    let entries = ingress_dns_entries.entry(tld).or_default();
                    for svc in admin_svcs.as_slice() {
                        assert!(entries.insert(svc.clone(), (*ingresses).clone()).is_none(), "Unexpected ingress here");
                    }
                }
                _ => {}
            }
        }
    }

    Ok(Projections {
        server_fqdns,
        default_used_nixpkgs_version,
        default_used_nixpkgs_checksum,
        default_used_nixpkgs_tarball_checksum,
        dns_checks,
        application_pg_shard_queries,
        application_ch_shard_queries,
        application_deployment_pg_wirings,
        application_deployment_ch_wirings,
        application_deployment_stream_wirings,
        application_deployment_bucket_wirings,
        application_deployment_configs,
        frontend_deployment_endpoint_wirings,
        frontend_deployment_page_wirings,
        frontend_deployment_link_wirings,
        versioned_types,
        rust_versioned_type_snippets,
        application_used_bw_types,
        checked_http_endpoints,
        checked_app_http_paths,
        bw_type_by_name,
        valid_http_method_by_name,
        valid_http_data_type,
        rust_sources_for_http_endpoints,
        transaction_steps,
        consul_network_iface,
        internet_network_iface,
        vpn_network_iface,
        server_runtime,
        parsed_alert_trigger_test_series,
        series_database,
        promtool_test_suite,
        checked_frontend_pages,
        checked_frontend_http_paths,
        backend_ingress_endpoints,
        ingress_dns_entries,
        loki_clusters,
        monitoring_clusters,
        tempo_clusters,
        backend_apps_in_region,
        frontend_apps_in_region,
        pg_schemas_in_region,
        ch_schemas_in_region,
        provisioning_server_in_region,
        vpn_gateways,
        vpn_p2p_links,
        region_ingresses,
        cloud_topologies,
        bm_topologies,
        server_kinds,
        used_architectures,
        used_architectures_per_region,
        label_database,
        server_disk_sizes,
        bb_depl_resources_per_region,
    })
}
