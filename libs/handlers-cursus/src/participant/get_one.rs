use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ParticipantPath, require_view_cursus, response::ParticipantDetailResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}",
    operation_id = "getCursusParticipant",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("participant_id" = mestier_core::ParticipantId, Path, description = "Participant identifier"),
    ),
    responses(
        (status = 200, description = "The participant with its apprentice record and company tutor records", body = inline(DataEnvelope<ParticipantDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Participant not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ParticipantPath {
        organization_id,
        participant_id,
    }: ParticipantPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ParticipantDetailResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state
        .usecase
        .get_participant(organization_id, participant_id)
        .await?;

    Ok(Response::OK(found.into()))
}
