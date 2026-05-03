use axum::extract::State;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;

use crate::{paths::OrganizationPath, response::OrganizationResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}",
    operation_id = "getOrganization",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Organization details", body = inline(DataEnvelope<OrganizationResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    OrganizationPath {
        organization_id: _organization_id,
    }: OrganizationPath,
    State(_state): State<AppState>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    todo!("get organization by id")
}
