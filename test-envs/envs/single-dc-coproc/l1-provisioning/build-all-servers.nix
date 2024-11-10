
let
  pkgs = import (fetchTarball { url = "https://github.com/NixOS/nixpkgs/archive/057f9aecfb71c4437d2b27d3323df7f93c010b7e.tar.gz"; sha256 = "1ndiv385w1qyb3b18vw13991fzb9wg4cl21wglk89grsfsnra41k"; }) {};
  evalConfig = import (pkgs.path + "/nixos/lib/eval-config.nix");
  buildServer = args: (evalConfig args).config.system.build.toplevel;
in
{

  server-e = buildServer {
    system = "x86_64-linux";
    modules = [
      ./server-e/configuration.nix
    ];
  };

  server-f = buildServer {
    system = "x86_64-linux";
    modules = [
      ./server-f/configuration.nix
    ];
  };

  server-a = buildServer {
    system = "x86_64-linux";
    modules = [
      ./server-a/configuration.nix
    ];
  };

  server-b = buildServer {
    system = "x86_64-linux";
    modules = [
      ./server-b/configuration.nix
    ];
  };

  server-c = buildServer {
    system = "x86_64-linux";
    modules = [
      ./server-c/configuration.nix
    ];
  };

  server-d = buildServer {
    system = "x86_64-linux";
    modules = [
      ./server-d/configuration.nix
    ];
  };

}
