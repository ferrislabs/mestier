use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, resolve_actor};
use mestier_core::DeleteDoeCommand;

use crate::doe::{DoePath, require_doe};

/// `DELETE` removes the DOE for good, so its project can get a new one. The
/// files it referenced stay in storage. A delegated (MCP) token never
/// reaches this route.
#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/does/{doe_id}",
    operation_id = "deleteDoe",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_id" = mestier_core::DoeId, Path, description = "DOE identifier"),
    ),
    responses(
        (status = 204, description = "DOE deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoePath {
        organization_id,
        doe_id,
    }: DoePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_doe(&state, &identity, organization_id, doe_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_doe(DeleteDoeCommand { actor, id: doe_id })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
