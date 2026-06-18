use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::PropertyId;

use crate::{paths::PropertyPath, require_customer_membership, response::PropertyResponse};

#[utoipa::path(
    get,
    path = "/api/v1/properties/{property_id}",
    operation_id = "getProperty",
    tag = super::super::TAG,
    params(
        ("property_id" = PropertyId, Path, description = "Property identifier"),
    ),
    responses(
        (status = 200, description = "Property details", body = inline(DataEnvelope<PropertyResponse>)),
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
) -> Result<Response<PropertyResponse>, ApiError> {
    let property = state.usecase.get_property(property_id).await?;
    require_customer_membership(&state, &identity, property.customer_id).await?;

    Ok(Response::OK(property.into()))
}
