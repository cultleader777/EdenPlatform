{modulesPath, ...}: {
  imports = [
    "${toString modulesPath}/virtualisation/proxmox-lxc.nix"
  ];
  formatAttr = "tarball";
  fileExtension = ".tar.xz";
}
