use std::pin::Pin;

use axum::{
    Json, Router,
    body::{Body, HttpBody},
    extract::{Path, Request, State},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::any,
};
use handlers::{ApiError, AppState, rate_limit::rate_limit_middleware};
use http::{
    HeaderMap, StatusCode,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
};
use mestier_core::{
    MAX_WEBHOOK_BODY_BYTES, OnInvalid, WebhookMethod, check_webhook_path, contract_violations,
    is_json_content_type,
};
use serde::Serialize;
use serde_json::{Value, json};
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

enum BodyFault {
    TooLarge,
    Unreadable,
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
    description = "Starts the workflow whose webhook trigger holds this path. Public: it is not under /api/v1 and takes no bearer token of its own; the credential chosen on the trigger, when there is one, is what authenticates the call. The route answers to the method chosen on the trigger (GET, POST, PUT, PATCH or DELETE), not only to POST. The body is JSON, at most 1 MiB; an empty body is read as {}. A path that does not exist, a method the trigger does not accept and a disabled or archived workflow all answer the same 404.",
    params(
        ("path" = String, Path, description = "The path chosen on the trigger, without the /webhooks/ prefix"),
    ),
    responses(
        (status = 202, description = "A run was started; its trigger payload is the body", body = WebhookAccepted),
        (status = 400, description = "The body is not JSON, or it breaks the contract of a trigger set to refuse; keys lists the keys at fault", body = WebhookBodyInvalid),
        (status = 401, description = "The credential chosen on the trigger is missing or wrong"),
        (status = 404, description = "No such webhook"),
        (status = 413, description = "The body is larger than 1 MiB"),
        (status = 415, description = "The body is not JSON"),
        (status = 429, description = "Too many requests"),
    ),
)]
pub async fn receive(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Response {
    let Some(method) = WebhookMethod::parse(request.method().as_str()) else {
        return not_found();
    };
    if check_webhook_path(&path).is_err() {
        return not_found();
    }
    let route = match state.usecase.resolve_webhook(&path, method).await {
        Ok(Some(route)) => route,
        Ok(None) => return not_found(),
        Err(error) => return ApiError::from(error).into_response(),
    };

    let (parts, body) = request.into_parts();

    if let Some(credential_id) = route.credential_id {
        let headers = &parts.headers;
        let allowed = state
            .usecase
            .verify_webhook_credential(route.org_id, credential_id, |name| {
                single_header(headers, name)
            })
            .await;
        if !allowed {
            return ApiError::Unauthorized.into_response();
        }
    }

    let content_type = parts
        .headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());
    if content_type.is_some_and(|value| !is_json_content_type(value)) {
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
    let bytes = match read_limited(body, MAX_WEBHOOK_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(BodyFault::TooLarge) => return too_large(),
        Err(BodyFault::Unreadable) => {
            return body_invalid(Vec::new(), "the body could not be read");
        }
    };

    let payload = if bytes.is_empty() {
        if content_type.is_none() {
            json!({})
        } else {
            return body_invalid(Vec::new(), "the body is not valid JSON");
        }
    } else if content_type.is_none() {
        return unsupported_media_type();
    } else {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(value) => value,
            Err(_) => return body_invalid(Vec::new(), "the body is not valid JSON"),
        }
    };

    let violations = contract_violations(&route.contract, &payload);
    if !violations.is_empty() && route.on_invalid == OnInvalid::Reject {
        return body_invalid(violations, "the body does not meet the webhook contract");
    }

    match state
        .usecase
        .start_trigger_run(route.org_id, route.workflow_id, &route.trigger_id, payload)
        .await
    {
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
}
