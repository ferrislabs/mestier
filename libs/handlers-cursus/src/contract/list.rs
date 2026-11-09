use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::TriadContractsPath, require_view_cursus, response::ContractResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}/contracts",
    operation_id = "listCursusContracts",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("triad_id" = mestier_core::TriadId, Path, description = "Triad identifier"),
    ),
    responses(
        (status = 200, description = "The contracts of the triad with their tutors", body = inline(DataEnvelope<Vec<ContractResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Triad not found in the organization"),
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
) -> Result<Response<Vec<ContractResponse>>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let items = state
        .usecase
        .list_contracts(organization_id, triad_id)
        .await?;

    Ok(Response::OK(
        items.into_iter().map(ContractResponse::from).collect(),
    ))
}
