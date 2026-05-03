use axum::{Router, extract::Request, routing::get};
use tower_http::trace::TraceLayer;
use tracing::info_span;

use handlers::ApiError;

pub fn internal_router() -> Result<Router, ApiError> {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request| {
            let method = request.method();
            let path = request.uri().path();
            let span = info_span!(
                "internal_http_request",
                otel.name = %format!("{method} {path}"),
                otel.kind = "server",
                http.request.method = %method,
                url.path = %path,
                http.response.status_code = tracing::field::Empty,
            );
            span
        })
        .on_response(
            |response: &http::Response<_>, _latency: std::time::Duration, span: &tracing::Span| {
                span.record("http.response.status_code", response.status().as_u16());
            },
        );

    let router = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .layer(trace_layer);

    Ok(router)
}

async fn health() -> &'static str {
    "ok"
}

async fn metrics() -> &'static str {
    ""
}
