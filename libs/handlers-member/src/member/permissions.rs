use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{paths::MyPermissionsPath, response::MyPermissionsResponse};

/// The caller's own aggregated permission bits in one organization (#307),
/// so the webapp can decide what to offer without waiting for a 403.
///
/// Presentational only — see this route's own doc comment on
/// `MyPermissionsResponse`: nothing here is a security boundary, the API
/// still refuses every write and redacts every read on its own account,
/// this route only lets the client stop asking for what it cannot have.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/members/me/permissions",
    operation_id = "getMyPermissions",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The caller's granted permission bits, by name", body = inline(DataEnvelope<MyPermissionsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MyPermissionsPath { organization_id }: MyPermissionsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MyPermissionsResponse>, ApiError> {
    let (user_id, _actor) = resolve_actor(&state, &identity).await?;

    let permissions = state
        .usecase
        .member_permissions(user_id, organization_id)
        .await?;

    Ok(Response::OK(MyPermissionsResponse {
        permissions: permissions
            .granted_names()
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }))
}
