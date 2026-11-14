use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::SchoolTutorPath, require_view_cursus, response::SchoolTutorResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}",
    operation_id = "getCursusSchoolTutor",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("school_tutor_id" = mestier_core::SchoolTutorId, Path, description = "School tutor identifier"),
    ),
    responses(
        (status = 200, description = "The school tutor", body = inline(DataEnvelope<SchoolTutorResponse>)),
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
) -> Result<Response<SchoolTutorResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state
        .usecase
        .get_school_tutor(organization_id, school_tutor_id)
        .await?;

    Ok(Response::OK(found.into()))
}
