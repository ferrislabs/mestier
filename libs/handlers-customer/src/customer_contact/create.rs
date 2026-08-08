use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::CreateCustomerContactCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::CustomerContactsPath, require_customer_membership, response::CustomerContactResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomerContactRequest {
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

#[utoipa::path(
    post,
    path = "/api/v1/customers/{customer_id}/contacts",
    operation_id = "createCustomerContact",
    tag = super::super::TAG,
    params(
        ("customer_id" = mestier_core::CustomerId, Path, description = "Customer identifier"),
    ),
    request_body = CreateCustomerContactRequest,
    responses(
        (status = 201, description = "Customer contact created", body = inline(DataEnvelope<CustomerContactResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
        (status = 409, description = "Customer contact conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomerContactsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCustomerContactRequest>,
) -> Result<Response<CustomerContactResponse>, ApiError> {
    require_customer_membership(&state, &identity, path.customer_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let customer_contact = state
        .usecase
        .acting_as(actor)
        .create_customer_contact(CreateCustomerContactCommand {
            customer_id: path.customer_id,
            first_name: payload.first_name,
            last_name: payload.last_name,
            role: payload.role,
            phone: payload.phone,
            email: payload.email,
            is_primary: payload.is_primary,
        })
        .await?;

    Ok(Response::Created(customer_contact.into()))
}
