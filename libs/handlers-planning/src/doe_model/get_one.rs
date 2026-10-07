use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::doe_model::{DoeModelPath, require_doe_model, response::DoeModelResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}",
    operation_id = "getDoeModel",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_model_id" = mestier_core::DoeModelId, Path, description = "DOE model identifier"),
    ),
    responses(
        (status = 200, description = "The DOE model", body = inline(DataEnvelope<DoeModelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE model not found"),
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
) -> Result<Response<DoeModelResponse>, ApiError> {
    let model = require_doe_model(&state, &identity, organization_id, doe_model_id).await?;

    Ok(Response::OK(model.into()))
}
