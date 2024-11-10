
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=4ecab3273592f27479a583fb6d975d4aba3486fe";
    nixos-generators = {
      url = "github:nix-community/nixos-generators?rev=246219bc21b943c6f6812bb7744218ba0df08600";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, nixos-generators, ... }: {
    packages.x86_64-linux = {
      default = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        modules = [
          # you can include your own nixos configuration here, i.e.
          ./aws-custom.nix
        ];
        format = "amazon";
      };
    };
  };
}
