use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateParticipantCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::ParticipantCollectionPath,
    request::{ApprenticeRequest, CompanyTutorRequest},
    require_org_membership,
    response::ParticipantDetailResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateParticipantRequest {
    pub last_name: String,
    pub first_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub apprentice: Option<ApprenticeRequest>,
    pub company_tutor: Option<CompanyTutorRequest>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/participants",
    operation_id = "createCursusParticipant",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateParticipantRequest,
    responses(
        (status = 201, description = "The participant was created with its apprentice or company tutor record", body = inline(DataEnvelope<ParticipantDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Host company not found in the organization"),
        (status = 409, description = "Invalid participant, duplicate email, or both an apprentice and a company tutor block"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ParticipantCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateParticipantRequest>,
) -> Result<Response<ParticipantDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_participant(CreateParticipantCommand {
            actor,
            organization_id: path.organization_id,
            last_name: payload.last_name,
            first_name: payload.first_name,
            email: payload.email,
            phone: payload.phone,
            apprentice: payload.apprentice.map(Into::into),
            company_tutor: payload.company_tutor.map(Into::into),
        })
        .await?;

    Ok(Response::Created(created.into()))
}
