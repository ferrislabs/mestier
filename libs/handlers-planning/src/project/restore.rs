use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{
    project::{ProjectRestorePath, require_project},
    response::ProjectResponse,
};

/// Brings an archived project back. Idempotent: restoring a live project is a
/// no-op rather than an error, because the caller's intent is already true.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/restore",
    operation_id = "restoreProject",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "Project restored", body = inline(DataEnvelope<ProjectResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectRestorePath {
        organization_id,
        project_id,
    }: ProjectRestorePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectResponse>, ApiError> {
    require_project(&state, &identity, organization_id, project_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .restore_project(actor, project_id)
        .await?;
    let project = state.usecase.get_project(project_id).await?;

    Ok(Response::OK(project.into()))
}
