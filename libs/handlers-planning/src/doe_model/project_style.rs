use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    DoeStyle, ProjectDoeStyle, ProjectDoeStyleCommand, UpdateProjectDoeStyleCommand,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{doe_model::ProjectDoeStylePath, require_org_membership};

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct ProjectDoeStyleResponse {
    /// The project's own style; `null` when its DOE follows the model or the
    /// organization.
    pub style: Option<DoeStyle>,
    /// The style the project's DOE gets without one: its model's, else the
    /// organization's, else the default.
    pub inherited: DoeStyle,
}

impl From<ProjectDoeStyle> for ProjectDoeStyleResponse {
    fn from(value: ProjectDoeStyle) -> Self {
        Self {
            style: value.style,
            inherited: value.inherited,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectDoeStyleRequest {
    pub style: DoeStyle,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/doe-style",
    operation_id = "getProjectDoeStyle",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "The project's style, if any, and the one it inherits", body = inline(DataEnvelope<ProjectDoeStyleResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_handler(
    path: ProjectDoeStylePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectDoeStyleResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let style = state
        .usecase
        .get_project_doe_style(ProjectDoeStyleCommand {
            actor,
            organization_id: path.organization_id,
            project_id: path.project_id,
        })
        .await?;

    Ok(Response::OK(style.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/doe-style",
    operation_id = "updateProjectDoeStyle",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    request_body = UpdateProjectDoeStyleRequest,
    responses(
        (status = 200, description = "The project's style, saved", body = inline(DataEnvelope<DoeStyle>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn put_handler(
    path: ProjectDoeStylePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateProjectDoeStyleRequest>,
) -> Result<Response<DoeStyle>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let style = state
        .usecase
        .acting_as(user_id)
        .update_project_doe_style(UpdateProjectDoeStyleCommand {
            actor,
            organization_id: path.organization_id,
            project_id: path.project_id,
            style: payload.style,
        })
        .await?;

    Ok(Response::OK(style))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/doe-style",
    operation_id = "clearProjectDoeStyle",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 204, description = "The project's DOE follows its model or the organization again"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_handler(
    path: ProjectDoeStylePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .clear_project_doe_style(ProjectDoeStyleCommand {
            actor,
            organization_id: path.organization_id,
            project_id: path.project_id,
        })
        .await?;

    Ok(Response::NoContent)
}
