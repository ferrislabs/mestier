use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{ProductId, ServiceRateUnit, UpdateProductCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ProductPath, require_org_membership, response::ProductResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProductRequest {
    pub name: String,
    pub sku: Option<String>,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    #[serde(default)]
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/products/{product_id}",
    operation_id = "updateProduct",
    tag = super::super::TAG,
    params(
        ("product_id" = ProductId, Path, description = "Product identifier"),
    ),
    request_body = UpdateProductRequest,
    responses(
        (status = 200, description = "Product updated", body = inline(DataEnvelope<ProductResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Product not found"),
        (status = 409, description = "Product conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProductPath { product_id }: ProductPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Response<ProductResponse>, ApiError> {
    let current = state.usecase.get_product(product_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let product = state
        .usecase
        .acting_as(actor)
        .update_product(UpdateProductCommand {
            id: product_id,
            name: payload.name,
            sku: payload.sku,
            unit: payload.unit,
            unit_price_cents: payload.unit_price_cents,
            default_vat_rate_bp: payload.default_vat_rate_bp,
            description: payload.description,
        })
        .await?;

    Ok(Response::OK(product.into()))
}
