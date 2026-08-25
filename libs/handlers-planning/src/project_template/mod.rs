//! Project templates — an ordered set of task shapes, not a project to be
//! copied. See `mestier_core::domain::project_template` for the model.
//!
//! Owns its own leaf paths, like `task` and `project` do and for the same
//! reason: `organization_id` and `project_template_id` are both part of
//! every single-item route.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{ProjectTemplate, ProjectTemplateId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod archive;
pub mod create;
pub mod get_one;
pub mod instantiate;
pub mod list;
pub mod replace_tasks;
pub mod restore;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_put(replace_tasks::handler)
        .typed_delete(archive::handler)
        .typed_post(restore::handler)
        .typed_post(instantiate::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/project-templates")]
pub struct ProjectTemplatesPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/project-templates/{project_template_id}")]
pub struct ProjectTemplatePath {
    pub organization_id: mestier_core::OrganizationId,
    pub project_template_id: ProjectTemplateId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/tasks"
)]
pub struct ProjectTemplateTasksPath {
    pub organization_id: mestier_core::OrganizationId,
    pub project_template_id: ProjectTemplateId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/restore"
)]
pub struct ProjectTemplateRestorePath {
    pub organization_id: mestier_core::OrganizationId,
    pub project_template_id: ProjectTemplateId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/instantiate"
)]
pub struct ProjectTemplateInstantiatePath {
    pub organization_id: mestier_core::OrganizationId,
    pub project_template_id: ProjectTemplateId,
}

/// Loads the template and checks both that the caller belongs to
/// `organization_id` and that the template actually belongs to it — same
/// discipline as `project::require_project`.
pub(crate) async fn require_project_template(
    state: &AppState,
    identity: &Identity,
    organization_id: mestier_core::OrganizationId,
    project_template_id: ProjectTemplateId,
) -> Result<ProjectTemplate, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let template = state
        .usecase
        .get_project_template(project_template_id)
        .await?;
    if template.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(template)
}
