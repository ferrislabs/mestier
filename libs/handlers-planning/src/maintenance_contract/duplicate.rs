use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::DuplicateMaintenanceContractCommand;

use crate::maintenance_contract::{
    MaintenanceContractDuplicatePath, require_contract, response::MaintenanceContractResponse,
};

/// Copies the contract and its lines into the next season, under a new
/// project and with no intervention. Week 53 becomes week 52 when the next
/// year has none. The quote link is not carried over.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/duplicate",
    operation_id = "duplicateMaintenanceContract",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::MaintenanceContractId, Path, description = "Maintenance contract identifier"),
    ),
    responses(
        (status = 201, description = "The next season's contract", body = inline(DataEnvelope<MaintenanceContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract, customer or site not found"),
        (status = 409, description = "The copied windows do not fit the next season"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MaintenanceContractDuplicatePath {
        organization_id,
        contract_id,
    }: MaintenanceContractDuplicatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MaintenanceContractResponse>, ApiError> {
    require_contract(&state, &identity, organization_id, contract_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let contract = state
        .usecase
        .duplicate_maintenance_contract(DuplicateMaintenanceContractCommand {
            actor,
            id: contract_id,
        })
        .await?;

    Ok(Response::Created(contract.into()))
}
