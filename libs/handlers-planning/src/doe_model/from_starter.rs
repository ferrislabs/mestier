use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateDoeModelFromStarterCommand, StarterModel};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    doe_model::{DoeModelFromStarterPath, response::DoeModelResponse},
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDoeModelFromStarterRequest {
    pub starter: StarterModel,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/doe-models/from-starter",
    operation_id = "createDoeModelFromStarter",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateDoeModelFromStarterRequest,
    responses(
        (status = 201, description = "A copy of the starter model, owned by the organization", body = inline(DataEnvelope<DoeModelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DoeModelFromStarterPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateDoeModelFromStarterRequest>,
) -> Result<Response<DoeModelResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let model = state
        .usecase
        .acting_as(user_id)
        .create_doe_model_from_starter(CreateDoeModelFromStarterCommand {
            actor,
            organization_id: path.organization_id,
            starter: payload.starter,
        })
        .await?;

    Ok(Response::Created(model.into()))
}
