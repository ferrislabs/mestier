use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateSchoolTutorCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::SchoolTutorPath, require_org_membership, response::SchoolTutorResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSchoolTutorRequest {
    pub job_title: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}",
    operation_id = "updateCursusSchoolTutor",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("school_tutor_id" = mestier_core::SchoolTutorId, Path, description = "School tutor identifier"),
    ),
    request_body = UpdateSchoolTutorRequest,
    responses(
        (status = 200, description = "The school tutor was updated", body = inline(DataEnvelope<SchoolTutorResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "School tutor not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SchoolTutorPath {
        organization_id,
        school_tutor_id,
    }: SchoolTutorPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateSchoolTutorRequest>,
) -> Result<Response<SchoolTutorResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_school_tutor(UpdateSchoolTutorCommand {
            actor,
            organization_id,
            id: school_tutor_id,
            job_title: payload.job_title,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
