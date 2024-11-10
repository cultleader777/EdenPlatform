
variable "bucket_name" {
  type = string
  default = "tnzatmjnrpbw"
}

variable "project" {
  type = string
  default = "test1-406308"
}

provider "google" {
  project = var.project
  region = "us-west1"
  alias = "us-west1"
  zone = "us-west1-b"
}

resource "google_compute_network" "main-vpc" {
  project                 = var.project
  name                    = "main-vpc"
  auto_create_subnetworks = false
  routing_mode            = "GLOBAL"
}

resource "google_compute_firewall" "public-node" {
  project = var.project
  name    = "public-node"
  network = google_compute_network.main-vpc.name

  allow {
    protocol = "icmp"
  }

  allow {
    protocol = "udp"
    ports    = ["53", "51820"]
  }

  allow {
    protocol = "tcp"
    ports    = ["22", "80", "443", "53", "51820"]
  }

  source_ranges = ["0.0.0.0/0"]
}

resource "google_compute_firewall" "private-node-ingress" {
  project = var.project
  name    = "private-node"
  network = google_compute_network.main-vpc.name
  direction = "INGRESS"

  allow {
    protocol = "all"
  }

  source_ranges = ["10.0.0.0/8"]
  destination_ranges = ["10.0.0.0/8"]
}

resource "google_compute_firewall" "private-node-egress" {
  project = var.project
  name    = "private-node-egress"
  network = google_compute_network.main-vpc.name
  direction = "EGRESS"

  allow {
    protocol = "all"
  }

  source_ranges = ["10.0.0.0/8"]
  destination_ranges = ["10.0.0.0/8"]
}

resource "google_storage_bucket" "artefacts" {
  project       = var.project
  name          = var.bucket_name
  location      = "US"
  storage_class = "STANDARD"

  uniform_bucket_level_access = true
}

resource "google_storage_bucket_object" "gce-image-x86_64" {
  name         = "os-default-image-x86_64.tar.gz"
  source       = trimspace(file("image-x86_64/gcloud-image.txt"))
  content_type = "application/octet-stream"
  bucket       = google_storage_bucket.artefacts.id
}

resource "google_compute_image" "os-default-image-x86_64" {
  project = var.project
  name = "os-default-image-x86-64"
  raw_disk {
    source = google_storage_bucket_object.gce-image-x86_64.self_link
  }
  guest_os_features {
    type = "MULTI_IP_SUBNET"
  }
  guest_os_features {
    type = "GVNIC"
  }
}

resource "google_compute_subnetwork" "us-west1-b_10-17-0" {
  project       = "test1-406308"
  name          = "us-west1-b-10-17-0-subnet"
  ip_cidr_range = "10.17.0.0/24"
  network       = google_compute_network.main-vpc.self_link
  region        = "us-west1"
}

resource "google_compute_router" "us-west1-nat-router" {
  project = "test1-406308"
  name    = "us-west1-nat-router"
  region  = "us-west1"
  network = google_compute_network.main-vpc.id
}

resource "google_compute_router_nat" "us-west1-nat" {
  project                            = "test1-406308"
  name                               = "us-west1-router-nat"
  router                             = google_compute_router.us-west1-nat-router.name
  region                             = "us-west1"
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
}

resource "google_compute_instance" "server-a" {
  project        = var.project
  name           = "server-a"
  machine_type   = "e2-standard-4"
  zone           = "us-west1-b"
  can_ip_forward = false


  network_interface {
    nic_type = "GVNIC"
    subnetwork = google_compute_subnetwork.us-west1-b_10-17-0.self_link
    network_ip = "10.17.0.10"
  }


  boot_disk {
    device_name = "sda"
    initialize_params {
      image = google_compute_image.os-default-image-x86_64.id
      size = 20
      type = "pd-balanced"

    }
  }

  metadata = {
    # our initial ssh key was wired into vm image
    enable-oslogin = "FALSE"
    serial-port-enable = "TRUE"
  }

  lifecycle {
    ignore_changes = [attached_disk]
  }
}

resource "google_compute_instance" "server-b" {
  project        = var.project
  name           = "server-b"
  machine_type   = "e2-standard-4"
  zone           = "us-west1-b"
  can_ip_forward = false


  network_interface {
    nic_type = "GVNIC"
    subnetwork = google_compute_subnetwork.us-west1-b_10-17-0.self_link
    network_ip = "10.17.0.11"
  }


  boot_disk {
    device_name = "sda"
    initialize_params {
      image = google_compute_image.os-default-image-x86_64.id
      size = 20
      type = "pd-balanced"

    }
  }

  metadata = {
    # our initial ssh key was wired into vm image
    enable-oslogin = "FALSE"
    serial-port-enable = "TRUE"
  }

  lifecycle {
    ignore_changes = [attached_disk]
  }
}

resource "google_compute_address" "public-ip-server-c" {
  project = "test1-406308"
  name = "public-ip-server-c"
  region = "us-west1"
}

resource "google_compute_instance" "server-c" {
  project        = var.project
  name           = "server-c"
  machine_type   = "e2-standard-4"
  zone           = "us-west1-b"
  can_ip_forward = true


  network_interface {
    nic_type = "GVNIC"
    subnetwork = google_compute_subnetwork.us-west1-b_10-17-0.self_link
    network_ip = "10.17.0.12"
    access_config {
      nat_ip = google_compute_address.public-ip-server-c.address
    }
  }


  boot_disk {
    device_name = "sda"
    initialize_params {
      image = google_compute_image.os-default-image-x86_64.id
      size = 20
      type = "pd-balanced"

    }
  }

  metadata = {
    # our initial ssh key was wired into vm image
    enable-oslogin = "FALSE"
    serial-port-enable = "TRUE"
  }

  lifecycle {
    ignore_changes = [attached_disk]
  }
}

resource "google_compute_address" "public-ip-server-d" {
  project = "test1-406308"
  name = "public-ip-server-d"
  region = "us-west1"
}

resource "google_compute_instance" "server-d" {
  project        = var.project
  name           = "server-d"
  machine_type   = "e2-standard-4"
  zone           = "us-west1-b"
  can_ip_forward = true


  network_interface {
    nic_type = "GVNIC"
    subnetwork = google_compute_subnetwork.us-west1-b_10-17-0.self_link
    network_ip = "10.17.0.13"
    access_config {
      nat_ip = google_compute_address.public-ip-server-d.address
    }
  }


  boot_disk {
    device_name = "sda"
    initialize_params {
      image = google_compute_image.os-default-image-x86_64.id
      size = 20
      type = "pd-balanced"

    }
  }

  metadata = {
    # our initial ssh key was wired into vm image
    enable-oslogin = "FALSE"
    serial-port-enable = "TRUE"
  }

  lifecycle {
    ignore_changes = [attached_disk]
  }
}

resource "google_compute_disk" "extra_disk_server-d_disk_vda" {
  project = var.project

  name = "extra-disk-server-d-disk-vda"
  type = "pd-extreme"
  zone = "us-west1-b"
  size = 20

  physical_block_size_bytes = 4096
}

resource "google_compute_attached_disk" "extra_disk_server-d_disk_vda" {
  project = var.project

  device_name = "vda"
  disk = google_compute_disk.extra_disk_server-d_disk_vda.id
  instance = google_compute_instance.server-d.id
}
