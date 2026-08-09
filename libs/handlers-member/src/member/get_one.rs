use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::MemberId;

use crate::{paths::MemberPath, require_org_membership, resolve_account, response::MemberResponse};

#[utoipa::path(
    get,
    path = "/api/v1/members/{member_id}",
    operation_id = "getMember",
    tag = super::super::TAG,
    params(
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    responses(
        (status = 200, description = "Member details", body = inline(DataEnvelope<MemberResponse>)),
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
) -> Result<Response<MemberResponse>, ApiError> {
    // Load first, then check the organization from the loaded resource —
    // never from the path — so a member from another organization is a 403
    // rather than an IDOR that leaks its data.
    let member = state.usecase.get_member(member_id).await?;
    require_org_membership(&state, &identity, member.organization_id).await?;

    let account = resolve_account(&state, member.user_id).await?;
    let mut response = MemberResponse::from(member);
    response.account = account;

    Ok(Response::OK(response))
}
