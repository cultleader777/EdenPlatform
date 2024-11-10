{
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}:

with lib;

let
  defaultConfigFile = pkgs.writeText "configuration.nix" ''
    { ... }:
    {
      imports = [
        <nixpkgs/nixos/modules/virtualisation/google-compute-image.nix>
      ];
    }
  '';
  defaultConfig = import "${toString modulesPath}/virtualisation/google-compute-config.nix" {};
in {
  # for virtio kernel drivers
  imports = [
    "${toString modulesPath}/virtualisation/google-compute-config.nix"
  ];

  networking.hostId = "daa82e91";
  boot.initrd.availableKernelModules = [ "nvme" ];
  boot.growPartition = true;

  formatAttr = "googleComputeImage";

  system.build.googleComputeImage = import "${toString modulesPath}/../lib/make-single-disk-zfs-image.nix" {
    bootSize = 1000; # 1G is the minimum EBS volume
    rootSize = 3000;
    rootPoolProperties = {
      ashift = 12;
      autoexpand = "on";
    };
    format = "raw";
    rootPoolName = "rpool";

    includeChannel = true;

    datasets = {
      "rpool/root".mount = "/";
      "rpool/var".mount = "/var";
      "rpool/var/log".mount = "/var/log";
      "rpool/nix".mount = "/nix";
    };

    name = "google-compute-image";
    postVM = ''
        PATH=$PATH:${with pkgs; lib.makeBinPath [ gnutar gzip ]}
        pushd $out
        mv "$rootDiskImage" disk.raw
        tar -Sc disk.raw | gzip -9 > \
          nixos-image-${config.system.nixos.label}-${pkgs.stdenv.hostPlatform.system}.raw.tar.gz
        rm $out/disk.raw
        popd
      '';
    configFile = defaultConfigFile;
    inherit config lib pkgs;
  };
}
