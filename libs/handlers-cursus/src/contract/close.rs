use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CloseContractCommand;

use crate::{
    paths::ContractClosePath, request::CloseRequest, require_org_membership,
    response::ContractResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/close",
    operation_id = "closeCursusContract",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::ContractId, Path, description = "Contract identifier"),
    ),
    request_body = CloseRequest,
    responses(
        (status = 200, description = "The contract is closed and its tutor links archived", body = inline(DataEnvelope<ContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract or closure reason not found in the organization"),
        (status = 409, description = "The contract is already closed or the closure reason is archived"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ContractClosePath {
        organization_id,
        contract_id,
    }: ContractClosePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CloseRequest>,
) -> Result<Response<ContractResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let closed = state
        .usecase
        .acting_as(user_id)
        .close_contract(CloseContractCommand {
            actor,
            organization_id,
            id: contract_id,
            closed_at: payload.closed_at,
            closure_reason_id: payload.closure_reason_id,
            closure_note: payload.closure_note,
        })
        .await?;

    Ok(Response::OK(closed.into()))
}
