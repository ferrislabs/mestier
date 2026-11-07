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
use handlers_field as field;
use handlers_files as files;
use handlers_invoice as invoice;
use handlers_member as member;
use handlers_organization as organization;
use handlers_planning as planning;
use handlers_purchase as purchase;
use handlers_quote as quote;
use handlers_reference as reference;
use handlers_reporting as reporting;
use handlers_support as support;

use crate::openapi::ApiDoc;

pub fn router(state: AppState) -> Result<Router, ApiError> {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(request_span)
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
        .merge(field::router(&state))
        .merge(files::router(&state))
        .merge(customer::router(&state))
        .merge(discord::router(&state))
        .merge(invoice::router(&state))
        .merge(member::router(&state))
        .merge(organization::router(&state))
        .merge(purchase::router(&state))
        .merge(quote::router(&state))
        .merge(reporting::router(&state))
        .merge(planning::router(&state))
        .merge(support::router(&state))
        .merge(reference::router(&state))
        .merge(Scalar::with_url("/scalar", openapi.clone()))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi.clone()))
        .layer(trace_layer)
        .layer(cors)
        .with_state(state);

    Ok(router)
}

const WEBHOOK_PREFIX: &str = "/webhooks";
const REDACTED_WEBHOOK_PATH: &str = "/webhooks/{redacted}";

fn is_webhook_path(path: &str) -> bool {
    path == WEBHOOK_PREFIX
        || path
            .strip_prefix(WEBHOOK_PREFIX)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn request_span(request: &Request) -> tracing::Span {
    let method = request.method();
    let redacted = is_webhook_path(request.uri().path());
    let path = if redacted {
        REDACTED_WEBHOOK_PATH
    } else {
        request.uri().path()
    };
    let span = info_span!(
        "http_request",
        otel.name = %format!("{method} {path}"),
        otel.kind = "server",
        http.request.method = %method,
        url.path = %path,
        url.query = tracing::field::Empty,
        http.response.status_code = tracing::field::Empty,
    );
    if !redacted && let Some(query) = request.uri().query() {
        span.record("url.query", query);
    }
    span
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::body::Body;
    use tracing_subscriber::fmt::MakeWriter;

    use super::*;

    #[derive(Clone, Default)]
    struct Captured(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for Captured {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Captured {
        type Writer = Captured;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn traced(uri: &str) -> String {
        let captured = Captured::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(captured.clone())
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let request = http::Request::builder()
                .method("POST")
                .uri(uri)
                .body(Body::empty())
                .unwrap();
            let span = request_span(&request);
            let _entered = span.enter();
            tracing::info!("handled");
        });
        let bytes = captured.0.lock().unwrap().clone();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn a_webhook_call_is_traced_without_its_path_or_its_query() {
        for uri in [
            "/webhooks/paiement/recu-secret-7f3a",
            "/webhooks/paiement-secret-7f3a?token=hunter2",
            "/webhooks/%ff-secret-7f3a",
        ] {
            let output = traced(uri);

            assert!(output.contains("/webhooks/{redacted}"), "{uri}: {output}");
            assert!(!output.contains("secret-7f3a"), "{uri}: {output}");
            assert!(!output.contains("hunter2"), "{uri}: {output}");
        }
    }

    #[test]
    fn the_bare_webhook_prefix_is_redacted_too() {
        for uri in ["/webhooks", "/webhooks/"] {
            let output = traced(uri);

            assert!(output.contains("/webhooks/{redacted}"), "{uri}: {output}");
        }
    }

    #[test]
    fn every_other_call_keeps_its_path_and_its_query() {
        let output = traced("/api/v1/organizations/abc/automation/workflows?archived=true");

        assert!(
            output.contains("/api/v1/organizations/abc/automation/workflows"),
            "{output}"
        );
        assert!(output.contains("archived=true"), "{output}");
    }

    #[test]
    fn a_path_that_only_starts_like_the_webhook_prefix_is_not_redacted() {
        let output = traced("/webhooksfoo/bar");

        assert!(output.contains("/webhooksfoo/bar"), "{output}");
    }
}
