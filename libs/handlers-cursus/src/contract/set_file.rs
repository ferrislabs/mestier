use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::SetContractFileCommand;

use crate::{
    paths::ContractFilePath, request::SetContractFileRequest, require_org_membership,
    response::ContractResponse,
};

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/files/{kind}",
    operation_id = "setCursusContractFile",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::ContractId, Path, description = "Contract identifier"),
        ("kind" = inline(mestier_core::ContractFileKind), Path, description = "File slot: mission_sheet"),
    ),
    request_body = SetContractFileRequest,
    responses(
        (status = 200, description = "The file was attached, or removed when `file_key` is null", body = inline(DataEnvelope<ContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract not found in the organization"),
        (status = 409, description = "Invalid file key"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ContractFilePath {
        organization_id,
        contract_id,
        kind,
    }: ContractFilePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetContractFileRequest>,
) -> Result<Response<ContractResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .set_contract_file(SetContractFileCommand {
            actor,
            organization_id,
            contract_id,
            kind,
            file_key: payload.file_key,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
