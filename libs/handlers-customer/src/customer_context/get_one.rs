use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CustomerContextId;

use crate::{
    paths::CustomerContextPath, require_customer_membership, response::CustomerContextResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/customer-contexts/{customer_context_id}",
    operation_id = "getCustomerContext",
    tag = super::super::TAG,
    params(
        ("customer_context_id" = CustomerContextId, Path, description = "Customer context identifier"),
    ),
    responses(
        (status = 200, description = "Customer context details", body = inline(DataEnvelope<CustomerContextResponse>)),
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
) -> Result<Response<CustomerContextResponse>, ApiError> {
    let customer_context = state
        .usecase
        .get_customer_context(customer_context_id)
        .await?;
    require_customer_membership(&state, &identity, customer_context.customer_id).await?;

    Ok(Response::OK(customer_context.into()))
}
