use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    project_template::{ProjectTemplateRestorePath, require_project_template},
    response::ProjectTemplateResponse,
};

/// Brings an archived template back. Idempotent, like `restoreProject`.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/restore",
    operation_id = "restoreProjectTemplate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_template_id" = mestier_core::ProjectTemplateId, Path, description = "Project template identifier"),
    ),
    responses(
        (status = 200, description = "Template restored", body = inline(DataEnvelope<ProjectTemplateResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectTemplateRestorePath {
        organization_id,
        project_template_id,
    }: ProjectTemplateRestorePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectTemplateResponse>, ApiError> {
    require_project_template(&state, &identity, organization_id, project_template_id).await?;

    state
        .usecase
        .restore_project_template(project_template_id)
        .await?;
    let template = state
        .usecase
        .get_project_template(project_template_id)
        .await?;

    Ok(Response::OK(template.into()))
}
