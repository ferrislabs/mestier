use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{
    paths::ParticipantInvitationPath, require_org_membership, response::ParticipantInviteResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/invitation",
    operation_id = "inviteCursusParticipant",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("participant_id" = mestier_core::ParticipantId, Path, description = "Participant identifier"),
    ),
    responses(
        (status = 201, description = "The invitation token, readable only in this response; a previous token is replaced", body = inline(DataEnvelope<ParticipantInviteResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Participant not found in the organization"),
        (status = 409, description = "The participant is already linked to a user or is deactivated"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ParticipantInvitationPath {
        organization_id,
        participant_id,
    }: ParticipantInvitationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ParticipantInviteResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let invite = state
        .usecase
        .acting_as(user_id)
        .invite_participant(actor, organization_id, participant_id)
        .await?;

    Ok(Response::Created(invite.into()))
}
