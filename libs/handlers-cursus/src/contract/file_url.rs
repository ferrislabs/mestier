use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ContractFilePath, require_view_cursus, response::ContractFileUrlResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/files/{kind}",
    operation_id = "getCursusContractFileUrl",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("contract_id" = mestier_core::ContractId, Path, description = "Contract identifier"),
        ("kind" = inline(mestier_core::ContractFileKind), Path, description = "File slot: cerfa or mission_sheet"),
    ),
    responses(
        (status = 200, description = "A short-lived presigned URL for the attached file", body = inline(DataEnvelope<ContractFileUrlResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract not found in the organization, or no file attached"),
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
) -> Result<Response<ContractFileUrlResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let key = state
        .usecase
        .get_contract_file_key(organization_id, contract_id, kind)
        .await?;
    let presigned = state.file_storage.presigned_get_url(key.as_str()).await?;

    Ok(Response::OK(ContractFileUrlResponse { url: presigned.url }))
}
