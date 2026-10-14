use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::ArchiveMaintenanceContractCommand;

use crate::maintenance_contract::{
    MaintenanceContractArchivePath, require_contract, response::MaintenanceContractResponse,
};

/// Archiving keeps the contract, its lines and its interventions readable.
/// Archiving twice keeps the first date.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/archive",
    operation_id = "archiveMaintenanceContract",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::MaintenanceContractId, Path, description = "Maintenance contract identifier"),
    ),
    responses(
        (status = 200, description = "The archived contract", body = inline(DataEnvelope<MaintenanceContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MaintenanceContractArchivePath {
        organization_id,
        contract_id,
    }: MaintenanceContractArchivePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MaintenanceContractResponse>, ApiError> {
    require_contract(&state, &identity, organization_id, contract_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let contract = state
        .usecase
        .archive_maintenance_contract(ArchiveMaintenanceContractCommand {
            actor,
            id: contract_id,
        })
        .await?;

    Ok(Response::OK(contract.into()))
}
