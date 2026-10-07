use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, resolve_actor};

use crate::doe_model::{DoeModelRestorePath, require_doe_model};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}/restore",
    operation_id = "restoreDoeModel",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_model_id" = mestier_core::DoeModelId, Path, description = "DOE model identifier"),
    ),
    responses(
        (status = 204, description = "DOE model restored"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE model not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoeModelRestorePath {
        organization_id,
        doe_model_id,
    }: DoeModelRestorePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_doe_model(&state, &identity, organization_id, doe_model_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .restore_doe_model(actor, doe_model_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
