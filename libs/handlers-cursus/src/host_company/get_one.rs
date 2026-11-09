use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::HostCompanyPath, require_view_cursus, response::HostCompanyResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/host-companies/{host_company_id}",
    operation_id = "getCursusHostCompany",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("host_company_id" = mestier_core::HostCompanyId, Path, description = "Host company identifier"),
    ),
    responses(
        (status = 200, description = "The host company", body = inline(DataEnvelope<HostCompanyResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Host company not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    HostCompanyPath {
        organization_id,
        host_company_id,
    }: HostCompanyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<HostCompanyResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state
        .usecase
        .get_host_company(organization_id, host_company_id)
        .await?;

    Ok(Response::OK(found.into()))
}
