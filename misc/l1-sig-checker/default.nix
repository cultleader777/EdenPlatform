{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  name = "l1-sig-checker";
  version = "0.1.0";
  src = pkgs.lib.sourceFilesBySuffices ./. [ ".ml" "dune" "dune-project" ];
  buildPhase = ''
    dune build
  '';
  installPhase = ''
    mkdir $out
    cp _build/default/bin/main.exe $out/checker
  '';
  buildInputs = with pkgs; [
    # ocaml development
    ocaml
    dune_3
    ocamlPackages.findlib
    ocamlPackages.sodium
    ocamlPackages.base64
    ocamlPackages.yojson
    ocamlPackages.ppx_deriving_yojson
    ocamlPackages.ocaml_sqlite3
  ];
}
