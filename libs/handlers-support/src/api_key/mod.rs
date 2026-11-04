use auth::Identity;
use authz::Subject;
use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, resolve_actor};
use mestier_core::{OrganizationId, Permissions};

use crate::require_org_membership;

pub mod create;
pub mod list;
pub mod revoke;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_post(create::handler)
        .typed_get(list::handler)
        .typed_delete(revoke::handler)
}

pub(crate) async fn require_key_manager(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<Subject, ApiError> {
    require_org_membership(state, identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(state, identity).await?;

    let permissions = state
        .usecase
        .member_permissions(user_id, organization_id)
        .await?;
    if !permissions.contains(Permissions::MANAGE_ORG) {
        return Err(ApiError::Forbidden);
    }

    Ok(actor)
}
