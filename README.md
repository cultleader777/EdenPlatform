# Developer setup

For testing with libvirt in NixOS you must enable docker and libvirt virtualization

```
    virtualisation.docker.enable = true;
    virtualisation.libvirtd.enable = true;
    networking.firewall.allowedTCPPorts = [
      12777 12778 # epl
    ]; # expose nix store for development

    # ensure virt-manager package is installed
    environment.systemPackages = with pkgs; [
      ..
      virt-manager
      ..
    ];
```
