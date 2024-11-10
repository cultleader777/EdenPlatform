
let
  pkgs = import (fetchTarball{ url = "https://github.com/NixOS/nixpkgs/archive/057f9aecfb71c4437d2b27d3323df7f93c010b7e.tar.gz"; sha256 = "1ndiv385w1qyb3b18vw13991fzb9wg4cl21wglk89grsfsnra41k"; }) {};
  lib = pkgs.lib;
in

{


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


  nix.settings = {
    substituters = [
      "http://10.17.0.1:12777/"

      "https://cache.nixos.org/"
    ];

    trusted-public-keys = [
      "epl-nix-cache:J1ra9/yUStIqnlqFf4FjS/pd9zwBW4bo+WP0FoBq72M="

    ];
  };


  # bare minimum packages for fast bootstrap l1 provisioning
  environment.systemPackages = with pkgs; [
    gzip
    sqlite
    tmux
    git
    procmail # lockfile command for l1
  ];

  services.sshd.enable = true;
  users.users.root.hashedPassword = "!";
  security.sudo.wheelNeedsPassword = false;
  users.users.admin = {
    isNormalUser = true;
    home = "/home/admin";
    extraGroups = [ "wheel" ];
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJfLsQZrFopCBQ0+md0XHgVAhz3i7tRJ+RL7ILH29IaB epl-root-ssh-key"

    ];
  };
  services.openssh.settings.PermitRootLogin = "prohibit-password";
  networking.usePredictableInterfaceNames = false;
  system.stateVersion = "23.11";
}
