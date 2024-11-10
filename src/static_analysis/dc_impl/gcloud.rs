use std::collections::{BTreeSet, BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{database::{TableRowPointerDatacenter, Database, TableRowPointerServer, TableRowPointerServerKind, TableRowPointerServerDisk}, static_analysis::{PlatformValidationError, networking::first_three_octets, get_global_settings, projections::Projection, dc_impl::node_eligibility_calculation}};

pub struct GcloudTopology {
    pub dcs: BTreeMap<TableRowPointerDatacenter, GcloudDatacenter>,
    pub disk_configs: BTreeMap<TableRowPointerServerDisk, GcloudDiskConfig>,
}

impl GcloudTopology {
    pub fn is_empty(&self) -> bool {
        self.dcs.is_empty()
    }
}

pub struct GcloudDatacenter {
    pub availability_zone: String,
    pub region: String,
    pub subnet_map: BTreeMap<String, GcloudSubnet>,
}

pub struct GcloudSubnet {
    pub vpn_count: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GcloudDatacenterArguments {
    availability_zone: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GcloudDiskConfig {
    pub provisioned_iops: Option<i64>,
    pub provisioned_throughput_mb: Option<i64>,
}

lazy_static! {
    static ref GCLOUD_EXTRA_DISK_ID_REGEX: regex::Regex =
        regex::Regex::new(r#"^vd[a-z]$"#).unwrap();
}

pub fn compute_gcloud_topology(
    db: &Database,
    server_kinds: &Projection<TableRowPointerServer, TableRowPointerServerKind>,
    server_disk_sizes: &BTreeMap<TableRowPointerServerDisk, i64>,
) -> Result<GcloudTopology, PlatformValidationError> {
    let mut res = GcloudTopology { dcs: BTreeMap::new(), disk_configs: BTreeMap::new() };
    let available_azs = gcloud_availability_zones();

    let dc_implementation = "gcloud";
    let server_kind_prefix = "gcloud.";
    let disk_kind_prefix = "gcloud.";

    let mut az_map: HashMap<String, String> = HashMap::new();
    let mut spent_gcloud_regions: HashMap<String, String> = HashMap::new();

    let mut google_server_kind_count = 0usize;
    for sk in db.server_kind().rows_iter() {
        let sk_name = db.server_kind().c_kind(sk);
        if sk_name.starts_with(server_kind_prefix) {
            google_server_kind_count += 1;

            let sub = &sk_name[server_kind_prefix.len()..sk_name.len()];
            if let Some(mt) = GCLOUD_SUPPORTED_MACHINES.get(sub) {
                assert_eq!(mt.architecture, db.server_kind().c_architecture(sk));
                let mem_derived_bytes = ((mt.memory_gb as f64) * 1024.0) as usize * 1024 * 1024;
                let mem_expected_bytes = db.server_kind().c_memory_bytes(sk) as f64;
                let diff = (mem_derived_bytes as f64) / mem_expected_bytes;
                let abs_diff = (diff - 1.0).abs();
                //println!("{mem_derived_bytes} / {mem_expected_bytes} = {diff}");
                assert!(abs_diff < 1.0, "{abs_diff} < 0.0001");
                assert_eq!(mt.cores as i64, db.server_kind().c_cores(sk));
            } else {
                return Err(PlatformValidationError::GcloudAddingCustomInstanceTypesIsNotAllowed {
                    alien_gcloud_server_kind: sk_name.to_string(),
                });
            }
        }
    }

    for dk in db.disk_kind().rows_iter() {
        let dk_name = db.disk_kind().c_kind(dk);
        if dk_name.starts_with(disk_kind_prefix) {
            if !GCLOUD_DISK_KINDS.contains(dk_name.as_str()) {
                return Err(PlatformValidationError::GcloudAddingCustomDiskKindsIsNotAllowed {
                    alien_gcloud_disk_kind: dk_name.to_string(),
                });
            }
        }
    }

    // users cannot add google cloud machine types
    assert_eq!(GCLOUD_SUPPORTED_MACHINES.len(), google_server_kind_count);

    for region in db.region().rows_iter() {
        let mut current_gcloud_regions: BTreeSet<String> = BTreeSet::new();
        for dc in db.region().c_referrers_datacenter__region(region) {
            if db.datacenter().c_implementation(*dc) == dc_implementation {
                let dc_name = db.datacenter().c_dc_name(*dc);
                let default_sk = db.server_kind().c_kind(db.datacenter().c_default_server_kind(*dc));
                if !default_sk.starts_with(server_kind_prefix) {
                    return Err(PlatformValidationError::GcloudDatacenterDefaultServerKindMustStartWithGcloud {
                        gcloud_datacenter: dc_name.clone(),
                        invalid_server_kind: default_sk.clone(),
                        expected_server_kind_prefix: server_kind_prefix.to_string(),
                    });
                }
                // gateways per subnet
                let mut subnet_map: BTreeMap<String, GcloudSubnet> = BTreeMap::new();

                let settings = db.datacenter().c_implementation_settings(*dc);
                let arguments: GcloudDatacenterArguments = serde_yaml::from_str(&settings)
                    .map_err(|e| {
                        let ex = example_settings();
                        PlatformValidationError::DatacenterImplementationInvalidSettings {
                            dc: dc_name.clone(),
                            dc_implementation: dc_implementation.to_string(),
                            current_settings: settings.clone(),
                            parsing_error: e.to_string(),
                            example_settings: serde_yaml::to_string(&ex).unwrap(),
                        }
                    })?;

                if !available_azs.contains(arguments.availability_zone.as_str()) {
                    return Err(PlatformValidationError::GcloudDatacenterUnknownAvailabilityZone {
                        dc: dc_name.clone(),
                        dc_implementation: dc_implementation.to_string(),
                        unknown_availability_zone: arguments.availability_zone.clone(),
                        current_settings: settings.clone(),
                    });
                }

                if let Some(prev) = az_map.insert(arguments.availability_zone.clone(), dc_name.clone()) {
                    return Err(PlatformValidationError::GcloudSameAvailabilityZoneUsedForMultipleDatacenters {
                        duplicate_az: arguments.availability_zone.clone(),
                        previous_dc: prev.clone(),
                        current_dc: dc_name.clone(),
                    });
                }

                let mut gcloud_region_v = arguments.availability_zone.split("-").collect::<Vec<_>>();
                assert_eq!(gcloud_region_v.len(), 3);
                // pop az
                let _ = gcloud_region_v.pop();
                let gcloud_region = gcloud_region_v.join("-");
                if let Some(prev_region) = spent_gcloud_regions.get(&gcloud_region) {
                    return Err(PlatformValidationError::GcloudRegionIsUsedInMoreThanOneEdenPlatformRegion {
                        overused_gcloud_region: gcloud_region.clone(),
                        epl_regions_using_gcloud_region: vec![prev_region.clone(), db.region().c_region_name(region).clone()],
                    });
                }

                let _ = current_gcloud_regions.insert(gcloud_region.clone());
                if current_gcloud_regions.len() > 1 {
                    return Err(PlatformValidationError::GcloudMoreThanGcloudOneRegionInsideEdenPlatformRegion {
                        eden_platform_region: db.region().c_region_name(region).clone(),
                        found_gcloud_regions: current_gcloud_regions.into_iter().collect::<Vec<_>>()
                    });
                }

                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    let server_kind = server_kinds.value(*server);
                    let sk_name = db.server_kind().c_kind(*server_kind);
                    let root_disk = db.server().c_root_disk(*server);
                    let root_disk_id = db.server_disk().c_disk_id(root_disk);
                    let mut total_disk_sizes_bytes: i64 = 0;
                    if root_disk_id != "sda" {
                        return Err(PlatformValidationError::GcloudRootDiskMustBeNamedSda {
                            gcloud_server: db.server().c_hostname(*server).clone(),
                            root_disk_id: root_disk_id.clone(),
                            root_disk_only_expected_id: "sda".to_string(),
                        });
                    }

                    if !sk_name.starts_with(server_kind_prefix) {
                        return Err(PlatformValidationError::GcloudEverServerKindInGoogleCloudMustStartWithGcloud {
                            gcloud_server: db.server().c_hostname(*server).clone(),
                            server_kind: sk_name.clone(),
                            expected_server_kind_prefix: server_kind_prefix.to_string(),
                        });
                    }

                    for disk in db.server().c_children_server_disk(*server) {
                        let disk_id = db.server_disk().c_disk_id(*disk);
                        let extra_config = db.server_disk().c_extra_config(*disk);
                        let disk_kind = db.server_disk().c_disk_kind(*disk);
                        let disk_kind_name = db.disk_kind().c_kind(disk_kind);
                        let is_root_disk = root_disk == *disk;
                        if !disk_kind_name.starts_with(disk_kind_prefix) {
                            return Err(PlatformValidationError::GcloudEveryDiskKindInGcloudMustStartWithGcloud {
                                gcloud_server: db.server().c_hostname(*server).clone(),
                                disk_id: disk_id.clone(),
                                disk_kind: disk_kind_name.to_string(),
                                expected_disk_kind_prefix: disk_kind_prefix.to_string(),
                            });
                        }

                        let disk_size = *server_disk_sizes.get(disk).unwrap();
                        total_disk_sizes_bytes += disk_size;
                        let size_rem = 1024 * 1024 * 1024;
                        let remainder = disk_size % size_rem;
                        if remainder != 0 {
                            return Err(PlatformValidationError::GcloudDiskSizeMustBeMultipleOfGigabyte {
                                gcloud_server: db.server().c_hostname(*server).clone(),
                                disk_id: db.server_disk().c_disk_id(*disk).clone(),
                                disk_size,
                                remainder_of_current_gigabyte: remainder,
                                bytes_until_next_gigabyte: size_rem - remainder,
                            });
                        }

                        if !is_root_disk && !GCLOUD_EXTRA_DISK_ID_REGEX.is_match(disk_id.as_str()) {
                            return Err(PlatformValidationError::GcloudNonRootDiskMustFollowThisConvention {
                                gcloud_server: db.server().c_hostname(*server).clone(),
                                non_root_disk_id: disk_id.clone(),
                                must_match_regex: GCLOUD_EXTRA_DISK_ID_REGEX.to_string(),
                            });
                        }

                        let disk_kind_real = &disk_kind_name[disk_kind_prefix.len()..];
                        let parsed_config = parse_gcloud_disk_config(
                            disk_kind_real, extra_config.as_str(),
                            db.server().c_hostname(*server).as_str(), disk_id,
                            disk_size / 1024 / 1024 / 1024,
                        )?;
                        if let Some(parsed_config) = parsed_config {
                            assert!(res.disk_configs.insert(*disk, parsed_config).is_none());
                        }
                    }

                    let disk_size_limit: i64 = 256 * 1024 * 1024 * 1024 * 1024;
                    if total_disk_sizes_bytes > disk_size_limit {
                        return Err(PlatformValidationError::GcloudTotalDiskSizeOnServerExceedsLimit {
                            gcloud_server: db.server().c_hostname(*server).clone(),
                            total_disk_size_limit: disk_size_limit,
                            total_disk_sizes_sum: total_disk_sizes_bytes,
                            explanation: "Total disk sizes attached to google compute instance cannot be above 256TiB".to_string(),
                        });
                    }

                    for net_if in db.server().c_children_network_interface(*server) {
                        let net_ptr = db.network_interface().c_if_network(*net_if);
                        let if_name = db.network_interface().c_if_name(*net_if);
                        let if_prefix = db.network_interface().c_if_prefix(*net_if);
                        match db.network().c_network_name(net_ptr).as_str() {
                            "dcrouter" => {
                                // this is needed to uniformly set public ips inside edendb source
                                if if_name.as_str() != "eth1" {
                                    return Err(PlatformValidationError::GcloudDcrouterNetworkInterfaceNameMustBeEth1 {
                                        gcloud_dc: dc_name.clone(),
                                        gcloud_server: db.server().c_hostname(*server).clone(),
                                        gcloud_dcrouter_network_interface: if_name.clone(),
                                        gcloud_dcrouter_network_interface_only_allowed_name: "eth1".to_string(),
                                    });
                                }
                            }
                            "lan" => {
                                if if_name.as_str() != "eth0" {
                                    return Err(PlatformValidationError::GcloudLanNetworkInterfaceNameMustBeEth0 {
                                        gcloud_dc: dc_name.clone(),
                                        gcloud_server: db.server().c_hostname(*server).clone(),
                                        gcloud_public_network_interface: if_name.clone(),
                                        gcloud_public_network_interface_only_allowed_name: "eth0".to_string(),
                                    });
                                }

                                let subnet_id = first_three_octets(db.network_interface().c_if_ip(*net_if));
                                let e = subnet_map.entry(subnet_id).or_insert_with(|| {
                                    GcloudSubnet {
                                        vpn_count: 0,
                                    }
                                });

                                if db.server().c_is_vpn_gateway(*server) {
                                    e.vpn_count += 1;
                                }
                            }
                            "internet" => {
                                // we do this so we could bootstrap nodes with colmena via public ip
                                if db.server().c_ssh_interface(*server) != *net_if {
                                    return Err(PlatformValidationError::GcloudSshInterfaceForPublicServerMustBePublic {
                                        gcloud_dc: dc_name.clone(),
                                        gcloud_server: db.server().c_hostname(*server).clone(),
                                        gcloud_server_expected_ssh_interface: "void".to_string(),
                                        gcloud_server_ssh_interface: db.network_interface().c_if_name(db.server().c_ssh_interface(*server)).clone(),
                                    });
                                }

                                if if_prefix != 32 {
                                    return Err(PlatformValidationError::GcloudInternetInterfaceMustHave32Prefix {
                                        gcloud_dc: dc_name.clone(),
                                        gcloud_server: db.server().c_hostname(*server).clone(),
                                        gcloud_server_interface: if_name.clone(),
                                        gcloud_server_mask: if_prefix,
                                        gcloud_server_only_allowed_mask: 32,
                                    });
                                }
                            }
                            "vpn" => {},
                            unexpected => {
                                panic!("Unexpected network {unexpected}")
                            }
                        }
                    }
                }

                for data in subnet_map.values_mut() {
                    match data.vpn_count {
                        0 | 2 => {},
                        _ => {
                            panic!("Should have been caught earlier by DcRoutingSubnetCannotMixDeclaredVpnGatewaysAndDeclaredRouters");
                        }
                    }
                }

                assert!(res.dcs.insert(*dc, GcloudDatacenter {
                    availability_zone: arguments.availability_zone.clone(),
                    region: gcloud_region,
                    subnet_map,
                }).is_none());
            }
        }

        for gcloud_reg in current_gcloud_regions {
            spent_gcloud_regions.insert(gcloud_reg, db.region().c_region_name(region).clone());
        }
    }

    google_cloud_meta_checks(db, !res.dcs.is_empty())?;

    Ok(res)
}

lazy_static! {
    static ref VALID_BUCKET_NAME_REGEX: regex::Regex = regex::Regex::new(r#"^[a-z0-9-]+$"#).unwrap();
}

fn google_cloud_meta_checks(db: &Database, is_google_cloud_used: bool) -> Result<(), PlatformValidationError> {
    let global_settings = get_global_settings(db);

    if is_google_cloud_used {
        if global_settings.google_cloud_project_id.is_empty() {
            return Err(PlatformValidationError::GcloudProjectIdIsUndefined {
                table_name: "global_settings".to_string(),
                table_column: "google_cloud_project_id".to_string(),
                current_value: global_settings.google_cloud_project_id.to_string(),
            });
        }

        if global_settings.google_cloud_artefacts_bucket_name.is_empty() {
            return Err(PlatformValidationError::GcloudArtefactsBucketIsUndefined {
                table_name: "global_settings".to_string(),
                table_column: "google_cloud_artefacts_bucket_name".to_string(),
                current_value: global_settings.google_cloud_artefacts_bucket_name.to_string(),
            });
        }

        let max_bucket_name_size = 32;
        if global_settings.google_cloud_artefacts_bucket_name.len() > max_bucket_name_size {
            return Err(PlatformValidationError::GcloudArtefactsBucketIsTooLong {
                table_name: "global_settings".to_string(),
                table_column: "google_cloud_artefacts_bucket_name".to_string(),
                current_value: global_settings.google_cloud_artefacts_bucket_name.to_string(),
                current_length: global_settings.google_cloud_artefacts_bucket_name.len(),
                max_length: max_bucket_name_size,
            });
        }

        if !VALID_BUCKET_NAME_REGEX.is_match(&global_settings.google_cloud_artefacts_bucket_name) {
            return Err(PlatformValidationError::GcloudArtefactsBucketHasNonKebabCaseName {
                table_name: "global_settings".to_string(),
                table_column: "google_cloud_artefacts_bucket_name".to_string(),
                current_value: global_settings.google_cloud_artefacts_bucket_name.to_string(),
            });
        }
    } else {
        if !global_settings.google_cloud_project_id.is_empty() {
            return Err(PlatformValidationError::GcloudNotUsedButProjectIdIsDefined {
                table_name: "global_settings".to_string(),
                table_column: "google_cloud_project_id".to_string(),
                current_value: global_settings.google_cloud_project_id.to_string(),
            });
        }
        if !global_settings.google_cloud_artefacts_bucket_name.is_empty() {
            return Err(PlatformValidationError::GcloudNotUsedButArtefactsBucketIsDefined {
                table_name: "global_settings".to_string(),
                table_column: "google_cloud_artefacts_bucket_name".to_string(),
                current_value: global_settings.google_cloud_artefacts_bucket_name.to_string(),
            });
        }
    }

    Ok(())
}

fn example_settings() -> GcloudDatacenterArguments {
    GcloudDatacenterArguments {
        availability_zone: "us-east1-b".to_string(),
    }
}

// update with 'gcloud compute zones list'
fn gcloud_availability_zones() -> BTreeSet<&'static str> {
    let zones = [
        "asia-east1-a",
        "asia-east1-b",
        "asia-east1-c",
        "asia-east2-a",
        "asia-east2-b",
        "asia-east2-c",
        "asia-northeast1-a",
        "asia-northeast1-b",
        "asia-northeast1-c",
        "asia-northeast2-a",
        "asia-northeast2-b",
        "asia-northeast2-c",
        "asia-northeast3-a",
        "asia-northeast3-b",
        "asia-northeast3-c",
        "asia-south1-a",
        "asia-south1-b",
        "asia-south1-c",
        "asia-south2-a",
        "asia-south2-b",
        "asia-south2-c",
        "asia-southeast1-a",
        "asia-southeast1-b",
        "asia-southeast1-c",
        "asia-southeast2-a",
        "asia-southeast2-b",
        "asia-southeast2-c",
        "australia-southeast1-a",
        "australia-southeast1-b",
        "australia-southeast1-c",
        "australia-southeast2-a",
        "australia-southeast2-b",
        "australia-southeast2-c",
        "europe-central2-a",
        "europe-central2-b",
        "europe-central2-c",
        "europe-north1-a",
        "europe-north1-b",
        "europe-north1-c",
        "europe-southwest1-a",
        "europe-southwest1-b",
        "europe-southwest1-c",
        "europe-west1-b",
        "europe-west1-c",
        "europe-west1-d",
        "europe-west10-a",
        "europe-west10-b",
        "europe-west10-c",
        "europe-west12-a",
        "europe-west12-b",
        "europe-west12-c",
        "europe-west2-a",
        "europe-west2-b",
        "europe-west2-c",
        "europe-west3-a",
        "europe-west3-b",
        "europe-west3-c",
        "europe-west4-a",
        "europe-west4-b",
        "europe-west4-c",
        "europe-west6-a",
        "europe-west6-b",
        "europe-west6-c",
        "europe-west8-a",
        "europe-west8-b",
        "europe-west8-c",
        "europe-west9-a",
        "europe-west9-b",
        "europe-west9-c",
        "me-central1-a",
        "me-central1-b",
        "me-central1-c",
        "me-central2-a",
        "me-central2-b",
        "me-central2-c",
        "me-west1-a",
        "me-west1-b",
        "me-west1-c",
        "northamerica-northeast1-a",
        "northamerica-northeast1-b",
        "northamerica-northeast1-c",
        "northamerica-northeast2-a",
        "northamerica-northeast2-b",
        "northamerica-northeast2-c",
        "southamerica-east1-a",
        "southamerica-east1-b",
        "southamerica-east1-c",
        "southamerica-west1-a",
        "southamerica-west1-b",
        "southamerica-west1-c",
        "us-central1-a",
        "us-central1-b",
        "us-central1-c",
        "us-central1-f",
        "us-east1-b",
        "us-east1-c",
        "us-east1-d",
        "us-east4-a",
        "us-east4-b",
        "us-east4-c",
        "us-east5-a",
        "us-east5-b",
        "us-east5-c",
        "us-south1-a",
        "us-south1-b",
        "us-south1-c",
        "us-west1-a",
        "us-west1-b",
        "us-west1-c",
        "us-west2-a",
        "us-west2-b",
        "us-west2-c",
        "us-west3-a",
        "us-west3-b",
        "us-west3-c",
        "us-west4-a",
        "us-west4-b",
        "us-west4-c",
    ];

    let mut res = BTreeSet::new();

    for zone in zones {
        assert!(res.insert(zone));
    }

    res
}

pub struct GcloudMachineType {
    pub architecture: &'static str,
    pub memory_gb: f32,
    pub cores: u32,
}

lazy_static! {
    pub static ref GCLOUD_SUPPORTED_MACHINES: BTreeMap<String, GcloudMachineType> = google_cloud_supported_machines();
    pub static ref GCLOUD_INSTANCE_TYPES_EDL_SOURCE: String = google_cloud_instance_types_edl_source();
}

fn google_cloud_supported_machines() -> BTreeMap<String, GcloudMachineType> {
    let mut res = BTreeMap::new();

    for mt in google_cloud_x86_machine_types_dump().lines().filter(|i| !i.is_empty()) {
        let spl: Vec<_> = mt.split(" ").collect();
        let machine_type = &spl[0];
        let architecture =
            if is_gcloud_machine_arm64(&machine_type) {
                "arm64"
            } else if is_gcloud_machine_x86(&machine_type) {
                "x86_64"
            } else {
                ""
            };
        if !architecture.is_empty() {
            assert!(res.insert(machine_type.to_string(), GcloudMachineType {
                architecture,
                cores: spl[1].parse().expect("Can't parse core count"),
                memory_gb: spl[2].parse().expect("Can't parse memory"),
            }).is_none());
        } else {
            eprintln!("WARNING: Machine type {machine_type} has no known architecture, not added to Eden platform");
        }
    }

    res
}

lazy_static! {
    static ref GCLOUD_DISK_KINDS: BTreeSet<&'static str> = google_cloud_disk_kinds();
}

fn google_cloud_disk_kinds() -> BTreeSet<&'static str> {
    let mut res = BTreeSet::new();

    assert!(res.insert("gcloud.pd-balanced"));
    assert!(res.insert("gcloud.pd-standard"));
    assert!(res.insert("gcloud.pd-ssd"));
    assert!(res.insert("gcloud.pd-extreme"));
    assert!(res.insert("gcloud.hyperdisk-balanced"));
    assert!(res.insert("gcloud.hyperdisk-throughput"));
    assert!(res.insert("gcloud.hyperdisk-extreme"));

    res
}

fn is_gcloud_machine_x86(name: &str) -> bool {
    let machines_prefixes = [
        "a2-",
        "a3-",
        "c2-",
        "c2d-",
        "c3-",
        "c3d-",
        "ct",
        "e2-",
        "f1-",
        "g1-",
        "g2-",
        "h3-",
        "m1-",
        "m2-",
        "m3-",
        "n1-",
        "n2-",
        "n2d-",
        "t2-",
        "t2d-",
        "z3-",
    ];

    for mp in machines_prefixes {
        if name.starts_with(mp) {
            return true;
        }
    }

    return false;
}

fn is_gcloud_machine_arm64(name: &str) -> bool {
    let machines_prefixes = [
        "t2a-",
    ];

    for mp in machines_prefixes {
        if name.starts_with(mp) {
            return true;
        }
    }

    return false;
}

fn google_cloud_instance_types_edl_source() -> String {
    use std::fmt::Write;

    let mut res = String::new();

    res += "DATA STRUCT server_kind [\n";

    for (instance_type, mt) in GCLOUD_SUPPORTED_MACHINES.iter() {
        let cores = mt.cores;
        let memory_bytes = ((mt.memory_gb as f64) * 1024.0) as i64 * 1024 * 1024;
        let arch = &mt.architecture;
        let non_eligible_reason = node_eligibility_calculation(memory_bytes);
        write!(&mut res, r#"  {{
    kind: gcloud.{instance_type},
    cores: {cores},
    memory_bytes: {memory_bytes},
    architecture: {arch},
    non_eligible_reason: "{non_eligible_reason}",
  }},
"#).unwrap();
    }

    res += "]\n";

    res
}

fn google_cloud_x86_machine_types_dump() -> &'static str {
    include_str!("gcloud-instance-types.txt")
}

