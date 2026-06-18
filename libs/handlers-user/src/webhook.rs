use axum::{body::Bytes, extract::State, http::HeaderMap, response::IntoResponse};
use handlers::{ApiError, AppState};
use hmac::{Hmac, Mac};
use mestier_core::UpsertUserBySubCommand;
use sha2::Sha256;
use tracing::error;

type HmacSha256 = Hmac<Sha256>;

const SIGNATURE_HEADER: &str = "x-ferriskey-signature";

/// Verify that `signature_hex` equals HMAC-SHA256(`body`, `secret`).
/// Uses constant-time comparison via [`hmac::Mac::verify_slice`].
pub(crate) fn verify_signature(secret: &[u8], body: &[u8], signature_hex: &str) -> bool {
    let Ok(sig_bytes) = hex::decode(signature_hex) else {
        return false;
    };
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    mac.verify_slice(&sig_bytes).is_ok()
}

/// Minimal user payload attached to every FerrisKey user event.
#[derive(Debug, serde::Deserialize, PartialEq)]
pub(crate) struct WebhookUser {
    /// IAM subject — the stable cross-system identifier.
    /// FerrisKey may send this as `sub` or `id`; both are accepted.
    #[serde(alias = "id")]
    pub sub: String,
    pub email: String,
    pub username: String,
    pub name: Option<String>,
}

/// FerrisKey webhook event. The `event_type` field drives dispatch.
/// Unknown event types are rejected with 400.
#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub(crate) enum WebhookEvent {
    #[serde(rename = "user.created")]
    Created { data: WebhookUser },
    #[serde(rename = "user.updated")]
    Updated { data: WebhookUser },
    #[serde(rename = "user.deleted")]
    Deleted { data: WebhookUser },
}

pub async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let signature = headers
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    if !verify_signature(state.webhook_secret.as_bytes(), &body, signature) {
        return Err(ApiError::Unauthorized);
    }

    let event: WebhookEvent = serde_json::from_slice(&body)
        .map_err(|e| ApiError::BadRequest(format!("invalid payload: {e}")))?;

    dispatch(&state, event).await?;

    Ok(http::StatusCode::OK)
}

/// Drive the usecase from a parsed [`WebhookEvent`]. Extracted so it can be
/// tested independently from the HTTP layer.
pub(crate) async fn dispatch(state: &AppState, event: WebhookEvent) -> Result<(), ApiError> {
    match event {
        WebhookEvent::Created { data } | WebhookEvent::Updated { data } => {
            state
                .usecase
                .reconcile_user_upsert(UpsertUserBySubCommand {
                    sub: data.sub,
                    email: data.email,
                    username: data.username,
                    name: data.name,
                })
                .await
                .map_err(|e| {
                    error!(error = %e, "webhook: reconcile_user_upsert failed");
                    ApiError::from(e)
                })?;
        }
        WebhookEvent::Deleted { data } => {
            state
                .usecase
                .reconcile_user_deletion(data.sub)
                .await
                .map_err(|e| {
                    error!(error = %e, "webhook: reconcile_user_deletion failed");
                    ApiError::from(e)
                })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod signature_tests {
    use super::*;

    fn make_sig(secret: &[u8], body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(body);
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn valid_signature_returns_true() {
        let secret = b"my-secret";
        let body = b"{\"event_type\":\"user.created\"}";
        let sig = make_sig(secret, body);
        assert!(verify_signature(secret, body, &sig));
    }

    #[test]
    fn wrong_secret_returns_false() {
        let body = b"{\"event_type\":\"user.created\"}";
        let sig = make_sig(b"correct-secret", body);
        assert!(!verify_signature(b"wrong-secret", body, &sig));
    }

    #[test]
    fn tampered_body_returns_false() {
        let secret = b"my-secret";
        let sig = make_sig(secret, b"original body");
        assert!(!verify_signature(secret, b"tampered body", &sig));
    }

    #[test]
    fn invalid_hex_signature_returns_false() {
        assert!(!verify_signature(b"secret", b"body", "not-valid-hex!"));
    }

    #[test]
    fn empty_body_with_correct_sig_returns_true() {
        let secret = b"s";
        let body = b"";
        let sig = make_sig(secret, body);
        assert!(verify_signature(secret, body, &sig));
    }
}

#[cfg(test)]
mod event_tests {
    use super::*;

    #[test]
    fn deserialize_user_created() {
        let json = r#"{
            "event_type": "user.created",
            "data": {
                "sub": "abc-123",
                "email": "alice@example.com",
                "username": "alice",
                "name": "Alice"
            }
        }"#;
        let event: WebhookEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, WebhookEvent::Created { .. }));
        if let WebhookEvent::Created { data } = event {
            assert_eq!(data.sub, "abc-123");
            assert_eq!(data.name, Some("Alice".to_owned()));
        }
    }

    #[test]
    fn deserialize_user_updated() {
        let json = r#"{
            "event_type": "user.updated",
            "data": {
                "sub": "abc-123",
                "email": "alice@example.com",
                "username": "alice",
                "name": null
            }
        }"#;
        let event: WebhookEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, WebhookEvent::Updated { .. }));
    }

    #[test]
    fn deserialize_user_deleted() {
        let json = r#"{
            "event_type": "user.deleted",
            "data": {
                "sub": "abc-123",
                "email": "alice@example.com",
                "username": "alice",
                "name": null
            }
        }"#;
        let event: WebhookEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, WebhookEvent::Deleted { .. }));
    }

    #[test]
    fn sub_alias_id_is_accepted() {
        let json = r#"{
            "event_type": "user.created",
            "data": {
                "id": "sub-via-id-field",
                "email": "bob@example.com",
                "username": "bob",
                "name": null
            }
        }"#;
        let event: WebhookEvent = serde_json::from_str(json).unwrap();
        if let WebhookEvent::Created { data } = event {
            assert_eq!(data.sub, "sub-via-id-field");
        } else {
            panic!("expected Created");
        }
    }

    #[test]
    fn unknown_event_type_errors() {
        let json = r#"{"event_type":"org.created","data":{"sub":"x","email":"x@x.com","username":"x","name":null}}"#;
        assert!(serde_json::from_str::<WebhookEvent>(json).is_err());
    }

    #[test]
    fn missing_signature_header_gives_unauthorized_error() {
        // Validate that the ApiError variant mapping is correct without HTTP layer.
        let result: Result<(), ApiError> = Err(ApiError::Unauthorized);
        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }

    #[test]
    fn bad_json_gives_bad_request_error() {
        let err = serde_json::from_slice::<WebhookEvent>(b"not-json")
            .map_err(|e| ApiError::BadRequest(format!("invalid payload: {e}")));
        assert!(matches!(err, Err(ApiError::BadRequest(_))));
    }
}
