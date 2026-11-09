use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::SetContractTutorsCommand;

use crate::{
    paths::ContractTutorsPath, request::SetContractTutorsRequest, require_org_membership,
    response::ContractResponse,
};

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/tutors",
    operation_id = "setCursusContractTutors",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::ContractId, Path, description = "Contract identifier"),
    ),
    request_body = SetContractTutorsRequest,
    responses(
        (status = 200, description = "The contract tutors were replaced", body = inline(DataEnvelope<ContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract or company tutor not found in the organization"),
        (status = 409, description = "The contract is closed, a tutor is listed twice, belongs to another host company, or the list does not have exactly one main tutor"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ContractTutorsPath {
        organization_id,
        contract_id,
    }: ContractTutorsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetContractTutorsRequest>,
) -> Result<Response<ContractResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .set_contract_tutors(SetContractTutorsCommand {
            actor,
            organization_id,
            contract_id,
            tutors: payload.tutors.into_iter().map(Into::into).collect(),
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
