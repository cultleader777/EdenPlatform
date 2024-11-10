{
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}:

with lib;

let
  cfg = {
    name = "nixos-amazon-image-${config.system.nixos.label}-${pkgs.stdenv.hostPlatform.system}";
    format = "vpc";
  };
  amiBootMode = if config.ec2.efi then "uefi" else "legacy-bios";
  configFile = pkgs.writeText "configuration.nix"
    ''
        { modulesPath, ... }: {
          imports = [ "''${modulesPath}/virtualisation/amazon-image.nix" ];
          ${optionalString config.ec2.efi ''
            ec2.efi = true;
          ''}
          ${optionalString config.ec2.zfs.enable ''
            ec2.zfs.enable = true;
            networking.hostId = "${config.networking.hostId}";
          ''}
        }
      '';
in {
  # for virtio kernel drivers
  imports = [
    "${toString modulesPath}/virtualisation/amazon-image.nix"
  ];

  networking.hostId = "daa82e91";

  ec2.zfs.enable = true;
  formatAttr = "amazonImage";
  fileExtension = ".vhd";

  system.build.amazonImage = import "${toString modulesPath}/../lib/make-single-disk-zfs-image.nix" {
    inherit lib config configFile;
    inherit (cfg) contents;
    pkgs = pkgs;

    includeChannel = true;
    rootPoolName = "rpool";

    bootSize = 1000; # 1G is the minimum EBS volume
    rootSize = 3000;
    rootPoolProperties = {
      ashift = 12;
      autoexpand = "on";
    };
    format = "vpc";

    datasets = {
      "rpool/root".mount = "/";
      "rpool/var".mount = "/var";
      "rpool/var/log".mount = "/var/log";
      "rpool/nix".mount = "/nix";
    };

    postVM = ''
        extension=''${rootDiskImage##*.}
        friendlyName=$out/${cfg.name}
        rootDisk="$friendlyName.root.$extension"
        mv "$rootDiskImage" "$rootDisk"

        mkdir -p $out/nix-support
        echo "file ${cfg.format} $rootDisk" >> $out/nix-support/hydra-build-products

       ${pkgs.jq}/bin/jq -n \
         --arg system_label ${lib.escapeShellArg config.system.nixos.label} \
         --arg system ${lib.escapeShellArg pkgs.stdenv.hostPlatform.system} \
         --arg root_logical_bytes "$(${pkgs.qemu_kvm}/bin/qemu-img info --output json "$rootDisk" | ${pkgs.jq}/bin/jq '."virtual-size"')" \
         --arg boot_mode "${amiBootMode}" \
         --arg root "$rootDisk" \
        '{}
          | .label = $system_label
          | .boot_mode = $boot_mode
          | .system = $system
          | .disks.root.logical_bytes = $root_logical_bytes
          | .disks.root.file = $root
          ' > $out/nix-support/image-info.json
      '';
  };
}
