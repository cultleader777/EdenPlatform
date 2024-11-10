use clap::{Parser, clap_derive::ArgEnum};

use crate::codegen::secrets::SecretKind;

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
        kind: SecretKind,
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
    /// target/release/eden_platform refresh-prometheus-metrics --prometheus-url http://10.17.0.10:9090 > edb-src/metrics-db.yml
    RefreshPrometheusMetrics {
        /// Prometheus URL
        #[clap(required = true, long)]
        prometheus_url: String,
    },
    /// Dump
    Dump {
        /// What kind of data to dump
        #[clap(arg_enum, long)]
        what: DataToDump,
    },
}

pub fn get_args() -> Cli {
    Cli::parse()
}
