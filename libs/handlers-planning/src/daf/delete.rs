use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, resolve_actor};
use mestier_core::DeleteDafCommand;

use crate::daf::{DafPath, require_daf};

/// `DELETE` removes the DAF for good. Its number is free again. The files it
/// referenced stay in storage. A delegated (MCP) token never reaches this
/// route.
#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/dafs/{daf_id}",
    operation_id = "deleteDaf",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("daf_id" = mestier_core::DafId, Path, description = "DAF identifier"),
    ),
    responses(
        (status = 204, description = "DAF deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DAF not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DafPath {
        organization_id,
        daf_id,
    }: DafPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_daf(&state, &identity, organization_id, daf_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_daf(DeleteDafCommand { actor, id: daf_id })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