fn parse_gcloud_disk_config(
    disk_kind_gcloud: &str,
    extra_config: &str,
    server: &str,
    disk_id: &str,
    disk_size_gb: i64,
) -> Result<Option<GcloudDiskConfig>, PlatformValidationError> {
    struct DiskCheck {
        throughput_mb_min_max: Option<(i64, i64)>,
        iops_per_gb_limit: Option<i64>,
        min_max_iops: Option<(i64, i64)>,
        iops_div_throughput_limit: Option<i64>,
    }

    let check = match disk_kind_gcloud {
        "hyperdisk-balanced" => {
            DiskCheck {
                throughput_mb_min_max: Some((140, 2400)),
                iops_per_gb_limit: Some(500),
                min_max_iops: Some((3000, 160000)),
                iops_div_throughput_limit: Some(4),
            }
        }
        "hyperdisk-extreme" => {
            DiskCheck {
                throughput_mb_min_max: None,
                iops_per_gb_limit: Some(1000),
                // this min is more than allowed minimum but if you want this
                // disk kind disk at least start with some decent IOPS?
                min_max_iops: Some((3000, 350000)),
                iops_div_throughput_limit: None,
            }
        }
        "hyperdisk-throughput" => {
            DiskCheck {
                throughput_mb_min_max: Some((10, 600)),
                iops_per_gb_limit: Some(1000),
                // this is more than allowed minimum but if you want this
                // disk kind disk at least start with some decent IOPS?
                min_max_iops: None,
                iops_div_throughput_limit: None,
            }
        }
        _ => {
            if extra_config.is_empty() {
                return Ok(None);
            } else {
                panic!("Should have been checked earlier {disk_kind_gcloud}");
            }
        }
    };

    let default =
        GcloudDiskConfig {
            // baseline minimum. default values might be more but we want to always
            // start with minimum if unspecified so google cloud fags don't drain our pockets
            provisioned_iops: check.min_max_iops.map(|(min, _)| min),
            provisioned_throughput_mb: check.throughput_mb_min_max.map(|(min, _)| min),
        };
    if extra_config.is_empty() {
        return Ok(Some(default));
    } else {
        let parsed_disk_conf: GcloudDiskConfig
            = serde_yaml::from_str(extra_config).map_err(|e| {
                PlatformValidationError::GcloudInvalidDiskExtraConfig {
                    gcloud_server: server.to_string(),
                    gcloud_disk_id: disk_id.to_string(),
                    gcloud_disk_kind: disk_kind_gcloud.to_string(),
                    config_provided: extra_config.to_string(),
                    example_valid_config: serde_yaml::to_string(&default).unwrap(),
                    error: e.to_string(),
                }
            })?;
        if let Some((min_iops, max_iops)) = &check.min_max_iops {
            if let Some(iops) = &parsed_disk_conf.provisioned_iops {
                if iops < min_iops || iops > max_iops {
                    return Err(PlatformValidationError::GcloudInvalidDiskExtraConfig {
                        gcloud_server: server.to_string(),
                        gcloud_disk_id: disk_id.to_string(),
                        gcloud_disk_kind: disk_kind_gcloud.to_string(),
                        config_provided: extra_config.to_string(),
                        example_valid_config: serde_yaml::to_string(&default).unwrap(),
                        error: format!("iops provided must be at least {min_iops} and no more than {max_iops}, got {iops}"),
                    })
                }

                if let Some(iops_per_gb_limit) = check.iops_per_gb_limit {
                    let max_space_bound_iops = disk_size_gb * iops_per_gb_limit;
                    if *iops > max_space_bound_iops {
                        return Err(PlatformValidationError::GcloudInvalidDiskExtraConfig {
                            gcloud_server: server.to_string(),
                            gcloud_disk_id: disk_id.to_string(),
                            gcloud_disk_kind: disk_kind_gcloud.to_string(),
                            config_provided: extra_config.to_string(),
                            example_valid_config: serde_yaml::to_string(&default).unwrap(),
                            error: format!("iops provided is more than disk size/iops ratio ({iops_per_gb_limit} IOPS/GB), maximum possible with {disk_size_gb}GB disk is {max_space_bound_iops}"),
                        })
                    }
                }
            }
        } else {
            if parsed_disk_conf.provisioned_iops.is_some() {
                return Err(PlatformValidationError::GcloudInvalidDiskExtraConfig {
                    gcloud_server: server.to_string(),
                    gcloud_disk_id: disk_id.to_string(),
                    gcloud_disk_kind: disk_kind_gcloud.to_string(),
                    config_provided: extra_config.to_string(),
                    example_valid_config: serde_yaml::to_string(&default).unwrap(),
                    error: format!("Specifying IOPS is not allowed for disk kind"),
                })
            }
        }

        if let Some(throughput) = &parsed_disk_conf.provisioned_throughput_mb {
            if let Some((min_mb, max_mb)) = &check.throughput_mb_min_max {
                if throughput < min_mb || throughput > max_mb {
                    return Err(PlatformValidationError::GcloudInvalidDiskExtraConfig {
                        gcloud_server: server.to_string(),
                        gcloud_disk_id: disk_id.to_string(),
                        gcloud_disk_kind: disk_kind_gcloud.to_string(),
                        config_provided: extra_config.to_string(),
                        example_valid_config: serde_yaml::to_string(&default).unwrap(),
                        error: format!("throughput provided must be at least {min_mb}MB/s and no more than {max_mb}MB/s, got {throughput}MB/s"),
                    })
                }

                if let Some(iops_div) = &check.iops_div_throughput_limit {
                    if let Some(iops) = &parsed_disk_conf.provisioned_iops {
                        let max_throughput = *iops / *iops_div;
                        if *throughput > max_throughput {
                            return Err(PlatformValidationError::GcloudInvalidDiskExtraConfig {
                                gcloud_server: server.to_string(),
                                gcloud_disk_id: disk_id.to_string(),
                                gcloud_disk_kind: disk_kind_gcloud.to_string(),
                                config_provided: extra_config.to_string(),
                                example_valid_config: serde_yaml::to_string(&default).unwrap(),
                                error: format!("provisioned throughput for disk kind must be not bigger than IOPS divided by {iops_div}. Maximum throughput for current config is {iops} IOPS / {iops_div} = {max_throughput} MB/s"),
                            })
                        }
                    }
                }
            } else {
                return Err(PlatformValidationError::GcloudInvalidDiskExtraConfig {
                    gcloud_server: server.to_string(),
                    gcloud_disk_id: disk_id.to_string(),
                    gcloud_disk_kind: disk_kind_gcloud.to_string(),
                    config_provided: extra_config.to_string(),
                    example_valid_config: serde_yaml::to_string(&default).unwrap(),
                    error: format!("provisioned_throughput_mb should not be set for disk type {disk_kind_gcloud}"),
                })
            }
        }

        return Ok(Some(parsed_disk_conf));
    }
}
