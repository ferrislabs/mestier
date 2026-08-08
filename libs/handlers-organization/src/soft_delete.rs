use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, resolve_user_id};
use http::StatusCode;
use mestier_core::OrganizationId;

use crate::{paths::OrganizationPath, require_org_membership};

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
    let user = require_org_membership(&state, &identity, organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;
    let organization = state.usecase.get_organization(organization_id).await?;

    if organization.owner_id != user.id {
        return Err(ApiError::Forbidden);
    }

    state
        .usecase
        .acting_as(actor)
        .soft_delete_organization(organization_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
