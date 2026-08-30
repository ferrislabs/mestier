use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::UnassignRoleCommand;

use crate::{EmptyResponse, paths::MemberRolePath};

/// Unassigns a role from a member (#308's documented gap) — symmetric with
/// `role::assign::handler`, but `role_id` sits in the path rather than the
/// body: this is a `DELETE`, and `role::delete::handler`'s own
/// `DELETE /api/v1/roles/{role_id}` already sets that precedent. Idempotent:
/// unassigning a role the member never held still answers `204`.
#[utoipa::path(
    delete,
    path = "/api/v1/members/{member_id}/roles/{role_id}",
    operation_id = "unassignRole",
    tag = super::super::TAG,
    params(
        ("member_id" = mestier_core::MemberId, Path, description = "Member identifier"),
        ("role_id" = mestier_core::RoleId, Path, description = "Role identifier"),
    ),
    responses(
        (status = 204, description = "Role unassigned"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberRolePath { member_id, role_id }: MemberRolePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .unassign_role(UnassignRoleCommand {
            actor,
            member_id,
            role_id,
        })
        .await?;

    Ok(Response::NoContent)
}
