use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http::{
    HeaderName, HeaderValue, StatusCode,
    header::{FORWARDED, RETRY_AFTER},
};
use rate_limit::{RateLimitDecision, RateLimitKey};
use tracing::{Span, field, warn};

use crate::{errors::ApiError, state::AppState};

const X_RATELIMIT_LIMIT: HeaderName = HeaderName::from_static("x-ratelimit-limit");
const X_RATELIMIT_REMAINING: HeaderName = HeaderName::from_static("x-ratelimit-remaining");
const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");
const X_REAL_IP: HeaderName = HeaderName::from_static("x-real-ip");

#[tracing::instrument(
    name = "rate_limit.check",
    skip_all,
    fields(
        otel.kind = "internal",
        client.ip = field::Empty,
        rate_limit.scope = "ip",
        rate_limit.limit = field::Empty,
        rate_limit.remaining = field::Empty,
        rate_limit.allowed = field::Empty,
        rate_limit.outcome = field::Empty,
    )
)]
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let span = Span::current();
    let ip = client_ip(&req, socket);
    span.record("client.ip", ip.as_str());

    let key = RateLimitKey::ip(&ip);
    let quota = state.rate_limit_quota;
    span.record("rate_limit.limit", quota.limit);

    let decision = match state.rate_limit.check(&key, quota).await {
        Ok(d) => d,
        Err(e) => {
            // Fail-open: a hiccupping Redis must not take the whole API down.
            span.record("rate_limit.outcome", "backend_error");
            warn!(error = %e, "rate limiter unavailable, allowing request");
            return next.run(req).await;
        }
    };

    span.record("rate_limit.allowed", decision.allowed);
    span.record("rate_limit.remaining", decision.remaining);

    if !decision.allowed {
        span.record("rate_limit.outcome", "blocked");
        return rate_limited_response(&decision);
    }

    span.record("rate_limit.outcome", "allowed");
    let mut response = next.run(req).await;
    inject_headers(response.headers_mut(), &decision);
    response
}

#[doc(hidden)]
pub fn client_ip(req: &Request, fallback: SocketAddr) -> String {
    if let Some(value) = req.headers().get(&X_FORWARDED_FOR)
        && let Ok(s) = value.to_str()
        && let Some(first) = s.split(',').next()
    {
        let trimmed = first.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(value) = req.headers().get(&X_REAL_IP)
        && let Ok(s) = value.to_str()
    {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(value) = req.headers().get(FORWARDED)
        && let Ok(s) = value.to_str()
    {
        for part in s.split(';') {
            if let Some(rest) = part.trim().strip_prefix("for=") {
                let cleaned = rest.trim_matches('"').trim_start_matches('[');
                if !cleaned.is_empty() {
                    return cleaned.to_string();
                }
            }
        }
    }

    fallback.ip().to_string()
}

fn rate_limited_response(decision: &RateLimitDecision) -> Response {
    let mut response = ApiError::TooManyRequests.into_response();
    let headers = response.headers_mut();
    inject_headers(headers, decision);
    if let Some(retry_after) = decision.retry_after {
        let secs = retry_after.as_secs().max(1);
        let mut buf = itoa::Buffer::new();
        if let Ok(value) = HeaderValue::from_str(buf.format(secs)) {
            headers.insert(RETRY_AFTER, value);
        }
    }
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
    response
}

#[doc(hidden)]
pub fn inject_headers(headers: &mut http::HeaderMap, decision: &RateLimitDecision) {
    let mut buf = itoa::Buffer::new();
    if let Ok(v) = HeaderValue::from_str(buf.format(decision.limit)) {
        headers.insert(X_RATELIMIT_LIMIT, v);
    }
    if let Ok(v) = HeaderValue::from_str(buf.format(decision.remaining)) {
        headers.insert(X_RATELIMIT_REMAINING, v);
    }
}
