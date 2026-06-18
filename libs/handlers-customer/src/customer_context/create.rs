use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateCustomerContextCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::CustomerContextsPath, require_customer_membership, response::CustomerContextResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomerContextRequest {
    pub label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub photo_key: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/customers/{customer_id}/customer-contexts",
    operation_id = "createCustomerContext",
    tag = super::super::TAG,
    params(
        ("customer_id" = mestier_core::CustomerId, Path, description = "Customer identifier"),
    ),
    request_body = CreateCustomerContextRequest,
    responses(
        (status = 201, description = "Customer context created", body = inline(DataEnvelope<CustomerContextResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
        (status = 409, description = "Customer context conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomerContextsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCustomerContextRequest>,
) -> Result<Response<CustomerContextResponse>, ApiError> {
    require_customer_membership(&state, &identity, path.customer_id).await?;

    let customer_context = state
        .usecase
        .create_customer_context(CreateCustomerContextCommand {
            customer_id: path.customer_id,
            label: payload.label,
            address_line: payload.address_line,
            postal_code: payload.postal_code,
            city: payload.city,
            photo_key: payload.photo_key,
        })
        .await?;

    Ok(Response::Created(customer_context.into()))
}
