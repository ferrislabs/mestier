use std::{future::Future, pin::Pin, time::Duration};

use axum::{
    Json, Router,
    body::{Body, HttpBody},
    extract::{Path, Request, State, rejection::PathRejection},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::any,
};
use handlers::{
    ApiError, AppState,
    rate_limit::{inject_headers, rate_limit_middleware},
};
use http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{CONTENT_LENGTH, CONTENT_TYPE, RETRY_AFTER},
};
use mestier_core::{
    MAX_WEBHOOK_BODY_BYTES, OnInvalid, SignatureSpec, WebhookAuthentication, WebhookMethod,
    check_webhook_path, contract_violations, holds_nul, is_json_content_type, path_fingerprint,
};
use rate_limit::{Quota, RateLimitDecision, RateLimitKey};
use serde::Serialize;
use serde_json::{Value, json};
use tracing::warn;
use utoipa::ToSchema;
use uuid::Uuid;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/webhooks", any(unknown))
        .route("/webhooks/", any(unknown))
        .route("/webhooks/{*path}", any(receive))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookAccepted {
    pub run_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookBodyInvalid {
    pub code: &'static str,
    pub message: String,
    pub status: u16,
    pub keys: Vec<String>,
}

#[derive(Serialize)]
struct Refusal {
    code: &'static str,
    message: &'static str,
    status: u16,
}

fn refusal(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    (
        status,
        Json(Refusal {
            code,
            message,
            status: status.as_u16(),
        }),
    )
        .into_response()
}

fn not_found() -> Response {
    ApiError::NotFound.into_response()
}

async fn unknown() -> Response {
    not_found()
}

fn body_invalid(keys: Vec<String>, message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(WebhookBodyInvalid {
            code: "E_WEBHOOK_BODY_INVALID",
            message: message.to_owned(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            keys,
        }),
    )
        .into_response()
}

fn single_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let mut values = headers.get_all(name).iter();
    let value = values.next()?;
    if values.next().is_some() {
        return None;
    }
    value.to_str().ok()
}

pub const WEBHOOK_PATH_LIMIT_PER_MINUTE: u32 = 60;
pub const WEBHOOK_WORKFLOW_LIMIT_PER_MINUTE: u32 = 120;
pub const WEBHOOK_BODY_TIMEOUT: Duration = Duration::from_secs(10);

enum BodyFault {
    TooLarge,
    Unreadable,
    TimedOut,
}

async fn read_within<F>(limit: Duration, reading: F) -> Result<Vec<u8>, BodyFault>
where
    F: Future<Output = Result<Vec<u8>, BodyFault>>,
{
    tokio::time::timeout(limit, reading)
        .await
        .unwrap_or(Err(BodyFault::TimedOut))
}

fn too_many(decision: &RateLimitDecision) -> Response {
    let mut response = ApiError::TooManyRequests.into_response();
    inject_headers(response.headers_mut(), decision);
    if let Some(retry_after) = decision.retry_after
        && let Ok(value) = HeaderValue::from_str(&retry_after.as_secs().max(1).to_string())
    {
        response.headers_mut().insert(RETRY_AFTER, value);
    }
    response
}

async fn within_limit(state: &AppState, key: RateLimitKey, limit: u32) -> Option<Response> {
    match state.rate_limit.check(&key, Quota::per_minute(limit)).await {
        Ok(decision) if !decision.allowed => Some(too_many(&decision)),
        Ok(_) => None,
        Err(error) => {
            warn!(%error, scope = %key.scope, "the webhook rate limiter is unavailable, allowing the call");
            None
        }
    }
}

