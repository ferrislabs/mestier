use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::AddCompanyTutorCommand;

use crate::{
    paths::ParticipantCompanyTutorsPath, request::CompanyTutorRequest, require_org_membership,
    response::ParticipantDetailResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/company-tutors",
    operation_id = "addCursusCompanyTutor",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("participant_id" = mestier_core::ParticipantId, Path, description = "Participant identifier"),
    ),
    request_body = CompanyTutorRequest,
    responses(
        (status = 201, description = "The company tutor record was added", body = inline(DataEnvelope<ParticipantDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Participant or host company not found in the organization"),
        (status = 409, description = "The participant already is a tutor for this host company"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ParticipantCompanyTutorsPath {
        organization_id,
        participant_id,
    }: ParticipantCompanyTutorsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CompanyTutorRequest>,
) -> Result<Response<ParticipantDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let detail = state
        .usecase
        .acting_as(user_id)
        .add_company_tutor(AddCompanyTutorCommand {
            actor,
            organization_id,
            participant_id,
            company_tutor: payload.into(),
        })
        .await?;

    Ok(Response::Created(detail.into()))
}
