use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    project::{ProjectPath, require_project},
    response::ProjectResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}",
    operation_id = "getProject",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "The project, archived or not", body = inline(DataEnvelope<ProjectResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectPath {
        organization_id,
        project_id,
    }: ProjectPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectResponse>, ApiError> {
    let project = require_project(&state, &identity, organization_id, project_id).await?;

    Ok(Response::OK(project.into()))
}
