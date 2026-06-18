use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::CustomerContextId;

use crate::{EmptyResponse, paths::CustomerContextPath, require_customer_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/customer-contexts/{customer_context_id}",
    operation_id = "deleteCustomerContext",
    tag = super::super::TAG,
    params(
        ("customer_context_id" = CustomerContextId, Path, description = "Customer context identifier"),
    ),
    responses(
        (status = 204, description = "Customer context soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer context not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CustomerContextPath {
        customer_context_id,
    }: CustomerContextPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let current = state
        .usecase
        .get_customer_context(customer_context_id)
        .await?;
    require_customer_membership(&state, &identity, current.customer_id).await?;
    state
        .usecase
        .soft_delete_customer_context(customer_context_id)
        .await?;

    Ok(Response::NoContent)
}