async fn read_limited(mut body: Body, limit: usize) -> Result<Vec<u8>, BodyFault> {
    let mut collected = Vec::new();
    loop {
        let frame = std::future::poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await;
        match frame {
            None => return Ok(collected),
            Some(Err(_)) => return Err(BodyFault::Unreadable),
            Some(Ok(frame)) => {
                if let Ok(data) = frame.into_data() {
                    if collected.len() + data.len() > limit {
                        return Err(BodyFault::TooLarge);
                    }
                    collected.extend_from_slice(&data);
                }
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/webhooks/{path}",
    operation_id = "receiveWebhook",
    tag = super::TAG,
    description = "Starts the workflow whose webhook trigger holds this path. Public: it is not under /api/v1 and takes no bearer token of its own; the credential chosen on the trigger, when there is one, is what authenticates the call: a token, a basic login or a header is checked before the body is read, a signature credential (HMAC) is checked on the raw body, still before the body is parsed or any run starts. The route answers to the method chosen on the trigger (GET, POST, PUT, PATCH or DELETE), not only to POST. The body is JSON, at most 1 MiB, and must arrive within 10 seconds; an empty body is read as {}. A path answers at most 60 calls a minute and a workflow at most 120. A path that does not exist, a method the trigger does not accept and a disabled or archived workflow all answer the same 404.",
    params(
        ("path" = String, Path, description = "The path chosen on the trigger, without the /webhooks/ prefix"),
    ),
    responses(
        (status = 202, description = "A run was started; its trigger payload is the body", body = WebhookAccepted),
        (status = 400, description = "The body is not valid JSON, holds a NUL character, or breaks the contract of a trigger set to refuse; keys lists the keys at fault", body = WebhookBodyInvalid),
        (status = 401, description = "The credential chosen on the trigger is missing or wrong"),
        (status = 404, description = "No such webhook"),
        (status = 408, description = "The body did not arrive within 10 seconds"),
        (status = 413, description = "The body is larger than 1 MiB"),
        (status = 415, description = "The body is not JSON"),
        (status = 429, description = "Too many requests"),
    ),
)]
pub async fn receive(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
    request: Request,
) -> Response {
    let Ok(Path(path)) = path else {
        return not_found();
    };
    let Some(method) = WebhookMethod::parse(request.method().as_str()) else {
        return not_found();
    };
    if check_webhook_path(&path).is_err() {
        return not_found();
    }
    let by_path = RateLimitKey {
        scope: "webhook-path".to_owned(),
        identifier: path_fingerprint(&path),
    };
    if let Some(refused) = within_limit(&state, by_path, WEBHOOK_PATH_LIMIT_PER_MINUTE).await {
        return refused;
    }
    let route = match state.usecase.resolve_webhook(&path, method).await {
        Ok(Some(route)) => route,
        Ok(None) => return not_found(),
        Err(error) => return ApiError::from(error).into_response(),
    };
    let by_workflow = RateLimitKey {
        scope: "webhook-workflow".to_owned(),
        identifier: format!("{}:{}", route.org_id.0, route.workflow_id),
    };
    if let Some(refused) =
        within_limit(&state, by_workflow, WEBHOOK_WORKFLOW_LIMIT_PER_MINUTE).await
    {
        return refused;
    }

    let (parts, body) = request.into_parts();

    let mut signature: Option<SignatureSpec> = None;
    if let Some(credential_id) = route.credential_id {
        let headers = &parts.headers;
        match state
            .usecase
            .authenticate_webhook(route.org_id, credential_id, |name| {
                single_header(headers, name)
            })
            .await
        {
            WebhookAuthentication::Accepted => {}
            WebhookAuthentication::Refused => return ApiError::Unauthorized.into_response(),
            WebhookAuthentication::SignatureRequired(spec) => signature = Some(spec),
        }
    }

    let content_type = parts
        .headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());
    let unsupported_content_type = content_type.is_some_and(|value| !is_json_content_type(value));
    if unsupported_content_type && signature.is_none() {
        return unsupported_media_type();
    }
    let declared = parts
        .headers
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());
    if declared.is_some_and(|length| length > MAX_WEBHOOK_BODY_BYTES) {
        return too_large();
    }
    let bytes = match read_within(
        WEBHOOK_BODY_TIMEOUT,
        read_limited(body, MAX_WEBHOOK_BODY_BYTES),
    )
    .await
    {
        Ok(bytes) => bytes,
        Err(BodyFault::TooLarge) => return too_large(),
        Err(BodyFault::TimedOut) => return body_timeout(),
        Err(BodyFault::Unreadable) => {
            return body_invalid(Vec::new(), "the body could not be read");
        }
    };

    if let Some(spec) = &signature {
        let headers = &parts.headers;
        if !spec.verify(|name| single_header(headers, name), &bytes) {
            return ApiError::Unauthorized.into_response();
        }
        if unsupported_content_type {
            return unsupported_media_type();
        }
    }

    let payload = if bytes.is_empty() {
        json!({})
    } else if content_type.is_none() {
        return unsupported_media_type();
    } else {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(value) => value,
            Err(_) => return body_invalid(Vec::new(), "the body is not valid JSON"),
        }
    };
    if holds_nul(&payload) {
        return body_invalid(
            Vec::new(),
            "the body holds a NUL character, which cannot be stored",
        );
    }

    let violations = contract_violations(&route.contract, &payload);
    if !violations.is_empty() && route.on_invalid == OnInvalid::Reject {
        return body_invalid(violations, "the body does not meet the webhook contract");
    }

    match state.usecase.start_webhook_run(&route, payload).await {
        Ok(run_id) => (StatusCode::ACCEPTED, Json(WebhookAccepted { run_id })).into_response(),
        Err(common::CoreError::NotFound) => not_found(),
        Err(error) => ApiError::from(error).into_response(),
    }
}

fn unsupported_media_type() -> Response {
    refusal(
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "E_WEBHOOK_UNSUPPORTED_MEDIA_TYPE",
        "the body must be JSON",
    )
}

fn body_timeout() -> Response {
    refusal(
        StatusCode::REQUEST_TIMEOUT,
        "E_WEBHOOK_BODY_TIMEOUT",
        "the body did not arrive in time",
    )
}

