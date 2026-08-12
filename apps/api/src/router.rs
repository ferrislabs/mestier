use axum::{Router, extract::Request};
use http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info_span;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

use handlers::{ApiError, AppState};
use handlers_automation as automation;
use handlers_customer as customer;
use handlers_discord as discord;
use handlers_files as files;
use handlers_member as member;
use handlers_organization as organization;
use handlers_planning as planning;
use handlers_quote as quote;
use handlers_reference as reference;

use crate::openapi::ApiDoc;

pub fn router(state: AppState) -> Result<Router, ApiError> {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request| {
            let method = request.method();
            let path = request.uri().path();
            let span = info_span!(
                "http_request",
                otel.name = %format!("{method} {path}"),
                otel.kind = "server",
                http.request.method = %method,
                url.path = %path,
                url.query = tracing::field::Empty,
                http.response.status_code = tracing::field::Empty,
            );
            if let Some(query) = request.uri().query() {
                span.record("url.query", query);
            }
            span
        })
        .on_response(
            |response: &http::Response<_>, _latency: std::time::Duration, span: &tracing::Span| {
                span.record("http.response.status_code", response.status().as_u16());
            },
        );

    let openapi = ApiDoc::openapi();

    let mut allowed_origins: Vec<HeaderValue> =
        vec![HeaderValue::from_static("http://localhost:3000")];
    allowed_origins.extend(
        state
            .args
            .server
            .allowed_origins
            .iter()
            .filter_map(|origin| HeaderValue::from_str(origin).ok()),
    );

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PUT,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_origin(allowed_origins)
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            CONTENT_LENGTH,
            ACCEPT,
            LOCATION,
        ])
        .allow_credentials(true);

    let router = Router::new()
        .merge(automation::router(&state))
        .merge(files::router(&state))
        .merge(customer::router(&state))
        .merge(discord::router(&state))
        .merge(member::router(&state))
        .merge(organization::router(&state))
        .merge(quote::router(&state))
        .merge(planning::router(&state))
        .merge(reference::router(&state))
        .merge(Scalar::with_url("/scalar", openapi.clone()))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi.clone()))
        .layer(trace_layer)
        .layer(cors)
        .with_state(state);

    Ok(router)
}
