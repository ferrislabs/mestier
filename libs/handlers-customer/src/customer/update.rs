use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{CustomerId, CustomerPipelineStage, CustomerStatus, UpdateCustomerCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::CustomerPath, require_org_membership, response::CustomerResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCustomerRequest {
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/customers/{customer_id}",
    operation_id = "updateCustomer",
    tag = super::super::TAG,
    params(
        ("customer_id" = CustomerId, Path, description = "Customer identifier"),
    ),
    request_body = UpdateCustomerRequest,
    responses(
        (status = 200, description = "Customer updated", body = inline(DataEnvelope<CustomerResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
        (status = 409, description = "Customer conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CustomerPath { customer_id }: CustomerPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> Result<Response<CustomerResponse>, ApiError> {
    let current = state.usecase.get_customer(customer_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let customer = state
        .usecase
        .acting_as(actor)
        .update_customer(UpdateCustomerCommand {
            id: customer_id,
            status: payload.status,
            pipeline_stage: payload.pipeline_stage,
            name: payload.name,
            phone: payload.phone,
            email: payload.email,
        })
        .await?;

    Ok(Response::OK(customer.into()))
}
