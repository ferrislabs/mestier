use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateCustomerCommand, CustomerPipelineStage, CustomerStatus};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::CustomersPath, require_org_membership, response::CustomerResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomerRequest {
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    #[serde(default)]
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/customers",
    operation_id = "createCustomer",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateCustomerRequest,
    responses(
        (status = 201, description = "Customer created", body = inline(DataEnvelope<CustomerResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Customer conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Response<CustomerResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let customer = state
        .usecase
        .acting_as(user_id)
        .create_customer(CreateCustomerCommand {
            actor,
            organization_id: path.organization_id,
            status: payload.status,
            pipeline_stage: payload.pipeline_stage,
            name: payload.name,
            registration_number: payload.registration_number,
            phone: payload.phone,
            email: payload.email,
        })
        .await?;

    Ok(Response::Created(customer.into()))
}
