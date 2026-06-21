use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CustomerContactId, UpdateCustomerContactCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::CustomerContactPath, require_customer_membership, response::CustomerContactResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCustomerContactRequest {
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

#[utoipa::path(
    patch,
    path = "/api/v1/customer-contacts/{customer_contact_id}",
    operation_id = "updateCustomerContact",
    tag = super::super::TAG,
    params(
        ("customer_contact_id" = CustomerContactId, Path, description = "Customer contact identifier"),
    ),
    request_body = UpdateCustomerContactRequest,
    responses(
        (status = 200, description = "Customer contact updated", body = inline(DataEnvelope<CustomerContactResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer contact not found"),
        (status = 409, description = "Customer contact conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CustomerContactPath {
        customer_contact_id,
    }: CustomerContactPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateCustomerContactRequest>,
) -> Result<Response<CustomerContactResponse>, ApiError> {
    let current = state
        .usecase
        .get_customer_contact(customer_contact_id)
        .await?;
    require_customer_membership(&state, &identity, current.customer_id).await?;

    let customer_contact = state
        .usecase
        .update_customer_contact(UpdateCustomerContactCommand {
            id: customer_contact_id,
            first_name: payload.first_name,
            last_name: payload.last_name,
            role: payload.role,
            phone: payload.phone,
            email: payload.email,
            is_primary: payload.is_primary,
        })
        .await?;

    Ok(Response::OK(customer_contact.into()))
}
