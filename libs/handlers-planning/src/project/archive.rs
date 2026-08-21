use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState};

use crate::project::{ProjectPath, require_project};

/// `DELETE` archives; it never removes a row.
///
/// The tasks attached to a project happened, and their cost is part of a period
/// somebody may still be reading. An archived project drops out of pickers and
/// out of the default listing, and `GET` still resolves it.
#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}",
    operation_id = "archiveProject",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 204, description = "Project archived"),
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
) -> Result<StatusCode, ApiError> {
    require_project(&state, &identity, organization_id, project_id).await?;

    state.usecase.archive_project(project_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
