use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, resolve_actor};
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
    OrganizationPath { organization_id }: OrganizationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .soft_delete_organization(organization_id, actor)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
