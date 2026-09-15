//! Credentials: list, create, update, delete, rotate. The secret is visible
//! exactly once, in `create` and `rotate`'s own response — see
//! `response::CredentialWithSecretResponse`.

use auth::Identity;
use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{Credential, OrganizationId};
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
