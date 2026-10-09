use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
};

use crate::{
    api::ApiClient,
    tools::{MestierTools, bearer},
};

/// Every request is served on its own: no MCP session, so the token of one
/// request can never serve another. The server sits behind the gateway, so
/// the DNS-rebinding `Host` allow-list is off.
pub fn router(api: ApiClient) -> Router {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .disable_allowed_hosts();
    let service = StreamableHttpService::new(
        move || Ok(MestierTools::new(api.clone())),
        std::sync::Arc::new(NeverSessionManager::default()),
        config,
    );
    Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn(require_bearer))
        .route("/healthz", get(|| async { StatusCode::OK }))
}

async fn require_bearer(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    if bearer(&parts).is_none() {
        let mut response = StatusCode::UNAUTHORIZED.into_response();
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        return response;
    }
    next.run(Request::from_parts(parts, body)).await
}
