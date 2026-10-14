use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateMaintenanceContractCommand, CustomerContextId, CustomerId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    maintenance_contract::{
        MaintenanceContractsPath,
        request::{ContractBody, ContractFields},
        response::MaintenanceContractResponse,
    },
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMaintenanceContractRequest {
    pub customer_id: CustomerId,
    /// The site. Must belong to the customer.
    pub customer_context_id: CustomerContextId,
    #[serde(flatten)]
    pub contract: ContractBody,
}

/// Also creates the contract's project, named after the customer, the site
/// and the season.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/maintenance/contracts",
    operation_id = "createMaintenanceContract",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateMaintenanceContractRequest,
    responses(
        (status = 201, description = "Contract created with its lines and its project", body = inline(DataEnvelope<MaintenanceContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer, site, manager or quote not found in the organization"),
        (status = 409, description = "Invalid season, cadence or line, a window outside the season, or a site that is not the customer's"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: MaintenanceContractsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateMaintenanceContractRequest>,
) -> Result<Response<MaintenanceContractResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let fields = ContractFields::try_from(payload.contract)?;

    let contract = state
        .usecase
        .create_maintenance_contract(CreateMaintenanceContractCommand {
            actor,
            organization_id: path.organization_id,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            reference: fields.reference,
            season: fields.season,
            manager_member_id: fields.manager_member_id,
            amount_cents: fields.amount_cents,
            quote_id: fields.quote_id,
            notes: fields.notes,
            status: fields.status,
            lines: fields.lines,
        })
        .await?;

    Ok(Response::Created(contract.into()))
}
