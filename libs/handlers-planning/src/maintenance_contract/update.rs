use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateMaintenanceContractCommand;

use crate::maintenance_contract::{
    MaintenanceContractPath,
    request::{ContractBody, ContractFields},
    require_contract,
    response::MaintenanceContractResponse,
};

/// Replaces the contract and its lines. The customer, the site and the year
/// stay as created.
#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}",
    operation_id = "updateMaintenanceContract",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::MaintenanceContractId, Path, description = "Maintenance contract identifier"),
    ),
    request_body = ContractBody,
    responses(
        (status = 200, description = "The contract, replaced", body = inline(DataEnvelope<MaintenanceContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract, manager or quote not found"),
        (status = 409, description = "Invalid season, cadence or line, a window outside the season, another year, a line that is not the contract's, or a removed line that has interventions"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MaintenanceContractPath {
        organization_id,
        contract_id,
    }: MaintenanceContractPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ContractBody>,
) -> Result<Response<MaintenanceContractResponse>, ApiError> {
    require_contract(&state, &identity, organization_id, contract_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let fields = ContractFields::try_from(payload)?;

    let contract = state
        .usecase
        .update_maintenance_contract(UpdateMaintenanceContractCommand {
            actor,
            id: contract_id,
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

    Ok(Response::OK(contract.into()))
}
