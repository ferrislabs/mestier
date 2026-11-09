use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{paths::HostCompanyArchivePath, require_org_membership, response::HostCompanyResponse};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/host-companies/{host_company_id}/archive",
    operation_id = "archiveCursusHostCompany",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("host_company_id" = mestier_core::HostCompanyId, Path, description = "Host company identifier"),
    ),
    responses(
        (status = 200, description = "The host company is archived", body = inline(DataEnvelope<HostCompanyResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Host company not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    HostCompanyArchivePath {
        organization_id,
        host_company_id,
    }: HostCompanyArchivePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<HostCompanyResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let archived = state
        .usecase
        .acting_as(user_id)
        .archive_host_company(actor, organization_id, host_company_id)
        .await?;

    Ok(Response::OK(archived.into()))
}
