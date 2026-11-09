use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ContractPath, require_view_cursus, response::ContractResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}",
    operation_id = "getCursusContract",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::ContractId, Path, description = "Contract identifier"),
    ),
    responses(
        (status = 200, description = "The contract with its tutors", body = inline(DataEnvelope<ContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ContractPath {
        organization_id,
        contract_id,
    }: ContractPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ContractResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state
        .usecase
        .get_contract(organization_id, contract_id)
        .await?;

    Ok(Response::OK(found.into()))
}
