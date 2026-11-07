use args::{log::LogArgs, observability::ObservabilityArgs};
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::SdkMeterProvider,
    trace::{RandomIdGenerator, SdkTracerProvider},
};
use tracing::info;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{
    EnvFilter, Layer, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("{0} is required when ACTIVE_OBSERVABILITY is true")]
    MissingEndpoint(&'static str),
    #[error("cannot build the {signal} exporter: {reason}")]
    Exporter {
        signal: &'static str,
        reason: String,
    },
}

fn exporter_error(
    signal: &'static str,
) -> impl FnOnce(opentelemetry_otlp::ExporterBuildError) -> TelemetryError {
    move |error| TelemetryError::Exporter {
        signal,
        reason: error.to_string(),
    }
}

pub fn init(
    log_args: &LogArgs,
    service_name: &str,
    observability_args: &ObservabilityArgs,
) -> Result<(), TelemetryError> {
    let filter = EnvFilter::try_new(&log_args.filter).unwrap_or_else(|err| {
        eprintln!("invalid log filter: {err}");
        eprintln!("using default log filter: info");
        EnvFilter::new("info")
    });

    let fmt_layer = if log_args.json {
        fmt::layer().with_writer(std::io::stderr).json().boxed()
    } else {
        fmt::layer().with_writer(std::io::stderr).boxed()
    };

    if !observability_args.active_observability {
        Registry::default().with(fmt_layer).with(filter).init();
        info!(
            service = service_name,
            log_filter = %log_args.filter,
            json = log_args.json,
            "observability disabled: set ACTIVE_OBSERVABILITY=true with OTLP_ENDPOINT and METRICS_ENDPOINT to enable OTLP export"
        );
        return Ok(());
    }

    let otlp_endpoint = observability_args
        .otlp_endpoint
        .as_ref()
        .ok_or(TelemetryError::MissingEndpoint("OTLP_ENDPOINT"))?;
    let metrics_endpoint = observability_args
        .metrics_endpoint
        .as_ref()
        .ok_or(TelemetryError::MissingEndpoint("METRICS_ENDPOINT"))?;

    let resource = || {
        Resource::builder()
            .with_service_name(service_name.to_string())
            .build()
    };

    let span_exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .map_err(exporter_error("trace"))?;
    let trace_provider = SdkTracerProvider::builder()
        .with_resource(resource())
        .with_id_generator(RandomIdGenerator::default())
        .with_batch_exporter(span_exporter)
        .build();
    let trace_layer =
        tracing_opentelemetry::layer().with_tracer(trace_provider.tracer(service_name.to_string()));

    let metric_exporter = MetricExporter::builder()
        .with_tonic()
        .with_protocol(Protocol::Grpc)
        .with_endpoint(metrics_endpoint)
        .build()
        .map_err(exporter_error("metric"))?;
    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource())
        .with_periodic_exporter(metric_exporter)
        .build();
    let metrics_layer = MetricsLayer::new(meter_provider);

    let log_exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .map_err(exporter_error("log"))?;
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource())
        .with_batch_exporter(log_exporter)
        .build();
    let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    Registry::default()
        .with(fmt_layer)
        .with(trace_layer)
        .with(metrics_layer)
        .with(otel_log_layer)
        .with(filter)
        .init();

    // The bridge only borrows the provider; dropping it would stop the log export.
    std::mem::forget(logger_provider);

    info!(
        service = service_name,
        otlp_endpoint = %otlp_endpoint,
        metrics_endpoint = %metrics_endpoint,
        log_filter = %log_args.filter,
        json = log_args.json,
        "observability enabled: exporting traces, metrics and logs via OTLP/gRPC"
    );
    Ok(())
}
