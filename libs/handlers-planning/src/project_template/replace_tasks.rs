use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ReplaceProjectTemplateTasksCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    project_template::{
        ProjectTemplateTasksPath, create::ProjectTemplateTaskShapeRequest, require_project_template,
    },
    response::ProjectTemplateTaskResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReplaceProjectTemplateTasksRequest {
    pub tasks: Vec<ProjectTemplateTaskShapeRequest>,
}

/// The complete replacement set of a template's task shapes — the builder's
/// "save" action. Mirrors `PATCH /tasks/{id}`'s `assignees`: always the full
/// list, never a delta.
#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/tasks",
    operation_id = "replaceProjectTemplateTasks",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_template_id" = mestier_core::ProjectTemplateId, Path, description = "Project template identifier"),
    ),
    request_body = ReplaceProjectTemplateTasksRequest,
    responses(
        (status = 200, description = "Task shapes replaced", body = inline(DataEnvelope<Vec<ProjectTemplateTaskResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
        (status = 409, description = "An invalid task shape, or an invalid hierarchy"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectTemplateTasksPath {
        organization_id,
        project_template_id,
    }: ProjectTemplateTasksPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReplaceProjectTemplateTasksRequest>,
) -> Result<Response<Vec<ProjectTemplateTaskResponse>>, ApiError> {
    require_project_template(&state, &identity, organization_id, project_template_id).await?;

    let tasks = state
        .usecase
        .replace_project_template_tasks(ReplaceProjectTemplateTasksCommand {
            template_id: project_template_id,
            tasks: payload.tasks.into_iter().map(Into::into).collect(),
        })
        .await?;

    Ok(Response::OK(tasks.into_iter().map(Into::into).collect()))
}
