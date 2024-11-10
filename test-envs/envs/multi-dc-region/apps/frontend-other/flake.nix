
{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=057f9aecfb71c4437d2b27d3323df7f93c010b7e";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils?rev=5aed5285a952e0b949eb3ba02c12fa4fcfef535f";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rust = pkgs.rust-bin.stable.latest.default.override {
          # Set the build targets supported by the toolchain,
          # wasm32-unknown-unknown is required for trunk
          targets = [ "wasm32-unknown-unknown" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;

        # When filtering sources, we want to allow assets other than .rs files
        src = lib.cleanSourceWith {
          src = ./.; # The original, unfiltered source
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            # Example of a folder for images, icons, etc
            (lib.hasInfix "/assets/" path) ||
            # Default filter from crane (allow .rs files)
            (craneLib.filterCargoSources path type)
          ;
        };

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          # We must force the target, otherwise cargo will attempt to use your native target
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };

        appName = "frontend-other";

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          # You cannot run cargo test on a wasm build
          doCheck = false;
        });

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        # This derivation is a directory you can put on a webserver.
        app-crate-uncompressed = craneLib.buildTrunkPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        app-crate = pkgs.stdenv.mkDerivation {
          name = "epl-frontend-app-dir";
          buildInputs = [ pkgs.gzip ];
          src = [ app-crate-uncompressed ];
          buildPhase = ''
            mkdir $out
            cp ${app-crate-uncompressed}/* $out/
            # convert artifact paths from absolute path to relative
            sed -i 's/\/epl-app-/OVERRIDE_ROOT_PATH\/epl-app-/g' $out/index.html
            gzip -k -9 $out/*
          '';
        };

        # Quick example on how to serve the app,
        # This is just an example, not useful for production environments
        nginxConfigFile = pkgs.writeText "nginx.conf.template"
        ''
          pcre_jit on;

          worker_processes 2;
          worker_rlimit_nofile 64;

          events {
              worker_connections  128;
          }

          http {
              log_format nginxlog_json escape=json '{"@timestamp":"$time_iso8601",'
                '"remote_addr":"$remote_addr",'
                '"body_bytes_sent":$body_bytes_sent,'
                '"gzip_ratio":"$gzip_ratio",'
                '"request_time":$request_time,'
                '"upstream_response_time":$upstream_response_time,'
                '"response_status":$status,'
                '"request":"$request",'
                '"request_method":"$request_method",'
                '"host":"$host",'
                '"upstream_addr":"$upstream_addr",'
                '"http_x_forwarded_for":"$http_x_forwarded_for",'
                '"http_referrer":"$http_referer",'
                '"http_user_agent":"$http_user_agent",'
                '"http_version":"$server_protocol",'
                '"server_port":"$server_port"}';
              access_log /dev/stdout nginxlog_json;
              error_log /dev/stderr;
              include ${pkgs.nginx}/conf/mime.types;
              sendfile on;

              server {
                  listen EPL_HTTP_SOCKET;
                  server_name _;
                  gzip on;
                  gzip_static on;
                  gzip_types application/javascript application/json application/wasm text/css;
                  root /html/;
                  location ~ ^/epl-app- {
                    # static resources
                    expires max;
                    if_modified_since off;
                    try_files $uri $uri/;
                  }
                  location / {
                    expires 10m;
                    try_files $uri $uri/ /index.html;
                  }
              }
          }
        '';
        # allow customization of listen address
        serve-app = pkgs.writeShellScriptBin "serve-app" ''
          # log something so integration tests can check it ends up in loki
          echo Starting EPL Frontend app
          EPL_HTTP_SOCKET="${"$"}{EPL_HTTP_SOCKET:-0.0.0.0:8081}"
          EPL_ENDPOINT_MAPPING="${"$"}{EPL_ENDPOINT_MAPPING:-{}}"
          EPL_EXTPAGES_MAPPING="${"$"}{EPL_EXTPAGES_MAPPING:-{}}"
          EPL_EXTLINKS_MAPPING="${"$"}{EPL_EXTLINKS_MAPPING:-{}}"
          cat ${nginxConfigFile} | sed "s/EPL_HTTP_SOCKET/$EPL_HTTP_SOCKET/g" > /nginx.conf
          mkdir /html/
          cp ${app-crate}/* /html/
          ESCAPED_REPLACE=$(printf '%s\n' "$OVERRIDE_ROOT_PATH" | sed -e 's/[\/&]/\\&/g')
          sed -i "s/OVERRIDE_ROOT_PATH/$ESCAPED_REPLACE/g" /html/index.html
          ESCAPED_ENDPOINTS=$(printf '%s\n' "$EPL_ENDPOINT_MAPPING" | sed 's/}}/}/' | sed -e 's/[\/&]/\\&/g')
          ESCAPED_EXTPAGES=$(printf '%s\n' "$EPL_EXTPAGES_MAPPING" | sed 's/}}/}/' | sed -e 's/[\/&]/\\&/g')
          ESCAPED_EXTLINKS=$(printf '%s\n' "$EPL_EXTLINKS_MAPPING" | sed 's/}}/}/' | sed -e 's/[\/&]/\\&/g')
          sed -i "s/<\!--EPL_HEAD_HOOK-->/<base href=\"$ESCAPED_REPLACE\/\" \/>/" /html/index.html
          sed -i "s/<\!--EPL_ENDPOINTS_HOOK-->/<script>var eplEndpointMapping = $ESCAPED_ENDPOINTS;<\/script>/" /html/index.html
          sed -i "s/<\!--EPL_EXTPAGES_HOOK-->/<script>var eplExtPagesMapping = $ESCAPED_EXTPAGES;<\/script>/" /html/index.html
          sed -i "s/<\!--EPL_EXTLINKS_HOOK-->/<script>var eplExtLinksMapping = $ESCAPED_EXTLINKS;<\/script>/" /html/index.html
          # make sure wasm is cached
          gzip -9 -k -f /html/index.html
          # set last modified time to uniform value
          # not to confuse nginx caching and etag
          find /html/ -exec touch -m -d '1977-07-07T07:07:07' {} +
          exec nginx -g 'daemon off;' -c /nginx.conf
        '';

        imageHash = pkgs.lib.head (pkgs.lib.strings.splitString "-" (baseNameOf app-crate.outPath));

        # reusable base
        dockerImageBase = pkgs.dockerTools.buildImage {
          name = "epl-frontend-nginx";
          copyToRoot = pkgs.buildEnv {
            name = "nginx";
            paths = [ pkgs.nginx pkgs.bash pkgs.gzip pkgs.toybox ];
            pathsToLink = [ "/bin" ];
          };
          extraCommands = ''
            mkdir -p var/log/nginx
            mkdir -p var/cache/nginx
            mkdir -p tmp
          '';
          runAsRoot = ''
            #!${pkgs.stdenv.shell}
            ${pkgs.dockerTools.shadowSetup}
            groupadd --system nogroup
            useradd --system --gid nogroup nobody
          '';
        };

        dockerImage = pkgs.dockerTools.buildImage {
          fromImage = dockerImageBase;
          name = appName;
          tag = "v${cargoArtifacts.version}-${imageHash}";
          config = {
            Entrypoint = [ "serve-app" ];
          };
          copyToRoot = pkgs.buildEnv {
            name = "app";
            paths = [ serve-app ];
            pathsToLink = [ "/bin" ];
          };
        };

      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit app-crate;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          app-crate-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Check formatting
          app-crate-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = serve-app;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          nativeBuildInputs = [
            rust
          ] ++ (with pkgs; [
            trunk
            cargo
            rust-analyzer
          ]);
        };

        packages.default = dockerImage;
      });
}