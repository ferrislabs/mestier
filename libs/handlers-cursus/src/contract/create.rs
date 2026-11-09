use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateContractCommand;

use crate::{
    paths::TriadContractsPath, request::ContractRequest, require_org_membership,
    response::ContractResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}/contracts",
    operation_id = "createCursusContract",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("triad_id" = mestier_core::TriadId, Path, description = "Triad identifier"),
    ),
    request_body = ContractRequest,
    responses(
        (status = 201, description = "The contract was created", body = inline(DataEnvelope<ContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Triad or host company not found in the organization"),
        (status = 409, description = "The period is invalid, the triad is closed or the host company is archived"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TriadContractsPath {
        organization_id,
        triad_id,
    }: TriadContractsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ContractRequest>,
) -> Result<Response<ContractResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_contract(CreateContractCommand {
            actor,
            organization_id,
            triad_id,
            host_company_id: payload.host_company_id,
            kind: payload.kind,
            number: payload.number,
            start_date: payload.start_date,
            end_date: payload.end_date,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
