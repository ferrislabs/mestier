use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{MemberId, application::policy};

use crate::{paths::MemberPath, response::MemberResponse};

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
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    // Loading the seat and checking its organization — never the path's —
    // now happens inside `MemberService::get_member`, which also hydrates
    // the occupant in the same call.
    let member = state.usecase.get_member(actor, member_id).await?;

    Ok(Response::OK(member.into()))
}
