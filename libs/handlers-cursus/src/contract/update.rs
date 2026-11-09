use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateContractCommand;

use crate::{
    paths::ContractPath, request::ContractRequest, require_org_membership,
    response::ContractResponse,
};

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}",
    operation_id = "updateCursusContract",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::ContractId, Path, description = "Contract identifier"),
    ),
    request_body = ContractRequest,
    responses(
        (status = 200, description = "The contract was updated", body = inline(DataEnvelope<ContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract or host company not found in the organization"),
        (status = 409, description = "The period is invalid, the contract is closed, the host company is archived or tutors are still linked"),
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
    Json(payload): Json<ContractRequest>,
) -> Result<Response<ContractResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_contract(UpdateContractCommand {
            actor,
            organization_id,
            id: contract_id,
            host_company_id: payload.host_company_id,
            kind: payload.kind,
            number: payload.number,
            start_date: payload.start_date,
            end_date: payload.end_date,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
