
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
      "epl-nix-cache:gi0dT5g3kUGLkTpP8WCgg79QSfAQ1yS7Ab5txcN5yag="

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
    "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC3AkuDzzPrMaDav0kN7PIoaBU1Vtw1TfkHxWzPMrleocCltYl8TljwCqEJtmizx5DGKbXFQg31mRVswzuAq2vP2RFdPHQxfl5nJnWsQkelvpPO/Q3LUdtrm19zAgbbDL+AtIg3/lif6/2qNiWCSTfaUpjM7WOPszBNmMRGz/UBZTYc7COTt+I3lK8f6sBn5YyD796LBw6tsNpqfqF9NTAsLT8/PqrXeTpdxFe375gMxeIpNWeE5exMGJKgqnZCcOMOoKMJy61+wdEAYzDFNgIX7ZFvpBYQPf/rTs7LWgtyTSw3fqvMDnfwAf7oIF8rZRwYdVnqTGCWA2h3f4lOf6BERIPkKEK7/DGjmekKnXJrRiLSfcgRjri3VuGBxrJ+Va/Dn6e7o7CdzdJ+fkw7KxTFKuf17Z2r3ZFi1xOduIxXW8/QY6zhq2A11e+HsMe/oaBh3bRcpdMFmW5mqQjGm05xvxArSCAARBKkHjywGs6mRLN2PjNPYdzlI2J8nF6bmSk= henlo"
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICeIqJYRvj53KPvzNz+N6/Ip2Kk5issinxZ/SeQedFF0 epl-root-ssh-key"

    ];
  };
  services.openssh.settings.PermitRootLogin = "prohibit-password";
  networking.usePredictableInterfaceNames = false;
  system.stateVersion = "23.11";
}
