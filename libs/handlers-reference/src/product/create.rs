use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateProductCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ProductsPath, require_org_membership, response::ProductResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductRequest {
    pub name: String,
    pub sku: Option<String>,
    /// One of the built-in `ServiceRateUnit` codes, or an organization's own
    /// custom unit code (#449) — a plain string because a fixed enum cannot
    /// name a value it does not know about yet.
    pub unit: String,
    pub unit_price_cents: i32,
    #[serde(default)]
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
    pub photo_keys: Vec<String>,
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
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let product = state
        .usecase
        .acting_as(user_id)
        .create_product(CreateProductCommand {
            actor,
            organization_id: path.organization_id,
            name: payload.name,
            sku: payload.sku,
            unit: payload.unit,
            unit_price_cents: payload.unit_price_cents,
            default_vat_rate_bp: payload.default_vat_rate_bp,
            description: payload.description,
            photo_keys: payload.photo_keys,
        })
        .await?;

    Ok(Response::Created(product.into()))
}
