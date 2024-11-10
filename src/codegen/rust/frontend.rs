use std::collections::BTreeMap;

use convert_case::{Case, Casing};

use crate::{
    codegen::Directory,
    database::{TableRowPointerFrontendApplication, TableRowPointerVersionedType},
    static_analysis::{
        http_endpoints::{CorePathSegment, PathArgs, ValidHttpPrimitiveType},
        CheckedDB,
    },
};

use super::http_type_to_rust_type;

pub fn generate_rust_frontend_app(
    checked: &CheckedDB,
    app: TableRowPointerFrontendApplication,
    dir: &mut Directory,
) {
    let db = &checked.db;
    let app_name = db
        .frontend_application()
        .c_application_name(app)
        .to_case(Case::Kebab);
    let comp_env = db.frontend_application().c_build_environment(app);
    let nixpkgs = db
        .rust_compilation_environment()
        .c_nixpkgs_environment(comp_env);
    let nixpkgs_hash = db
        .nixpkgs_version()
        .c_checksum(db.nixpkgs_environment().c_version(nixpkgs));
    let edition = db.rust_compilation_environment().c_rust_edition(comp_env);
    let src_dir = dir.create_directory("src");
    src_dir.create_file("main.rs", rust_frontend_main_rs());
    src_dir.create_file("generated.rs", rust_frontend_generated_part(checked, app));
    src_dir.create_file_if_not_exists("implementation.rs", implementation_frontend_mock_rs());
    dir.create_file(
        "Cargo.toml",
        super::rust_cargo_toml(edition.as_str(), checked, comp_env),
    );
    dir.create_file(
        "flake.nix",
        generate_rust_frontend_flake(&app_name, nixpkgs_hash),
    );
    dir.create_file(
        ".envrc",
        "use flake".to_string(),
    );
    dir.create_file("README.md", generate_rust_frontend_readme());
    dir.create_file(
        "index.html",
        generate_rust_frontend_html(
            checked
                .db
                .frontend_application()
                .c_index_page_title(app)
                .as_str(),
        ),
    );
}

