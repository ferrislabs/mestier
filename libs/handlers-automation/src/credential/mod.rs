//! Credentials: list, create, update, delete, rotate. A generated secret is
//! visible exactly once, in `create` and `rotate`'s own response — see
//! `dto::CredentialWithSecretResponse`.

use auth::Identity;
use axum::Router;
use axum_extra::routing::RouterExt;
use common::CoreError;
use handlers::{ApiError, AppState};
use mestier_core::{Credential, CredentialError, OrganizationId};
use uuid::Uuid;

use crate::require_manage_automation;

pub mod create;
pub mod delete;
pub mod dto;
pub mod list;
pub mod oauth;
pub mod rotate;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
        .typed_post(rotate::handler)
        .typed_post(oauth::start)
        .typed_post(oauth::disconnect)
}

/// Loads a credential of `organization_id`, refusing a caller without
/// `MANAGE_AUTOMATION`.
///
/// Gating is fused in rather than taken as a parameter because every caller
/// is a write. A read route must not reach for this.
pub(crate) async fn require_credential(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    credential_id: Uuid,
) -> Result<Credential, ApiError> {
    require_manage_automation(state, identity, organization_id).await?;

    state
        .usecase
        .find_credential(organization_id, credential_id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub(crate) fn credential_failure(error: CoreError) -> ApiError {
    match &error {
        CoreError::Conflict(message) => match CredentialError::validation_message(message) {
            Some(text) => ApiError::Validation(text.to_owned()),
            None => error.into(),
        },
        _ => error.into(),
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, response::IntoResponse};
    use http::StatusCode;

    use super::*;

    async fn answer(error: CoreError) -> (StatusCode, serde_json::Value) {
        let response = credential_failure(error).into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn the_field_errors_are_validation_errors_in_french_without_the_marker() {
        for error in [
            CredentialError::BlankName,
            CredentialError::ReservedHeaderName,
            CredentialError::MissingField {
                label: "Jeton".to_owned(),
            },
            CredentialError::NullValue {
                label: "Jeton".to_owned(),
            },
        ] {
            let (status, json) = answer(CoreError::from(error.clone())).await;

            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(json["code"], "E_VALIDATION");
            let message = json["message"].as_str().unwrap();
            assert!(message.contains(&error.to_string()), "{message}");
            assert!(!message.contains("[validation]"), "{message}");
        }
    }

    #[tokio::test]
    async fn the_other_credential_conflicts_keep_their_ordinary_mapping() {
        for error in [
            CoreError::from(CredentialError::NameTaken),
            CoreError::from(CredentialError::NotRotatable),
            CoreError::Conflict("slug taken".to_owned()),
        ] {
            let (status, json) = answer(error).await;

            assert_eq!(status, StatusCode::CONFLICT);
            assert_eq!(json["code"], "E_CONFLICT");
        }
        let (_, json) = answer(CoreError::from(CredentialError::NameTaken)).await;
        assert!(!json["message"].as_str().unwrap().contains("uq_"));
        let (status, _) = answer(CoreError::NotFound).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
