//! Credentials: list, create, update, delete, rotate. The secret is visible
//! exactly once, in `create` and `rotate`'s own response — see
//! `response::CredentialWithSecretResponse`.

use auth::Identity;
use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{Credential, OrganizationId};
use uuid::Uuid;

use crate::require_org_membership;

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

/// Loads the credential and checks both that the caller belongs to
/// `organization_id` and that the credential actually belongs to it —
/// `find_credential` is itself scoped by `organization_id`, so a real
/// `credential_id` from a different organization already reads back as
/// absent; this only adds the membership check every route needs anyway.
/// Mirrors `handlers-planning::task::require_task`.
pub(crate) async fn require_credential(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    credential_id: Uuid,
) -> Result<Credential, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    state
        .usecase
        .find_credential(organization_id, credential_id)
        .await?
        .ok_or(ApiError::NotFound)
}
