use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::CustomerId;

use crate::{EmptyResponse, paths::CustomerPath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/customers/{customer_id}",
    operation_id = "deleteCustomer",
    tag = super::super::TAG,
    params(
        ("customer_id" = CustomerId, Path, description = "Customer identifier"),
    ),
    responses(
        (status = 204, description = "Customer soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CustomerPath { customer_id }: CustomerPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let current = state.usecase.get_customer(customer_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    state.usecase.soft_delete_customer(customer_id).await?;

    Ok(Response::NoContent)
}
