use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    project::{channel::ProjectChannelPath, require_project},
    response::ProjectChannelResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/channel",
    operation_id = "getProjectChannel",
    tag = super::super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "The project's channel", body = inline(DataEnvelope<ProjectChannelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found, or it has no channel"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectChannelPath {
        organization_id,
        project_id,
    }: ProjectChannelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectChannelResponse>, ApiError> {
    require_project(&state, &identity, organization_id, project_id).await?;

    let channel = state.usecase.get_project_channel(project_id).await?;

    Ok(Response::OK(channel.into()))
}
