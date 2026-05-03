#[derive(clap::Args, Debug, Clone)]
pub struct ObservabilityArgs {
    #[arg(
        long = "active-observability",
        env = "ACTIVE_OBSERVABILITY",
        name = "ACTIVE_OBSERVABILITY",
        default_value_t = false,
        long_help = "Whether to enable observability features like tracing and metrics",
        required = false
    )]
    pub active_observability: bool,
    #[arg(
        short = 'O',
        long = "otlp-endpoint",
        env = "OTLP_ENDPOINT",
        name = "OTLP_ENDPOINT",
        long_help = "The endpoint for the traces collector",
        required = false
    )]
    pub otlp_endpoint: Option<String>,
    #[arg(
        short = 'M',
        long = "metrics-endpoint",
        env = "METRICS_ENDPOINT",
        name = "METRICS_ENDPOINT",
        long_help = "The endpoint for the metrics collector",
        required = false
    )]
    pub metrics_endpoint: Option<String>,
}

impl Default for ObservabilityArgs {
    fn default() -> Self {
        Self {
            active_observability: false,
            otlp_endpoint: Some("http://localhost:4317".to_string()),
            metrics_endpoint: Some("http://localhost:4317".to_string()),
        }
    }
}
