{
  config,
  lib,
  pkgs,
  modulesPath,
  specialArgs,
  ...
}: {
  # for virtio kernel drivers
  imports = [
    "${toString modulesPath}/profiles/qemu-guest.nix"
  ];

  networking.hostId = "daa82e91";

  fileSystems."/" = {
    device = "rpool/root";
    fsType = "zfs";
  };

  boot.growPartition = true;
  boot.kernelParams = ["console=ttyS0"];
  boot.loader.grub.device =
    if (pkgs.stdenv.system == "x86_64-linux")
    then (lib.mkDefault "/dev/vda")
    else (lib.mkDefault "nodev");

  boot.loader.grub.efiSupport = lib.mkIf (pkgs.stdenv.system != "x86_64-linux") (lib.mkDefault true);
  boot.loader.grub.efiInstallAsRemovable = lib.mkIf (pkgs.stdenv.system != "x86_64-linux") (lib.mkDefault true);
  boot.loader.timeout = 0;

  system.build.qcow = import "${toString modulesPath}/../lib/make-single-disk-zfs-image.nix" {
    inherit lib config pkgs;
    format = "qcow2";
    datasets = {
      "rpool/root".mount = "/";
      "rpool/var".mount = "/var";
      "rpool/var/log".mount = "/var/log";
      "rpool/nix".mount = "/nix";
    };
    rootPoolName = "rpool";
    bootSize = 1024;
    rootSize = 4096;
    rootPoolProperties = {
      ashift = 12;
      autoexpand = "on";
    };
  };

  formatAttr = "qcow";
  fileExtension = ".qcow2";
}
