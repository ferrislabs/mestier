use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    project::{channel::ProjectChannelPath, require_project},
    response::ProjectChannelResponse,
};

/// `name` is optional: the whole point of "creating it from the project" is
/// that the caller does not have to think of a name — omitting it (or
/// sending `null`) names the channel after the project.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectChannelRequest {
    #[serde(default)]
    pub name: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/channel",
    operation_id = "createProjectChannel",
    tag = super::super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    request_body = CreateProjectChannelRequest,
    responses(
        (status = 201, description = "Channel created", body = inline(DataEnvelope<ProjectChannelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "The project already has a channel, or a blank name"),
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
    Json(payload): Json<CreateProjectChannelRequest>,
) -> Result<Response<ProjectChannelResponse>, ApiError> {
    require_project(&state, &identity, organization_id, project_id).await?;

    let channel = state
        .usecase
        .create_project_channel(project_id, payload.name)
        .await?;

    Ok(Response::Created(channel.into()))
}
