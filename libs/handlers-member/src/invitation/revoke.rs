use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};
use mestier_core::{RevokeInvitationCommand, application::policy};

use crate::{EmptyResponse, paths::InvitationPath};

/// Revokes a *pending* invitation — distinct from removing an already
/// accepted membership, which is `DELETE /members/{id}` (`soft_delete`).
#[utoipa::path(
    delete,
    path = "/api/v1/invitations/{invitation_id}",
    operation_id = "revokeInvitation",
    tag = super::super::TAG,
    params(
        ("invitation_id" = mestier_core::InvitationId, Path, description = "Invitation identifier"),
    ),
    responses(
        (status = 204, description = "Invitation revoked"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invitation not found"),
        (status = 409, description = "Invitation already accepted"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvitationPath { invitation_id }: InvitationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    // Loading the invitation and checking its organization — never a
    // caller-supplied one — happens inside `InvitationService::revoke_invitation`.
    state
        .usecase
        .acting_as(user_id)
        .revoke_invitation(RevokeInvitationCommand {
            actor,
            invitation_id,
        })
        .await?;

    Ok(Response::NoContent)
}
