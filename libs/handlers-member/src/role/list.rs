use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{paths::RolesPath, response::RoleResponse};

/// An organization's roles, with their permissions (#308). Gated on
/// `role.manage`, not a separate view bit: unlike cost visibility, seeing
/// how roles are defined is itself a role-management concern, so this
/// route is reachable only by whoever could also change one.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/roles",
    operation_id = "listRoles",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The organization's roles", body = inline(DataEnvelope<Vec<RoleResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RolesPath { organization_id }: RolesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<RoleResponse>>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let roles = state
        .usecase
        .acting_as(user_id)
        .list_roles(organization_id, actor)
        .await?;

    Ok(Response::OK(
        roles.into_iter().map(RoleResponse::from).collect(),
    ))
}
