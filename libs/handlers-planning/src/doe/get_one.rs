use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::doe::{DoePath, require_doe, response::DoeResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/does/{doe_id}",
    operation_id = "getDoe",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_id" = mestier_core::DoeId, Path, description = "DOE identifier"),
    ),
    responses(
        (status = 200, description = "The DOE", body = inline(DataEnvelope<DoeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoePath {
        organization_id,
        doe_id,
    }: DoePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DoeResponse>, ApiError> {
    let doe = require_doe(&state, &identity, organization_id, doe_id).await?;

    Ok(Response::OK(doe.into()))
}
