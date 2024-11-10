
{ pkgs, ... }:
{

  users.users.root.hashedPassword = "!";
  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINx1ICTCwzUHWYauEiXYTVxQOlf5qNzKxiQuwRCpbtbA epl-root-ssh-key
"
  ];

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
  ];

}
