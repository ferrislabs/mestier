use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::CustomerContactId;

use crate::{EmptyResponse, paths::CustomerContactPath, require_customer_contact_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/customer-contacts/{customer_contact_id}",
    operation_id = "deleteCustomerContact",
    tag = super::super::TAG,
    params(
        ("customer_contact_id" = CustomerContactId, Path, description = "Customer contact identifier"),
    ),
    responses(
        (status = 204, description = "Customer contact soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer contact not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CustomerContactPath {
        customer_contact_id,
    }: CustomerContactPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_customer_contact_membership(&state, &identity, customer_contact_id).await?;
    state
        .usecase
        .soft_delete_customer_contact(customer_contact_id)
        .await?;
    Ok(Response::NoContent)
}
