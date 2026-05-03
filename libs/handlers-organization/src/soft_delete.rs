use axum::extract::State;
use handlers::{ApiError, AppState};
use http::StatusCode;
use mestier_core::OrganizationId;

use crate::paths::OrganizationPath;

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}",
    operation_id = "deleteOrganization",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 204, description = "Organization soft-deleted"),
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
) -> Result<StatusCode, ApiError> {
    todo!("soft delete organization")
}
