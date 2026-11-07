use axum::{Router, routing::get};

use handlers::ApiError;

// No trace layer: this router only serves the kubelet probes and the metrics
// scrape, which would otherwise fill the traces with one span every few seconds.
pub fn internal_router() -> Result<Router, ApiError> {
    Ok(Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics)))
}

async fn health() -> &'static str {
    "ok"
}

async fn metrics() -> &'static str {
    ""
}
