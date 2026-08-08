use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};
use mestier_core::ProductId;

use crate::{EmptyResponse, paths::ProductPath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/products/{product_id}",
    operation_id = "deleteProduct",
    tag = super::super::TAG,
    params(
        ("product_id" = ProductId, Path, description = "Product identifier"),
    ),
    responses(
        (status = 204, description = "Product soft-deleted"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
    let current = state.usecase.get_product(product_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;
    state
        .usecase
        .acting_as(actor)
        .soft_delete_product(product_id)
        .await?;

    Ok(Response::NoContent)
}
