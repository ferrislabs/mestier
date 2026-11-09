use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateParticipantCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ParticipantPath, require_org_membership, response::ParticipantDetailResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateParticipantRequest {
    pub last_name: String,
    pub first_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}",
    operation_id = "updateCursusParticipant",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("participant_id" = mestier_core::ParticipantId, Path, description = "Participant identifier"),
    ),
    request_body = UpdateParticipantRequest,
    responses(
        (status = 200, description = "The participant was updated", body = inline(DataEnvelope<ParticipantDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Participant not found in the organization"),
        (status = 409, description = "Invalid participant or duplicate email"),
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
    Json(payload): Json<UpdateParticipantRequest>,
) -> Result<Response<ParticipantDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_participant(UpdateParticipantCommand {
            actor,
            organization_id,
            id: participant_id,
            last_name: payload.last_name,
            first_name: payload.first_name,
            email: payload.email,
            phone: payload.phone,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
