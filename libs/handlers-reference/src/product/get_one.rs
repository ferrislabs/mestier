use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ProductId;

use crate::{paths::ProductPath, require_org_membership, response::ProductResponse};

#[utoipa::path(
    get,
    path = "/api/v1/products/{product_id}",
    operation_id = "getProduct",
    tag = super::super::TAG,
    params(
        ("product_id" = ProductId, Path, description = "Product identifier"),
    ),
    responses(
        (status = 200, description = "Product details", body = inline(DataEnvelope<ProductResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Product not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProductPath { product_id }: ProductPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProductResponse>, ApiError> {
    let product = state.usecase.get_product(product_id).await?;
    require_org_membership(&state, &identity, product.organization_id).await?;

    Ok(Response::OK(product.into()))
}
