use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{paths::MemberRolesPath, response::MemberRoleIdsResponse};

#[utoipa::path(
    get,
    path = "/api/v1/members/{member_id}/roles",
    operation_id = "listMemberRoles",
    tag = super::super::TAG,
    params(
        ("member_id" = mestier_core::MemberId, Path, description = "Member identifier"),
    ),
    responses(
        (status = 200, description = "The role ids this member holds", body = inline(DataEnvelope<MemberRoleIdsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberRolesPath { member_id }: MemberRolesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MemberRoleIdsResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let role_ids = state
        .usecase
        .acting_as(user_id)
        .list_role_ids(member_id, actor)
        .await?;

    Ok(Response::OK(MemberRoleIdsResponse { role_ids }))
}
