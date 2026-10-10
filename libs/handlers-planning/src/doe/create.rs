use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateDoeCommand, DoeModelId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    doe::{ProjectDoePath, response::DoeResponse},
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDoeRequest {
    pub model_id: DoeModelId,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/doe",
    operation_id = "createDoe",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    request_body = CreateDoeRequest,
    responses(
        (status = 201, description = "DOE created from the model", body = inline(DataEnvelope<DoeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project or model not found"),
        (status = 409, description = "The project already has a DOE, or the model is archived"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectDoePath {
        organization_id,
        project_id,
    }: ProjectDoePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateDoeRequest>,
) -> Result<Response<DoeResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let doe = state
        .usecase
        .create_doe(CreateDoeCommand {
            actor,
            organization_id,
            project_id,
            model_id: payload.model_id,
        })
        .await?;

    Ok(Response::Created(doe.into()))
}
