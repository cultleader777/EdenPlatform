
let
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/4ecab3273592f27479a583fb6d975d4aba3486fe.tar.gz")) {};
in

{ config, lib, pkgs, ... }:
{
  nix.settings = {
    substituters = [
      "http://10.17.0.1:12777/"

      "https://cache.nixos.org/"
    ];

    trusted-public-keys = [
      "epl-nix-cache:3QbATnsHn1DB7mQxFvNWKubUzMyHLsLnpkBkuUlqtPI="

    ];
  };


  # bare minimum packages for fast bootstrap l1 provisioning
  environment.systemPackages = with pkgs; [
    gzip
    sqlite
    tmux
    git
  ];

  services.sshd.enable = true;

  users.users.root.hashedPassword = "!";
  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC3AkuDzzPrMaDav0kN7PIoaBU1Vtw1TfkHxWzPMrleocCltYl8TljwCqEJtmizx5DGKbXFQg31mRVswzuAq2vP2RFdPHQxfl5nJnWsQkelvpPO/Q3LUdtrm19zAgbbDL+AtIg3/lif6/2qNiWCSTfaUpjM7WOPszBNmMRGz/UBZTYc7COTt+I3lK8f6sBn5YyD796LBw6tsNpqfqF9NTAsLT8/PqrXeTpdxFe375gMxeIpNWeE5exMGJKgqnZCcOMOoKMJy61+wdEAYzDFNgIX7ZFvpBYQPf/rTs7LWgtyTSw3fqvMDnfwAf7oIF8rZRwYdVnqTGCWA2h3f4lOf6BERIPkKEK7/DGjmekKnXJrRiLSfcgRjri3VuGBxrJ+Va/Dn6e7o7CdzdJ+fkw7KxTFKuf17Z2r3ZFi1xOduIxXW8/QY6zhq2A11e+HsMe/oaBh3bRcpdMFmW5mqQjGm05xvxArSCAARBKkHjywGs6mRLN2PjNPYdzlI2J8nF6bmSk= henlo"
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIP1uxUv5pWtBLKUSinFlvV1Aqyv/VmhhHijrWzeSYlAE epl-root-ssh-key
"

  ];
  services.openssh.settings.PermitRootLogin = "prohibit-password";

  system.stateVersion = "23.05";
}