fn too_large() -> Response {
    refusal(
        StatusCode::PAYLOAD_TOO_LARGE,
        "E_WEBHOOK_BODY_TOO_LARGE",
        "the body is larger than 1 MiB",
    )
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use http::HeaderValue;

    use super::*;

    async fn body_of(response: Response) -> (StatusCode, Value) {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[test]
    fn a_header_is_read_only_when_it_is_present_once_and_readable() {
        let mut headers = HeaderMap::new();
        headers.insert("x-once", HeaderValue::from_static("a"));
        headers.append("x-twice", HeaderValue::from_static("a"));
        headers.append("x-twice", HeaderValue::from_static("a"));
        headers.insert("x-binary", HeaderValue::from_bytes(&[0xff, 0xfe]).unwrap());

        assert_eq!(single_header(&headers, "x-once"), Some("a"));
        assert_eq!(single_header(&headers, "X-Once"), Some("a"));
        assert_eq!(single_header(&headers, "x-twice"), None);
        assert_eq!(single_header(&headers, "x-binary"), None);
        assert_eq!(single_header(&headers, "x-absent"), None);
        assert_eq!(single_header(&headers, "not a header name"), None);
    }

    #[tokio::test]
    async fn a_body_is_read_up_to_the_limit_and_refused_one_byte_above() {
        let at_the_limit = read_limited(Body::from(vec![b'a'; 10]), 10).await;
        let above = read_limited(Body::from(vec![b'a'; 11]), 10).await;
        let empty = read_limited(Body::empty(), 10).await;

        assert_eq!(at_the_limit.ok(), Some(vec![b'a'; 10]));
        assert!(matches!(above, Err(BodyFault::TooLarge)));
        assert_eq!(empty.ok(), Some(Vec::new()));
    }

    #[tokio::test]
    async fn the_refusals_carry_a_code_a_message_and_their_status() {
        let (status, json) = body_of(too_large()).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json["code"], "E_WEBHOOK_BODY_TOO_LARGE");
        assert_eq!(json["status"], 413);

        let (status, json) = body_of(unsupported_media_type()).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(json["code"], "E_WEBHOOK_UNSUPPORTED_MEDIA_TYPE");

        let (status, json) = body_of(body_invalid(
            vec!["montant".to_owned()],
            "the body does not meet the webhook contract",
        ))
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "E_WEBHOOK_BODY_INVALID");
        assert_eq!(json["status"], 400);
        assert_eq!(json["keys"], json!(["montant"]));
    }

    #[tokio::test]
    async fn the_uniform_not_found_does_not_say_why() {
        let (status, json) = body_of(not_found()).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["code"], "E_NOT_FOUND");
        assert!(json.get("keys").is_none());
    }

    #[tokio::test]
    async fn a_body_that_does_not_arrive_is_given_up_on() {
        let stalled = read_within(Duration::from_millis(30), std::future::pending());

        let outcome = tokio::time::timeout(Duration::from_secs(5), stalled)
            .await
            .expect("the wait is bounded by its own timeout");

        assert!(matches!(outcome, Err(BodyFault::TimedOut)));
    }

    #[tokio::test]
    async fn a_body_that_arrives_in_time_is_passed_through() {
        let outcome = read_within(Duration::from_secs(5), async { Ok(b"{}".to_vec()) }).await;

        assert_eq!(outcome.ok(), Some(b"{}".to_vec()));
    }

    #[tokio::test]
    async fn the_body_timeout_is_ten_seconds_and_answers_408() {
        assert_eq!(WEBHOOK_BODY_TIMEOUT, Duration::from_secs(10));

        let (status, json) = body_of(body_timeout()).await;

        assert_eq!(status, StatusCode::REQUEST_TIMEOUT);
        assert_eq!(json["code"], "E_WEBHOOK_BODY_TIMEOUT");
        assert_eq!(json["status"], 408);
    }

    #[test]
    fn the_limits_are_sixty_a_minute_per_path_and_one_hundred_twenty_per_workflow() {
        assert_eq!(WEBHOOK_PATH_LIMIT_PER_MINUTE, 60);
        assert_eq!(WEBHOOK_WORKFLOW_LIMIT_PER_MINUTE, 120);
    }

    #[tokio::test]
    async fn a_refused_call_is_told_to_wait_in_the_format_of_the_ip_limiter() {
        let decision = RateLimitDecision {
            allowed: false,
            limit: 60,
            remaining: 0,
            retry_after: Some(Duration::from_millis(1500)),
        };

        let response = too_many(&decision);
        let headers = response.headers().clone();
        let (status, json) = body_of(response).await;

        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(json["code"], "E_TOO_MANY_REQUESTS");
        assert_eq!(json["status"], 429);
        assert_eq!(headers["retry-after"], "1");
        assert_eq!(headers["x-ratelimit-limit"], "60");
        assert_eq!(headers["x-ratelimit-remaining"], "0");
    }

    #[tokio::test]
    async fn a_refused_call_with_a_long_wait_says_so_in_whole_seconds() {
        let decision = RateLimitDecision {
            allowed: false,
            limit: 120,
            remaining: 0,
            retry_after: Some(Duration::from_secs(42)),
        };

        assert_eq!(too_many(&decision).headers()["retry-after"], "42");
    }
}
