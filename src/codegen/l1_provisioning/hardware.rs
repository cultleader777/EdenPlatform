use crate::{codegen::{nixplan::{NixAllServerPlans, NixServerPlan}, makefile::vms_exist, l1_provisioning::routing::get_hetnzer_internal_dc_vlan_ip}, static_analysis::{CheckedDB, networking::first_three_octets, get_global_settings}, database::TableRowPointerServer};

pub fn provision_spec_hardware(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    let zfs_args_qemu = zfs_args_qemu();
    let zfs_args_gcloud = zfs_args_gcloud();
    let zfs_args_aws = zfs_args_aws();
    let vms_exist = vms_exist(db);
    let settings = get_global_settings(&db.db);
    for server in db.db.server().rows_iter() {
        let dc = db.db.server().c_dc(server);
        let dc_net = db.sync_res.network.networking_answers.dcs.get(&dc).unwrap();
        let dc_routers = dc_net.all_routers_set();
        let plan = plans.fetch_plan(server);
        plan.add_custom_nix_block("# NIX REGION custom_hardware START".to_string());

        let add_internet_lan_iface_default_gateway = |plans: &mut NixAllServerPlans| {
            let plan = plans.fetch_plan(server);
            let lan_iface = db.projections.consul_network_iface.value(server);
            let lan_iface_name = db.db.network_interface().c_if_name(*lan_iface);
            let lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
            let lan_first_gw = format!("{}.1", first_three_octets(&lan_ip));
            plan.add_interface_route(
                lan_iface_name,
                format!(r#"{{ address = "0.0.0.0"; prefixLength = 0; via = "{lan_first_gw}"; }}"#)
            );
        };
        let add_aws_dclocal_default_gateway_route = |plans: &mut NixAllServerPlans| {
            let plan = plans.fetch_plan(server);
            let lan_iface = db.projections.consul_network_iface.value(server);
            let lan_iface_name = db.db.network_interface().c_if_name(*lan_iface);
            let lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
            let lan_first_gw = format!("{}.1", first_three_octets(&lan_ip));
            // Intra AWS/gcloud datacenters are connected by transit gateway
            for aws_dc in db.projections.cloud_topologies.aws.dcs.keys() {
                let dc_cidr = db.db.datacenter().c_network_cidr(*aws_dc).split("/").collect::<Vec<_>>();
                let dc_subnet = dc_cidr[0];
                let dc_prefix = dc_cidr[1];
                plan.add_interface_route(
                    lan_iface_name,
                    format!(r#"{{ address = "{dc_subnet}"; prefixLength = {dc_prefix}; via = "{lan_first_gw}"; }}"#)
                );
            }
        };
        let add_aws_magic_route = |plans: &mut NixAllServerPlans| {
            let plan = plans.fetch_plan(server);
            let lan_iface = db.projections.consul_network_iface.value(server);
            let lan_iface_name = db.db.network_interface().c_if_name(*lan_iface);
            let lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
            let lan_first_gw = format!("{}.1", first_three_octets(&lan_ip));
            // Intra AWS/gcloud datacenters are connected by transit gateway
            plan.add_interface_route(
                lan_iface_name,
                format!(r#"{{ address = "169.254.169.254"; prefixLength = 32; via = "{lan_first_gw}"; }}"#)
            );
        };
        let dc_impl = db.db.datacenter().c_implementation(dc).as_str();
        match dc_impl {
            "manual" => {
                // think of what to do here
            },
            "hetzner" => {
                let plan = plans.fetch_plan(server);
                add_default_internet_route(db, plan, server);
                hetzner_hardware_config(db, plan, server);
            }
            "bm_simple" => {
                // add gateway ip
                let plan = plans.fetch_plan(server);
                let bm_simple_info = db.projections.bm_topologies.bm_simple.get(&dc).unwrap();
                let gw_ip = &bm_simple_info.gateway_ip;
                let lan_iface = db.projections.consul_network_iface.value(server);
                let lan_iface_name = db.db.network_interface().c_if_name(*lan_iface);
                plan.add_interface_route(
                    lan_iface_name,
                    format!(r#"{{ address = "0.0.0.0"; prefixLength = 0; via = "{gw_ip}"; }}"#)
                );

                auto_hardware_config(plan, true);
            }
            "gcloud" => {
                add_internet_lan_iface_default_gateway(plans);

                let plan = plans.fetch_plan(server);
                plan.add_custom_nix_block(format!(r#"
    imports = [ "${{modulesPath}}/virtualisation/google-compute-image.nix" ];

{zfs_args_gcloud}

    networking.usePredictableInterfaceNames = false;
"#));
            }
            "aws" => {
                add_aws_magic_route(plans);
                // if ospf is needed then this will be added anyway on the router
                // we only need this when we have no routing, and node has internet
                if !dc_net.is_ospf_routing_needed || !dc_routers.contains(&server) {
                    if db.projections.internet_network_iface.contains_key(&server) {
                        add_internet_lan_iface_default_gateway(plans);
                    }
                }

                // router nodes already have such static routes
                if !db.db.server().c_is_vpn_gateway(server) && !db.db.server().c_is_router(server) {
                    add_aws_dclocal_default_gateway_route(plans);
                }

                // TODO: how to support ipv6 in all implementations? pick interface automatically, next to internet?
                let maybe_ipv6_block =
                    if settings.enable_ipv6 {
                        if let Some(_public_ipv6) = db.sync_res.network.node_public_ipv6_addrs.get(&server) {
                            let ipv6_addr = db.db.server().c_public_ipv6_address(server);
                            let prefix = db.db.server().c_public_ipv6_address_prefix(server);
                            let lan_iface = db.projections.consul_network_iface.value(server);
                            let lan_iface_name = db.db.network_interface().c_if_name(*lan_iface);
                            format!(r#"
    networking.interfaces.{lan_iface_name}.ipv6.addresses = [
      {{ address = "{ipv6_addr}"; prefixLength = {prefix}; }}
    ];
"#)
                        } else { "".to_string() }
                    } else { "".to_string() };

                let plan = plans.fetch_plan(server);
                plan.add_nix_package("parted");
                plan.add_custom_nix_block(format!(r#"
    imports = [ "${{modulesPath}}/virtualisation/amazon-image.nix" ];

{zfs_args_aws}{maybe_ipv6_block}

    networking.usePredictableInterfaceNames = false;
"#));
            },
            "coprocessor" => {
                let plan = plans.fetch_plan(server);

                add_default_internet_route(db, plan, server);

                if !vms_exist {
                    auto_hardware_config(plan, false);
                } else {
                    let lan_iface = db.projections.consul_network_iface.value(server);
                    let lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
                    let lan_iface_name = db.db.network_interface().c_if_name(*lan_iface);
                    let lan_ip_subnet: ipnet::Ipv4Net =
                        format!("{}.0/24", first_three_octets(&lan_ip)).parse().unwrap();
                    let first_gw = lan_ip_subnet.hosts().next().unwrap().to_string();
                    if let Some(docker_registry_gw_ip) = &db.sync_res.network.test_docker_registry_gw_address {
                        if &first_gw != docker_registry_gw_ip {
                            let internet_iface = db.projections.internet_network_iface.get(&server).unwrap();
                            let internet_ip = db.db.network_interface().c_if_ip(*internet_iface);
                            let internet_ip_subnet: ipnet::Ipv4Net =
                                format!("{}.0/24", first_three_octets(&internet_ip)).parse().unwrap();
                            let first_gw = internet_ip_subnet.hosts().next().unwrap().to_string();
                            plan.add_interface_route(
                                &lan_iface_name,
                                format!(r#"{{ address = "{docker_registry_gw_ip}"; prefixLength = 32; via = "{first_gw}"; }}"#)
                            );
                        }
                    }

                    plan.add_custom_nix_block(format!(r#"
    imports =
      [ "${{modulesPath}}/profiles/qemu-guest.nix" ];

{zfs_args_qemu}

    boot.initrd.availableKernelModules = [ "zfs" "ahci" "xhci_pci" "virtio_pci" "virtio_blk" "virtio_console" ];
    boot.initrd.kernelModules = [ ];
    boot.kernelModules = [ "zfs" "kvm-amd" ];
    boot.kernelParams = [ "console=ttyS0,115200n8" ];
    boot.extraModulePackages = [ ];
    boot.loader.grub.extraConfig = "
      serial --speed=115200 --unit=0 --word=8 --parity=no --stop=1
      terminal_input serial
      terminal_output serial
    ";

    hardware.cpu.amd.updateMicrocode = lib.mkDefault true;
"#));
                }
            },
            "testvms" => {
                let plan = plans.fetch_plan(server);
                let lan_iface = db.projections.consul_network_iface.value(server);
                let lan_ip = db.db.network_interface().c_if_ip(*lan_iface);
                let lan_iface_name = db.db.network_interface().c_if_name(*lan_iface);
                let lan_ip_subnet: ipnet::Ipv4Net =
                    format!("{}.0/24", first_three_octets(&lan_ip)).parse().unwrap();
                let first_gw = lan_ip_subnet.hosts().next().unwrap().to_string();
                if let Some(docker_registry_gw_ip) = &db.sync_res.network.test_docker_registry_gw_address {
                    if &first_gw != docker_registry_gw_ip {
                        plan.add_interface_route(
                            &lan_iface_name,
                            format!(r#"{{ address = "{docker_registry_gw_ip}"; prefixLength = 32; via = "{first_gw}"; }}"#)
                        );
                    }
                }

                add_default_internet_route(db, plan, server);

                // without these virsh console doesnt work to debug
                let serial_params = r#"
    boot.kernelParams = [ "console=ttyS0,115200n8" ];
    boot.loader.grub.extraConfig = "
      serial --speed=115200 --unit=0 --word=8 --parity=no --stop=1
      terminal_input serial
      terminal_output serial
    ";
"#;

                // for testing
                let test_auto_config = false;

                if test_auto_config {
                    auto_hardware_config(plan, false);
                    plan.add_custom_nix_block(serial_params.to_string());
                } else {
                    plan.add_custom_nix_block(format!(r#"
    imports =
      [ "${{modulesPath}}/profiles/qemu-guest.nix" ];

{zfs_args_qemu}

    boot.initrd.availableKernelModules = [ "zfs" "ahci" "xhci_pci" "virtio_pci" "virtio_blk" "virtio_console" ];
    boot.initrd.kernelModules = [ ];
    boot.kernelModules = [ "zfs" "kvm-amd" ];
    boot.extraModulePackages = [ ];
{serial_params}

    hardware.cpu.amd.updateMicrocode = lib.mkDefault true;
"#));

                }
            },
            other => {
                panic!("Unexpected datacenter implementation: {other}")
            }
        }

        let plan = plans.fetch_plan(server);
        plan.add_custom_nix_block("# NIX REGION custom_hardware END".to_string());
    }
}

fn auto_hardware_config(plan: &mut NixServerPlan, efi_enabled: bool) {
    plan.add_pre_l1_provisioning_shell_hook("nixos-generate-config --show-hardware-config --no-filesystems | grep -v -e ' networking.' -e ' # ' > /etc/nixos/hardware-configuration.nix".to_string());
    let zfs_args_bm = zfs_args_bm();

    let boot_loader =
        if efi_enabled {
            r#"
  boot.loader = {
    efi = {
      canTouchEfiVariables = false;
    };
    grub = {
      enable = true;
      device = "nodev";
      efiSupport = true;
      zfsSupport = true;
      efiInstallAsRemovable = true;
    };
  };
"#
        } else {
            r#"
  boot.loader.grub = {
    enable = true;
    zfsSupport = true;
    efiSupport = false;
    efiInstallAsRemovable = false;
    mirroredBoots = [
      { devices = [ "nodev" ]; path = "/boot"; }
    ];
  };
"#
        };

        plan.add_custom_nix_block(format!(r#"
  imports = [ ./hardware-configuration.nix ];
  networking.usePredictableInterfaceNames = false;

  boot.zfs.devNodes = "/dev/disk/by-label/rpool";
  services.zfs.expandOnBoot = "all";

{boot_loader}

{zfs_args_bm}
"#));
}

fn hetzner_hardware_config(db: &CheckedDB, plan: &mut NixServerPlan, server: TableRowPointerServer) {
    plan.add_pre_l1_provisioning_shell_hook("nixos-generate-config --show-hardware-config --no-filesystems | grep -v -e ' networking.' -e ' # ' > /etc/nixos/hardware-configuration.nix".to_string());
    let root_disk = db.db.server_disk().c_disk_id(db.db.server().c_root_disk(server));
    let zfs_args_bm = zfs_args_bm();

    plan.add_custom_nix_block(format!(r#"
  imports = [ ./hardware-configuration.nix ];
  networking.usePredictableInterfaceNames = false;

  boot.loader.grub = {{
    enable = true;
    zfsSupport = true;
    efiSupport = false;
    efiInstallAsRemovable = false;
  }};
  boot.loader.grub.devices = [
    "/dev/disk/by-id/{root_disk}"
  ];

{zfs_args_bm}
"#));

    if db.db.server().c_public_ipv6_address(server) != "" {
        let address = db.db.server().c_public_ipv6_address(server);
        let prefix = db.db.server().c_public_ipv6_address_prefix(server);
        let internet_iface = db.projections.internet_network_iface.get(&server).unwrap();
        let internet_iface_name = db.db.network_interface().c_if_name(*internet_iface);

        plan.add_custom_nix_block(format!(r#"
    networking.interfaces.eth0.ipv6.addresses = [
      {{ address = "{address}";
         prefixLength = {prefix}; }}
    ];

    networking.defaultGateway6 = {{
      address = "fe80::1";
      interface = "{internet_iface_name}";
    }};
"#));
    }

    // provision VLAN for inter hetzner DC communication
    if db.db.server().c_is_vpn_gateway(server) {
        let gobal_conf = get_global_settings(&db.db);
        let vlan_id = gobal_conf.hetzner_inter_dc_vlan_id;
        assert!(vlan_id >= 4000, "that's only what hetzner allows, should be caught earlier");

        let internet_iface = db.projections.internet_network_iface.get(&server).unwrap();
        let internet_iface_name = db.db.network_interface().c_if_name(*internet_iface);
        let vpn_iface = db.projections.vpn_network_iface.get(&server).unwrap();
        let vpn_ip = db.db.network_interface().c_if_ip(*vpn_iface);
        // use VPN address last two octets but replace start with 172.42
        let iface_ip = get_hetnzer_internal_dc_vlan_ip(vpn_ip);

        plan.add_custom_nix_block(format!(r#"
    networking.vlans.vlan{vlan_id} = {{ id = {vlan_id}; interface = "{internet_iface_name}"; }};
    networking.interfaces.vlan{vlan_id}.ipv4.addresses = [
      {{ address = "{iface_ip}";
         prefixLength = 16; }}
    ];
"#));
    }
}

// try first host address in the subnet
fn add_default_internet_route(db: &CheckedDB, plan: &mut NixServerPlan, server: TableRowPointerServer) {
    if let Some(internet_iface) = db.projections.internet_network_iface.get(&server) {
        let internet_iface_name = db.db.network_interface().c_if_name(*internet_iface);
        let internet_ip = db.db.network_interface().c_if_ip(*internet_iface);
        let internet_cidr = db.db.network_interface().c_if_prefix(*internet_iface);
        let net: ipnet::Ipv4Net = format!("{internet_ip}/{internet_cidr}").parse().unwrap();
        let first_internet_gw = net.hosts().next().unwrap();
        plan.add_interface_route(
            internet_iface_name,
            format!(r#"{{ address = "0.0.0.0"; prefixLength = 0; via = "{first_internet_gw}"; }}"#)
        );
    }
}

pub fn zfs_args_qemu() -> &'static str {
    r#"
  boot.zfs.devNodes = "/dev/disk/by-label/rpool";
  services.zfs.expandOnBoot = "all";

  boot.loader.grub = {
    enable = true;
    zfsSupport = true;
    efiSupport = false;
    efiInstallAsRemovable = false;
    mirroredBoots = [
      { devices = [ "nodev"]; path = "/boot"; }
    ];
  };

  fileSystems."/" =
    { device = "rpool/root";
      fsType = "zfs";
    };

  fileSystems."/nix" =
    { device = "rpool/nix";
      fsType = "zfs";
    };

  fileSystems."/var" =
    { device = "rpool/var";
      fsType = "zfs";
    };

  fileSystems."/var/log" =
    { device = "rpool/var/log";
      fsType = "zfs";
    };

  fileSystems."/boot" =
    { device = "/dev/vda2";
      fsType = "vfat";
    };
"#
}

pub fn zfs_args_bm() -> &'static str {
    r#"
  fileSystems."/" =
    { device = "rpool/root";
      fsType = "zfs";
    };

  fileSystems."/nix" =
    { device = "rpool/nix";
      fsType = "zfs";
    };

  fileSystems."/var" =
    { device = "rpool/var";
      fsType = "zfs";
    };

  fileSystems."/var/log" =
    { device = "rpool/var/log";
      fsType = "zfs";
    };

  fileSystems."/boot" = {
    # The ZFS image uses a partition labeled ESP whether or not we're
    # booting with EFI.
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };
"#
}

pub fn zfs_args_aws() -> &'static str {
    r#"
  ec2.zfs.enable = true;

  fileSystems."/" =
    { device = "rpool/root";
      fsType = "zfs";
    };

  fileSystems."/nix" =
    { device = "rpool/nix";
      fsType = "zfs";
    };

  fileSystems."/var" =
    { device = "rpool/var";
      fsType = "zfs";
    };

  fileSystems."/var/log" =
    { device = "rpool/var/log";
      fsType = "zfs";
    };

  fileSystems."/boot" = {
    # The ZFS image uses a partition labeled ESP whether or not we're
    # booting with EFI.
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };
"#
}

pub fn zfs_args_gcloud() -> &'static str {
    r#"
  services.zfs.expandOnBoot = "all";

  fileSystems."/" =
    # force because google-compute-config.nix makes it ext4
    pkgs.lib.mkForce
    { device = "rpool/root";
      fsType = "zfs";
    };

  fileSystems."/nix" =
    { device = "rpool/nix";
      fsType = "zfs";
    };

  fileSystems."/var" =
    { device = "rpool/var";
      fsType = "zfs";
    };

  fileSystems."/var/log" =
    { device = "rpool/var/log";
      fsType = "zfs";
    };

  fileSystems."/boot" = {
    # The ZFS image uses a partition labeled ESP whether or not we're
    # booting with EFI.
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };
"#
}
