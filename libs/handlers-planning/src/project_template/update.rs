use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::UpdateProjectTemplateCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    project_template::{ProjectTemplatePath, require_project_template},
    response::ProjectTemplateResponse,
};

/// `name`/`description` only — task shapes go through their own route
/// (`PUT .../tasks`), so renaming a template never needs to resend every
/// shape.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}",
    operation_id = "patchProjectTemplate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_template_id" = mestier_core::ProjectTemplateId, Path, description = "Project template identifier"),
    ),
    request_body = UpdateProjectTemplateRequest,
    responses(
        (status = 200, description = "Template updated", body = inline(DataEnvelope<ProjectTemplateResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
        (status = 409, description = "A blank name"),
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
    Json(payload): Json<UpdateProjectTemplateRequest>,
) -> Result<Response<ProjectTemplateResponse>, ApiError> {
    require_project_template(&state, &identity, organization_id, project_template_id).await?;

    let template = state
        .usecase
        .update_project_template(UpdateProjectTemplateCommand {
            id: project_template_id,
            name: payload.name,
            description: payload.description,
        })
        .await?;

    Ok(Response::OK(template.into()))
}
