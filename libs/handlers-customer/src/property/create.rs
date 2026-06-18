use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreatePropertyCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::PropertiesPath, require_customer_membership, response::PropertyResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePropertyRequest {
    pub label: String,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub photo_key: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/customers/{customer_id}/properties",
    operation_id = "createProperty",
    tag = super::super::TAG,
    params(
        ("customer_id" = mestier_core::CustomerId, Path, description = "Customer identifier"),
    ),
    request_body = CreatePropertyRequest,
    responses(
        (status = 201, description = "Property created", body = inline(DataEnvelope<PropertyResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
        (status = 409, description = "Property conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: PropertiesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreatePropertyRequest>,
) -> Result<Response<PropertyResponse>, ApiError> {
    require_customer_membership(&state, &identity, path.customer_id).await?;

    let property = state
        .usecase
        .create_property(CreatePropertyCommand {
            customer_id: path.customer_id,
            label: payload.label,
            street: payload.street,
            zip: payload.zip,
            city: payload.city,
            photo_key: payload.photo_key,
        })
        .await?;

    Ok(Response::Created(property.into()))
}
