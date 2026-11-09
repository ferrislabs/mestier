use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::AcceptParticipantInvitationCommand;

use crate::{paths::AcceptInvitationPath, response::CursusSpaceResponse};

#[utoipa::path(
    post,
    path = "/api/v1/cursus/invitations/{token}/accept",
    operation_id = "acceptCursusInvitation",
    tag = crate::TAG,
    params(
        ("token" = String, Path, description = "Clear invitation token"),
    ),
    responses(
        (status = 200, description = "Invitation accepted; the caller is now the participant", body = inline(DataEnvelope<CursusSpaceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "The participant is linked to another user, or the caller already is a participant of this organization"),
        (status = 410, description = "Unknown, expired or already used token, or deactivated participant"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AcceptInvitationPath { token }: AcceptInvitationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<CursusSpaceResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;

    let space = state
        .usecase
        .acting_as(user_id)
        .accept_participant_invitation(AcceptParticipantInvitationCommand { token, user_id })
        .await?;

    Ok(Response::OK(space.into()))
}
