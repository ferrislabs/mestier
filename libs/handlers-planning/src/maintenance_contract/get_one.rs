use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::maintenance_contract::{
    MaintenanceContractPath, require_contract, response::MaintenanceContractResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}",
    operation_id = "getMaintenanceContract",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::MaintenanceContractId, Path, description = "Maintenance contract identifier"),
    ),
    responses(
        (status = 200, description = "The contract with its lines", body = inline(DataEnvelope<MaintenanceContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract not found"),
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
) -> Result<Response<MaintenanceContractResponse>, ApiError> {
    let contract = require_contract(&state, &identity, organization_id, contract_id).await?;

    Ok(Response::OK(contract.into()))
}
