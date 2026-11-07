use args::{log::LogArgs, observability::ObservabilityArgs};
use opentelemetry::{
    global,
    propagation::{Extractor, Injector},
    trace::TracerProvider,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, SdkTracerProvider},
};
use tracing::{Span, info, info_span};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetrySpanExt};
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

    global::set_text_map_propagator(TraceContextPropagator::new());

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

struct HeaderReader<'a>(&'a http::HeaderMap);

impl Extractor for HeaderReader<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(http::HeaderName::as_str).collect()
    }
}

struct HeaderWriter<'a>(&'a mut http::HeaderMap);

impl Injector for HeaderWriter<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(name), Ok(value)) = (
            http::HeaderName::from_bytes(key.as_bytes()),
            http::HeaderValue::from_str(&value),
        ) {
            self.0.insert(name, value);
        }
    }
}

/// Makes `span` a child of the trace carried by the request's `traceparent`
/// header, if any. A no-op while observability is off.
pub fn set_remote_parent(span: &Span, headers: &http::HeaderMap) {
    let parent =
        global::get_text_map_propagator(|propagator| propagator.extract(&HeaderReader(headers)));
    let _ = span.set_parent(parent);
}

/// Writes the current span's `traceparent` into outgoing request headers.
pub fn inject_current_context(headers: &mut http::HeaderMap) {
    let context = Span::current().context();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut HeaderWriter(headers));
    });
}

/// The span of an incoming HTTP request, for `TraceLayer::make_span_with`.
pub fn server_span<B>(request: &http::Request<B>) -> Span {
    let method = request.method();
    let path = request.uri().path();
    let span = info_span!(
        "http_request",
        otel.name = %format!("{method} {path}"),
        otel.kind = "server",
        http.request.method = %method,
        url.path = %path,
        http.response.status_code = tracing::field::Empty,
    );
    set_remote_parent(&span, request.headers());
    span
}

/// Records the status on the span of `server_span`, for `TraceLayer::on_response`.
pub fn record_status<B>(response: &http::Response<B>, _latency: std::time::Duration, span: &Span) {
    span.record("http.response.status_code", response.status().as_u16());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_traceparent_header_survives_extract_and_inject() {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let mut incoming = http::HeaderMap::new();
        incoming.insert(
            "traceparent",
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
                .parse()
                .unwrap(),
        );
        let context = global::get_text_map_propagator(|p| p.extract(&HeaderReader(&incoming)));

        let mut outgoing = http::HeaderMap::new();
        global::get_text_map_propagator(|p| {
            p.inject_context(&context, &mut HeaderWriter(&mut outgoing))
        });

        let sent = outgoing.get("traceparent").unwrap().to_str().unwrap();
        assert!(
            sent.starts_with("00-0af7651916cd43dd8448eb211c80319c-"),
            "{sent}"
        );
    }

    #[test]
    fn nothing_is_injected_without_an_active_span() {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let mut headers = http::HeaderMap::new();
        inject_current_context(&mut headers);
        assert!(headers.is_empty());
    }
}
