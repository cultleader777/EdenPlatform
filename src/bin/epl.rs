use std::sync::Arc;

use edendb::{db_parser::InputSource, checker::logic::AllData};
use epl::{static_analysis::bench_start, codegen::MetricScrapePlan};
use colored::*;

use clap::{Parser, clap_derive::ArgEnum};

#[derive(ArgEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum DataToDump {
    /// AWS instance types in eden data language format
    AwsInstanceTypes,
    /// Google cloud instance types in eden data language format
    GcloudInstanceTypes,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub enum Cli {
    /// Compile the project
    Compile {
        /// Project output directory
        #[clap(required = true, long)]
        output_directory: String,

        /// Input sources to compile and check
        #[clap(required = true, min_values(1))]
        inputs: Vec<String>,
    },
    /// Override secret
    OverrideSecret {
        /// Project output directory
        #[clap(required = true, long)]
        output_directory: String,
        /// Key of the secret
        #[clap(required = true, long)]
        key: String,
        /// Value of the secret
        #[clap(required = true, long)]
        // yeah, passing secret as command line flag is insecure, but should only be done in admin machine
        value: String,
        /// Secret kind
        #[clap(required = true, long)]
        kind: epl::codegen::secrets::SecretKind,
    },
    OverrideSecretWithFile {
        /// Project output directory
        #[clap(required = true, long)]
        output_directory: String,
        /// Key of the secret
        #[clap(required = true, long)]
        key: String,
        /// FIle with the value of the secret
        #[clap(required = true, long)]
        // yeah, passing secret as command line flag is insecure, but should only be done in admin machine
        value_file: String,
        /// Secret kind
        #[clap(required = true, long)]
        kind: epl::codegen::secrets::SecretKind,
    },
    /// Get secret
    GetSecret {
        /// Project output directory
        #[clap(required = true, long)]
        output_directory: String,
        /// Key of the secret
        #[clap(required = true, long)]
        key: String,
    },
    /// Refresh infrastructure metrics.
    /// To refresh existing database from prometheus running at 10.17.0.10:9090:
    /// target/release/eden_platform refresh-prometheus-metrics --prometheus-url cluster-a,http://10.17.0.10:9090 > edb-src/metrics-db.yml
    RefreshPrometheusMetrics {
        /// Prometheus URL
        #[clap(required = true, long)]
        prometheus_url: Vec<String>,
    },
    /// Scrape prometheus metrics from the plan and dump to sqlite database
    ScrapePrometheusMetrics {
        /// Plan yml file
        plan_file: String,
        /// Sqlite database to dump to
        sqlite_db_dump: String,
    },
    /// Dump
    Dump {
        /// What kind of data to dump
        #[clap(arg_enum, long)]
        what: DataToDump,
    },
}

fn get_args() -> Cli {
    Cli::parse()
}

