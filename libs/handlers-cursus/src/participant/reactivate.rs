use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{
    paths::ParticipantReactivatePath, require_org_membership, response::ParticipantResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/reactivate",
    operation_id = "reactivateCursusParticipant",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("participant_id" = mestier_core::ParticipantId, Path, description = "Participant identifier"),
    ),
    responses(
        (status = 200, description = "The participant after the change", body = inline(DataEnvelope<ParticipantResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Participant not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ParticipantReactivatePath {
        organization_id,
        participant_id,
    }: ParticipantReactivatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ParticipantResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let participant = state
        .usecase
        .acting_as(user_id)
        .reactivate_participant(actor, organization_id, participant_id)
        .await?;

    Ok(Response::OK(participant.into()))
}
