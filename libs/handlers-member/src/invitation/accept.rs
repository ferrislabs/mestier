use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::AcceptInvitationCommand;

use crate::{paths::AcceptInvitationPath, response::MemberResponse};

/// The one route in this whole crate that is authenticated **without**
/// requiring organization membership. That is deliberate — see
/// `AcceptInvitationCommand`'s doc comment: the caller holds a valid
/// FerrisKey token (so `auth_middleware` already ran and the local `users`
/// row exists), but has no standing yet in the organization the token
/// targets. The usual `resolve_user_id` + `policy::user_subject` +
/// `enrich_for_organization` shape every other handler in this crate goes
/// through would reject exactly the request this route exists to serve, so
/// it stops one step earlier: resolve the caller's local id, and nothing
/// else.
#[utoipa::path(
    post,
    path = "/api/v1/invitations/{token}/accept",
    operation_id = "acceptInvitation",
    tag = super::super::TAG,
    params(
        ("token" = String, Path, description = "Clear invitation token"),
    ),
    responses(
        (status = 200, description = "Invitation accepted — the seat now belongs to the caller", body = inline(DataEnvelope<MemberResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Unknown, expired, or already-accepted token"),
        (status = 409, description = "Caller is already a member of this organization"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    AcceptInvitationPath { token }: AcceptInvitationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MemberResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;

    let member = state
        .usecase
        .acting_as(user_id)
        .accept_invitation(AcceptInvitationCommand { token, user_id })
        .await?;

    Ok(Response::OK(member.into()))
}
