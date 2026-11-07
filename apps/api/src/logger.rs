use args::{log::LogArgs, observability::ObservabilityArgs};
use handlers::ApiError;

pub fn init_tracing_and_logging(
    log_args: &LogArgs,
    service_name: &str,
    observability_args: &ObservabilityArgs,
) -> Result<(), ApiError> {
    telemetry::init(log_args, service_name, observability_args).map_err(|error| {
        eprintln!("telemetry: {error}");
        ApiError::Internal
    })
}
