use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;

use crate::{paths::OrganizationPath, require_org_membership, response::OrganizationResponse};

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
    OrganizationPath { organization_id }: OrganizationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let organization = state.usecase.get_organization(organization_id).await?;

    Ok(Response::OK(organization.into()))
}
