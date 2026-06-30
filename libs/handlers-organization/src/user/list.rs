use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::OrganizationUsersPath,
    require_org_membership,
    response::UserResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/users",
    operation_id = "listOrganizationUsers",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "List of users in the organization", body = inline(DataEnvelope<Vec<UserResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — not a member of this organization"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0), err)]
pub async fn handler(
    path: OrganizationUsersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<UserResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let users = state
        .usecase
        .list_users_by_org(path.organization_id)
        .await?;

    let items: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Response::OK(items))
}