fn generate_rust_frontend_readme() -> String {
    r#"
# Eden Platform frontend app

## Requirements

- Nix package manager with [flakes support](https://nixos.wiki/wiki/Flakes)
- Docker

## Develop with trunk

### Enter the nix shell environment

```
nix develop
```

### Build trunk app

```
trunk build
```

### Serve trunk app

`--public-url` is needed because Trunk.toml uses relative path for building.
```
trunk serve --public-url=/
```

### Serve trunk app with possibly live backend
```
trunk serve --public-url=/ --proxy-backend https://www.epl-infra.net/api/
```

more trunk documentation can be found [here](https://trunkrs.dev/configuration/)

## Build the project with running tests

```
nix build
```

## Loading and running docker image

Should build a docker image file named `result`.

You can load docker image locally
```
docker load -i result
```

Then run the image on custom port 12421
```
docker run --rm -e EPL_HTTP_SOCKET=0.0.0.0:12421 -p 127.0.0.1:12421:12421 -it frontend-test:v0.1.0-l9z821n112ria1hv5w1hyl3zdwgp9xby
```

Page should be available in the browser address http://127.0.0.1:12421/


"#.to_string()
}

fn generate_rust_frontend_flake(app_name: &str, nixpkgs_rev: &str) -> String {
    format!(
        r#"
{{
  description = "Build a cargo project";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs?rev={nixpkgs_rev}";

    crane = {{
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    }};

    flake-utils.url = "github:numtide/flake-utils?rev=5aed5285a952e0b949eb3ba02c12fa4fcfef535f";

    rust-overlay = {{
      url = "github:oxalica/rust-overlay";
      inputs = {{
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      }};
    }};
  }};

  outputs = {{ self, nixpkgs, crane, flake-utils, rust-overlay, ... }}:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {{
          inherit system;
          overlays = [ (import rust-overlay) ];
        }};

        inherit (pkgs) lib;

        rust = pkgs.rust-bin.stable.latest.default.override {{
          # Set the build targets supported by the toolchain,
          # wasm32-unknown-unknown is required for trunk
          targets = [ "wasm32-unknown-unknown" ];
        }};
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;

        # When filtering sources, we want to allow assets other than .rs files
        src = lib.cleanSourceWith {{
          src = ./.; # The original, unfiltered source
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            # Example of a folder for images, icons, etc
            (lib.hasInfix "/assets/" path) ||
            # Default filter from crane (allow .rs files)
            (craneLib.filterCargoSources path type)
          ;
        }};

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {{
          inherit src;
          # We must force the target, otherwise cargo will attempt to use your native target
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        }};

        appName = "{app_name}";

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {{
          # You cannot run cargo test on a wasm build
          doCheck = false;
        }});

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        # This derivation is a directory you can put on a webserver.
        app-crate-uncompressed = craneLib.buildTrunkPackage (commonArgs // {{
          inherit cargoArtifacts;
        }});

        app-crate = pkgs.stdenv.mkDerivation {{
          name = "epl-frontend-app-dir";
          buildInputs = [ pkgs.gzip ];
          src = [ app-crate-uncompressed ];
          buildPhase = ''
            mkdir $out
            cp ${{app-crate-uncompressed}}/* $out/
            # convert artifact paths from absolute path to relative
            sed -i 's/\/epl-app-/OVERRIDE_ROOT_PATH\/epl-app-/g' $out/index.html
            gzip -k -9 $out/*
          '';
        }};

        # Quick example on how to serve the app,
        # This is just an example, not useful for production environments
        nginxConfigFile = pkgs.writeText "nginx.conf.template"
        ''
          pcre_jit on;

          worker_processes 2;
          worker_rlimit_nofile 64;

          events {{
              worker_connections  128;
          }}

          http {{
              log_format nginxlog_json escape=json '{{"@timestamp":"$time_iso8601",'
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
                '"server_port":"$server_port"}}';
              access_log /dev/stdout nginxlog_json;
              error_log /dev/stderr;
              include ${{pkgs.nginx}}/conf/mime.types;
              sendfile on;

              server {{
                  listen EPL_HTTP_SOCKET;
                  server_name _;
                  gzip on;
                  gzip_static on;
                  gzip_types application/javascript application/json application/wasm text/css;
                  root /html/;
                  location ~ ^/epl-app- {{
                    # static resources
                    expires max;
                    if_modified_since off;
                    try_files $uri $uri/;
                  }}
                  location / {{
                    expires 10m;
                    try_files $uri $uri/ /index.html;
                  }}
              }}
          }}
        '';
        # allow customization of listen address
        serve-app = pkgs.writeShellScriptBin "serve-app" ''
          # log something so integration tests can check it ends up in loki
          echo Starting EPL Frontend app
          EPL_HTTP_SOCKET="${{"$"}}{{EPL_HTTP_SOCKET:-0.0.0.0:8081}}"
          EPL_ENDPOINT_MAPPING="${{"$"}}{{EPL_ENDPOINT_MAPPING:-{{}}}}"
          EPL_EXTPAGES_MAPPING="${{"$"}}{{EPL_EXTPAGES_MAPPING:-{{}}}}"
          EPL_EXTLINKS_MAPPING="${{"$"}}{{EPL_EXTLINKS_MAPPING:-{{}}}}"
          cat ${{nginxConfigFile}} | sed "s/EPL_HTTP_SOCKET/$EPL_HTTP_SOCKET/g" > /nginx.conf
          mkdir /html/
          cp ${{app-crate}}/* /html/
          ESCAPED_REPLACE=$(printf '%s\n' "$OVERRIDE_ROOT_PATH" | sed -e 's/[\/&]/\\&/g')
          sed -i "s/OVERRIDE_ROOT_PATH/$ESCAPED_REPLACE/g" /html/index.html
          ESCAPED_ENDPOINTS=$(printf '%s\n' "$EPL_ENDPOINT_MAPPING" | sed 's/}}}}/}}/' | sed -e 's/[\/&]/\\&/g')
          ESCAPED_EXTPAGES=$(printf '%s\n' "$EPL_EXTPAGES_MAPPING" | sed 's/}}}}/}}/' | sed -e 's/[\/&]/\\&/g')
          ESCAPED_EXTLINKS=$(printf '%s\n' "$EPL_EXTLINKS_MAPPING" | sed 's/}}}}/}}/' | sed -e 's/[\/&]/\\&/g')
          sed -i "s/<\!--EPL_HEAD_HOOK-->/<base href=\"$ESCAPED_REPLACE\/\" \/>/" /html/index.html
          sed -i "s/<\!--EPL_ENDPOINTS_HOOK-->/<script>var eplEndpointMapping = $ESCAPED_ENDPOINTS;<\/script>/" /html/index.html
          sed -i "s/<\!--EPL_EXTPAGES_HOOK-->/<script>var eplExtPagesMapping = $ESCAPED_EXTPAGES;<\/script>/" /html/index.html
          sed -i "s/<\!--EPL_EXTLINKS_HOOK-->/<script>var eplExtLinksMapping = $ESCAPED_EXTLINKS;<\/script>/" /html/index.html
          # make sure wasm is cached
          gzip -9 -k -f /html/index.html
          # set last modified time to uniform value
          # not to confuse nginx caching and etag
          find /html/ -exec touch -m -d '1977-07-07T07:07:07' {{}} +
          exec nginx -g 'daemon off;' -c /nginx.conf
        '';

        imageHash = pkgs.lib.head (pkgs.lib.strings.splitString "-" (baseNameOf app-crate.outPath));

        # reusable base
        dockerImageBase = pkgs.dockerTools.buildImage {{
          name = "epl-frontend-nginx";
          copyToRoot = pkgs.buildEnv {{
            name = "nginx";
            paths = [ pkgs.nginx pkgs.bash pkgs.gzip pkgs.toybox ];
            pathsToLink = [ "/bin" ];
          }};
          extraCommands = ''
            mkdir -p var/log/nginx
            mkdir -p var/cache/nginx
            mkdir -p tmp
          '';
          runAsRoot = ''
            #!${{pkgs.stdenv.shell}}
            ${{pkgs.dockerTools.shadowSetup}}
            groupadd --system nogroup
            useradd --system --gid nogroup nobody
          '';
        }};

        dockerImage = pkgs.dockerTools.buildImage {{
          fromImage = dockerImageBase;
          name = appName;
          tag = "v${{cargoArtifacts.version}}-${{imageHash}}";
          config = {{
            Entrypoint = [ "serve-app" ];
          }};
          copyToRoot = pkgs.buildEnv {{
            name = "app";
            paths = [ serve-app ];
            pathsToLink = [ "/bin" ];
          }};
        }};

      in
      {{
        checks = {{
          # Build the crate as part of `nix flake check` for convenience
          inherit app-crate;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          app-crate-clippy = craneLib.cargoClippy (commonArgs // {{
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }});

          # Check formatting
          app-crate-fmt = craneLib.cargoFmt {{
            inherit src;
          }};
        }};

        apps.default = flake-utils.lib.mkApp {{
          drv = serve-app;
        }};

        devShells.default = pkgs.mkShell {{
          inputsFrom = builtins.attrValues self.checks;

          nativeBuildInputs = [
            rust
          ] ++ (with pkgs; [
            trunk
            cargo
            rust-analyzer
          ]);
        }};

        packages.default = dockerImage;
      }});
}}"#
    )
}

// TODO: make this a column so it could be easily customized?
// Add custom css resources and stuff?
fn generate_rust_frontend_html(title: &str) -> String {
    let title = html_escape::encode_safe(title);
    format!(
        r#"
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>{title}</title>
    <!--EPL_HEAD_HOOK-->
    <!--EPL_ENDPOINTS_HOOK-->
    <!--EPL_EXTPAGES_HOOK-->
    <!--EPL_EXTLINKS_HOOK-->
  </head>
</html>
"#
    )
}

fn implementation_frontend_mock_rs() -> String {
    r#"
use yew::prelude::*;
use crate::generated::AppRouteExt;

pub fn switch(route: AppRouteExt) -> Html {
    match route {
        // implement your matched routes here
    }
}
"#
    .to_string()
}

fn rust_frontend_main_rs() -> String {
    r#"
mod generated;
mod implementation;

fn main() {
    yew::Renderer::<crate::generated::Main>::new().render();
}
"#
    .to_string()
}

fn rust_frontend_generated_part(
    checked: &CheckedDB,
    app: TableRowPointerFrontendApplication,
) -> String {
    let mut res = r#"
use yew::prelude::*;
use yew_router::prelude::*;

"#
    .to_string();

    res += &rust_frontend_generate_routes(checked, app);
    res += &rust_frontend_generate_api_calls(checked, app);
    res += &rust_frontend_generate_page_and_link_urls(checked, app);

    let endpoint_mapping_needed = !checked
        .db
        .frontend_application()
        .c_children_frontend_application_used_endpoint(app)
        .is_empty()
        || !checked
            .db
            .frontend_application()
            .c_children_frontend_application_external_page(app)
            .is_empty()
        || !checked
            .db
            .frontend_application()
            .c_children_frontend_application_external_link(app)
            .is_empty();

    if endpoint_mapping_needed {
        res += r#"
fn get_endpoint_mapping(glob_var: &str, endpoint: &str) -> Option<String> {
    let mapping: ::js_sys::Object = gloo::utils::window().get(glob_var)
        .unwrap()
        .try_into()
        .unwrap();

    ::js_sys::Reflect::get(&mapping, &::wasm_bindgen::JsValue::from_str(endpoint)).ok().map(|i| i.as_string()).flatten()
}
"#;
    }

    res
}

fn rust_frontend_generate_api_calls(
    checked: &CheckedDB,
    app: TableRowPointerFrontendApplication,
) -> String {
    let mut res = String::new();

    #[derive(Default)]
    struct BwTypeFlags {
        json_serialization: bool,
        json_deserialization: bool,
    }

    let mut needed_bw_types: BTreeMap<TableRowPointerVersionedType, BwTypeFlags> = BTreeMap::new();

    for ue in checked
        .db
        .frontend_application()
        .c_children_frontend_application_used_endpoint(app)
    {
        let backend_http_endpoint = checked
            .db
            .frontend_application_used_endpoint()
            .c_backend_endpoint(*ue);
        let this_type = checked
            .db
            .http_endpoint_data_type()
            .c_http_endpoint_data_type(
                checked
                    .db
                    .backend_http_endpoint()
                    .c_data_type(backend_http_endpoint),
            )
            .as_str();
        // TODO: support binary
        assert_eq!(this_type, "json");

        let content_type = match this_type {
            "json" => "application/json",
            _ => panic!("Unsupported content type {this_type}"),
        };

        let checked_endpoint = checked
            .projections
            .checked_http_endpoints
            .value(backend_http_endpoint);
        let has_req_args = checked_endpoint.has_required_arguments();
        let has_opt_args = checked_endpoint.has_optional_arguments();
        let endpoint_name = checked
            .db
            .frontend_application_used_endpoint()
            .c_endpoint_name(*ue);

        // if has optional arguments generate Default struct
        let req_args_struct_name = format!("Api{}Args", endpoint_name.to_case(Case::Pascal));
        let opt_args_struct_name = format!("Api{}OptArgs", endpoint_name.to_case(Case::Pascal));

        let mut fun_args = Vec::with_capacity(2);

        res += "\n";

        let mut args_list = Vec::new();
        if has_req_args {
            args_list.push("required_args");
            fun_args.push(format!("required_args: {}", req_args_struct_name));
            res += "pub struct ";
            res += &req_args_struct_name;
            res += " {\n";
            if let Some(inp_body) = &checked_endpoint.input_body_type {
                let the_type = checked
                    .projections
                    .rust_versioned_type_snippets
                    .value(*inp_body);
                let e: &mut _ = needed_bw_types.entry(*inp_body).or_default();
                e.json_serialization = true;
                res += "    pub input_body: ";
                res += &the_type.nominal_type_name;
                res += ",\n";
            }
            for ra in &checked_endpoint.path_args.required_args {
                if let CorePathSegment::Argument(vn, vt) = ra {
                    res += "    pub ";
                    res += vn;
                    res += ": ";
                    res += http_type_to_rust_type(*vt);
                    res += ",\n";
                }
            }
            // add path + payload body?
            res += "}\n";
        }
        res += "\n";

        if has_opt_args {
            args_list.push("optional_args");
            fun_args.push(format!("optional_args: {}", opt_args_struct_name));
            res += "#[derive(Default)]\n";
            res += "pub struct ";
            res += &opt_args_struct_name;
            res += " {\n";

            for (an, at) in &checked_endpoint.path_args.opt_query_args {
                res += "    pub ";
                res += an;
                res += ": ";
                if !at.is_multiple {
                    res += "Option<";
                } else {
                    res += "Vec<";
                }
                res += http_type_to_rust_type(at.the_type);
                res += ">,\n";
            }

            res += "}\n";
        }
        res += "\n";

        res += "pub async fn api_";
        res += endpoint_name;
        res += "(";
        res += &fun_args.join(", ");
        res += ") -> ";

        let result_type = if let Some(out_type) = &checked_endpoint.output_body_type {
            let the_type = checked
                .projections
                .rust_versioned_type_snippets
                .value(*out_type);
            let e = needed_bw_types.entry(*out_type).or_default();
            e.json_deserialization = true;
            format!(
                "Result<{}, Box<dyn ::std::error::Error>>",
                the_type.nominal_type_name
            )
        } else {
            "Result<(), Box<dyn ::std::error::Error>>".to_string()
        };

        res += &result_type;

        res += " {\n";

        // concat slash args from path and add question mark args from opts?
        let maybe_mut = if has_req_args || has_opt_args {
            "mut"
        } else {
            ""
        };
        res += &format!("    let {maybe_mut} url = get_endpoint_mapping(\"eplEndpointMapping\", \"{endpoint_name}\").unwrap_or_else(|| \"/\".to_string());\n");

        for (idx, pa) in checked_endpoint.path_args.required_args.iter().enumerate() {
            if idx > 0 && !matches!(pa, CorePathSegment::LastSlash) {
                res += "    url += '/';\n";
            }
            match pa {
                CorePathSegment::Text(t) => {
                    res += "    url += \"";
                    res += t;
                    res += "\";\n";
                }
                CorePathSegment::Argument(n, _) => {
                    res += "    url += &::urlencoding::encode(required_args.";
                    res += n;
                    res += ");\n";
                }
                CorePathSegment::LastSlash => {
                    res += "    url += '/';\n";
                }
            }
        }

        if has_opt_args {
            res += "    let mut query_args = Vec::new();\n";
            for (k, a) in &checked_endpoint.path_args.opt_query_args {
                if a.is_multiple {
                    res += "    for arg in &optional_args.";
                    res += k;
                    res += " {\n";
                    res +=
                        &format!("        query_args.push((\"{k}\", format!(\"{{}}\", arg)));\n");
                    res += "    }\n";
                } else {
                    res += "    if let Some(arg) = &optional_args.";
                    res += k;
                    res += " {\n";
                    res +=
                        &format!("        query_args.push((\"{k}\", format!(\"{{}}\", arg)));\n");
                    res += "    }\n";
                }
            }
        }

        let method = checked
            .db
            .backend_http_endpoint()
            .c_http_method(backend_http_endpoint);
        let method_str = checked
            .db
            .http_methods()
            .c_http_method_name(method)
            .to_lowercase();

        // how to http POST?
        // http method set
        res += &format!("    let fetch = ::gloo_net::http::Request::{method_str}(&url)\n");
        if has_opt_args {
            res += "        .query(query_args)\n";
        }
        if let Some(inp_body) = &checked_endpoint.input_body_type {
            let the_type = checked
                .projections
                .rust_versioned_type_snippets
                .value(*inp_body);
            res += &format!("        .header(\"Content-Type\", \"{content_type}\")\n");
            res += &format!(
                "        .body({}(&required_args.input_body))?\n",
                the_type.json_serialization_function.function_name
            );
        }
        res += "        .send()\n";
        res += "        .await?\n";
        res += "        .binary()\n";
        res += "        .await?;\n";

        if let Some(out_type) = &checked_endpoint.output_body_type {
            let the_type = checked
                .projections
                .rust_versioned_type_snippets
                .value(*out_type);
            res += "    let deser = ";
            res += &the_type.json_deserialization_function.function_name;
            res += "(&fetch)?;\n";
            res += "    Ok(deser)\n";
        } else {
            res += "    drop(fetch);\n";
            res += "    Ok(())\n";
        }

        res += "}\n";
        res += "\n";

        // generate closure call instead of async
        res += "#[allow(dead_code)]\n";
        res += "pub fn apicl_";
        res += endpoint_name;
        res += "(";
        res += &fun_args.join(", ");
        res += ", closure: impl FnOnce(";
        res += &result_type;
        res += ") + 'static) {\n";
        res += "    wasm_bindgen_futures::spawn_local(async move {\n";
        res += "        closure(api_";
        res += endpoint_name;
        res += "(";
        res += &args_list.join(", ");
        res += ").await);\n";
        res += "    })\n";
        res += "}\n";
        res += "\n";
        // does have input payload? yes it does have args always!!
        // it may not have args if nothing in url and no post body?
    }

    for (bw_type, needs) in &needed_bw_types {
        let the_type = checked
            .projections
            .rust_versioned_type_snippets
            .value(*bw_type);

        res += &the_type.struct_definitions;
        if needs.json_deserialization {
            res += &the_type.json_deserialization_function.function_body;
            res += &the_type.migration_functions;
        }
        if needs.json_serialization {
            res += &the_type.json_serialization_function.function_body;
        }
    }

    res += "\n";
    res += super::json_deserialization_error_types();

    res
}

fn rust_frontend_generate_routes(
    checked: &CheckedDB,
    app: TableRowPointerFrontendApplication,
) -> String {
    let mut res = r#"
#[derive(Clone, PartialEq, Routable)]
pub enum AppRoute {
"#
    .to_string();

    for page in checked
        .db
        .frontend_application()
        .c_children_frontend_page(app)
    {
        let path_args = checked.projections.checked_frontend_pages.value(*page);
        let enum_name = checked
            .db
            .frontend_page()
            .c_page_name(*page)
            .to_case(Case::Pascal);
        let (path, args_count) = gen_rust_http_path_frontend(path_args, false);
        res += "    #[at(\"";
        res += &path;
        res += "\")]\n";
        res += "    ";
        res += &enum_name;
        let mut args_set = 0;
        if args_count > 0 {
            res += " { ";
            for pa in &path_args.required_args {
                if let CorePathSegment::Argument(n, t) = pa {
                    res += n;
                    res += ": ";
                    res += super::http_type_to_rust_type(*t);
                    args_set += 1;
                    if args_set < args_count {
                        res += ", ";
                    }
                }
            }
            res += " }";
        }
        res += ",\n";
    }

    res += "}\n";

    res += r#"
#[derive(Clone, PartialEq)]
pub enum AppRouteExt {
"#;

    for page in checked
        .db
        .frontend_application()
        .c_children_frontend_page(app)
    {
        let path_args = checked.projections.checked_frontend_pages.value(*page);
        let enum_name = checked
            .db
            .frontend_page()
            .c_page_name(*page)
            .to_case(Case::Pascal);
        let (_, args_count) = gen_rust_http_path_frontend(path_args, true);
        res += "    ";
        res += &enum_name;
        let mut args_set = 0;
        if args_count > 0 {
            res += " { ";
            for pa in &path_args.required_args {
                if let CorePathSegment::Argument(n, t) = pa {
                    res += n;
                    res += ": ";
                    res += super::http_type_to_rust_type(*t);
                    args_set += 1;
                    if args_set < args_count {
                        res += ", ";
                    }
                }
            }
            for (opt_n, opt_t) in &path_args.opt_query_args {
                res += opt_n;
                res += ": ";
                if opt_t.is_multiple {
                    res += "Vec<";
                } else {
                    res += "Option<";
                }
                res += super::http_type_to_rust_type(opt_t.the_type);
                res += ">";
                args_set += 1;
                if args_set < args_count {
                    res += ", ";
                }
            }
            res += " }";
        }
        res += ",\n";
    }

    res += "}\n";

    res += r#"
impl Routable for AppRouteExt {
    fn from_path(_path: &str, _params: &std::collections::HashMap<&str, &str>) -> Option<Self> {
        None
    }

    fn to_path(&self) -> String {
        match self {
"#;

    for page in checked
        .db
        .frontend_application()
        .c_children_frontend_page(app)
    {
        let path_args = checked.projections.checked_frontend_pages.value(*page);
        let enum_name = checked
            .db
            .frontend_page()
            .c_page_name(*page)
            .to_case(Case::Pascal);
        let (_, args_count) = gen_rust_http_path_frontend(path_args, true);
        res += "            AppRouteExt::";
        res += &enum_name;
        let mut args_set = 0;
        if args_count > 0 {
            res += " { ";
            for pa in &path_args.required_args {
                if let CorePathSegment::Argument(n, _) = pa {
                    res += n;
                    args_set += 1;
                    if args_set < args_count {
                        res += ", ";
                    }
                }
            }
            for (n, _) in &path_args.opt_query_args {
                res += n;
                args_set += 1;
                if args_set < args_count {
                    res += ", ";
                }
            }
            res += " }";
        }
        res += " => {\n";
        let opt_args_needed = !path_args.opt_query_args.is_empty();
        let format_expr = gen_rust_http_path_format(path_args);
        if !opt_args_needed {
            res += "                ";
            res += &format_expr;
            res += "\n";
        } else {
            res += "                let mut res = ";
            res += &format_expr;
            res += ";\n";
            res += "                let mut arg_added = false;\n";
            res += "                let mut append = |i: &str| {
                    if arg_added {
                        res += \"&\";
                    } else {
                        res += \"?\";
                        arg_added = true;
                    }
                    res += i;
                };
";
            for (on, ot) in &path_args.opt_query_args {
                match ot.the_type {
                    ValidHttpPrimitiveType::Int => {
                        res += &format!(
                            r#"                {on}.iter().for_each(|arg| append(&format!("{on}={{arg}}")));"#
                        );
                    }
                    ValidHttpPrimitiveType::Float => {
                        res += &format!(
                            r#"                {on}.iter().for_each(|arg| append(&format!("{on}={{arg}}")));"#
                        );
                    }
                    ValidHttpPrimitiveType::Bool => {
                        res += &format!(
                            r#"                {on}.iter().for_each(|arg| append(&format!("{on}={{arg}}")));"#
                        );
                    }
                    ValidHttpPrimitiveType::Text => {
                        res += &format!(
                            r#"                {on}.iter().for_each(|arg| append(&format!("{on}={{}}", ::urlencoding::encode(arg))));"#
                        );
                    }
                }
                res += "\n";
            }
            res += "                res\n";
        }
        res += "            },\n";
    }

    res += r#"
        }
    }

    fn routes() -> Vec<&'static str> {
        vec![]
    }

    fn not_found_route() -> Option<Self> {
        None
    }

    fn recognize(_pathname: &str) -> Option<Self> {
        None
    }
}
"#;

    res += r#"
#[function_component(Main)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<AppRoute> render={switch} />
        </BrowserRouter>
    }
}

fn switch(route: AppRoute) -> Html {
    let route_ext: AppRouteExt = match route {
"#;

    let mut has_any_query_args = false;
    for page in checked
        .db
        .frontend_application()
        .c_children_frontend_page(app)
    {
        let path_args = checked.projections.checked_frontend_pages.value(*page);
        let enum_name = checked
            .db
            .frontend_page()
            .c_page_name(*page)
            .to_case(Case::Pascal);
        let (_, args_count) = gen_rust_http_path_frontend(path_args, true);
        res += "        AppRoute::";
        res += &enum_name;

        let (_, req_args_count) = gen_rust_http_path_frontend(path_args, false);
        if req_args_count > 0 {
            let mut passed_count: u32 = 0;
            res += " { ";
            for req in &path_args.required_args {
                if let CorePathSegment::Argument(an, _) = req {
                    res += an;
                    passed_count += 1;
                    if passed_count < req_args_count {
                        res += ", ";
                    }
                }
            }
            res += " }";
        }

        res += " => {\n";
        if !path_args.opt_query_args.is_empty() {
            has_any_query_args = true;
            // if user picks argument name that matches this name, tough
            res += "            let current_url_args = current_url_args();\n";
        }
        for (qn, qt) in &path_args.opt_query_args {
            res += "            let ";
            res += qn;
            res += " = ";
            if !qt.is_multiple {
                match &qt.the_type {
                    ValidHttpPrimitiveType::Int => {
                        res += &format!(
                            r#"current_url_args.get("{qn}").map(|v| v.parse::<i64>().ok()).flatten();"#
                        );
                    }
                    ValidHttpPrimitiveType::Float => {
                        res += &format!(
                            r#"current_url_args.get("{qn}").map(|v| v.parse::<f64>().ok()).flatten();"#
                        );
                    }
                    ValidHttpPrimitiveType::Bool => {
                        res += &format!(
                            r#"current_url_args.get("{qn}").map(|v| v.parse::<bool>().ok()).flatten();"#
                        );
                    }
                    ValidHttpPrimitiveType::Text => {
                        res += &format!(r#"current_url_args.get("{qn}");"#);
                    }
                }
            } else {
                match &qt.the_type {
                    ValidHttpPrimitiveType::Int => {
                        res += &format!(
                            r#"current_url_args.get_all("{qn}").iter().filter_map(|i| i.as_string().map(|j| j.parse::<i64>().ok()).flatten()).collect::<Vec<_>>();"#
                        );
                    }
                    ValidHttpPrimitiveType::Float => {
                        res += &format!(
                            r#"current_url_args.get_all("{qn}").iter().filter_map(|i| i.as_string().map(|j| j.parse::<f64>().ok()).flatten()).collect::<Vec<_>>();"#
                        );
                    }
                    ValidHttpPrimitiveType::Bool => {
                        res += &format!(
                            r#"current_url_args.get_all("{qn}").iter().filter_map(|i| i.as_string().map(|j| j.parse::<bool>().ok()).flatten()).collect::<Vec<_>>();"#
                        );
                    }
                    ValidHttpPrimitiveType::Text => {
                        res += &format!(
                            r#"current_url_args.get_all("{qn}").iter().filter_map(|i| i.as_string()).collect::<Vec<_>>();"#
                        );
                    }
                }
            }
            res += "\n";
        }
        res += "            AppRouteExt::";
        res += &enum_name;

        let mut passed_count: u32 = 0;
        if args_count > 0 {
            res += " { ";
            for req in &path_args.required_args {
                if let CorePathSegment::Argument(an, _) = req {
                    res += an;
                    passed_count += 1;
                    if passed_count < args_count {
                        res += ", ";
                    }
                }
            }
            for (qn, _) in &path_args.opt_query_args {
                res += qn;
                passed_count += 1;
                if passed_count < args_count {
                    res += ", ";
                }
            }
            res += " }";
        }
        res += "\n";
        res += "        },\n";
    }

    res += r#"    };
    crate::implementation::switch(route_ext)
}
"#;

    if has_any_query_args {
        res += r#"
fn current_url_args() -> web_sys::UrlSearchParams {
    web_sys::UrlSearchParams::new_with_str(
        &gloo::utils::document().location().unwrap().search().unwrap()
    ).unwrap()
}
"#;
    }

    res
}

fn gen_rust_http_path_frontend(path_args: &PathArgs, include_query: bool) -> (String, u32) {
    let mut res = String::new();
    let mut path_arguments_count = 0;

    for segment in &path_args.required_args {
        if path_args.required_args.len() > 1 && !matches!(segment, CorePathSegment::LastSlash) {
            res += "/";
        }
        match segment {
            CorePathSegment::Text(t) => {
                res += t;
            }
            CorePathSegment::Argument(n, _) => {
                path_arguments_count += 1;
                res += ":";
                res += n;
            }
            CorePathSegment::LastSlash => {
                res += "/";
            }
        }
    }

    if include_query {
        if !path_args.opt_query_args.is_empty() {
            res += "?";
        }

        for (idx, (opt_arg_k, _)) in path_args.opt_query_args.iter().enumerate() {
            path_arguments_count += 1;
            let is_last = idx == path_args.opt_query_args.len() - 1;
            res += opt_arg_k;
            res += "=:";
            res += opt_arg_k;
            if !is_last {
                res += "&";
            }
        }
    }

    (res, path_arguments_count)
}

fn gen_rust_http_path_format(path_args: &PathArgs) -> String {
    let mut res = "format!(\"".to_string();

    if path_args.required_args.len() == 1 {
        assert_eq!(
            path_args.required_args[0],
            crate::static_analysis::http_endpoints::CorePathSegment::LastSlash
        );
        res += "/\")";
        return res;
    }
    for segment in &path_args.required_args {
        if path_args.required_args.len() > 1 && !matches!(segment, CorePathSegment::LastSlash) {
            res += "/";
        }
        match segment {
            crate::static_analysis::http_endpoints::CorePathSegment::Text(t) => {
                res += t;
            }
            crate::static_analysis::http_endpoints::CorePathSegment::Argument(_, _) => {
                res += "{}";
            }
            crate::static_analysis::http_endpoints::CorePathSegment::LastSlash => {
                res += "/";
            }
        }
    }

    res += "\"";
    for segment in &path_args.required_args {
        if let crate::static_analysis::http_endpoints::CorePathSegment::Argument(n, t) = segment {
            match t {
                ValidHttpPrimitiveType::Int => {
                    res += ", ";
                    res += n;
                }
                ValidHttpPrimitiveType::Float => {
                    res += ", ";
                    res += n;
                }
                ValidHttpPrimitiveType::Bool => {
                    res += ", ";
                    res += n;
                }
                ValidHttpPrimitiveType::Text => {
                    res += ", ::urlencoding::encode(";
                    res += n;
                    res += ")"
                }
            }
        }
    }

    res += ")";

    res
}

fn lib_epl_url() -> &'static str {
    r#"
pub struct EplUrl {
    url: String,
}

impl EplUrl {
    /// Navigate in the same tab
    #[allow(dead_code)]
    pub fn navigate(&self) {
        let _ = ::gloo::utils::window().location().set_href(self.url.as_str());
    }

    /// Navigate in the new tab
    #[allow(dead_code)]
    pub fn navigate_new_tab(&self) {
        let _ = ::gloo::utils::window().open_with_url_and_target(self.url.as_str(), "_blank");
    }

    /// Get raw url
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }
}

impl yew::html::IntoPropValue<AttrValue> for EplUrl {
    fn into_prop_value(self) -> AttrValue {
        AttrValue::Rc(std::rc::Rc::from(self.url.clone()))
    }
}

impl yew::html::IntoPropValue<Option<AttrValue>> for EplUrl {
    fn into_prop_value(self) -> Option<AttrValue> {
        Some(AttrValue::Rc(std::rc::Rc::from(self.url.clone())))
    }
}
"#
}

fn rust_frontend_generate_page_and_link_urls(
    checked: &CheckedDB,
    app: TableRowPointerFrontendApplication,
) -> String {
    let mut res = String::new();

    let has_ext_pages = !checked
        .db
        .frontend_application()
        .c_children_frontend_application_external_page(app)
        .is_empty();
    let has_ext_links = !checked
        .db
        .frontend_application()
        .c_children_frontend_application_external_link(app)
        .is_empty();
    let epl_url_needed = has_ext_pages || has_ext_links;

    generate_frontend_to_frontend_pages(checked, app, &mut res);
    generate_frontend_to_backend_links(checked, app, &mut res);

    if epl_url_needed {
        res += lib_epl_url();
    }

    res
}

fn generate_frontend_to_frontend_pages(
    checked: &CheckedDB,
    app: TableRowPointerFrontendApplication,
    res: &mut String,
) {
    // All required args are listed and optional args are in struct?
    for ep in checked
        .db
        .frontend_application()
        .c_children_frontend_application_external_page(app)
    {
        let link_name = checked
            .db
            .frontend_application_external_page()
            .c_link_name(*ep);
        let frontend_page = checked
            .db
            .frontend_application_external_page()
            .c_frontend_page(*ep);
        let checked_page = checked
            .projections
            .checked_frontend_pages
            .value(frontend_page);
        let req_needed = !checked_page.required_args.is_empty();
        let opt_needed = !checked_page.opt_query_args.is_empty();
        let req_args_struct_name = format!("UrlP{}Args", link_name.to_case(Case::Pascal));
        let opt_args_struct_name = format!("UrlP{}OptArgs", link_name.to_case(Case::Pascal));

        let mut fun_args = Vec::new();

        if req_needed {
            fun_args.push(format!("required_args: {req_args_struct_name}"));
            *res += "pub struct ";
            *res += &req_args_struct_name;
            *res += " {\n";
            for ra in &checked_page.required_args {
                if let CorePathSegment::Argument(vn, vt) = ra {
                    *res += "    pub ";
                    *res += vn;
                    *res += ": ";
                    *res += http_type_to_rust_type(*vt);
                    *res += ",\n";
                }
            }
            *res += "}\n";
            *res += "\n";
        }
        if opt_needed {
            fun_args.push(format!("optional_args: {opt_args_struct_name}"));
            *res += "#[derive(Default)]\n";
            *res += "pub struct ";
            *res += &opt_args_struct_name;
            *res += " {\n";

            for (an, at) in &checked_page.opt_query_args {
                *res += "    pub ";
                *res += an;
                *res += ": ";
                if !at.is_multiple {
                    *res += "Option<";
                } else {
                    *res += "Vec<";
                }
                *res += http_type_to_rust_type(at.the_type);
                *res += ">,\n";
            }

            *res += "}\n";
            *res += "\n";
        }

        *res += "pub fn page_link_";
        *res += checked
            .db
            .frontend_application_external_page()
            .c_link_name(*ep);
        *res += "(";
        *res += &fun_args.join(", ");
        *res += ") -> EplUrl {\n";
        let maybe_mut = if req_needed || opt_needed { "mut" } else { "" };
        *res += &format!("    let {maybe_mut} url = get_endpoint_mapping(\"eplExtPagesMapping\", \"{link_name}\").unwrap_or_else(|| \"/\".to_string());\n");
        for (idx, pa) in checked_page.required_args.iter().enumerate() {
            if idx > 0 && !matches!(pa, CorePathSegment::LastSlash) {
                *res += "    url += \"/\";\n";
            }
            match pa {
                CorePathSegment::Text(t) => {
                    *res += "    url += \"";
                    *res += t;
                    *res += "\";\n";
                }
                CorePathSegment::Argument(n, t) => {
                    if matches!(t, ValidHttpPrimitiveType::Text) {
                        *res += "    url += &::urlencoding::encode(&required_args.";
                        *res += n;
                        *res += ");\n";
                    } else {
                        *res += "    url += &::urlencoding::encode(&format!(\"{}\", required_args.";
                        *res += n;
                        *res += "));\n";
                    }
                }
                CorePathSegment::LastSlash => {
                    *res += "    url += \"/\";\n";
                }
            }
        }

        if opt_needed {
            *res += "    let mut opt_added = false;\n";
            for (k, a) in &checked_page.opt_query_args {
                if a.is_multiple {
                    *res += "    for arg in &optional_args.";
                    *res += k;
                    *res += " {\n";
                    *res += "        #[allow(unused_assignments)]\n";
                    *res += "        if opt_added { url += \"&\" } else { opt_added = true; url += \"?\"; }\n";
                    *res += &format!("        url += \"{k}=\";\n");
                    if matches!(a.the_type, ValidHttpPrimitiveType::Text) {
                        *res += "        url += &::urlencoding::encode(&arg);\n";
                    } else {
                        *res += "        url += &::urlencoding::encode(&format!(\"{}\", arg));\n";
                    }
                    *res += "    }\n";
                } else {
                    *res += "    if let Some(arg) = &optional_args.";
                    *res += k;
                    *res += " {\n";
                    *res += "        #[allow(unused_assignments)]\n";
                    *res += "        if opt_added { url += \"&\" } else { opt_added = true; url += \"?\"; }\n";
                    *res += &format!("        url += \"{k}=\";\n");
                    if matches!(a.the_type, ValidHttpPrimitiveType::Text) {
                        *res += "        url += &::urlencoding::encode(&arg);\n";
                    } else {
                        *res += "        url += &::urlencoding::encode(&format!(\"{}\", arg));\n";
                    }
                    *res += "    }\n";
                }
            }
        }

        *res += "    EplUrl { url }\n";

        *res += "}\n";
        *res += "\n";
    }
}

fn generate_frontend_to_backend_links(
    checked: &CheckedDB,
    app: TableRowPointerFrontendApplication,
    res: &mut String,
) {
    for el in checked
        .db
        .frontend_application()
        .c_children_frontend_application_external_link(app)
    {
        let link_name = checked
            .db
            .frontend_application_external_link()
            .c_link_name(*el);
        let be = checked
            .db
            .frontend_application_external_link()
            .c_backend_endpoint(*el);
        let checked_endpoint = checked.projections.checked_http_endpoints.value(be);
        let req_needed = !checked_endpoint.path_args.required_args.is_empty();
        let opt_needed = !checked_endpoint.path_args.opt_query_args.is_empty();

        let req_args_struct_name = format!("UrlL{}Args", link_name.to_case(Case::Pascal));
        let opt_args_struct_name = format!("UrlL{}OptArgs", link_name.to_case(Case::Pascal));

        let mut fun_args = Vec::new();
        if req_needed {
            fun_args.push(format!("required_args: {req_args_struct_name}"));
            *res += "pub struct ";
            *res += &req_args_struct_name;
            assert!(checked_endpoint.input_body_type.is_none());
            *res += " {\n";
            for ra in &checked_endpoint.path_args.required_args {
                if let CorePathSegment::Argument(vn, vt) = ra {
                    *res += "    pub ";
                    *res += vn;
                    *res += ": ";
                    *res += http_type_to_rust_type(*vt);
                    *res += ",\n";
                }
            }
            *res += "}\n";
            *res += "\n";
        }
        if opt_needed {
            fun_args.push(format!("optional_args: {opt_args_struct_name}"));
            *res += "#[derive(Default)]\n";
            *res += "pub struct ";
            *res += &opt_args_struct_name;
            *res += " {\n";

            for (an, at) in &checked_endpoint.path_args.opt_query_args {
                *res += "    pub ";
                *res += an;
                *res += ": ";
                if !at.is_multiple {
                    *res += "Option<";
                } else {
                    *res += "Vec<";
                }
                *res += http_type_to_rust_type(at.the_type);
                *res += ">,\n";
            }

            *res += "}\n";
            *res += "\n";
        }

        *res += "pub fn backend_link_";
        *res += checked
            .db
            .frontend_application_external_link()
            .c_link_name(*el);
        *res += "(";
        *res += &fun_args.join(", ");
        *res += ") -> EplUrl {\n";

        let maybe_mut = if req_needed || opt_needed { "mut" } else { "" };
        *res += &format!("    let {maybe_mut} url = get_endpoint_mapping(\"eplExtLinksMapping\", \"{link_name}\").unwrap_or_else(|| \"/\".to_string());\n");

        for (idx, pa) in checked_endpoint.path_args.required_args.iter().enumerate() {
            if idx > 0 && !matches!(pa, CorePathSegment::LastSlash) {
                *res += "    url += \"/\";\n";
            }
            match pa {
                CorePathSegment::Text(t) => {
                    *res += "    url += \"";
                    *res += t;
                    *res += "\";\n";
                }
                CorePathSegment::Argument(n, t) => {
                    if matches!(t, ValidHttpPrimitiveType::Text) {
                        *res += "    url += &::urlencoding::encode(&required_args.";
                        *res += n;
                        *res += ");\n";
                    } else {
                        *res += "    url += &::urlencoding::encode(&format!(\"{}\", required_args.";
                        *res += n;
                        *res += "));\n";
                    }
                }
                CorePathSegment::LastSlash => {
                    *res += "    url += \"/\";\n";
                }
            }
        }

        if opt_needed {
            *res += "    let mut opt_added = false;\n";
            for (k, a) in &checked_endpoint.path_args.opt_query_args {
                if a.is_multiple {
                    *res += "    for arg in &optional_args.";
                    *res += k;
                    *res += " {\n";
                    *res += "        #[allow(unused_assignments)]\n";
                    *res += "        if opt_added { url += \"&\" } else { opt_added = true; url += \"?\"; }\n";
                    *res += &format!("        url += \"{k}=\";\n");
                    if matches!(a.the_type, ValidHttpPrimitiveType::Text) {
                        *res += "        url += &::urlencoding::encode(&arg);\n";
                    } else {
                        *res += "        url += &::urlencoding::encode(&format!(\"{}\", arg));\n";
                    }
                    *res += "    }\n";
                } else {
                    *res += "    if let Some(arg) = &optional_args.";
                    *res += k;
                    *res += " {\n";
                    *res += "        #[allow(unused_assignments)]\n";
                    *res += "        if opt_added { url += \"&\" } else { opt_added = true; url += \"?\"; }\n";
                    *res += &format!("        url += \"{k}=\";\n");
                    if matches!(a.the_type, ValidHttpPrimitiveType::Text) {
                        *res += "        url += &::urlencoding::encode(&arg);\n";
                    } else {
                        *res += "        url += &::urlencoding::encode(&format!(\"{}\", arg));\n";
                    }
                    *res += "    }\n";
                }
            }
        }

        *res += "    EplUrl { url }\n";

        *res += "}\n";
        *res += "\n";
    }
}
