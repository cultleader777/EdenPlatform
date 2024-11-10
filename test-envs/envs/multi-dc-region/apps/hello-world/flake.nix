{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=057f9aecfb71c4437d2b27d3323df7f93c010b7e";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils?rev=5aed5285a952e0b949eb3ba02c12fa4fcfef535f";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.lib.${system};
        appName = "hello-world";

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          # Additional arguments specific to this derivation can be added here.
          # Be warned that using `//` will not do a deep copy of nested
          # structures
          pname = "epl-rust";
          version = "0.1.0";
        });

        myCrate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        imageHash = pkgs.lib.head (pkgs.lib.strings.splitString "-" (baseNameOf myCrate.outPath));

        dockerImage = pkgs.dockerTools.buildImage {
          name = appName;
          tag = "v${cargoArtifacts.version}-${imageHash}";
          config = { Entrypoint = [ "${myCrate}/bin/epl-app" ]; };
        };
      in
    {
      packages.default = dockerImage;
    });
}