fn main() {
    if !std::env::var("EPL_SHELL").is_ok() {
        eprintln!("We assume to run only in eden platform shell.");
        std::process::exit(7);
    }

    sodiumoxide::init().expect("Can't initialize sodiumoxide");

    let cli = get_args();

    match cli {
        Cli::Compile { output_directory, inputs } => {
            let precompiled_bytes = include_bytes!("source_schema.bin");
            let total = bench_start("Total compilation time");

            let bench = bench_start("Precompile deserialization");
            let mut precompiled_sources =
                match edendb::db_parser::deserialize_source_outputs(precompiled_bytes) {
                    Ok(ok) => ok,
                    Err(e) => {
                        err_print("Precompile deserialization error", e.as_ref());
                    }
                };
            bench.end();

            let mut input = inputs.iter().map(|i| {
                InputSource {
                    path: i.clone(),
                    contents: None,
                    source_dir: None,
                    line_comments: Vec::new(),
                }
            }).collect::<Vec<_>>();


            let bench = bench_start("EDB parse");
            if let Err(e) = precompiled_sources.parse_into_external(input.as_mut_slice()) {
                err_print("platform sources parsing error", e.as_ref());
            }
            bench.end();

            let bench = bench_start("EDB checks");
            let all_data = match AllData::new_with_flags(precompiled_sources, true) {
                Ok(verified) => verified,
                Err(e) => {
                    err_print("platform data verification error", &e);
                }
            };
            bench.end();

            let bench = bench_start("EDB deserialize");
            let vecs = all_data.serialization_vectors();
            let bytes = edendb::codegen::dump_as_bytes(&vecs);
            let db = match epl::database::Database::deserialize(&bytes) {
                Ok(deserialized) => Arc::new(deserialized),
                Err(_) => {
                    eprintln!("{}", "deserialization error, definition of new tables is now allowed in the inputs.".red());
                    std::process::exit(7);
                }
            };
            bench.end();

            let checked = match epl::static_analysis::run_static_checks(db) {
                Ok(ok) => ok,
                Err(e) => {
                    err_print("eden platform error", &e);
                },
            };

            std::fs::create_dir_all(&output_directory).expect("Cannot create output directory");
            let secrets_path = format!("{}/secrets.yml", output_directory);
            let checksums_path = format!("{}/secrets-checksums.yml", output_directory);
            let mut secrets = match epl::codegen::secrets::SecretsStorage::new(&secrets_path, &checksums_path) {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("secrets error: {}", e);
                    std::process::exit(7);
                }
            };

            let bench = bench_start("Codegen plan in memory");
            let code_plan = epl::codegen::generate_outputs(&checked, &mut secrets);
            bench.end();
            let bench = bench_start("Codegen write to disk");
            epl::codegen::write_outputs_to_disk(&output_directory, &code_plan);
            bench.end();

            if let Err(e) = secrets.flush_to_disk() {
                eprintln!("{}: {}", err_color("secrets saving error"), e);
                std::process::exit(7);
            }

            // we do this after flushing code to disk because makefile might not be generated
            // to conveniently store secrets yet and couldn't be used
            if let Err(err) = epl::codegen::secrets::check_secrets_are_defined(&checked, &secrets) {
                eprintln!("undefined secrets error: {}", err);
                std::process::exit(7);
            }

            total.end();
        },
        Cli::OverrideSecret { key, value, kind, output_directory } => {
            let secrets_path = format!("{}/secrets.yml", output_directory);
            let checksums_path = format!("{}/secrets-checksums.yml", output_directory);
            let mut secrets = match epl::codegen::secrets::SecretsStorage::new(&secrets_path, &checksums_path) {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{}: {}", err_color("secrets error"), e);
                    std::process::exit(7);
                }
            };

            secrets.override_secret(key, kind, value);

            if let Err(e) = secrets.flush_to_disk() {
                eprintln!("{}: {}", err_color("secrets saving error"), e);
                std::process::exit(7);
            }
        },
        Cli::OverrideSecretWithFile { key, value_file, kind, output_directory } => {
            let secrets_path = format!("{}/secrets.yml", output_directory);
            let checksums_path = format!("{}/secrets-checksums.yml", output_directory);
            let mut secrets = match epl::codegen::secrets::SecretsStorage::new(&secrets_path, &checksums_path) {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{}: {}", err_color("secrets error"), e);
                    std::process::exit(7);
                }
            };

            let value = String::from_utf8(
                std::fs::read(&value_file).expect("Can't read the file")
            ).expect("invalid utf-8 value");

            secrets.override_secret(key, kind, value);

            if let Err(e) = secrets.flush_to_disk() {
                eprintln!("{}: {}", err_color("secrets saving error"), e);
                std::process::exit(7);
            }
        },
        Cli::GetSecret { output_directory, key } => {
            let secrets_path = format!("{}/secrets.yml", output_directory);
            let checksums_path = format!("{}/secrets-checksums.yml", output_directory);
            let secrets = match epl::codegen::secrets::SecretsStorage::new(&secrets_path, &checksums_path) {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{}: {}", err_color("secrets error"), e);
                    std::process::exit(7);
                }
            };

            match secrets.get_secret(&key) {
                Some(v) => {
                    print!("{}", v.value())
                },
                None => {
                    panic!("Cannot find secret with key {} in secrets storage", key);
                },
            }
        },
        Cli::RefreshPrometheusMetrics { prometheus_url } => {
            epl::prom_metrics_dump::refresh_prometheus_metrics(&prometheus_url);
        },
        Cli::ScrapePrometheusMetrics { plan_file, sqlite_db_dump } => {
            let plan = std::fs::read(plan_file).expect("Can't read metrics db file");
            let plan: Vec<MetricScrapePlan> = serde_yaml::from_slice(&plan)
                .expect("Can't deserialize metrics scrape plan");
            epl::prom_metrics_dump::dump_prometheus_metrics(&plan, &sqlite_db_dump);
        }
        Cli::Dump { what } => {
            match what {
                DataToDump::AwsInstanceTypes => {
                    print!("{}", epl::static_analysis::dc_impl::aws::AWS_INSTANCE_TYPES_EDL_SOURCE.as_str());
                }
                DataToDump::GcloudInstanceTypes => {
                    print!("{}", epl::static_analysis::dc_impl::gcloud::GCLOUD_INSTANCE_TYPES_EDL_SOURCE.as_str());
                }
            }
        }
    }
}

fn err_color(input: &str) -> String {
    input.white()
         .bold()
         .on_red()
         .to_string()
}

fn err_print(prefix: &'static str, e: &dyn std::error::Error) -> ! {
    let out = format!("{}: {:#?}", err_color(prefix), e);
    let repl = out.replace("\\n", "\n").replace("\\\"", "\"");
    eprintln!("{}", repl);
    std::process::exit(7);
}
