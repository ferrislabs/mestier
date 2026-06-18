use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CustomerId;

use crate::{paths::CustomerPath, require_org_membership, response::CustomerResponse};

#[utoipa::path(
    get,
    path = "/api/v1/customers/{customer_id}",
    operation_id = "getCustomer",
    tag = super::super::TAG,
    params(
        ("customer_id" = CustomerId, Path, description = "Customer identifier"),
    ),
    responses(
        (status = 200, description = "Customer details", body = inline(DataEnvelope<CustomerResponse>)),
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
) -> Result<Response<CustomerResponse>, ApiError> {
    let customer = state.usecase.get_customer(customer_id).await?;
    require_org_membership(&state, &identity, customer.organization_id).await?;

    Ok(Response::OK(customer.into()))
}
