use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    project_template::{ProjectTemplatePath, require_project_template},
    response::ProjectTemplateResponse,
};

/// The one surface that loads the full shape list alongside the template —
/// the builder and the "start from a template" preview both need it, and
/// neither should pay for a second call.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}",
    operation_id = "getProjectTemplate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_template_id" = mestier_core::ProjectTemplateId, Path, description = "Project template identifier"),
    ),
    responses(
        (status = 200, description = "The template, archived or not, with its task shapes", body = inline(DataEnvelope<ProjectTemplateResponse>)),
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
) -> Result<Response<ProjectTemplateResponse>, ApiError> {
    let template =
        require_project_template(&state, &identity, organization_id, project_template_id).await?;
    let tasks = state
        .usecase
        .list_project_template_tasks(project_template_id)
        .await?;

    Ok(Response::OK(ProjectTemplateResponse {
        tasks: Some(tasks.into_iter().map(Into::into).collect()),
        ..template.into()
    }))
}
