use std::collections::{BTreeSet, HashMap, BTreeMap};

use serde::{Serialize, Deserialize};

use crate::{database::{TableRowPointerDatacenter, Database, TableRowPointerServer, TableRowPointerServerKind, TableRowPointerServerDisk}, static_analysis::{PlatformValidationError, networking::first_three_octets, get_global_settings, projections::Projection, dc_impl::node_eligibility_calculation}};

pub struct AwsTopology {
    pub dcs: BTreeMap<TableRowPointerDatacenter, AwsDatacenter>,
    pub disk_configs: BTreeMap<TableRowPointerServerDisk, AwsDiskConfig>,
}

impl AwsTopology {
    pub fn is_empty(&self) -> bool {
        self.dcs.is_empty()
    }
}

pub struct AwsSubnet {
    pub is_public: bool,
    pub vpn_count: usize,
}

pub struct AwsDatacenter {
    pub availability_zone: String,
    pub region: String,
    pub subnet_map: BTreeMap<String, AwsSubnet>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AwsDatacenterArguments {
    availability_zone: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AwsDiskConfig {
    pub provisioned_iops: Option<i64>,
    pub provisioned_throughput_mb: Option<i64>,
}

lazy_static! {
    // Recommended config as per
    // https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/device_naming.html#available-ec2-device-names
    static ref AWS_EXTRA_EBS_DISK_ID_REGEX: regex::Regex =
        regex::Regex::new(r#"^sd[f-p]$"#).unwrap();
}

pub fn compute_aws_topology(
    db: &Database,
    server_kinds: &Projection<TableRowPointerServer, TableRowPointerServerKind>,
    server_disk_sizes: &BTreeMap<TableRowPointerServerDisk, i64>,
) -> Result<AwsTopology, PlatformValidationError> {
    let mut res = AwsTopology { dcs: BTreeMap::new(), disk_configs: BTreeMap::new() };
    let available_azs = aws_availability_zones();

    let dc_implementation = "aws";
    let server_kind_prefix = "aws.";
    let disk_kind_prefix = "aws.";

    let mut az_map: HashMap<String, String> = HashMap::new();
    let mut spent_aws_regions: HashMap<String, String> = HashMap::new();

    let mut aws_server_kind_count = 0usize;
    for sk in db.server_kind().rows_iter() {
        let sk_name = db.server_kind().c_kind(sk);
        if sk_name.starts_with(server_kind_prefix) {
            aws_server_kind_count += 1;

            let sub = &sk_name[server_kind_prefix.len()..sk_name.len()];
            if let Some(mt) = AWS_SUPPORTED_MACHINES.get(sub) {
                assert_eq!(&mt.arch, db.server_kind().c_architecture(sk));
                let mem_derived_bytes = mt.memory_mb as f64 * 1024.0 * 1024.0;
                let mem_expected_bytes = db.server_kind().c_memory_bytes(sk) as f64;
                let diff = mem_derived_bytes / mem_expected_bytes;
                //println!("{mem_derived_bytes} / {mem_expected_bytes} = {diff}");
                assert!((diff - 1.0).abs() < 0.0001);
                assert_eq!(mt.cores as i64, db.server_kind().c_cores(sk));
            } else {
                return Err(PlatformValidationError::AwsAddingCustomInstanceTypesIsNotAllowed {
                    alien_aws_server_kind: sk_name.to_string(),
                });
            }
        }
    }

    let mut aws_disk_kind_count = 0usize;
    for dk in db.disk_kind().rows_iter() {
        let dk_name = db.disk_kind().c_kind(dk);
        if dk_name.starts_with(disk_kind_prefix) {
            aws_disk_kind_count += 1;
            if !AWS_SUPPORTED_DISK_KINDS.contains(dk_name) {
                return Err(PlatformValidationError::AwsAddingCustomDiskKindsIsNotAllowed {
                    alien_aws_disk_kind: dk_name.to_string(),
                });
            }
        }
    }

    // users cannot add aws machine types
    assert_eq!(AWS_SUPPORTED_MACHINES.len(), aws_server_kind_count);
    assert_eq!(AWS_SUPPORTED_DISK_KINDS.len(), aws_disk_kind_count);

    for region in db.region().rows_iter() {
        let mut current_aws_regions: BTreeSet<String> = BTreeSet::new();
        for dc in db.region().c_referrers_datacenter__region(region) {
            if db.datacenter().c_implementation(*dc) == dc_implementation {
                let dc_name = db.datacenter().c_dc_name(*dc);
                let default_sk = db.server_kind().c_kind(db.datacenter().c_default_server_kind(*dc));
                if !default_sk.starts_with(server_kind_prefix) {
                    return Err(PlatformValidationError::AwsDatacenterDefaultServerKindMustStartWithAws {
                        aws_datacenter: dc_name.clone(),
                        invalid_server_kind: default_sk.clone(),
                        expected_server_kind_prefix: server_kind_prefix.to_string(),
                    });
                }
                // gateways per subnet
                let mut subnet_map: BTreeMap<String, AwsSubnet> = BTreeMap::new();

                let settings = db.datacenter().c_implementation_settings(*dc);
                let arguments: AwsDatacenterArguments = serde_yaml::from_str(&settings)
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
                    return Err(PlatformValidationError::AwsDatacenterUnknownAvailabilityZone {
                        dc: dc_name.clone(),
                        dc_implementation: dc_implementation.to_string(),
                      unknown_availability_zone: arguments.availability_zone.clone(),
                        current_settings: settings.clone(),
                    });
                }

                if let Some(prev) = az_map.insert(arguments.availability_zone.clone(), dc_name.clone()) {
                    return Err(PlatformValidationError::AwsSameAvailabilityZoneUsedForMultipleDatacenters {
                        duplicate_az: arguments.availability_zone.clone(),
                        previous_dc: prev.clone(),
                        current_dc: dc_name.clone(),
                    });
                }

                let mut aws_region = arguments.availability_zone.clone();
                let _ = aws_region.pop();
                if let Some(prev_region) = spent_aws_regions.get(&aws_region) {
                    return Err(PlatformValidationError::AwsRegionIsUsedInMoreThanOneEdenPlatformRegion {
                        overused_aws_region: aws_region.clone(),
                        epl_regions_using_aws_region: vec![prev_region.clone(), db.region().c_region_name(region).clone()],
                    });
                }

                let _ = current_aws_regions.insert(aws_region.clone());
                if current_aws_regions.len() > 1 {
                    return Err(PlatformValidationError::AwsMoreThanAwsOneRegionInsideEdenPlatformRegion {
                        eden_platform_region: db.region().c_region_name(region).clone(),
                        found_aws_regions: current_aws_regions.into_iter().collect::<Vec<_>>()
                    });
                }

                for server in db.datacenter().c_referrers_server__dc(*dc) {
                    let server_kind = server_kinds.value(*server);
                    let root_disk = db.server().c_root_disk(*server);
                    let root_disk_id = db.server_disk().c_disk_id(root_disk);
                    let root_disk_kind = db.server_disk().c_disk_kind(root_disk);
                    let root_disk_kind_name = db.disk_kind().c_kind(root_disk_kind);

                    let sk_name = db.server_kind().c_kind(*server_kind);
                    if !sk_name.starts_with(server_kind_prefix) {
                        return Err(PlatformValidationError::AwsEveryServerKindInAwsMustStartWithAws {
                            aws_server: db.server().c_hostname(*server).clone(),
                            server_kind: sk_name.clone(),
                            expected_server_kind_prefix: server_kind_prefix.to_string(),
                        });
                    }

                    let mut hypervisor = "";
                    for sk_attr in db.server_kind().c_children_server_kind_attribute(*server_kind) {
                        if db.server_kind_attribute().c_key(*sk_attr) == "hypervisor" {
                            hypervisor = db.server_kind_attribute().c_value(*sk_attr);
                        }
                    }
                    let aws_sk_name = &sk_name[server_kind_prefix.len()..];
                    let extra_machine_info = AWS_SUPPORTED_MACHINES.get(aws_sk_name).unwrap();
                    let maximum_machine_iops = &extra_machine_info.maximum_iops;
                    assert!(!hypervisor.is_empty(), "All hypervisor attributes should have been set now");

                    let can_use_io1_io2 = hypervisor == "nitro" || hypervisor == "metal";
                    for disk in db.server().c_children_server_disk(*server) {
                        let disk_kind = db.server_disk().c_disk_kind(*disk);
                        let disk_kind_name = db.disk_kind().c_kind(disk_kind);
                        let disk_id = db.server_disk().c_disk_id(*disk);
                        let extra_config = db.server_disk().c_extra_config(*disk);
                        let is_root = *disk == root_disk;
                        if !disk_kind_name.starts_with(disk_kind_prefix) {
                            return Err(PlatformValidationError::AwsEveryDiskKindInAwsMustStartWithAws {
                                aws_server: db.server().c_hostname(*server).clone(),
                                disk_id: disk_id.clone(),
                                disk_kind: disk_kind_name.clone(),
                                expected_disk_kind_prefix: disk_kind_prefix.to_string(),
                            });
                        }

                        let disk_size = *server_disk_sizes.get(disk).unwrap();
                        let size_rem = 1024 * 1024 * 1024;
                        let remainder = disk_size % size_rem;
                        if remainder != 0 {
                            return Err(PlatformValidationError::AwsDiskSizeMustBeMultipleOfGigabyte {
                                aws_server: db.server().c_hostname(*server).clone(),
                                disk_id: disk_id.clone(),
                                disk_size,
                                remainder_of_current_gigabyte: remainder,
                                bytes_until_next_gigabyte: size_rem - remainder,
                            });
                        }

                        let disk_kind_real = &disk_kind_name[disk_kind_prefix.len()..];
                        let parsed_config = parse_aws_disk_config(
                            disk_kind_real, extra_config.as_str(),
                            db.server().c_hostname(*server).as_str(), disk_id,
                            disk_size / 1024 / 1024 / 1024,
                            &maximum_machine_iops,
                            aws_sk_name,
                        )?;
                        if let Some(parsed_config) = parsed_config {
                            assert!(res.disk_configs.insert(*disk, parsed_config).is_none());
                        }

                        if (disk_kind_real == "io1" || disk_kind_real == "io2") && !can_use_io1_io2 {
                            return Err(PlatformValidationError::AwsCannotUseDiskKindOnHypervisorInstance {
                                aws_server: db.server().c_hostname(*server).clone(),
                                aws_disk_id: disk_id.clone(),
                                aws_disk_kind: disk_kind_name.clone(),
                                server_hypervisor: hypervisor.to_string(),
                                only_allowed_hypervisors_for_disk_kind: vec![
                                    "nitro".to_string(),
                                    "metal".to_string(),
                                ],
                            });
                        }

                        if !is_root {
                            let disk_id = db.server_disk().c_disk_id(*disk);
                            if !AWS_EXTRA_EBS_DISK_ID_REGEX.is_match(&disk_id) {
                                return Err(PlatformValidationError::AwsNonRootDiskMustFollowRecommendedConvention {
                                    aws_server: db.server().c_hostname(*server).clone(),
                                    non_root_disk_id: disk_id.clone(),
                                    must_match_regex: AWS_EXTRA_EBS_DISK_ID_REGEX.to_string(),
                                });
                            }
                        }
                    }

                    if root_disk_kind_name == "aws.st1" || root_disk_kind_name == "aws.sc1" {
                        return Err(PlatformValidationError::AwsDiskKindIsNotAllowedToBeRoot {
                            aws_server: db.server().c_hostname(*server).clone(),
                            disk_id: root_disk_id.clone(),
                            disk_kind: root_disk_kind_name.clone(),
                            forbidden_root_disk_kinds: vec![
                                "aws.st1".to_string(),
                                "aws.sc1".to_string(),
                            ]
                        });
                    }

                    match hypervisor {
                        "nitro" => {
                            let expected_id = "nvme0n1";
                            if root_disk_id != expected_id {
                                return Err(PlatformValidationError::AwsRootDiskOnHypervisorMustBeNamedThis {
                                    aws_server: db.server().c_hostname(*server).clone(),
                                    root_disk_id: root_disk_id.clone(),
                                    hypervisor: hypervisor.to_string(),
                                    root_disk_only_expected_id: expected_id.to_string(),
                                });
                            }
                        }
                        "xen" => {
                            let expected_id = "xvda";
                            if root_disk_id != expected_id {
                                return Err(PlatformValidationError::AwsRootDiskOnHypervisorMustBeNamedThis {
                                    aws_server: db.server().c_hostname(*server).clone(),
                                    root_disk_id: root_disk_id.clone(),
                                    hypervisor: hypervisor.to_string(),
                                    root_disk_only_expected_id: expected_id.to_string(),
                                });
                            }
                        }
                        "metal" => {
                            let expected_id = "nvme0n1";
                            if root_disk_id != expected_id {
                                return Err(PlatformValidationError::AwsRootDiskOnHypervisorMustBeNamedThis {
                                    aws_server: db.server().c_hostname(*server).clone(),
                                    root_disk_id: root_disk_id.clone(),
                                    hypervisor: hypervisor.to_string(),
                                    root_disk_only_expected_id: expected_id.to_string(),
                                });
                            }
                        }
                        _ => {
                            panic!("Unknown hypervisor {hypervisor}")
                        }
                    }

                    for net_if in db.server().c_children_network_interface(*server) {
                        let net_ptr = db.network_interface().c_if_network(*net_if);
                        let if_name = db.network_interface().c_if_name(*net_if);
                        let if_prefix = db.network_interface().c_if_prefix(*net_if);
                        match db.network().c_network_name(net_ptr).as_str() {
                            "dcrouter" => {
                                // this is needed to uniformly set public ips inside edendb source
                                if if_name.as_str() != "eth1" {
                                    return Err(PlatformValidationError::AwsDcrouterNetworkInterfaceNameMustBeEth1 {
                                        aws_dc: dc_name.clone(),
                                        aws_server: db.server().c_hostname(*server).clone(),
                                        aws_network_interface: if_name.clone(),
                                        aws_network_interface_only_allowed_name: "eth1".to_string(),
                                    });
                                }
                            }
                            "lan" => {
                                if if_name.as_str() != "eth0" {
                                    return Err(PlatformValidationError::AwsLanNetworkInterfaceNameMustBeEth0 {
                                        aws_dc: dc_name.clone(),
                                        aws_server: db.server().c_hostname(*server).clone(),
                                        aws_lan_network_interface: if_name.clone(),
                                        aws_lan_network_interface_only_allowed_name: "eth0".to_string(),
                                    });
                                }

                                let subnet_id = first_three_octets(db.network_interface().c_if_ip(*net_if));
                                let e = subnet_map.entry(subnet_id).or_insert_with(|| {
                                    AwsSubnet {
                                        is_public: false,
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
                                    return Err(PlatformValidationError::AwsSshInterfaceForPublicServerMustBePublic {
                                        aws_dc: dc_name.clone(),
                                        aws_server: db.server().c_hostname(*server).clone(),
                                        aws_server_expected_ssh_interface: "void".to_string(),
                                        aws_server_ssh_interface: db.network_interface().c_if_name(db.server().c_ssh_interface(*server)).clone(),
                                    });
                                }

                                if if_prefix != 32 {
                                    return Err(PlatformValidationError::AwsInternetInterfaceMustHave32Prefix {
                                        aws_dc: dc_name.clone(),
                                        aws_server: db.server().c_hostname(*server).clone(),
                                        aws_server_interface: if_name.clone(),
                                        aws_server_mask: if_prefix,
                                        aws_server_only_allowed_mask: 32,
                                    });
                                }
                            },
                            "vpn" => {},
                            unexpected => {
                                panic!("Unexpected network {unexpected}")
                            }
                        }
                    }

                    if db.server().c_public_ipv6_address_prefix(*server) != 128 {
                        return Err(PlatformValidationError::AwsPublicIpv6AddressMustHave128AsPrefix {
                            aws_dc: dc_name.clone(),
                            aws_server: db.server().c_hostname(*server).clone(),
                            aws_server_ipv6_prefix: db.server().c_public_ipv6_address_prefix(*server),
                            aws_server_only_allowed_prefix: 128,
                        });
                    }
                }

                for data in &mut subnet_map.values_mut() {
                    match data.vpn_count {
                        0 => {}, // private subnet
                        2 => {
                            data.is_public = true;
                        },
                        _ => {
                            panic!("Should have been caught earlier by DcRoutingSubnetCannotMixDeclaredVpnGatewaysAndDeclaredRouters");
                        }
                    }
                }

                assert!(res.dcs.insert(*dc, AwsDatacenter {
                    availability_zone: arguments.availability_zone.clone(),
                    region: aws_region,
                    subnet_map,
                }).is_none());
            }
        }

        for aws_reg in current_aws_regions {
            spent_aws_regions.insert(aws_reg, db.region().c_region_name(region).clone());
        }
    }

    aws_meta_checks(db, !res.dcs.is_empty())?;

    Ok(res)
}

lazy_static! {
    static ref VALID_BUCKET_NAME_REGEX: regex::Regex = regex::Regex::new(r#"^[a-z0-9-]+$"#).unwrap();
}

fn aws_meta_checks(db: &Database, is_aws_used: bool) -> Result<(), PlatformValidationError> {
    let global_settings = get_global_settings(db);

    if is_aws_used {
        if global_settings.aws_artefacts_s3_bucket_name.is_empty() {
            return Err(PlatformValidationError::AwsArtefactsBucketIsUndefined {
                table_name: "global_settings".to_string(),
                table_column: "aws_artefacts_s3_bucket_name".to_string(),
                current_value: global_settings.aws_artefacts_s3_bucket_name.to_string(),
            });
        }

        let max_bucket_name_size = 32;
        if global_settings.aws_artefacts_s3_bucket_name.len() > max_bucket_name_size {
            return Err(PlatformValidationError::AwsArtefactsBucketIsTooLong {
                table_name: "global_settings".to_string(),
                table_column: "aws_artefacts_s3_bucket_name".to_string(),
                current_value: global_settings.aws_artefacts_s3_bucket_name.to_string(),
                current_length: global_settings.aws_artefacts_s3_bucket_name.len(),
                max_length: max_bucket_name_size,
            });
        }

        if !VALID_BUCKET_NAME_REGEX.is_match(&global_settings.aws_artefacts_s3_bucket_name) {
            return Err(PlatformValidationError::AwsArtefactsBucketHasNonKebabCaseName {
                table_name: "global_settings".to_string(),
                table_column: "aws_artefacts_s3_bucket_name".to_string(),
                current_value: global_settings.aws_artefacts_s3_bucket_name.to_string(),
            });
        }
    } else {
        if !global_settings.aws_artefacts_s3_bucket_name.is_empty() {
            return Err(PlatformValidationError::AwsNotUsedButArtefactsBucketIsDefined {
                table_name: "global_settings".to_string(),
                table_column: "aws_artefacts_s3_bucket_name".to_string(),
                current_value: global_settings.aws_artefacts_s3_bucket_name.to_string(),
            });
        }
    }

    Ok(())
}

fn example_settings() -> AwsDatacenterArguments {
    AwsDatacenterArguments {
        availability_zone: "us-west-1b".to_string(),
    }
}

fn aws_availability_zones() -> BTreeSet<&'static str> {
    let zones = [
        "af-south-1a",
        "af-south-1b",
        "af-south-1c",
        "ap-east-1a",
        "ap-east-1b",
        "ap-east-1c",
        "ap-northeast-1a",
        "ap-northeast-1c",
        "ap-northeast-1d",
        "ap-northeast-2a",
        "ap-northeast-2b",
        "ap-northeast-2c",
        "ap-northeast-2d",
        "ap-northeast-3a",
        "ap-northeast-3b",
        "ap-northeast-3c",
        "ap-south-1a",
        "ap-south-1b",
        "ap-south-1c",
        "ap-south-2a",
        "ap-south-2b",
        "ap-south-2c",
        "ap-southeast-1a",
        "ap-southeast-1b",
        "ap-southeast-1c",
        "ap-southeast-2a",
        "ap-southeast-2b",
        "ap-southeast-2c",
        "ap-southeast-3a",
        "ap-southeast-3b",
        "ap-southeast-3c",
        "ap-southeast-4a",
        "ap-southeast-4b",
        "ap-southeast-4c",
        "ca-central-1a",
        "ca-central-1b",
        "ca-central-1d",
        "eu-central-1a",
        "eu-central-1b",
        "eu-central-1c",
        "eu-central-2a",
        "eu-central-2b",
        "eu-central-2c",
        "eu-north-1a",
        "eu-north-1b",
        "eu-north-1c",
        "eu-south-1a",
        "eu-south-1b",
        "eu-south-1c",
        "eu-south-2a",
        "eu-south-2b",
        "eu-south-2c",
        "eu-west-1a",
        "eu-west-1b",
        "eu-west-1c",
        "eu-west-2a",
        "eu-west-2b",
        "eu-west-2c",
        "eu-west-3a",
        "eu-west-3b",
        "eu-west-3c",
        "il-central-1a",
        "il-central-1b",
        "il-central-1c",
        "me-central-1a",
        "me-central-1b",
        "me-central-1c",
        "me-south-1a",
        "me-south-1b",
        "me-south-1c",
        "sa-east-1a",
        "sa-east-1b",
        "sa-east-1c",
        "us-east-1a",
        "us-east-1b",
        "us-east-1c",
        "us-east-1d",
        "us-east-1e",
        "us-east-1f",
        "us-east-2a",
        "us-east-2b",
        "us-east-2c",
        "us-west-1b",
        "us-west-1c",
        "us-west-2a",
        "us-west-2b",
        "us-west-2c",
        "us-west-2d",
    ];

    let mut res = BTreeSet::new();

    for zone in zones {
        assert!(res.insert(zone));
    }

    res
}

#[derive(Deserialize)]
pub struct AwsMachineType {
    pub arch: String,
    pub memory_mb: f32,
    pub cores: u32,
    pub instance_type: String,
    pub hypervisor: Option<String>,
    pub bare_metal: bool,
    pub maximum_iops: Option<i64>,
}

lazy_static! {
    pub static ref AWS_SUPPORTED_DISK_KINDS: BTreeSet<String> = aws_supported_disk_kinds();
    pub static ref AWS_SUPPORTED_MACHINES: BTreeMap<String, AwsMachineType> = aws_supported_machines();
    pub static ref AWS_INSTANCE_TYPES_EDL_SOURCE: String = aws_instance_types_edl_source();
}

fn aws_supported_disk_kinds() -> BTreeSet<String> {
    let mut res = BTreeSet::new();

    assert!(res.insert("aws.standard".to_string()));
    assert!(res.insert("aws.gp2".to_string()));
    assert!(res.insert("aws.gp3".to_string()));
    assert!(res.insert("aws.io1".to_string()));
    assert!(res.insert("aws.io2".to_string()));
    assert!(res.insert("aws.sc1".to_string()));
    assert!(res.insert("aws.st1".to_string()));

    res
}

fn aws_supported_machines() -> BTreeMap<String, AwsMachineType> {
    let mut res = BTreeMap::new();

    let parsed: Vec<AwsMachineType> =
        serde_json::from_str(include_str!("aws-instance-types.json"))
            .expect("Can't parse aws machine types");

    for mt in parsed {
        assert!(res.insert(mt.instance_type.clone(), mt).is_none());
    }

    res
}

fn aws_instance_types_edl_source() -> String {
    use std::fmt::Write;

    let mut res = String::new();

    res += "DATA STRUCT server_kind [\n";

    for mt in AWS_SUPPORTED_MACHINES.values() {
        let instance_type = &mt.instance_type;
        let cores = mt.cores;
        let memory_bytes = mt.memory_mb as i64 * 1024 * 1024;
        let arch = &mt.arch;
        let mut non_eligible_reason = aws_non_eligible_reason(mt);
        if non_eligible_reason.is_empty() {
            non_eligible_reason = node_eligibility_calculation(memory_bytes);
        }
        let hypervisor = mt.hypervisor.as_ref().map(|i| i.as_str()).unwrap_or("metal");
        let bare_metal = mt.bare_metal;
        if !bare_metal {
            assert!(mt.hypervisor.is_some());
        } else {
            assert!(mt.hypervisor.is_none());
        }
        write!(&mut res, r#"  {{
    kind: aws.{instance_type},
    cores: {cores},
    memory_bytes: {memory_bytes},
    architecture: {arch},
    non_eligible_reason: "{non_eligible_reason}",
    bare_metal: {bare_metal},
    WITH server_kind_attribute {{
      key: hypervisor,
      value: {hypervisor},
    }}
  }},
"#).unwrap();
    }

    res += "]\n";

    res
}

fn aws_non_eligible_reason(mt: &AwsMachineType) -> String {
    if mt.arch != "x86_64" {
        return format!("Architecture {} not supported in eden platform", mt.arch)
    }

    "".to_string()
}

fn parse_aws_disk_config(
    disk_kind_aws: &str,
    extra_config: &str,
    server: &str,
    disk_id: &str,
    disk_size_gb: i64,
    max_instance_iops: &Option<i64>,
    aws_server_kind: &str
) -> Result<Option<AwsDiskConfig>, PlatformValidationError> {
    struct DiskCheck {
        throughput_mb_min_max: Option<(i64, i64)>,
        iops_per_gb_limit: Option<i64>,
        min_iops: i64,
        max_iops: i64,
    }

    let check = match disk_kind_aws {
        "gp3" => {
            DiskCheck {
                throughput_mb_min_max: Some((125, 1000)),
                iops_per_gb_limit: None,
                min_iops: 3000,
                max_iops: 16000,
            }
        }
        "io1" => {
            DiskCheck {
                throughput_mb_min_max: None,
                iops_per_gb_limit: Some(50),
                min_iops: 100,
                max_iops: 64000,
            }
        }
        "io2" => {
            DiskCheck {
                throughput_mb_min_max: None,
                iops_per_gb_limit: Some(1000),
                min_iops: 100,
                max_iops: 256000,
            }
        }
        _ => {
            if extra_config.is_empty() {
                return Ok(None);
            } else {
                panic!("Should have been checked earlier {disk_kind_aws}");
            }
        }
    };

    let default =
        AwsDiskConfig {
            // baseline
            provisioned_iops: Some(check.min_iops),
            provisioned_throughput_mb: check.throughput_mb_min_max.map(|(min, _)| {
                min
            }),
        };
    if extra_config.is_empty() {
        return Ok(Some(default));
    } else {
        let parsed_disk_conf: AwsDiskConfig
            = serde_yaml::from_str(extra_config).map_err(|e| {
                PlatformValidationError::AwsInvalidDiskExtraConfig {
                    aws_server: server.to_string(),
                    aws_disk_id: disk_id.to_string(),
                    aws_disk_kind: disk_kind_aws.to_string(),
                    config_provided: extra_config.to_string(),
                    example_valid_config: serde_yaml::to_string(&default).unwrap(),
                    error: e.to_string(),
                }
            })?;
        if let Some(iops) = &parsed_disk_conf.provisioned_iops {
            let min_iops = check.min_iops;
            let max_iops = check.max_iops;
            if *iops < check.min_iops || *iops > check.max_iops {
                return Err(PlatformValidationError::AwsInvalidDiskExtraConfig {
                    aws_server: server.to_string(),
                    aws_disk_id: disk_id.to_string(),
                    aws_disk_kind: disk_kind_aws.to_string(),
                    config_provided: extra_config.to_string(),
                    example_valid_config: serde_yaml::to_string(&default).unwrap(),
                    error: format!("iops provided must be at least {min_iops} and no more than {max_iops}, got {iops}"),
                })
            }

            if let Some(iops_per_gb_limit) = check.iops_per_gb_limit {
                let max_space_bound_iops = disk_size_gb * iops_per_gb_limit;
                if *iops > max_space_bound_iops {
                    return Err(PlatformValidationError::AwsInvalidDiskExtraConfig {
                        aws_server: server.to_string(),
                        aws_disk_id: disk_id.to_string(),
                        aws_disk_kind: disk_kind_aws.to_string(),
                        config_provided: extra_config.to_string(),
                        example_valid_config: serde_yaml::to_string(&default).unwrap(),
                        error: format!("iops provided is more than disk size/iops ratio ({iops_per_gb_limit} IOPS/GB), maximum possible with {disk_size_gb}GB disk is {max_space_bound_iops}"),
                    })
                }
            }

            if let Some(max_instance_iops) = max_instance_iops {
                if iops > max_instance_iops {
                    return Err(PlatformValidationError::AwsInvalidDiskExtraConfig {
                        aws_server: server.to_string(),
                        aws_disk_id: disk_id.to_string(),
                        aws_disk_kind: disk_kind_aws.to_string(),
                        config_provided: extra_config.to_string(),
                        example_valid_config: serde_yaml::to_string(&default).unwrap(),
                        error: format!("iops {iops} provisioned for disk is more than maximum available for EC2 instance kind of {aws_server_kind} which has maximum of {max_instance_iops}"),
                    })
                }
            }
        }
        if let Some(throughput) = &parsed_disk_conf.provisioned_throughput_mb {
            if let Some((min_mb, max_mb)) = &check.throughput_mb_min_max {
                if throughput < min_mb || throughput > max_mb {
                    return Err(PlatformValidationError::AwsInvalidDiskExtraConfig {
                        aws_server: server.to_string(),
                        aws_disk_id: disk_id.to_string(),
                        aws_disk_kind: disk_kind_aws.to_string(),
                        config_provided: extra_config.to_string(),
                        example_valid_config: serde_yaml::to_string(&default).unwrap(),
                        error: format!("throughput provided must be at least {min_mb}MB/s and no more than {max_mb}MB/s, got {throughput}MB/s"),
                    })
                }
            } else {
                return Err(PlatformValidationError::AwsInvalidDiskExtraConfig {
                    aws_server: server.to_string(),
                    aws_disk_id: disk_id.to_string(),
                    aws_disk_kind: disk_kind_aws.to_string(),
                    config_provided: extra_config.to_string(),
                    example_valid_config: serde_yaml::to_string(&default).unwrap(),
                    error: format!("provisioned_throughput_mb should not be set for disk type {disk_kind_aws}"),
                })
            }
        }
        return Ok(Some(parsed_disk_conf));
    }
}
