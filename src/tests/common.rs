#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use crate::codegen::nixplan::nix_strip_for_diff;
#[cfg(test)]
use crate::static_analysis::CheckedDB;
#[cfg(test)]
use edendb::checker::errors::DatabaseValidationError;

#[cfg(test)]
use crate::codegen::CodegenPlan;

#[cfg(test)]
pub fn assert_eden_db_error_wcustom_data(source: &str) -> DatabaseValidationError {
    use edendb::checker::logic::AllData;
    use edendb::db_parser::InputSource;

    let input = &mut [
        InputSource {
            path: "main.edl".to_string(),
            contents: None,
            source_dir: Some("edb-src".to_string()),
            line_comments: Vec::new(),
        },
        InputSource {
            path: "test".to_string(),
            contents: Some(source.to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
        InputSource {
            path: "glob_flags".to_string(),
            contents: Some(custom_global_flags().to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
        InputSource {
            path: "aux_src".to_string(),
            contents: Some(aux_test_source().to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
    ];

    let parsed = edendb::db_parser::parse_sources_with_external(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    assert!(parsed.table_definitions().len() + parsed.table_data_segments().len() > 0);
    let all_data = AllData::new_with_flags(parsed, false);
    match all_data {
        Ok(_) => panic!("Expected EdenDB validation error, test passed"),
        Err(e) => e,
    }
}

#[cfg(test)]
pub struct TestArgs {
    pub add_default_global_flags: bool,
    pub add_default_data: bool,
}

#[cfg(test)]
impl Default for TestArgs {
    fn default() -> Self {
        Self { add_default_global_flags: true, add_default_data: true }
    }
}

#[cfg(test)]
pub fn assert_platform_validation_error_wcustom_data(
    source: &str,
) -> crate::static_analysis::PlatformValidationError {
    assert_platform_validation_error_wcustom_data_wargs(
        TestArgs::default(),
        source
    )
}

#[cfg(test)]
pub fn assert_platform_validation_error_wcustom_data_raw(
    source: &str,
) -> crate::static_analysis::PlatformValidationError {
    assert_platform_validation_error_wcustom_data_wargs(
        TestArgs {
            add_default_data: false,
            add_default_global_flags: false,
        },
        source
    )
}

#[cfg(test)]
pub fn assert_platform_validation_error_wcustom_data_wargs(
    args: TestArgs,
    source: &str,
) -> crate::static_analysis::PlatformValidationError {
    use edendb::checker::logic::AllData;
    use edendb::db_parser::InputSource;
    use std::sync::Arc;

    let mut input = vec![
        InputSource {
            path: "main.edl".to_string(),
            contents: None,
            source_dir: Some("edb-src".to_string()),
            line_comments: Vec::new(),
        },
        InputSource {
            path: "test".to_string(),
            contents: Some(source.to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
    ];

    if args.add_default_global_flags {
        input.push(
            InputSource {
                path: "glob_flags".to_string(),
                contents: Some(custom_global_flags().to_string()),
                source_dir: None,
                line_comments: Vec::new(),
            }
        );
    }
    if args.add_default_data {
        input.push(
            InputSource {
                path: "aux_src".to_string(),
                contents: Some(aux_test_source().to_string()),
                source_dir: None,
                line_comments: Vec::new(),
            }
        );
    }

    let parsed = edendb::db_parser::parse_sources_with_external(&mut input);
    if let Err(err) = parsed {
        panic!("{}", err);
    }
    let parsed = parsed.unwrap();
    assert!(parsed.table_definitions().len() + parsed.table_data_segments().len() > 0);
    let all_data = AllData::new_with_flags(parsed, false).unwrap();
    let vecs = all_data.serialization_vectors();
    let bytes = edendb::codegen::dump_as_bytes(&vecs);
    let res = crate::database::Database::deserialize(&bytes)
        .expect("Failed to deserialize dynamically generated data");
    match crate::static_analysis::run_static_checks(Arc::new(res)) {
        Ok(_) => panic!("Expected platform validation error, test passed"),
        Err(e) => e,
    }
}

#[cfg(test)]
pub fn assert_platform_validation_success_plain(source: &str) -> CheckedDB {
    use edendb::checker::logic::AllData;
    use edendb::db_parser::InputSource;
    use std::sync::Arc;

    let input = &mut [
        InputSource {
            path: "main.edl".to_string(),
            contents: None,
            source_dir: Some("edb-src".to_string()),
            line_comments: Vec::new(),
        },
        InputSource {
            path: "test".to_string(),
            contents: Some(source.to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
    ];

    let parsed = edendb::db_parser::parse_sources_with_external(input);
    if let Err(err) = parsed {
        panic!("{}", err);
    }
    let parsed = parsed.unwrap();
    assert!(parsed.table_definitions().len() + parsed.table_data_segments().len() > 0);
    let all_data = AllData::new_with_flags(parsed, false).unwrap();
    let vecs = all_data.serialization_vectors();
    let bytes = edendb::codegen::dump_as_bytes(&vecs);
    let res = crate::database::Database::deserialize(&bytes)
        .expect("Failed to deserialize dynamically generated data");
    match crate::static_analysis::run_static_checks(Arc::new(res)) {
        Ok(checked) => checked,
        Err(e) => {
            panic!(
                "Expected test to pass, got platform validation error: {:#?}",
                e
            )
        }
    }
}

#[cfg(test)]
pub fn assert_platform_validation_success(source: &str) -> CheckedDB {
    use edendb::checker::logic::AllData;
    use edendb::db_parser::InputSource;
    use std::sync::Arc;

    let input = &mut [
        InputSource {
            path: "main.edl".to_string(),
            contents: None,
            source_dir: Some("edb-src".to_string()),
            line_comments: Vec::new(),
        },
        InputSource {
            path: "test".to_string(),
            contents: Some(source.to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
        InputSource {
            path: "glob_flags".to_string(),
            contents: Some(custom_global_flags().to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
        InputSource {
            path: "aux_src".to_string(),
            contents: Some(aux_test_source().to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
    ];

    let parsed = edendb::db_parser::parse_sources_with_external(input);
    if let Err(err) = parsed {
        panic!("{}", err);
    }
    let parsed = parsed.unwrap();
    assert!(parsed.table_definitions().len() + parsed.table_data_segments().len() > 0);
    let all_data = AllData::new_with_flags(parsed, false).unwrap();
    let vecs = all_data.serialization_vectors();
    let bytes = edendb::codegen::dump_as_bytes(&vecs);
    let res = crate::database::Database::deserialize(&bytes)
        .expect("Failed to deserialize dynamically generated data");
    match crate::static_analysis::run_static_checks(Arc::new(res)) {
        Ok(checked) => checked,
        Err(e) => {
            panic!(
                "Expected test to pass, got platform validation error: {:#?}",
                e
            )
        }
    }
}

#[cfg(test)]
pub fn assert_platform_validation_success_wargs(args: TestArgs, source: &str) -> CheckedDB {
    use edendb::checker::logic::AllData;
    use edendb::db_parser::InputSource;
    use std::sync::Arc;

    let mut input = vec![
        InputSource {
            path: "main.edl".to_string(),
            contents: None,
            source_dir: Some("edb-src".to_string()),
            line_comments: Vec::new(),
        },
        InputSource {
            path: "test".to_string(),
            contents: Some(source.to_string()),
            source_dir: None,
            line_comments: Vec::new(),
        },
    ];

    if args.add_default_global_flags {
        input.push(
            InputSource {
                path: "glob_flags".to_string(),
                contents: Some(custom_global_flags().to_string()),
                source_dir: None,
                line_comments: Vec::new(),
            },
        );
    }

    if args.add_default_data {
        input.push(
            InputSource {
                path: "aux_src".to_string(),
                contents: Some(aux_test_source().to_string()),
                source_dir: None,
                line_comments: Vec::new(),
            },
        );
    }

    let parsed = edendb::db_parser::parse_sources_with_external(&mut input);
    if let Err(err) = parsed {
        panic!("{}", err);
    }
    let parsed = parsed.unwrap();
    assert!(parsed.table_definitions().len() + parsed.table_data_segments().len() > 0);
    let all_data = AllData::new_with_flags(parsed, false).unwrap();
    let vecs = all_data.serialization_vectors();
    let bytes = edendb::codegen::dump_as_bytes(&vecs);
    let res = crate::database::Database::deserialize(&bytes)
        .expect("Failed to deserialize dynamically generated data");
    match crate::static_analysis::run_static_checks(Arc::new(res)) {
        Ok(checked) => checked,
        Err(e) => {
            panic!(
                "Expected test to pass, got platform validation error: {:#?}",
                e
            )
        }
    }
}

#[test]
#[should_panic]
fn test_always_panic_when_defining_new_tables() {
    // serialization vectors should not match the schema
    // given that we cannot undefine tables from standard library
    // adding any new table will fail
    let _ = assert_platform_validation_error_wcustom_data(
        r#"
TABLE this_is_a_damage {
   id INT
}
"#,
    );
}

#[cfg(test)]
fn custom_global_flags() -> &'static str {
    r#"
DATA STRUCT global_settings {
    project_name: test-env,
    admin_email: admin@epl-infra.net,
    disable_consul_quorum_tests: true,
    disable_nomad_quorum_tests: true,
    disable_vault_quorum_tests: true,
    disable_dns_quorum_tests: true,
    disable_deployment_min_server_tests: true,
    disable_deployment_min_ingress_tests: true,
    disable_vpn_gateway_tests: true,
}
"#
}

#[cfg(test)]
fn aux_test_source() -> &'static str {
    r#"
DEFAULTS {
    server.dc dc1,
    datacenter.region us-west,
    datacenter.implementation manual,
    datacenter.default_server_kind testvm.cpu4ram8192,
    server_disk.disk_kind default-ssd,
    region.tld epl-infra.net,
    server.nixpkgs_environment default_nixpkgs,
    rust_compilation_environment.nixpkgs_environment default_nixpkgs,
    frontend_application_deployment.region us-west,
    backend_application_deployment.region us-west,
    grafana.region us-west,
    pg_deployment.region us-west,
    nats_cluster.region us-west,
    minio_cluster.region us-west,
    tempo_cluster.region us-west,
    monitoring_cluster.region us-west,
    loki_cluster.region us-west,
    ch_deployment.region us-west,
    ch_keeper_deployment.region us-west,
    blackbox_deployment.region us-west,
}

DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    architecture: x86_64,
}

DATA STRUCT disk_kind {
    kind: default-ssd,
    medium: ssd,
    capacity_bytes: 21474836480,
}

DATA region {
    us-west;
}

DATA STRUCT datacenter {
    dc_name: dc1,
    network_cidr: '10.17.0.0/16',
    allow_small_subnets: true,
}

DATA STRUCT EXCLUSIVE tld {
    domain: 'epl-infra.net',
    expose_admin: true,
}
"#
}

#[cfg(test)]
pub fn replace_sources(replacements: &[(&str, &str)], input: &str) -> String {
    assert!(replacements.len() > 0);
    let (find, replace) = &replacements[0];
    let mut res = input.replace(*find, *replace);
    assert!(res != input, "First replacement find [{find}] replace [{replace}] failed");
    for (find, replace) in &replacements[1..] {
        let new = res.replace(*find, *replace);
        assert!(res != new, "Replacement find [{find}] replace [{replace}] failed");
        res = new;
    }
    res
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    name: String,
    value: Option<String>,
}

#[cfg(test)]
impl Config {
    pub fn new(cfg_name: &str, cfg_output: Option<String>) -> Config {
        let value =
            cfg_output.map(|i| {
                nix_strip_for_diff(i.as_str())
            });
        Config { name: cfg_name.to_string(), value }
    }
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub struct ServerDescription {
    pub server: String,
    pub configs: Vec<Config>,
}

#[cfg(test)]
impl ServerDescription {
    pub fn new(server: &str, configs: Vec<Config>) -> ServerDescription {
        let mut existing: HashSet<&str> = HashSet::new();
        for cfg in &configs {
            let name = cfg.name.as_str();
            assert!(
                existing.insert(cfg.name.as_str()),
                "Config duplicate {name}",
            );
        }
        ServerDescription { server: server.to_string(), configs }
    }
}

#[cfg(test)]
pub fn ensure_config_plans(plan: &CodegenPlan, expected: Vec<ServerDescription>) {
    use crate::codegen::{find_server_config_region, nixplan::find_nix_region};

    let mut output_plans = Vec::with_capacity(expected.len());
    let mut existing: HashSet<&str> = HashSet::new();

    for exp in &expected {
        let name = exp.server.as_str();
        assert!(
            existing.insert(exp.server.as_str()),
            "Duplicate server {name}",
        );
        let mut actual_server_desc = ServerDescription {
            server: exp.server.clone(),
            configs: Vec::with_capacity(exp.configs.len()),
        };
        for conf in &exp.configs {
           let cfg_server = find_server_config_region(&exp.server, plan);
           let conf_res = find_nix_region(&conf.name, &cfg_server).map(|i| i.to_string());
           let this_desc = Config::new(&conf.name, conf_res);
           actual_server_desc.configs.push(this_desc);
        }
        output_plans.push(actual_server_desc);
    }

    pretty_assertions::assert_eq!(
        expected,
        output_plans,
    );
}
