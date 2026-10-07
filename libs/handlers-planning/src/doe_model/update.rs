use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{DoeModelSection, UpdateDoeModelCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::doe_model::{DoeModelPath, require_doe_model, response::DoeModelResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDoeModelRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}",
    operation_id = "updateDoeModel",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_model_id" = mestier_core::DoeModelId, Path, description = "DOE model identifier"),
    ),
    request_body = UpdateDoeModelRequest,
    responses(
        (status = 200, description = "DOE model updated", body = inline(DataEnvelope<DoeModelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE model not found"),
        (status = 409, description = "Invalid content, or the model is archived"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoeModelPath {
        organization_id,
        doe_model_id,
    }: DoeModelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateDoeModelRequest>,
) -> Result<Response<DoeModelResponse>, ApiError> {
    require_doe_model(&state, &identity, organization_id, doe_model_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let model = state
        .usecase
        .acting_as(user_id)
        .update_doe_model(UpdateDoeModelCommand {
            actor,
            id: doe_model_id,
            name: payload.name,
            description: payload.description,
            sections: payload.sections,
        })
        .await?;

    Ok(Response::OK(model.into()))
}
