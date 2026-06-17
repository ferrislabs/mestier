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

use handlers::ApiError;

pub fn init_tracing_and_logging(
    log_args: &LogArgs,
    service_name: &str,
    observability_args: &ObservabilityArgs,
) -> Result<(), ApiError> {
    let filter = EnvFilter::try_new(&log_args.filter).unwrap_or_else(|err| {
        eprint!("invalid log filter: {err}");
        eprint!("using default log filter: info");
        EnvFilter::new("info")
    });

    let fmt_layer = if log_args.json {
        fmt::layer().with_writer(std::io::stderr).json().boxed()
    } else {
        fmt::layer().with_writer(std::io::stderr).boxed()
    };

    if observability_args.active_observability {
        let otlp_endpoint = observability_args
            .otlp_endpoint
            .as_ref()
            .ok_or(ApiError::Internal)?;

        let metrics_endpoints = observability_args
            .metrics_endpoint
            .as_ref()
            .ok_or(ApiError::Internal)?;

        let resource = Resource::builder()
            .with_service_name(service_name.to_string())
            .build();

        let span_exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build()
            .map_err(|_| ApiError::Internal)?;

        let trace_provider = SdkTracerProvider::builder()
            .with_resource(
                Resource::builder()
                    .with_service_name(service_name.to_string())
                    .build(),
            )
            .with_id_generator(RandomIdGenerator::default())
            .with_batch_exporter(span_exporter)
            .build();

        let tracer = trace_provider.tracer(service_name.to_string());
        let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        let metric_exporter = MetricExporter::builder()
            .with_tonic()
            .with_protocol(Protocol::Grpc)
            .with_endpoint(metrics_endpoints)
            .build()
            .map_err(|_| ApiError::Internal)?;

        let meter_provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder()
                    .with_service_name(service_name.to_string())
                    .build(),
            )
            .with_periodic_exporter(metric_exporter)
            .build();

        let metrics_layer = MetricsLayer::new(meter_provider);

        let log_exporter = LogExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build()
            .map_err(|_| ApiError::Internal)?;

        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource)
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

        std::mem::forget(logger_provider);

        info!(
            service = service_name,
            otlp_endpoint = %otlp_endpoint,
            metrics_endpoint = %metrics_endpoints,
            log_filter = %log_args.filter,
            json = log_args.json,
            "observability enabled: exporting traces, metrics and logs via OTLP/gRPC"
        );
    } else {
        let subscriber = Registry::default().with(fmt_layer).with(filter);

        subscriber.init();

        info!(
            service = service_name,
            log_filter = %log_args.filter,
            json = log_args.json,
            "observability disabled: set ACTIVE_OBSERVABILITY=true with OTLP_ENDPOINT and METRICS_ENDPOINT to enable OTLP export"
        );
    }

    Ok(())
}
