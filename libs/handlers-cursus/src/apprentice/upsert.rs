use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpsertApprenticeCommand;

use crate::{
    paths::ParticipantApprenticePath, request::ApprenticeRequest, require_org_membership,
    response::ParticipantDetailResponse,
};

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/apprentice",
    operation_id = "upsertCursusApprentice",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("participant_id" = mestier_core::ParticipantId, Path, description = "Participant identifier"),
    ),
    request_body = ApprenticeRequest,
    responses(
        (status = 200, description = "The apprentice record was created or replaced", body = inline(DataEnvelope<ParticipantDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Participant not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ParticipantApprenticePath {
        organization_id,
        participant_id,
    }: ParticipantApprenticePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ApprenticeRequest>,
) -> Result<Response<ParticipantDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let detail = state
        .usecase
        .acting_as(user_id)
        .upsert_apprentice(UpsertApprenticeCommand {
            actor,
            organization_id,
            participant_id,
            apprentice: payload.into(),
        })
        .await?;

    Ok(Response::OK(detail.into()))
}
