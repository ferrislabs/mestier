use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{AdminOrganizationPath, platform_admin, response::AdminOrganizationResponse};

#[utoipa::path(
    get,
    path = "/api/v1/admin/organizations/{organization_id}",
    operation_id = "getOrganizationForAdmin",
    tag = crate::ADMIN_TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Members, feature flags and flag history of one organization", body = inline(DataEnvelope<AdminOrganizationResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AdminOrganizationPath { organization_id }: AdminOrganizationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AdminOrganizationResponse>, ApiError> {
    let admin = platform_admin(&identity)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;

    let detail = state
        .usecase
        .acting_as(user_id)
        .get_organization_for_admin(admin, organization_id)
        .await?;

    Ok(Response::OK(detail.into()))
}
