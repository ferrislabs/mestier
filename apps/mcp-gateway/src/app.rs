use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use mcp_tools::{CLIENT_HEADER, SCOPE_DOE_WRITE, SCOPE_READ};
use reqwest::Client;
use serde_json::json;
use tracing::{info, warn};
use url::Url;

use crate::{
    exchange::TokenExchanger,
    pdp::PolicyDecisionPoint,
    rpc::{self, Rejection},
    verifier::{Principal, TokenVerifier, VerifyError},
};

/// Room for a 5 MB file sent as base64 to `add_doe_file` (about 6.7 MB), the
/// same ceiling as the MCP server behind.
const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
const METADATA_PATH: &str = "/.well-known/oauth-protected-resource";

const FORWARDED_REQUEST_HEADERS: [&str; 5] = [
    "content-type",
    "accept",
    "last-event-id",
    "mcp-session-id",
    "mcp-protocol-version",
];

const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

pub struct Gateway<V, P, X> {
    pub verifier: V,
    pub pdp: P,
    pub exchanger: X,
    pub http: Client,
    pub resource_url: String,
    pub issuer: String,
    pub upstream_url: String,
}

pub struct AppState<V, P, X>(Arc<Gateway<V, P, X>>);

impl<V, P, X> Clone for AppState<V, P, X> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<V, P, X> AppState<V, P, X> {
    pub fn new(gateway: Gateway<V, P, X>) -> Self {
        Self(Arc::new(gateway))
    }

    fn metadata_url(&self) -> String {
        Url::parse(&self.0.resource_url)
            .ok()
            .map(|u| format!("{}{METADATA_PATH}", u.origin().ascii_serialization()))
            .unwrap_or_else(|| format!("{}{METADATA_PATH}", self.0.resource_url))
    }
}

pub fn router<V, P, X>(state: AppState<V, P, X>) -> Router
where
    V: TokenVerifier + Send + Sync + 'static,
    P: PolicyDecisionPoint + Send + Sync + 'static,
    X: TokenExchanger + Send + Sync + 'static,
{
    Router::new()
        .route(METADATA_PATH, get(metadata::<V, P, X>))
        .route(
            &format!("{METADATA_PATH}/{{*rest}}"),
            get(metadata::<V, P, X>),
        )
        .route("/mcp", axum::routing::any(mcp::<V, P, X>))
        .with_state(state)
}

async fn metadata<V, P, X>(State(state): State<AppState<V, P, X>>) -> Response {
    Json(json!({
        "resource": state.0.resource_url,
        "authorization_servers": [state.0.issuer],
        "scopes_supported": [SCOPE_READ, SCOPE_DOE_WRITE],
        "bearer_methods_supported": ["header"],
    }))
    .into_response()
}

fn unauthorized(metadata_url: &str, invalid_token: bool) -> Response {
    let challenge = if invalid_token {
        format!("Bearer error=\"invalid_token\", resource_metadata=\"{metadata_url}\"")
    } else {
        format!("Bearer resource_metadata=\"{metadata_url}\"")
    };
    let mut response = StatusCode::UNAUTHORIZED.into_response();
    if let Ok(value) = HeaderValue::from_str(&challenge) {
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, value);
    }
    response
}

fn generic(status: StatusCode, message: &'static str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

fn rejection_response(rejection: &Rejection) -> Response {
    let status = StatusCode::from_u16(rejection.http_status).unwrap_or(StatusCode::OK);
    (status, Json(rejection.body())).into_response()
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty()).then(|| token.trim())
}

async fn mcp<V, P, X>(State(state): State<AppState<V, P, X>>, request: Request) -> Response
where
    V: TokenVerifier + Send + Sync,
    P: PolicyDecisionPoint + Send + Sync,
    X: TokenExchanger + Send + Sync,
{
    let started = Instant::now();
    let (parts, body) = request.into_parts();
    let metadata_url = state.metadata_url();

    let Some(token) = bearer(&parts.headers) else {
        return unauthorized(&metadata_url, false);
    };
    let principal = match state.0.verifier.verify(token).await {
        Ok(principal) => principal,
        Err(VerifyError::Invalid) => return unauthorized(&metadata_url, true),
        Err(VerifyError::Unavailable) => {
            return generic(StatusCode::SERVICE_UNAVAILABLE, "service unavailable");
        }
    };

    let Ok(body) = to_bytes(body, MAX_BODY_BYTES).await else {
        return generic(StatusCode::PAYLOAD_TOO_LARGE, "request too large");
    };

    let calls = if parts.method == http::Method::POST {
        match rpc::inspect(&body) {
            Ok(calls) => calls,
            Err(rejection) => {
                info!(sub = %principal.sub, azp = %principal.client_id, decision = "rejected", code = rejection.code);
                return rejection_response(&rejection);
            }
        }
    } else {
        Vec::new()
    };

    for call in &calls {
        let allowed = state
            .0
            .pdp
            .decide(&principal, call.tool, call.organization.as_deref())
            .await;
        let decision = match allowed {
            Ok(true) => "allow",
            Ok(false) => "deny",
            Err(_) => "pdp_error",
        };
        info!(
            sub = %principal.sub,
            azp = %principal.client_id,
            tool = %call.tool,
            decision,
            latency_ms = started.elapsed().as_millis() as u64,
        );
        if !matches!(allowed, Ok(true)) {
            return rejection_response(&Rejection::denied(call.id.clone()));
        }
    }

    let delegated = match state.0.exchanger.exchange(&principal, token).await {
        Ok(delegated) => delegated,
        Err(_) => return generic(StatusCode::BAD_GATEWAY, "upstream unavailable"),
    };

    match forward(
        &state.0,
        &parts.method,
        &parts.headers,
        body,
        &principal,
        &delegated.access_token,
    )
    .await
    {
        Ok(response) => {
            info!(
                sub = %principal.sub,
                azp = %principal.client_id,
                upstream_status = response.status().as_u16(),
                latency_ms = started.elapsed().as_millis() as u64,
            );
            response
        }
        Err(()) => generic(StatusCode::BAD_GATEWAY, "upstream unavailable"),
    }
}

async fn forward<V, P, X>(
    gateway: &Gateway<V, P, X>,
    method: &http::Method,
    headers: &HeaderMap,
    body: bytes::Bytes,
    principal: &Principal,
    delegated: &str,
) -> Result<Response, ()> {
    let mut outgoing = reqwest::header::HeaderMap::new();
    for (name, value) in headers {
        let name_str = name.as_str();
        if FORWARDED_REQUEST_HEADERS.contains(&name_str) || name_str.starts_with("mcp-") {
            outgoing.append(name.clone(), value.clone());
        }
    }
    let mut bearer = HeaderValue::from_str(&format!("Bearer {delegated}")).map_err(|_| ())?;
    bearer.set_sensitive(true);
    outgoing.insert(header::AUTHORIZATION, bearer);
    outgoing.insert(
        HeaderName::from_static(CLIENT_HEADER),
        HeaderValue::from_str(&principal.client_id).map_err(|_| ())?,
    );

    let upstream = gateway
        .http
        .request(method.clone(), &gateway.upstream_url)
        .headers(outgoing)
        .body(body)
        .send()
        .await
        .map_err(|error| {
            warn!(%error, "upstream call failed");
        })?;

    let mut response = Response::builder().status(upstream.status());
    for (name, value) in upstream.headers() {
        if !HOP_BY_HOP.contains(&name.as_str()) {
            response = response.header(name, value);
        }
    }
    response
        .body(Body::from_stream(upstream.bytes_stream()))
        .map_err(|_| ())
}
