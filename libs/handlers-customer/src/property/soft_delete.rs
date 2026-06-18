use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::PropertyId;

use crate::{EmptyResponse, paths::PropertyPath, require_customer_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/properties/{property_id}",
    operation_id = "deleteProperty",
    tag = super::super::TAG,
    params(
        ("property_id" = PropertyId, Path, description = "Property identifier"),
    ),
    responses(
        (status = 204, description = "Property soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Property not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PropertyPath { property_id }: PropertyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let current = state.usecase.get_property(property_id).await?;
    require_customer_membership(&state, &identity, current.customer_id).await?;
    state.usecase.soft_delete_property(property_id).await?;

    Ok(Response::NoContent)
}
