use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateSchoolTutorCommand, MemberId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::SchoolTutorCollectionPath, require_org_membership, response::SchoolTutorResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSchoolTutorRequest {
    pub member_id: MemberId,
    pub job_title: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/school-tutors",
    operation_id = "createCursusSchoolTutor",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateSchoolTutorRequest,
    responses(
        (status = 201, description = "The member is declared school tutor", body = inline(DataEnvelope<SchoolTutorResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
        (status = 409, description = "The member is already a school tutor"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: SchoolTutorCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateSchoolTutorRequest>,
) -> Result<Response<SchoolTutorResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_school_tutor(CreateSchoolTutorCommand {
            actor,
            organization_id: path.organization_id,
            member_id: payload.member_id,
            job_title: payload.job_title,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
