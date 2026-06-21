use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateProductCommand, ServiceRateUnit};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ProductsPath, require_org_membership, response::ProductResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductRequest {
    pub name: String,
    pub sku: Option<String>,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub description: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/products",
    operation_id = "createProduct",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created", body = inline(DataEnvelope<ProductResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Product conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ProductsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Response<ProductResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let product = state
        .usecase
        .create_product(CreateProductCommand {
            organization_id: path.organization_id,
            name: payload.name,
            sku: payload.sku,
            unit: payload.unit,
            unit_price_cents: payload.unit_price_cents,
            description: payload.description,
        })
        .await?;

    Ok(Response::Created(product.into()))
}
