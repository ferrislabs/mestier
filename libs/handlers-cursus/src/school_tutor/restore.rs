use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{paths::SchoolTutorRestorePath, require_org_membership, response::SchoolTutorResponse};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}/restore",
    operation_id = "restoreCursusSchoolTutor",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("school_tutor_id" = mestier_core::SchoolTutorId, Path, description = "School tutor identifier"),
    ),
    responses(
        (status = 200, description = "The school tutor is active again", body = inline(DataEnvelope<SchoolTutorResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "School tutor not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SchoolTutorRestorePath {
        organization_id,
        school_tutor_id,
    }: SchoolTutorRestorePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SchoolTutorResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let tutor = state
        .usecase
        .acting_as(user_id)
        .unarchive_school_tutor(actor, organization_id, school_tutor_id)
        .await?;

    Ok(Response::OK(tutor.into()))
}
