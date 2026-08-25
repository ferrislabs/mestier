use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateProjectTemplateCommand, ProjectTemplateTaskShapeCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    project_template::ProjectTemplatesPath, require_org_membership,
    response::ProjectTemplateResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProjectTemplateTaskShapeRequest {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub day_offset: i32,
    #[serde(default)]
    pub starts_minute: Option<i16>,
    #[serde(default)]
    pub ends_minute: Option<i16>,
    #[serde(default)]
    pub all_day: bool,
    pub blocks_availability: bool,
    #[serde(default)]
    pub expenses_cents: i32,
    #[serde(default)]
    pub expenses_label: Option<String>,
    /// The `position` (0-based rank in `tasks` below) of another shape of
    /// this same request, or `null` for a root shape.
    #[serde(default)]
    pub parent_index: Option<i32>,
}

impl From<ProjectTemplateTaskShapeRequest> for ProjectTemplateTaskShapeCommand {
    fn from(value: ProjectTemplateTaskShapeRequest) -> Self {
        Self {
            title: value.title,
            description: value.description,
            day_offset: value.day_offset,
            starts_minute: value.starts_minute,
            ends_minute: value.ends_minute,
            all_day: value.all_day,
            blocks_availability: value.blocks_availability,
            expenses_cents: value.expenses_cents,
            expenses_label: value.expenses_label,
            parent_index: value.parent_index,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tasks: Vec<ProjectTemplateTaskShapeRequest>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/project-templates",
    operation_id = "createProjectTemplate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateProjectTemplateRequest,
    responses(
        (status = 201, description = "Template created", body = inline(DataEnvelope<ProjectTemplateResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A blank name, an invalid task shape, or an invalid hierarchy"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ProjectTemplatesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateProjectTemplateRequest>,
) -> Result<Response<ProjectTemplateResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let template = state
        .usecase
        .create_project_template(CreateProjectTemplateCommand {
            organization_id: path.organization_id,
            name: payload.name,
            description: payload.description,
            tasks: payload.tasks.into_iter().map(Into::into).collect(),
        })
        .await?;

    let tasks = state
        .usecase
        .list_project_template_tasks(template.id)
        .await?;

    Ok(Response::Created(ProjectTemplateResponse {
        tasks: Some(tasks.into_iter().map(Into::into).collect()),
        ..template.into()
    }))
}
