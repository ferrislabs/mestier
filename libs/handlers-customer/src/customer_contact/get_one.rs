use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CustomerContactId;

use crate::{
    paths::CustomerContactPath, require_customer_contact_membership,
    response::CustomerContactResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/customer-contacts/{customer_contact_id}",
    operation_id = "getCustomerContact",
    tag = super::super::TAG,
    params(
        ("customer_contact_id" = CustomerContactId, Path, description = "Customer contact identifier"),
    ),
    responses(
        (status = 200, description = "Customer contact details", body = inline(DataEnvelope<CustomerContactResponse>)),
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
) -> Result<Response<CustomerContactResponse>, ApiError> {
    require_customer_contact_membership(&state, &identity, customer_contact_id).await?;
    let customer_contact = state
        .usecase
        .get_customer_contact(customer_contact_id)
        .await?;

    Ok(Response::OK(customer_contact.into()))
}
