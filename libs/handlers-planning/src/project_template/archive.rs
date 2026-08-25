use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState};

use crate::project_template::{ProjectTemplatePath, require_project_template};

/// `DELETE` archives; it never removes a row — a template that produced
/// real projects stays available for `GET`, it just drops out of pickers.
#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}",
    operation_id = "archiveProjectTemplate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_template_id" = mestier_core::ProjectTemplateId, Path, description = "Project template identifier"),
    ),
    responses(
        (status = 204, description = "Template archived"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectTemplatePath {
        organization_id,
        project_template_id,
    }: ProjectTemplatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_project_template(&state, &identity, organization_id, project_template_id).await?;

    state
        .usecase
        .archive_project_template(project_template_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
