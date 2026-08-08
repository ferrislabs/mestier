use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{CustomerContextId, UpdateCustomerContextCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::CustomerContextPath, require_customer_membership, response::CustomerContextResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCustomerContextRequest {
    pub label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub photo_key: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/customer-contexts/{customer_context_id}",
    operation_id = "updateCustomerContext",
    tag = super::super::TAG,
    params(
        ("customer_context_id" = CustomerContextId, Path, description = "Customer context identifier"),
    ),
    request_body = UpdateCustomerContextRequest,
    responses(
        (status = 200, description = "Customer context updated", body = inline(DataEnvelope<CustomerContextResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer context not found"),
        (status = 409, description = "Customer context conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CustomerContextPath {
        customer_context_id,
    }: CustomerContextPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateCustomerContextRequest>,
) -> Result<Response<CustomerContextResponse>, ApiError> {
    let current = state
        .usecase
        .get_customer_context(customer_context_id)
        .await?;
    require_customer_membership(&state, &identity, current.customer_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let customer_context = state
        .usecase
        .acting_as(actor)
        .update_customer_context(UpdateCustomerContextCommand {
            id: customer_context_id,
            label: payload.label,
            address_line: payload.address_line,
            postal_code: payload.postal_code,
            city: payload.city,
            photo_key: payload.photo_key,
        })
        .await?;

    Ok(Response::OK(customer_context.into()))
}
