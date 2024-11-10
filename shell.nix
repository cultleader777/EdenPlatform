{ extraPackages ? { pkgs }: [] }:
let
  pkgsLock = import ./nix-pkgs-lock.nix;
  pkgs = pkgsLock.pkgs;
  prometheus = pkgsLock.prometheusPkg;

in pkgs.mkShell {
  buildInputs = (with pkgs; [
    cargo
    rustc
    rustfmt
    rust-analyzer
    luajit
    binutils
    nix-serve
    consul
    cfssl
    clippy
    trunk

    # terraform codegen
    terraform
    google-cloud-sdk
    # for key generation
    wireguard-tools
    # for running webassembly browser tests
    ungoogled-chromium
    # for generating dnssec keys
    bind
    # dnssec validation
    dig
    # run nix serve in the background
    tmux
    # set terraform ips to edb souce
    jq
    # file locking
    procmail
    sqlite

    # ocaml development
    ocaml
    dune_3
    opam
    ocamlPackages.findlib
    ocamlPackages.merlin
    ocamlPackages.ocp-indent
    ocamlPackages.ocaml-lsp
    ocamlPackages.utop
    ocamlPackages.sodium
    ocamlPackages.base64
    ocamlPackages.yojson
    ocamlPackages.ppx_deriving_yojson
    ocamlPackages.ocaml_sqlite3
  ]) ++ [
    prometheus # old version includes promtool which we need
  ] ++ (extraPackages { pkgs = pkgs; });

  # Certain Rust tools won't work without this
  # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
  # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  EPL_SHELL = "true";
  AWS_IMAGE_UPLOAD_SCRIPT = builtins.toString ./. + "/misc/create-amis.sh";
  NIXOS_GENERATORS = builtins.toString ./. + "/third-party/nixos-generators";
  NIXOS_GENERATE = builtins.toString ./. + "/third-party/nixos-generators/nixos-generate";
}
