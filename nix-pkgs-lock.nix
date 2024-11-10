let
  # 23.11
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/057f9aecfb71c4437d2b27d3323df7f93c010b7e.tar.gz")) {
    config.allowUnfreePredicate = pkg: builtins.elem (pkgs.lib.getName pkg) [
      "terraform"
    ];
  };
  # Nixpkgs version 22.11, includes promtool (23.05 doesn't)
  pkgsOld = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/4d2b37a84fad1091b9de401eb450aae66f1a741e.tar.gz")) {};
in
{
  prometheusPkg = pkgsOld.prometheus;
  pkgs = pkgs;
}
