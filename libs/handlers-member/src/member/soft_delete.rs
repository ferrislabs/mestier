use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};
use mestier_core::MemberId;

use crate::{EmptyResponse, paths::MemberPath, require_permission};

#[utoipa::path(
    delete,
    path = "/api/v1/members/{member_id}",
    operation_id = "deleteMember",
    tag = super::super::TAG,
    params(
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    responses(
        (status = 204, description = "Member soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberPath { member_id }: MemberPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    // Load first, then check the organization from the loaded resource —
    // never from the path — so a member from another organization is a 403
    // rather than an IDOR that leaks its data.
    let current = state.usecase.get_member(member_id).await?;
    require_permission(&state, &identity, current.organization_id, "member.remove").await?;
    let actor = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .acting_as(actor)
        .remove_member(member_id)
        .await?;

    Ok(Response::NoContent)
}
