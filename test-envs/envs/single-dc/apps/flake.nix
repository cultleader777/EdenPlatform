{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=057f9aecfb71c4437d2b27d3323df7f93c010b7e";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    app-hello-world.url = "path:./hello-world";
  };

  outputs = { self, nixpkgs, crane, flake-utils, app-hello-world, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
    {
      packages.default = pkgs.mkDerivation {
      };
    });
}
