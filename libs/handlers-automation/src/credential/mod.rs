//! Credentials: list, create, update, delete, rotate. The secret is visible
//! exactly once, in `create` and `rotate`'s own response — see
//! `response::CredentialWithSecretResponse`.

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
pub mod list;
pub mod rotate;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
        .typed_post(rotate::handler)
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
        CoreError::Conflict(message) if CredentialError::from_conflict(message).is_some() => {
            ApiError::Validation(message.clone())
        }
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
    async fn a_blank_secret_and_a_reserved_header_are_validation_errors() {
        for error in [
            CredentialError::BlankField,
            CredentialError::ReservedHeaderName,
        ] {
            let (status, json) = answer(CoreError::from(error.clone())).await;

            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(json["code"], "E_VALIDATION");
            assert!(
                json["message"]
                    .as_str()
                    .unwrap()
                    .contains(&error.to_string())
            );
        }
    }

    #[tokio::test]
    async fn the_other_credential_conflicts_keep_their_ordinary_mapping() {
        for error in [
            CoreError::from(CredentialError::MissingField {
                field: "token".to_owned(),
            }),
            CoreError::Conflict("slug taken".to_owned()),
        ] {
            let (status, json) = answer(error).await;

            assert_eq!(status, StatusCode::CONFLICT);
            assert_eq!(json["code"], "E_CONFLICT");
        }
        let (status, _) = answer(CoreError::NotFound).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
