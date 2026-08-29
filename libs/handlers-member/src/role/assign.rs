use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::{AssignRoleCommand, RoleId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{EmptyResponse, paths::MemberRolesPath};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: RoleId,
}

/// Assigns a role to a member (#308) — additive, not a replace: a member
/// can hold more than one role, and this never removes an existing one.
#[utoipa::path(
    post,
    path = "/api/v1/members/{member_id}/roles",
    operation_id = "assignRole",
    tag = super::super::TAG,
    params(
        ("member_id" = mestier_core::MemberId, Path, description = "Member identifier"),
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 204, description = "Role assigned"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member or role not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberRolesPath { member_id }: MemberRolesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AssignRoleRequest>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .assign_role(AssignRoleCommand {
            actor,
            member_id,
            role_id: payload.role_id,
        })
        .await?;

    Ok(Response::NoContent)
}
