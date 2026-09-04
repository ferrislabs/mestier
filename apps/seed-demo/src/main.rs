mod config;
mod error;
mod seed;

use std::process::ExitCode;

use clap::Parser;
use config::SeedArgs;
use dotenvy::dotenv;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    dotenv().ok();
    let args = SeedArgs::parse();

    init_tracing(&args.log.filter, args.log.json);

    match seed::run(&args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = %err, "seed-demo failed");
            ExitCode::FAILURE
        }
    }
}

/// A stripped-down version of `apps/api`'s `init_tracing_and_logging`: same
/// filter/JSON switch, no OTLP export — a one-shot Job has no long-running
/// process for a trace/metrics pipeline to attach to.
fn init_tracing(filter: &str, json: bool) {
    let filter = EnvFilter::try_new(filter).unwrap_or_else(|err| {
        eprintln!("invalid log filter `{filter}`: {err}, falling back to `info`");
        EnvFilter::new("info")
    });

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);

    if json {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}
