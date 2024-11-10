use std::{fmt::Write, collections::{BTreeSet, BTreeMap}};

use crate::{static_analysis::{CheckedDB, networking::first_two_octets, get_global_settings}, codegen::Directory, database::TableRowPointerDatacenter};

pub fn generate_aws_outputs(checked: &CheckedDB, dir: &mut Directory) {
    dir.create_file("main.tf", generate_main_tf_file(checked));
}

struct AwsDatacenterAttrs {
    vpc_name: String,
}

impl AwsDatacenterAttrs {
    fn new(checked: &CheckedDB, dc: TableRowPointerDatacenter) -> AwsDatacenterAttrs {
        let aws_dc = checked.projections.cloud_topologies.aws.dcs.get(&dc).unwrap();
        Self { vpc_name: format!("epl-vpc-{}", aws_dc.region) }
    }
}

fn generate_main_tf_file(checked: &CheckedDB) -> String {
    let topology = &checked.projections.cloud_topologies.aws;
    let global_settings = get_global_settings(&checked.db);

    let mut res = String::new();

    let mut unique_regions: BTreeSet<String> = BTreeSet::new();
    let mut region_cidrs: BTreeMap<String, BTreeSet<String>> = BTreeMap::default();
    for (dc, top) in &topology.dcs {
        let _ = unique_regions.insert(top.region.clone());
        let cidrs = region_cidrs.entry(top.region.clone()).or_default();
        let cidr = checked.db.datacenter().c_network_cidr(*dc);
        assert!(cidrs.insert(cidr.clone()), "Should be all unique");
    }
    let other_regions_concat = unique_regions.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");

    let mut regions_with_amis: BTreeSet<String> = BTreeSet::new();

    let first_region = unique_regions.iter().next().unwrap();

    let transit_gw_needed = unique_regions.len() > 1;
    let project_name = &global_settings.project_name;
    write!(&mut res, r#"
variable "project_name" {{
    type = string
    default = "{project_name}"
}}
"#).unwrap();

    let mut ipv6_unique_cidrs_for_cidrs: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let maybe_vpc_ipv6 = if global_settings.enable_ipv6 {
        "
    assign_generated_ipv6_cidr_block = true"
    } else { "" };
    for ur in &unique_regions {
        let region_cidrs = region_cidrs.get(ur).unwrap();
        let mut cidr_iter = region_cidrs.iter();
        let main_cidr = cidr_iter.next().unwrap();
        write!(&mut res, r#"
provider "aws" {{
    region = "{ur}"
    alias = "{ur}"
}}

resource "aws_vpc" "epl-vpc-{ur}" {{
    provider = aws.{ur}
    enable_dns_support = "true"
    enable_dns_hostnames = "false"
    instance_tenancy = "default"
    cidr_block = "{main_cidr}"{maybe_vpc_ipv6}
}}
"#).unwrap();

        let mut secondary_cidr_idx = 0;
        for secondary_cidr in cidr_iter {
            secondary_cidr_idx += 1;
            write!(&mut res, r#"
resource "aws_vpc_ipv4_cidr_block_association" "{ur}_cidr_{secondary_cidr_idx}" {{
    provider = aws.{ur}
    vpc_id = aws_vpc.epl-vpc-{ur}.id
    cidr_block = "{secondary_cidr}"
}}
"#).unwrap();
        }
    }

    if transit_gw_needed {
        write!(&mut res, r#"
resource "aws_ec2_transit_gateway" "global" {{
    provider = aws.{first_region}

    auto_accept_shared_attachments = "enable"
}}
"#).unwrap();
    }

    write!(&mut res, r#"
resource "aws_s3_bucket" "artefacts_bucket" {{
    provider = aws.{first_region}

    bucket = "${{var.project_name}}-artefacts"
}}

resource "aws_iam_role" "vmimport_role" {{
    provider = aws.{first_region}

    name = "${{var.project_name}}-vmimport"
    assume_role_policy = jsonencode({{
        Version = "2012-10-17"
        Statement = [
            {{
                Effect = "Allow"
                Principal = {{ Service = "vmie.amazonaws.com" }}
                Action = "sts:AssumeRole"
                Condition = {{
                  StringEquals = {{
                    "sts:Externalid" = "vmimport"
                  }}
                }}
            }},
        ]
    }})
}}

resource "aws_iam_role_policy" "vmimport_role_policy" {{
    provider = aws.{first_region}

    name = "${{var.project_name}}-vmimport"
    role = aws_iam_role.vmimport_role.name

    policy = jsonencode({{
       Version = "2012-10-17"
       Statement = [
          {{
             Effect = "Allow",
             Action = [
                "s3:GetBucketLocation",
                "s3:GetObject",
                "s3:ListBucket",
             ],
             Resource = [
                "arn:aws:s3:::${{aws_s3_bucket.artefacts_bucket.id}}",
                "arn:aws:s3:::${{aws_s3_bucket.artefacts_bucket.id}}/*"
             ]
          }},
          {{
             Effect = "Allow"
             Action = [
                "ec2:ModifySnapshotAttribute",
                "ec2:CopySnapshot",
                "ec2:RegisterImage",
                "ec2:Describe*",
             ],
             Resource = "*"
          }}
       ]
    }})
}}
"#).unwrap();

    for ua in &checked.projections.used_architectures {
        write!(&mut res, r#"
resource "aws_s3_object" "default_vm_image_object_{ua}" {{
    provider = aws.{first_region}

    bucket = aws_s3_bucket.artefacts_bucket.id
    key = basename(trimspace(file("image-{ua}/aws-image.txt")))
    source = trimspace(file("image-{ua}/aws-image.txt"))
}}

resource "null_resource" "ami_images_all_regions_{ua}" {{
    depends_on = [ aws_s3_object.default_vm_image_object_{ua} ]

    provisioner "local-exec" {{
      command = "$AWS_IMAGE_UPLOAD_SCRIPT ${{path.cwd}}/image-{ua}/result"
      working_dir = "${{path.cwd}}/image-{ua}"
      environment = {{
        home_region = "{first_region}"
        regions = "{other_regions_concat}"
        bucket = aws_s3_bucket.artefacts_bucket.id
        service_role_name = aws_iam_role.vmimport_role.name
        state_dir = "${{path.cwd}}/image-{ua}/ec2-images"
        image_s3_key = aws_s3_object.default_vm_image_object_{ua}.key
        project_name = var.project_name
      }}
    }}
}}
"#).unwrap();
    }

    let mut region_subnet_ids: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for aws_region in &unique_regions {
        let vpc_name = format!("epl-vpc-{aws_region}");
        let private_sg_name = format!("private-node-{vpc_name}");
        let public_sg_name = format!("public-node-{vpc_name}");

        let private_sec_group = format!(r#"
resource "aws_security_group" "{private_sg_name}" {{
    provider = aws.{aws_region}

    vpc_id = aws_vpc.{vpc_name}.id
    name = "Private node"
}}

resource "aws_vpc_security_group_egress_rule" "private-node-sg-{vpc_name}-all" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.private-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    ip_protocol = "-1"
}}

resource "aws_vpc_security_group_ingress_rule" "private-node-sg-{vpc_name}-all" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.private-node-{vpc_name}.id

    cidr_ipv4   = "10.0.0.0/8"
    ip_protocol = "-1"
}}
"#);

        let maybe_ipv6_public_rules = if global_settings.enable_ipv6 {
            format!(r#"
resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-{vpc_name}-http" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv6   = "::/0"
    from_port   = 80
    ip_protocol = "tcp"
    to_port     = 80
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-{vpc_name}-https" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv6   = "::/0"
    from_port   = 443
    ip_protocol = "tcp"
    to_port     = 443
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-{vpc_name}-dns-tcp" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv6   = "::/0"
    from_port   = 53
    ip_protocol = "tcp"
    to_port     = 53
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-{vpc_name}-dns-udp" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv6   = "::/0"
    from_port   = 53
    ip_protocol = "udp"
    to_port     = 53
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-{vpc_name}-ping" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv6   = "::/0"
    from_port   = -1
    to_port     = -1
    ip_protocol = "icmpv6"
}}
"#)
        } else { "".to_string() };

        let public_sec_group = format!(r#"
resource "aws_security_group" "{public_sg_name}" {{
    provider = aws.{aws_region}

    vpc_id = aws_vpc.{vpc_name}.id
    name = "Public node"
}}

resource "aws_vpc_security_group_egress_rule" "public-node-sg-{vpc_name}-all" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    ip_protocol = "-1"
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-all" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "10.0.0.0/8"
    ip_protocol = "-1"
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-ssh" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 22
    ip_protocol = "tcp"
    to_port     = 22
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-http" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 80
    ip_protocol = "tcp"
    to_port     = 80
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-https" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 443
    ip_protocol = "tcp"
    to_port     = 443
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-dns-tcp" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 53
    ip_protocol = "tcp"
    to_port     = 53
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-dns-udp" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 53
    ip_protocol = "udp"
    to_port     = 53
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-ping" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = -1
    to_port     = -1
    ip_protocol = "icmp"
}}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-{vpc_name}-wireguard" {{
    provider = aws.{aws_region}

    security_group_id = aws_security_group.public-node-{vpc_name}.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 51820
    ip_protocol = "udp"
    to_port     = 51820
}}
"#);

        write!(&mut res, r#"
{public_sec_group}

{maybe_ipv6_public_rules}

{private_sec_group}

resource "aws_internet_gateway" "igw-{vpc_name}" {{
    provider = aws.{aws_region}

    vpc_id = aws_vpc.{vpc_name}.id
}}

resource "aws_route_table" "internet-{vpc_name}" {{
    provider = aws.{aws_region}

    vpc_id = aws_vpc.{vpc_name}.id
}}

resource "aws_route" "igw-route-{vpc_name}" {{
    provider = aws.{aws_region}

    route_table_id         = aws_route_table.internet-{vpc_name}.id
    destination_cidr_block = "0.0.0.0/0"
    gateway_id             = aws_internet_gateway.igw-{vpc_name}.id
}}
"#).unwrap();

        if global_settings.enable_ipv6 {
            write!(&mut res, r#"
resource "aws_route" "igw-route-ipv6-{vpc_name}" {{
    provider = aws.{aws_region}

    route_table_id              = aws_route_table.internet-{vpc_name}.id
    destination_ipv6_cidr_block = "::/0"
    gateway_id                  = aws_internet_gateway.igw-{vpc_name}.id
}}
"#).unwrap();
        }

    }

    for (dc, top) in &topology.dcs {
        let attrs = AwsDatacenterAttrs::new(checked, *dc);
        let cidr = checked.db.datacenter().c_network_cidr(*dc);
        let dc_name = checked.db.datacenter().c_dc_name(*dc);
        let dc_region = checked.db.datacenter().c_region(*dc);
        let vpc_name = &attrs.vpc_name;
        let aws_region = &top.region;
        let availability_zone = &top.availability_zone;
        let more_than_one_subnet = top.subnet_map.len() > 1;
        let router_subnet_id = format!("router_subnet_{dc_name}");
        let private_sg_name = format!("private-node-{vpc_name}");
        let public_sg_name = format!("public-node-{vpc_name}");

        if regions_with_amis.insert(aws_region.clone()) {
            for ua in &checked.projections.used_architectures {
                write!(&mut res, r#"

data "local_file" "ami_image_{ua}_{aws_region}" {{
    filename = "${{path.cwd}}/image-{ua}/ec2-images/{aws_region}.{ua}-linux.ami_id"

    depends_on = [ null_resource.ami_images_all_regions_{ua} ]
}}
"#).unwrap();
            }
        }

        if more_than_one_subnet {
            let router_network = format!("{}.252.0/22", first_two_octets(cidr));
            write!(&mut res, r#"
resource "aws_subnet" "{router_subnet_id}" {{
    provider = aws.{aws_region}

    vpc_id = aws_vpc.{vpc_name}.id
    cidr_block = "{router_network}"
    availability_zone = "{availability_zone}"
}}
"#).unwrap();
        }

        let subnet_ids = region_subnet_ids.entry(aws_region.clone()).or_default();
        // generate subnets
        for (subnet_key, sdata) in &top.subnet_map {
            let subnet = format!("{subnet_key}.0/24");
            let maybe_ipv6 = if global_settings.enable_ipv6 && sdata.is_public {
                let cidr_add = extract_ipv6_subnet_from_cidr(&subnet);
                let cidrs = ipv6_unique_cidrs_for_cidrs.entry(vpc_name.clone()).or_default();
                if !cidrs.insert(cidr_add) {
                    panic!("Hash collision {cidr_add} for 8 bit value for aws VPC {vpc_name}, gz boi!")
                }
                format!(r#"
    ipv6_cidr_block = "${{cidrsubnet(aws_vpc.{vpc_name}.ipv6_cidr_block, 8, {cidr_add})}}""#)
            } else {
                "".to_string()
            };
            let subnet_name = subnet.replace(".", "_").replace("/", "p");
            let subnet_id = format!("subnet_{subnet_name}");
            subnet_ids.push(subnet_id.clone());
            write!(&mut res, r#"
resource "aws_subnet" "{subnet_id}" {{
    provider = aws.{aws_region}

    vpc_id = aws_vpc.{vpc_name}.id
    cidr_block = "{subnet}"
    availability_zone = "{availability_zone}"{maybe_ipv6}
}}
"#).unwrap();

            if sdata.is_public {
                write!(&mut res, r#"
resource "aws_route_table_association" "igw-{subnet_id}" {{
    provider = aws.{aws_region}

    subnet_id = aws_subnet.{subnet_id}.id
    route_table_id = aws_route_table.internet-{vpc_name}.id
}}
"#).unwrap();
            }
        }

        if transit_gw_needed {
            // add route tables for transit gateways
            for peer_dc in topology.dcs.keys() {
                let peer_attrs = AwsDatacenterAttrs::new(checked, *peer_dc);
                let peer_vpc_name = &peer_attrs.vpc_name;
                let peer_cidr = checked.db.datacenter().c_network_cidr(*peer_dc);
                let peer_dc_region = checked.db.datacenter().c_region(*peer_dc);
                let peer_dc_name = checked.db.datacenter().c_dc_name(*peer_dc);
                if dc_region != peer_dc_region {
            write!(&mut res, r#"
resource "aws_route" "aig-route-{vpc_name}-{dc_name}-2-{peer_vpc_name}-{peer_dc_name}" {{
    provider = aws.{aws_region}

    route_table_id         = aws_route_table.internet-{vpc_name}.id
    destination_cidr_block = "{peer_cidr}"
    transit_gateway_id     = aws_ec2_transit_gateway.global.id
}}
"#).unwrap();
                }
            }
        }

        let maybe_depends_on_subnet = if more_than_one_subnet {
            format!("    depends_on = [ aws_subnet.{router_subnet_id} ]")
        } else { "".to_string() };

        let sk_prefix = "aws.";
        for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
            let server_kind = checked.projections.server_kinds.value(*server);
            let sk_name = checked.db.server_kind().c_kind(*server_kind);
            let ami_arch = checked.db.server_kind().c_architecture(*server_kind);
            let instance_type = &sk_name[sk_prefix.len()..sk_name.len()];
            let hostname = checked.db.server().c_hostname(*server);

            let lan_iface = checked.projections.consul_network_iface.value(*server);
            let lan_ip = checked.db.network_interface().c_if_ip(*lan_iface);
            let has_public_ip = checked.projections.internet_network_iface.get(&server).is_some();
            // enable routing
            let source_dest_check = !has_public_ip && !checked.db.server().c_is_router(*server);
            let net: ipnet::Ipv4Net = format!("{lan_ip}/24").parse().unwrap();
            let aws_subnet = net.network().to_string().replace(".", "_");
            let sg_id = if has_public_ip { &public_sg_name } else { &private_sg_name };
            let root_disk = checked.db.server().c_root_disk(*server);
            let disk_size = checked.projections.server_disk_sizes.get(&root_disk).unwrap();
            let disk_kind = checked.db.server_disk().c_disk_kind(root_disk);
            let disk_kind_name = checked.db.disk_kind().c_kind(disk_kind);
            let mut extra_disk_conf = String::new();
            if let Some(extra_conf) = topology.disk_configs.get(&root_disk) {
                if let Some(iops) = &extra_conf.provisioned_iops {
                    writeln!(&mut extra_disk_conf, "      iops = {iops}").unwrap();
                }
                if let Some(throughput) = &extra_conf.provisioned_throughput_mb {
                    writeln!(&mut extra_disk_conf, "      throughput = {throughput}").unwrap();
                }
            }
            assert!(disk_kind_name.starts_with("aws."), "We should have checked this earlier.");
            assert!(disk_size % 1024 * 1024 * 1024 == 0, "We should have checked this earlier.");
            let aws_vol_type = disk_kind_name.split(".").skip(1).next().unwrap();
            let disk_size_gb = disk_size / 1024 / 1024 / 1024;
            let maybe_public_ipv6_address = if has_public_ip && global_settings.enable_ipv6 {
                "
    ipv6_address_count = 1"
            } else { "" };

            write!(&mut res, r#"
resource "aws_instance" "{hostname}" {{
    provider = aws.{aws_region}

    ami = trimspace(data.local_file.ami_image_{ami_arch}_{aws_region}.content)
    source_dest_check = {source_dest_check}
    instance_type = "{instance_type}"
    associate_public_ip_address = {has_public_ip}{maybe_public_ipv6_address}

    tags = {{
        Name = "{project_name} - {hostname}"
    }}

    root_block_device {{
      tags = {{
          Name = "{project_name} - {hostname} - root"
      }}
      volume_size = {disk_size_gb}
      volume_type = "{aws_vol_type}"
{extra_disk_conf}
    }}

    private_ip = "{lan_ip}"
    subnet_id = aws_subnet.subnet_{aws_subnet}p24.id
    vpc_security_group_ids = [ aws_security_group.{sg_id}.id ]
{maybe_depends_on_subnet}
}}
"#).unwrap();

            for ni in checked.db.server().c_children_network_interface(*server) {
                let network_name = checked.db.network().c_network_name(checked.db.network_interface().c_if_network(*ni));
                // we made higher level checks that there's only one interface
                // like that
                if network_name == "dcrouter" {
                    let ip = checked.db.network_interface().c_if_ip(*ni);
                    write!(&mut res, r#"
resource "aws_network_interface" "{hostname}-dcrouter-interface" {{
    provider = aws.{aws_region}

    subnet_id         = aws_subnet.{router_subnet_id}.id
    private_ips       = ["{ip}"]
    security_groups   = [aws_security_group.{private_sg_name}.id]
    source_dest_check = false

    attachment {{
      instance     = aws_instance.{hostname}.id
      device_index = 1
    }}
}}
"#).unwrap();
                }
            }

            let mut disks = checked.db.server().c_children_server_disk(*server).to_vec();
            disks.sort_by_key(|d| {
                checked.db.server_disk().c_disk_id(*d)
            });
            for disk in &disks {
                if *disk == root_disk {
                    continue;
                }

                let disk_id = checked.db.server_disk().c_disk_id(*disk);
                let disk_size = *checked.projections.server_disk_sizes.get(disk).unwrap();
                let disk_kind = checked.db.server_disk().c_disk_kind(*disk);
                let disk_kind_name = checked.db.disk_kind().c_kind(disk_kind);
                assert!(disk_kind_name.starts_with("aws."), "We should have checked this earlier.");
                let aws_vol_type = disk_kind_name.split(".").skip(1).next().unwrap();
                let mut extra_disk_conf = String::new();
                if let Some(extra_conf) = topology.disk_configs.get(&disk) {
                    if let Some(iops) = &extra_conf.provisioned_iops {
                        writeln!(&mut extra_disk_conf, "    iops = {iops}").unwrap();
                    }
                    if let Some(throughput) = &extra_conf.provisioned_throughput_mb {
                        writeln!(&mut extra_disk_conf, "    throughput = {throughput}").unwrap();
                    }
                }

                assert!(disk_size % 1024 * 1024 * 1024 == 0, "Should have been checked earliet");
                let disk_size_gb = disk_size / 1024 / 1024 / 1024;

                write!(&mut res, r#"
resource "aws_ebs_volume" "{hostname}-disk-{disk_id}" {{
    provider = aws.{aws_region}

    availability_zone = "{availability_zone}"
    size = {disk_size_gb}
    type = "{aws_vol_type}"
{extra_disk_conf}
    tags = {{
      Name = "{project_name} - {hostname} - {disk_id}"
    }}
}}

resource "aws_volume_attachment" "{hostname}-disk-{disk_id}" {{
    provider = aws.{aws_region}

    device_name = "/dev/{disk_id}"
    volume_id = aws_ebs_volume.{hostname}-disk-{disk_id}.id
    instance_id = aws_instance.{hostname}.id
}}
"#).unwrap();
            }
        }
    }

    if transit_gw_needed {
        for (aws_region, subnet_ids) in &region_subnet_ids {
            let mut mapped: Vec<String> = subnet_ids.iter().map(|i| format!("aws_subnet.{i}.id")).collect();
            mapped.sort();
            let joined_ids = mapped.join(", ");
            write!(&mut res, r#"
resource "aws_ec2_transit_gateway_vpc_attachment" "aig-attachment-{aws_region}" {{
    provider = aws.{aws_region}

    subnet_ids         = [{joined_ids}]
    transit_gateway_id = aws_ec2_transit_gateway.global.id
    vpc_id             = aws_vpc.epl-vpc-{aws_region}.id
}}
"#).unwrap();
        }
    }

    res
}

fn extract_ipv6_subnet_from_cidr(inp: &str) -> u32 {
    let col = inp.split(".").collect::<Vec<_>>();
    // we need deterministic and dumb hash algorithm to fit into 256 bit value
    (257 * col[1].parse::<u32>().unwrap() + col[2].parse::<u32>().unwrap()) % 256
}

#[test]
fn test_extract_ipv6_subnet_from_cidr() {
    assert_eq!(extract_ipv6_subnet_from_cidr("10.1.2.0/24"), 3);
}
