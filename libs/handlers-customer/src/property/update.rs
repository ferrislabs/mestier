use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{PropertyId, UpdatePropertyCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::PropertyPath, require_customer_membership, response::PropertyResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePropertyRequest {
    pub label: String,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub photo_key: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/properties/{property_id}",
    operation_id = "updateProperty",
    tag = super::super::TAG,
    params(
        ("property_id" = PropertyId, Path, description = "Property identifier"),
    ),
    request_body = UpdatePropertyRequest,
    responses(
        (status = 200, description = "Property updated", body = inline(DataEnvelope<PropertyResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Property not found"),
        (status = 409, description = "Property conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PropertyPath { property_id }: PropertyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdatePropertyRequest>,
) -> Result<Response<PropertyResponse>, ApiError> {
    let current = state.usecase.get_property(property_id).await?;
    require_customer_membership(&state, &identity, current.customer_id).await?;

    let property = state
        .usecase
        .update_property(UpdatePropertyCommand {
            id: property_id,
            label: payload.label,
            street: payload.street,
            zip: payload.zip,
            city: payload.city,
            photo_key: payload.photo_key,
        })
        .await?;

    Ok(Response::OK(property.into()))
}
