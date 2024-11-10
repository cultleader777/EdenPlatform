use std::collections::BTreeSet;
use std::fmt::Write;

use crate::{static_analysis::{CheckedDB, networking::first_three_octets, get_global_settings}, codegen::Directory};

pub fn generate_gcloud_outputs(checked: &CheckedDB, dir: &mut Directory) {
    dir.create_file("main.tf", generate_main_tf_file(checked));
}

fn generate_main_tf_file(checked: &CheckedDB) -> String {
    let topology = &checked.projections.cloud_topologies.gcloud;
    let global_settings = get_global_settings(&checked.db);

    let mut res = String::new();

    let mut unique_regions: BTreeSet<String> = BTreeSet::new();
    for top in topology.dcs.values() {
        let _ = unique_regions.insert(top.region.clone());
    }

    let first_dc = topology.dcs.values().next().unwrap();
    let first_region = &first_dc.region;
    let first_az = &first_dc.availability_zone;
    let project = &global_settings.google_cloud_project_id;
    let bucket_name = &global_settings.google_cloud_artefacts_bucket_name;

    write!(&mut res, r#"
variable "bucket_name" {{
  type = string
  default = "{bucket_name}"
}}

variable "project" {{
  type = string
  default = "{project}"
}}

provider "google" {{
  project = var.project
  region = "{first_region}"
  alias = "{first_region}"
  zone = "{first_az}"
}}

resource "google_compute_network" "main-vpc" {{
  project                 = var.project
  name                    = "main-vpc"
  auto_create_subnetworks = false
  routing_mode            = "GLOBAL"
}}

resource "google_compute_firewall" "public-node" {{
  project = var.project
  name    = "public-node"
  network = google_compute_network.main-vpc.name

  allow {{
    protocol = "icmp"
  }}

  allow {{
    protocol = "udp"
    ports    = ["53", "51820"]
  }}

  allow {{
    protocol = "tcp"
    ports    = ["22", "80", "443", "53", "51820"]
  }}

  source_ranges = ["0.0.0.0/0"]
}}

resource "google_compute_firewall" "private-node-ingress" {{
  project = var.project
  name    = "private-node"
  network = google_compute_network.main-vpc.name
  direction = "INGRESS"

  allow {{
    protocol = "all"
  }}

  source_ranges = ["10.0.0.0/8"]
  destination_ranges = ["10.0.0.0/8"]
}}

resource "google_compute_firewall" "private-node-egress" {{
  project = var.project
  name    = "private-node-egress"
  network = google_compute_network.main-vpc.name
  direction = "EGRESS"

  allow {{
    protocol = "all"
  }}

  source_ranges = ["10.0.0.0/8"]
  destination_ranges = ["10.0.0.0/8"]
}}

resource "google_storage_bucket" "artefacts" {{
  project       = var.project
  name          = var.bucket_name
  location      = "US"
  storage_class = "STANDARD"

  uniform_bucket_level_access = true
}}
"#).unwrap();

    for ua in &checked.projections.used_architectures {
        let ua_repl = ua.replace("_", "-");
        write!(&mut res, r#"
resource "google_storage_bucket_object" "gce-image-{ua}" {{
  name         = "os-default-image-{ua}.tar.gz"
  source       = trimspace(file("image-{ua}/gcloud-image.txt"))
  content_type = "application/octet-stream"
  bucket       = google_storage_bucket.artefacts.id
}}

resource "google_compute_image" "os-default-image-{ua}" {{
  project = var.project
  name = "os-default-image-{ua_repl}"
  raw_disk {{
    source = google_storage_bucket_object.gce-image-{ua}.self_link
  }}
  guest_os_features {{
    type = "MULTI_IP_SUBNET"
  }}
  guest_os_features {{
    type = "GVNIC"
  }}
}}
"#).unwrap();
    }

    let mut region_routers_covered: BTreeSet<String> = BTreeSet::new();
    // we create subnetwork per dc
    for (dc, top) in &topology.dcs {
        let region = &top.region;
        let zone = &top.availability_zone;

        for subnet_key in top.subnet_map.keys() {
            let sk_repl = subnet_key.replace(".", "-");
            write!(&mut res, r#"
resource "google_compute_subnetwork" "{zone}_{sk_repl}" {{
  project       = "{project}"
  name          = "{zone}-{sk_repl}-subnet"
  ip_cidr_range = "{subnet_key}.0/24"
  network       = google_compute_network.main-vpc.self_link
  region        = "{region}"
}}
"#).unwrap();
        }

        if region_routers_covered.insert(region.clone()) {
            write!(&mut res, r#"
resource "google_compute_router" "{region}-nat-router" {{
  project = "{project}"
  name    = "{region}-nat-router"
  region  = "{region}"
  network = google_compute_network.main-vpc.id
}}

resource "google_compute_router_nat" "{region}-nat" {{
  project                            = "{project}"
  name                               = "{region}-router-nat"
  router                             = google_compute_router.{region}-nat-router.name
  region                             = "{region}"
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
}}
"#).unwrap();
        }

        let sk_prefix = "gcloud.";
        for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
            let server_kind = checked.projections.server_kinds.value(*server);
            let architecture = checked.db.server_kind().c_architecture(*server_kind);
            let sk_name = checked.db.server_kind().c_kind(*server_kind);
            let instance_type = &sk_name[sk_prefix.len()..sk_name.len()];
            let hostname = checked.db.server().c_hostname(*server);
            let nic_type = "GVNIC";
            let root_disk = checked.db.server().c_root_disk(*server);
            let root_disk_id = checked.db.server_disk().c_disk_id(root_disk);
            let root_disk_kind = checked.db.server_disk().c_disk_kind(root_disk);
            let root_disk_kind_name = checked.db.disk_kind().c_kind(root_disk_kind);
            let root_disk_size = checked.projections.server_disk_sizes.get(&root_disk).unwrap();
            assert_eq!(root_disk_size % 1024 * 1024 * 1024, 0, "We should have checked this earlier");
            let root_disk_size_gb = root_disk_size / 1024 / 1024 / 1024;
            let disk_kind_prefix = "gcloud.";
            assert!(root_disk_kind_name.starts_with(disk_kind_prefix), "We should have checked this earlier");
            let final_root_disk_kind = &root_disk_kind_name[disk_kind_prefix.len()..];
            let mut extra_disk_conf = String::new();
            if let Some(extra_conf) = topology.disk_configs.get(&root_disk) {
                if let Some(iops) = &extra_conf.provisioned_iops {
                    writeln!(&mut extra_disk_conf, "  provisioned_iops = {iops}").unwrap();
                }
                if let Some(throughput) = &extra_conf.provisioned_throughput_mb {
                    writeln!(&mut extra_disk_conf, "  provisioned_throughput = {throughput}").unwrap();
                }
            }

            let lan_iface = checked.projections.consul_network_iface.value(*server);
            let lan_ip = checked.db.network_interface().c_if_ip(*lan_iface);
            let subnet_key = first_three_octets(&lan_ip).replace(".", "-");
            let has_public_ip = checked.projections.internet_network_iface.get(&server).is_some();
            let can_ip_forward = checked.db.server().c_is_vpn_gateway(*server);
            let maybe_public_ip = if has_public_ip {
                write!(&mut res, r#"
resource "google_compute_address" "public-ip-{hostname}" {{
  project = "{project}"
  name = "public-ip-{hostname}"
  region = "{region}"
}}
"#).unwrap();
                format!(r#"
  network_interface {{
    nic_type = "{nic_type}"
    subnetwork = google_compute_subnetwork.{zone}_{subnet_key}.self_link
    network_ip = "{lan_ip}"
    access_config {{
      nat_ip = google_compute_address.public-ip-{hostname}.address
    }}
  }}
"#)
            } else {
                format!(r#"
  network_interface {{
    nic_type = "{nic_type}"
    subnetwork = google_compute_subnetwork.{zone}_{subnet_key}.self_link
    network_ip = "{lan_ip}"
  }}
"#)
            };

            // enable routing
            write!(&mut res, r#"
resource "google_compute_instance" "{hostname}" {{
  project        = var.project
  name           = "{hostname}"
  machine_type   = "{instance_type}"
  zone           = "{zone}"
  can_ip_forward = {can_ip_forward}

{maybe_public_ip}

  boot_disk {{
    device_name = "{root_disk_id}"
    initialize_params {{
      image = google_compute_image.os-default-image-{architecture}.id
      size = {root_disk_size_gb}
      type = "{final_root_disk_kind}"
{extra_disk_conf}
    }}
  }}

  metadata = {{
    # our initial ssh key was wired into vm image
    enable-oslogin = "FALSE"
    serial-port-enable = "TRUE"
  }}

  lifecycle {{
    ignore_changes = [attached_disk]
  }}
}}
"#).unwrap();

            let mut disks = checked.db.server().c_children_server_disk(*server).to_vec();
            disks.sort_by_key(|d| {
                checked.db.server_disk().c_disk_id(*d)
            });
            for server_disk in &disks {
                if *server_disk == root_disk {
                    continue;
                }

                let disk_id = checked.db.server_disk().c_disk_id(*server_disk);
                let disk_kind = checked.db.server_disk().c_disk_kind(*server_disk);
                let disk_kind_name = checked.db.disk_kind().c_kind(disk_kind);
                assert!(disk_kind_name.starts_with(disk_kind_prefix), "We should have checked this earlier");
                let final_disk_kind = &disk_kind_name[disk_kind_prefix.len()..];
                let disk_size = *checked.projections.server_disk_sizes.get(server_disk).unwrap();
                assert_eq!(disk_size % 1024 * 1024 * 1024, 0, "We should have checked this earlier");
                let disk_size_gb = disk_size / 1024 / 1024 / 1024;

                let mut extra_disk_conf = String::new();
                if let Some(extra_conf) = topology.disk_configs.get(&server_disk) {
                    if let Some(iops) = &extra_conf.provisioned_iops {
                        writeln!(&mut extra_disk_conf, "  provisioned_iops = {iops}").unwrap();
                    }
                    if let Some(throughput) = &extra_conf.provisioned_throughput_mb {
                        writeln!(&mut extra_disk_conf, "  provisioned_throughput = {throughput}").unwrap();
                    }
                }

                write!(&mut res, r#"
resource "google_compute_disk" "extra_disk_{hostname}_disk_{disk_id}" {{
  project = var.project

  name = "extra-disk-{hostname}-disk-{disk_id}"
  type = "{final_disk_kind}"
  zone = "{zone}"
  size = {disk_size_gb}
{extra_disk_conf}
  physical_block_size_bytes = 4096
}}

resource "google_compute_attached_disk" "extra_disk_{hostname}_disk_{disk_id}" {{
  project = var.project

  device_name = "{disk_id}"
  disk = google_compute_disk.extra_disk_{hostname}_disk_{disk_id}.id
  instance = google_compute_instance.{hostname}.id
}}
"#).unwrap();
            }
        }
    }

    res
}
