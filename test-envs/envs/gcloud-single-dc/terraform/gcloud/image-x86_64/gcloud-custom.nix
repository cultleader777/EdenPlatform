
{ pkgs, modulesPath, ... }:
{
  imports = [
    "${modulesPath}/virtualisation/google-compute-config.nix"
  ];


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



  users.users.root.hashedPassword = "!";
  security.sudo.wheelNeedsPassword = false;
  users.users.admin = {
    isNormalUser = true;
    home = "/home/admin";
    extraGroups = [ "wheel" ];
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGu3/SFtnfsWErjXZ3eBgREPSzfZxtEdP6/tOZ/tL3hi epl-root-ssh-key"
    ];
  };

  # we use cloud native firewalls
  networking.firewall.enable = false;
  # always have lan interface as eth0, anything else is confusing
  networking.usePredictableInterfaceNames = false;
  # bare minimum packages for fast bootstrap l1 provisioning
  environment.systemPackages = with pkgs; [
    gzip
    sqlite
    tmux
    git
    procmail # lockfile command for l1
  ];

}
